//! Hand-drawn lines. Every circle, arc and line the game draws on the map (ranges, radial
//! rings, hover, torpedo lane, route, bars, progress) is a band textured with the pencil
//! stroke `ui/stroke.png`, the same width at any radius. Systems queue shapes each frame;
//! one mesh is rebuilt from them. Without the drawing a soft ink band stands in.
//! Sole writer of `Strokes` and of the canvas mesh.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite_render::AlphaMode2d;

use crate::balance::Z_UI;
use crate::game::art::exists;

pub fn plugin(app: &mut App) {
    app.init_resource::<Strokes>()
        .add_systems(Startup, spawn_canvas)
        .add_systems(PostUpdate, build);
}

/// World units across a stroke band. The drawing's line sits in its middle.
const WIDTH: f32 = 3.0;
/// The stroke texture is 96 x 8 world units; its length repeats every this many widths.
const REPEAT: f32 = 12.0;
/// Above map labels and radial discs, below nothing but the screen-space UI.
const Z_STROKES: f32 = Z_UI + 2.0;

/// Shapes queued this frame, drawn and cleared in `PostUpdate`.
#[derive(Resource, Default)]
pub struct Strokes {
    paths: Vec<Path>,
}

struct Path {
    points: Vec<Vec2>,
    closed: bool,
    color: Color,
}

impl Strokes {
    pub fn line(&mut self, a: Vec2, b: Vec2, color: Color) {
        self.paths.push(Path {
            points: vec![a, b],
            closed: false,
            color,
        });
    }

    pub fn circle(&mut self, centre: Vec2, radius: f32, color: Color) {
        let n = segments(radius);
        self.paths.push(Path {
            points: (0..n)
                .map(|i| {
                    centre + Vec2::from_angle(i as f32 / n as f32 * std::f32::consts::TAU) * radius
                })
                .collect(),
            closed: true,
            color,
        });
    }

    /// `angle` radians of a circle, clockwise from straight up, like a clock hand.
    pub fn arc(&mut self, centre: Vec2, radius: f32, angle: f32, color: Color) {
        let n = ((segments(radius) as f32 * angle / std::f32::consts::TAU).ceil() as usize).max(1);
        self.paths.push(Path {
            points: (0..=n)
                .map(|i| {
                    let a = std::f32::consts::FRAC_PI_2 - angle * i as f32 / n as f32;
                    centre + Vec2::from_angle(a) * radius
                })
                .collect(),
            closed: false,
            color,
        });
    }
}

fn segments(radius: f32) -> usize {
    ((radius / 4.0) as usize).clamp(16, 128)
}

/// One band along `points`: two vertices per point, mitred so neighbouring quads never
/// overlap (overlaps would show as darker knots in a translucent stroke). `u` runs along
/// the length in stroke repeats, `v` across.
pub fn band(points: &[Vec2], closed: bool, width: f32) -> (Vec<Vec2>, Vec<Vec2>) {
    let n = points.len();
    let mut positions = Vec::with_capacity(2 * (n + 1));
    let mut uvs = Vec::with_capacity(2 * (n + 1));
    let mut along = 0.0;
    let count = if closed { n + 1 } else { n };
    for i in 0..count {
        let p = points[i % n];
        let prev = if i > 0 || closed {
            Some(points[(i + n - 1) % n])
        } else {
            None
        };
        let next = if i + 1 < n || closed {
            Some(points[(i + 1) % n])
        } else {
            None
        };
        let dir_in = prev.map(|q| (p - q).normalize_or_zero());
        let dir_out = next.map(|q| (q - p).normalize_or_zero());
        let tangent = match (dir_in, dir_out) {
            (Some(a), Some(b)) => (a + b).normalize_or(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => Vec2::X,
        };
        let normal = tangent.perp();
        let mitre = dir_in
            .or(dir_out)
            .map_or(1.0, |d| normal.dot(d.perp()).abs().max(0.3));
        let offset = normal * width * 0.5 / mitre;
        if let Some(q) = prev.filter(|_| i > 0) {
            along += p.distance(q);
        }
        let u = along / (width * REPEAT);
        positions.push(p + offset);
        positions.push(p - offset);
        uvs.push(Vec2::new(u, 0.0));
        uvs.push(Vec2::new(u, 1.0));
    }
    (positions, uvs)
}

#[derive(Component)]
struct Canvas(Handle<Mesh>);

/// A soft vertical ink profile: the stand-in for the drawn stroke.
fn soft_band() -> Image {
    const HEIGHT: u32 = 16;
    let data = (0..HEIGHT)
        .flat_map(|y| {
            let v = (y as f32 + 0.5) / HEIGHT as f32;
            let alpha = ((1.0 - (v - 0.5).abs() * 2.0) * 2.5).clamp(0.0, 1.0);
            [255, 255, 255, (alpha * 255.0) as u8]
        })
        .collect();
    Image::new(
        Extent3d {
            width: 1,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn spawn_canvas(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture = if exists("ui/stroke.png") {
        assets
            .load_builder()
            .with_settings(|s: &mut ImageLoaderSettings| {
                s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    ..ImageSamplerDescriptor::linear()
                });
            })
            .load("ui/stroke.png")
    } else {
        images.add(soft_band())
    };
    let mesh = meshes.add(Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    ));
    commands.spawn((
        Name::new("Strokes"),
        Canvas(mesh.clone()),
        Mesh2d(mesh),
        MeshMaterial2d(materials.add(ColorMaterial {
            texture: Some(texture),
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
        Transform::from_translation(Vec3::Z * Z_STROKES),
        NoFrustumCulling,
    ));
}

fn build(
    mut strokes: ResMut<Strokes>,
    mut meshes: ResMut<Assets<Mesh>>,
    canvas: Single<(&Canvas, &mut Visibility)>,
) {
    let (canvas, mut visibility) = canvas.into_inner();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    for path in strokes.paths.drain(..) {
        if path.points.len() < 2 {
            continue;
        }
        let (band_positions, band_uvs) = band(&path.points, path.closed, WIDTH);
        let base = positions.len() as u32;
        let color = path.color.to_linear().to_f32_array();
        for (p, uv) in band_positions.iter().zip(&band_uvs) {
            positions.push([p.x, p.y, 0.0]);
            uvs.push(uv.to_array());
            colors.push(color);
        }
        for i in 0..(band_positions.len() as u32 / 2 - 1) {
            let a = base + 2 * i;
            indices.extend([a, a + 1, a + 2, a + 1, a + 3, a + 2]);
        }
    }
    let wanted = if indices.is_empty() {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    if *visibility != wanted {
        *visibility = wanted;
    }
    if indices.is_empty() {
        return;
    }
    let Some(mut mesh) = meshes.get_mut(&canvas.0) else {
        return;
    };
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_circle_band_keeps_its_width_and_its_u_keeps_growing() {
        let points: Vec<Vec2> = (0..64)
            .map(|i| Vec2::from_angle(i as f32 / 64.0 * std::f32::consts::TAU) * 100.0)
            .collect();
        let (positions, uvs) = band(&points, true, WIDTH);
        for pair in positions.chunks(2) {
            let across = pair[0].distance(pair[1]);
            assert!((across - WIDTH).abs() < 0.05, "band is {across} wide");
        }
        for w in uvs.windows(2) {
            assert!(w[1].x >= w[0].x);
        }
        let circumference = uvs.last().unwrap().x * WIDTH * REPEAT;
        assert!((circumference - std::f32::consts::TAU * 100.0).abs() < 2.0);
    }

    #[test]
    fn a_straight_line_is_one_quad_of_the_given_width() {
        let (positions, _) = band(&[Vec2::ZERO, Vec2::new(10.0, 0.0)], false, 2.0);
        assert_eq!(positions.len(), 4);
        assert_eq!(positions[0].distance(positions[1]), 2.0);
    }
}

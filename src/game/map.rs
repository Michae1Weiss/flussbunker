//! Static world: the river centreline as a polyline from the sea (`s = 0`) to the
//! bunker (`s = length`), in world units, plus the arc-length maths every mover uses,
//! and the fixed build spots. Sole writer of `RiverPath`, `BuildSpot`.

use bevy::prelude::*;

use crate::balance::{
    BUILD_SPOT_RADIUS, MAP_HEIGHT_TILES, MAP_WIDTH_TILES, RIVER_HALF_WIDTH, TILE_SIZE, Z_TILE,
    Z_TOWER,
};
use crate::game::palette::FAINT;
use crate::game::stroke::Strokes;
use crate::game::wave::WaveState;
use crate::game_plugin::Frame;
use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    let points = CENTRELINE_TILES
        .iter()
        .map(|&(x, y)| tile_to_world(x, y))
        .collect();
    app.insert_resource(RiverPath::new(points))
        .add_systems(OnEnter(Screen::Gameplay), (spawn_river, spawn_build_spots))
        .add_systems(Update, draw_build_spots.in_set(Frame::Present));
}

/// Tile coordinates, x rightwards and y downwards from the top-left corner of the map.
/// Straights of different lengths on purpose; the longest must exceed `TORPEDO_RUN`.
const CENTRELINE_TILES: &[(f32, f32)] = &[
    (-1.0, 3.0),
    (9.0, 3.0),
    (12.0, 8.0),
    (23.0, 8.0),
    (23.0, 13.0),
    (28.0, 13.0),
    (28.0, 17.5),
];

/// Tile coordinates like `CENTRELINE_TILES`. Mostly at bends, where turrets matter and
/// torpedoes do not; deliberately too few to cover everything.
const BUILD_SPOT_TILES: &[(f32, f32)] = &[
    (7.5, 5.5),
    (13.0, 4.5),
    (14.0, 10.5),
    (20.5, 5.5),
    (25.5, 10.5),
    (20.5, 14.5),
    (25.5, 15.5),
    (30.5, 14.5),
];

/// Greybox water; hidden when a painted map exists (`art`).
#[derive(Component)]
pub struct RiverSegment;

/// The bunker sprite at the river's end.
#[derive(Component)]
pub struct BunkerSprite;

/// A place a turret can be built. Occupancy is tracked by `tower::Occupied`.
#[derive(Component)]
pub struct BuildSpot;

pub fn tile_to_world(x: f32, y: f32) -> Vec2 {
    Vec2::new(
        (x - MAP_WIDTH_TILES * 0.5) * TILE_SIZE,
        (MAP_HEIGHT_TILES * 0.5 - y) * TILE_SIZE,
    )
}

#[derive(Resource)]
pub struct RiverPath {
    points: Vec<Vec2>,
    /// Arc length at each point; `cumulative[0] == 0`.
    cumulative: Vec<f32>,
}

impl RiverPath {
    /// # Panics
    /// With fewer than two points.
    pub fn new(points: Vec<Vec2>) -> Self {
        assert!(points.len() >= 2, "a river needs at least two points");
        let mut cumulative = Vec::with_capacity(points.len());
        let mut total = 0.0;
        cumulative.push(0.0);
        for pair in points.windows(2) {
            total += pair[0].distance(pair[1]);
            cumulative.push(total);
        }
        Self { points, cumulative }
    }

    pub fn length(&self) -> f32 {
        *self.cumulative.last().unwrap()
    }

    pub fn spawn_point(&self) -> Vec2 {
        self.points[0]
    }

    pub fn bunker_point(&self) -> Vec2 {
        *self.points.last().unwrap()
    }

    pub fn segments(&self) -> impl Iterator<Item = (Vec2, Vec2)> + '_ {
        self.points.windows(2).map(|pair| (pair[0], pair[1]))
    }

    /// Segment containing `s`; `s` at a vertex belongs to the segment starting there.
    fn segment_index(&self, s: f32) -> usize {
        let after = self.cumulative.partition_point(|&c| c <= s);
        after.clamp(1, self.points.len() - 1) - 1
    }

    /// `s` is clamped to the river.
    pub fn point_at(&self, s: f32) -> Vec2 {
        let s = s.clamp(0.0, self.length());
        let i = self.segment_index(s);
        let t = (s - self.cumulative[i]) / (self.cumulative[i + 1] - self.cumulative[i]);
        self.points[i].lerp(self.points[i + 1], t)
    }

    /// Unit direction of increasing `s`, i.e. downstream.
    pub fn tangent_at(&self, s: f32) -> Vec2 {
        let i = self.segment_index(s.clamp(0.0, self.length()));
        (self.points[i + 1] - self.points[i]).normalize()
    }

    /// Arc length of the nearest point on the centreline, and the distance to it.
    pub fn project(&self, p: Vec2) -> (f32, f32) {
        let mut best = (0.0, f32::INFINITY);
        for (i, (a, b)) in self.segments().enumerate() {
            let ab = b - a;
            let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
            let distance = p.distance(a + ab * t);
            if distance < best.1 {
                best = (self.cumulative[i] + t * ab.length(), distance);
            }
        }
        best
    }

    pub fn contains(&self, p: Vec2) -> bool {
        self.project(p).1 <= RIVER_HALF_WIDTH
    }

    /// Arc length where the straight containing `s` ends, walking downstream or upstream.
    /// At a vertex, upstream means the straight that ends there.
    pub fn straight_end(&self, s: f32, downstream: bool) -> f32 {
        let s = s.clamp(0.0, self.length());
        if downstream {
            self.cumulative[self.segment_index(s) + 1]
        } else {
            let before = self.cumulative.partition_point(|&c| c < s);
            self.cumulative[before.clamp(1, self.points.len() - 1) - 1]
        }
    }
}

fn spawn_river(
    mut commands: Commands,
    path: Res<RiverPath>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let water = Color::srgb(0.10, 0.24, 0.34);
    let bend = meshes.add(Circle::new(RIVER_HALF_WIDTH));
    let water_material = materials.add(water);

    for (a, b) in path.segments() {
        let along = b - a;
        commands.spawn((
            Name::new("River segment"),
            RiverSegment,
            DespawnOnExit(Screen::Gameplay),
            Sprite {
                color: water,
                custom_size: Some(Vec2::new(along.length(), RIVER_HALF_WIDTH * 2.0)),
                ..default()
            },
            Transform::from_translation(a.midpoint(b).extend(Z_TILE))
                .with_rotation(Quat::from_rotation_z(along.to_angle())),
        ));
        commands.spawn((
            Name::new("River bend"),
            RiverSegment,
            DespawnOnExit(Screen::Gameplay),
            Mesh2d(bend.clone()),
            MeshMaterial2d(water_material.clone()),
            Transform::from_translation(b.extend(Z_TILE)),
        ));
    }

    commands.spawn((
        Name::new("Bunker"),
        BunkerSprite,
        DespawnOnExit(Screen::Gameplay),
        Sprite {
            color: Color::srgb(0.34, 0.32, 0.30),
            custom_size: Some(Vec2::splat(TILE_SIZE * 1.6)),
            ..default()
        },
        Transform::from_translation(path.bunker_point().extend(Z_TOWER)),
    ));
}

fn spawn_build_spots(mut commands: Commands) {
    for &(x, y) in BUILD_SPOT_TILES {
        commands.spawn((
            Name::new("Build spot"),
            BuildSpot,
            DespawnOnExit(Screen::Gameplay),
            Transform::from_translation(tile_to_world(x, y).extend(Z_TILE)),
        ));
    }
}

/// Greybox only: a drawn spot sprite replaces the circle.
fn draw_build_spots(
    mut strokes: ResMut<Strokes>,
    wave: Res<WaveState>,
    spots: Query<&Transform, (With<BuildSpot>, Without<Sprite>)>,
) {
    if wave.prologue {
        return;
    }
    for transform in &spots {
        strokes.circle(transform.translation.truncate(), BUILD_SPOT_RADIUS, FAINT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balance::TORPEDO_RUN;

    fn map() -> RiverPath {
        let points = CENTRELINE_TILES
            .iter()
            .map(|&(x, y)| tile_to_world(x, y))
            .collect();
        RiverPath::new(points)
    }

    #[test]
    fn build_spots_are_on_land_within_reach_of_the_river() {
        let river = map();
        for &(x, y) in BUILD_SPOT_TILES {
            let (_, distance) = river.project(tile_to_world(x, y));
            assert!(
                distance > RIVER_HALF_WIDTH + BUILD_SPOT_RADIUS,
                "spot ({x}, {y}) touches the water"
            );
            assert!(
                distance < 3.0 * TILE_SIZE,
                "spot ({x}, {y}) is too far from the river"
            );
        }
    }

    #[test]
    fn build_spots_do_not_overlap() {
        for (i, &(ax, ay)) in BUILD_SPOT_TILES.iter().enumerate() {
            for &(bx, by) in &BUILD_SPOT_TILES[i + 1..] {
                let distance = tile_to_world(ax, ay).distance(tile_to_world(bx, by));
                assert!(distance > 2.0 * BUILD_SPOT_RADIUS);
            }
        }
    }

    fn river() -> RiverPath {
        RiverPath::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            Vec2::new(100.0, 50.0),
            Vec2::new(160.0, 50.0),
        ])
    }

    #[test]
    fn arc_length_sums_the_segments() {
        assert_eq!(river().length(), 210.0);
    }

    #[test]
    fn project_inverts_point_at() {
        let river = river();
        for s in [0.0, 37.5, 100.0, 120.0, 210.0] {
            let (back, distance) = river.project(river.point_at(s));
            assert!((back - s).abs() < 1e-3, "s = {s}, got {back}");
            assert!(distance < 1e-3);
        }
    }

    #[test]
    fn points_beside_the_channel_are_outside() {
        let river = river();
        assert!(river.contains(Vec2::new(50.0, RIVER_HALF_WIDTH * 0.5)));
        assert!(!river.contains(Vec2::new(50.0, RIVER_HALF_WIDTH * 1.5)));
    }

    #[test]
    fn straight_end_stops_at_the_bend() {
        let river = river();
        assert_eq!(river.straight_end(30.0, true), 100.0);
        assert_eq!(river.straight_end(30.0, false), 0.0);
        assert_eq!(river.straight_end(120.0, false), 100.0);
        assert_eq!(river.straight_end(100.0, false), 0.0);
        assert_eq!(river.straight_end(100.0, true), 150.0);
    }

    #[test]
    fn tangent_points_downstream() {
        let river = river();
        assert_eq!(river.tangent_at(50.0), Vec2::X);
        assert_eq!(river.tangent_at(120.0), Vec2::Y);
    }

    #[test]
    fn the_map_has_a_straight_longer_than_a_torpedo_run() {
        let river = map();
        let longest = river
            .segments()
            .map(|(a, b)| a.distance(b))
            .fold(0.0, f32::max);
        assert!(
            longest > TORPEDO_RUN,
            "longest straight {longest} <= {TORPEDO_RUN}"
        );
    }
}

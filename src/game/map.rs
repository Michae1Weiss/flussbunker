//! The static world: which tiles exist, and where the river goes.
//!
//! Two resources, deliberately not one:
//!
//! * `MapGrid`   — what is at cell (x, y), and grid <-> world maths.
//! * `RiverPath` — the ordered waypoints from the sea to the bunker.
//!
//! Splitting them is information hiding in practice. `enemy.rs` needs the path
//! and nothing else; if both lived in one `Map` resource, every movement system
//! would carry a dependency on tile layout it never reads. Small resources with
//! one job each beat one `GameState` god-object every time.

use bevy::prelude::*;
use std::collections::HashSet;

use crate::balance::{TILE_SIZE, Z_TILE};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // Parsing happens once at startup and panics loudly on a broken map. A
    // malformed level is a programmer error, not a runtime condition to
    // recover from — fail at the earliest possible moment, with a message.
    let (grid, river) = parse(MAP).expect("map is malformed");
    app.insert_resource(grid)
        .insert_resource(river)
        .add_systems(OnEnter(Screen::Gameplay), spawn_tiles);
}

/// `S` sea (enemy spawn) · `~` river · `.` buildable land · `B` the bunker.
///
/// An ASCII level is the highest-value-per-line data format there is: you can
/// see the map while editing it, and a typo is visible rather than subtle.
const MAP: &str = "\
S~~~~.......
....~.......
....~~~~~~..
.........~..
..~~~~~~~~..
..~.........
..~~~~~~....
.......~....
.......~....
.......~~~..
.........~..
.........B..";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Land,
    River,
    Bunker,
}

impl Tile {
    /// Everything a boat or a u-boat can travel through.
    pub fn is_navigable(self) -> bool {
        matches!(self, Tile::River | Tile::Bunker)
    }
}

#[derive(Resource)]
pub struct MapGrid {
    pub width: i32,
    pub height: i32,
    tiles: Vec<Tile>,
}

impl MapGrid {
    pub fn tile(&self, cell: IVec2) -> Option<Tile> {
        if cell.x < 0 || cell.y < 0 || cell.x >= self.width || cell.y >= self.height {
            return None;
        }
        Some(self.tiles[(cell.y * self.width + cell.x) as usize])
    }

    /// Centre of a cell, in world space.
    ///
    /// The *only* place this conversion is written. Every "my sprite is half a
    /// tile off" bug in a grid game comes from this maths existing in three
    /// slightly different versions.
    pub fn grid_to_world(&self, cell: IVec2) -> Vec2 {
        Vec2::new(
            (cell.x as f32 - (self.width - 1) as f32 * 0.5) * TILE_SIZE,
            // Screen-space Y grows upward, map rows grow downward. Negate once,
            // here, and never think about it again.
            -(cell.y as f32 - (self.height - 1) as f32 * 0.5) * TILE_SIZE,
        )
    }

    pub fn world_to_grid(&self, world: Vec2) -> IVec2 {
        IVec2::new(
            (world.x / TILE_SIZE + (self.width - 1) as f32 * 0.5).round() as i32,
            (-world.y / TILE_SIZE + (self.height - 1) as f32 * 0.5).round() as i32,
        )
    }
}

/// Ordered river cells and their world positions, sea first, bunker last.
///
/// Enemies walk it forwards, u-boats walk it backwards. Storing both
/// representations costs a few hundred bytes and saves a conversion in two hot
/// loops.
#[derive(Resource)]
pub struct RiverPath {
    pub cells: Vec<IVec2>,
    pub points: Vec<Vec2>,
}

impl RiverPath {
    pub fn spawn_point(&self) -> Vec2 {
        self.points[0]
    }

    pub fn bunker_point(&self) -> Vec2 {
        self.points[self.points.len() - 1]
    }

    /// Index of the river cell a world position sits on, if any.
    pub fn index_of(&self, cell: IVec2) -> Option<usize> {
        self.cells.iter().position(|c| *c == cell)
    }
}

/// Parses the ASCII map and walks the river from `S` to `B`.
///
/// The walk is a deliberate design constraint: the river must be a single
/// unbranched 4-connected channel. That makes pathfinding unnecessary — no A*,
/// no navmesh, no flow field, just a `Vec` of waypoints. When a rule about
/// your *content* removes an entire subsystem from your *code*, take it.
fn parse(source: &str) -> Result<(MapGrid, RiverPath), String> {
    let rows: Vec<&str> = source.lines().collect();
    let height = rows.len() as i32;
    let width = rows[0].len() as i32;

    let mut tiles = Vec::with_capacity((width * height) as usize);
    let mut start = None;
    let mut bunker = None;

    for (y, row) in rows.iter().enumerate() {
        if row.len() as i32 != width {
            return Err(format!("row {y} has width {}, expected {width}", row.len()));
        }
        for (x, ch) in row.chars().enumerate() {
            let cell = IVec2::new(x as i32, y as i32);
            tiles.push(match ch {
                '.' => Tile::Land,
                '~' => Tile::River,
                'S' => {
                    start = Some(cell);
                    Tile::River
                }
                'B' => {
                    bunker = Some(cell);
                    Tile::Bunker
                }
                other => return Err(format!("unknown tile '{other}' at {cell}")),
            });
        }
    }

    let grid = MapGrid { width, height, tiles };
    let start = start.ok_or("map has no 'S' spawn")?;
    let bunker = bunker.ok_or("map has no 'B' bunker")?;

    // Walk: from each cell, step to the one navigable neighbour we have not
    // visited. Exactly one must exist, or the river is branched/ambiguous and
    // the level is broken.
    let mut cells = vec![start];
    let mut visited = HashSet::new();
    visited.insert(start);

    while *cells.last().unwrap() != bunker {
        let current = *cells.last().unwrap();
        let mut next = None;
        for step in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
            let candidate = current + step;
            let navigable = grid.tile(candidate).is_some_and(Tile::is_navigable);
            if navigable && !visited.contains(&candidate) {
                if next.is_some() {
                    return Err(format!("river branches at {current}"));
                }
                next = Some(candidate);
            }
        }
        let next = next.ok_or_else(|| format!("river dead-ends at {current}"))?;
        visited.insert(next);
        cells.push(next);
    }

    let points = cells.iter().map(|c| grid.grid_to_world(*c)).collect();
    Ok((grid, RiverPath { cells, points }))
}

/// One sprite per tile. 144 sprites is nothing; a tilemap crate would be
/// premature optimisation with a migration cost attached.
fn spawn_tiles(mut commands: Commands, grid: Res<MapGrid>) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = IVec2::new(x, y);
            let Some(tile) = grid.tile(cell) else { continue };
            let color = match tile {
                Tile::Land => Color::srgb(0.16, 0.20, 0.15),
                Tile::River => Color::srgb(0.10, 0.24, 0.34),
                Tile::Bunker => Color::srgb(0.34, 0.32, 0.30),
            };
            commands.spawn((
                Name::new(format!("Tile {x},{y}")),
                DespawnOnExit(Screen::Gameplay),
                Sprite {
                    color,
                    // 1px gap so the grid reads as a grid.
                    custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_translation(grid.grid_to_world(cell).extend(Z_TILE)),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn river_runs_from_sea_to_bunker() {
        let (grid, river) = parse(MAP).unwrap();

        assert_eq!(grid.tile(river.cells[0]), Some(Tile::River));
        assert_eq!(grid.tile(*river.cells.last().unwrap()), Some(Tile::Bunker));
    }

    /// Every step must be exactly one cell. If this fails, the walk jumped a
    /// gap — and boats would visibly teleport across land.
    #[test]
    fn river_is_contiguous() {
        let (_, river) = parse(MAP).unwrap();

        for pair in river.cells.windows(2) {
            let delta = pair[1] - pair[0];
            assert_eq!(delta.x.abs() + delta.y.abs(), 1, "gap at {:?}", pair[0]);
        }
    }

    #[test]
    fn every_river_tile_is_on_the_path() {
        let (grid, river) = parse(MAP).unwrap();

        let on_path = river.cells.len();
        let navigable = (0..grid.height)
            .flat_map(|y| (0..grid.width).map(move |x| IVec2::new(x, y)))
            .filter(|c| grid.tile(*c).is_some_and(Tile::is_navigable))
            .count();

        // A river tile the walk never reached is a disconnected puddle: it
        // looks like water, boats ignore it, and u-boats can be ordered to it.
        assert_eq!(on_path, navigable);
    }

    #[test]
    fn grid_and_world_round_trip() {
        let (grid, _) = parse(MAP).unwrap();
        let cell = IVec2::new(3, 7);

        assert_eq!(grid.world_to_grid(grid.grid_to_world(cell)), cell);
    }

    /// A level author's typo should fail at startup with a message, not
    /// produce boats that pick a fork at random.
    #[test]
    fn a_branching_river_is_rejected() {
        let branched = "S~~.\n.~~.\n..~B";
        assert!(parse(branched).is_err());
    }
}

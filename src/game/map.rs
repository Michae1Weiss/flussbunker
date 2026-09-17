//! Static world: `MapGrid` (tiles, grid ↔ world maths) and `RiverPath` (waypoints
//! from the sea to the bunker). Parsed once at startup; a malformed map panics.

use bevy::prelude::*;
use std::collections::HashSet;

use crate::balance::{TILE_SIZE, Z_TILE};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    let (grid, river) = parse(MAP).expect("map is malformed");
    app.insert_resource(grid)
        .insert_resource(river)
        .add_systems(OnEnter(Screen::Gameplay), spawn_tiles);
}

/// `S` sea spawn, `~` river, `.` land, `B` bunker.
/// The river must be one unbranched, 4-connected channel from `S` to `B`.
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
    /// `None` outside the grid.
    pub fn tile(&self, cell: IVec2) -> Option<Tile> {
        if cell.x < 0 || cell.y < 0 || cell.x >= self.width || cell.y >= self.height {
            return None;
        }
        Some(self.tiles[(cell.y * self.width + cell.x) as usize])
    }

    /// Cell centre in world units. Row 0 is the top of the map, so y is negated.
    pub fn grid_to_world(&self, cell: IVec2) -> Vec2 {
        Vec2::new(
            (cell.x as f32 - (self.width - 1) as f32 * 0.5) * TILE_SIZE,
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

/// Sea first, bunker last. Enemies walk it forwards, the U-boat backwards.
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

    /// `None` for cells that are not on the river.
    pub fn index_of(&self, cell: IVec2) -> Option<usize> {
        self.cells.iter().position(|c| *c == cell)
    }
}

/// `Err` for unknown characters, ragged rows, a missing `S` or `B`, or a river
/// that branches or dead-ends.
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

    let grid = MapGrid {
        width,
        height,
        tiles,
    };
    let start = start.ok_or("map has no 'S' spawn")?;
    let bunker = bunker.ok_or("map has no 'B' bunker")?;

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

fn spawn_tiles(mut commands: Commands, grid: Res<MapGrid>) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            let cell = IVec2::new(x, y);
            let Some(tile) = grid.tile(cell) else {
                continue;
            };
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

        assert_eq!(on_path, navigable);
    }

    #[test]
    fn grid_and_world_round_trip() {
        let (grid, _) = parse(MAP).unwrap();
        let cell = IVec2::new(3, 7);

        assert_eq!(grid.world_to_grid(grid.grid_to_world(cell)), cell);
    }

    #[test]
    fn a_branching_river_is_rejected() {
        let branched = "S~~.\n.~~.\n..~B";
        assert!(parse(branched).is_err());
    }
}

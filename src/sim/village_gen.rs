//! Procedural village layout generator.
//!
//! Road topology
//! ─────────────
//!   • One main east–west road (2 tiles wide) at the vertical centre.
//!   • North–south cross streets every BLOCK_W tiles.
//!   • Buildings fill the rectangular plots between cross streets, set back
//!     2 tiles from the main road.
//!
//! Returns: (tiles, actor_spawns, locations)

use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::constants::{MAP_WIDTH, MAP_HEIGHT};
use crate::sim::world::{TileKind, Location, LocationKind};
use crate::sim::blueprint::{self, stamp, BUILDINGS, TAVERN, MARKET, WELL};

/// Spacing between cross streets (in tiles).
const BLOCK_W: usize = 12;
/// Gap between road edge and building face (tiles).
const SETBACK:  usize = 2;
/// Width of the main road (tiles).
const ROAD_W:   usize = 2;

pub struct VillageLayout {
    pub tiles:    Vec<Vec<TileKind>>,
    /// Per-cell visual tileset override (populated by blueprint stamp).
    pub tile_ids: Vec<Vec<Option<(u8, u8)>>>,
    /// Walkable tile coords where actors may be spawned.
    pub spawns:  Vec<(i32, i32)>,
    pub locations: Vec<Location>,
}

pub fn generate(seed: u64) -> VillageLayout {
    let mut rng = StdRng::seed_from_u64(seed);
    let w = MAP_WIDTH;
    let h = MAP_HEIGHT;

    let mut tiles = vec![vec![TileKind::Grass; w]; h];
    let mut tile_ids: Vec<Vec<Option<(u8, u8)>>> = vec![vec![None; w]; h];

    // ── Main E–W road ─────────────────────────────────────────────────────────
    let road_y = h / 2;               // centre row of the road
    for x in 0..w {
        for dy in 0..ROAD_W {
            let ty = road_y + dy;
            if ty < h { tiles[ty][x] = TileKind::Path; }
        }
    }

    // ── N–S cross streets ─────────────────────────────────────────────────────
    let street_xs: Vec<usize> = (0..w).step_by(BLOCK_W).collect();
    let street_reach = 10usize;        // tiles north and south of road

    for &sx in &street_xs {
        for dy in 0..street_reach {
            // north
            let ny = road_y.saturating_sub(dy);
            tiles[ny][sx] = TileKind::Path;
            // south
            let sy = (road_y + ROAD_W + dy).min(h - 1);
            tiles[sy][sx] = TileKind::Path;
        }
    }

    // ── Building blocks ───────────────────────────────────────────────────────
    // Iterate over each block slot between consecutive cross streets.
    let mut locations: Vec<Location> = Vec::new();

    // special placement: tavern at the first interior block, centered
    let tavern_slot = 1usize;

    for (slot, &left_x) in street_xs.iter().enumerate() {
        let right_x = left_x + BLOCK_W;
        if right_x > w { break; }

        // Inner column range (between the two cross streets, with 1-tile margin)
        let inner_x  = left_x + 1;
        let inner_w  = BLOCK_W.saturating_sub(2);  // leave 1 tile on each side

        // ── North block ──────────────────────────────────────────────────────
        let is_tavern = slot == tavern_slot;
        let bp_north: &blueprint::Blueprint = if is_tavern {
            &TAVERN
        } else {
            BUILDINGS[rng.gen_range(0..BUILDINGS.len())]
        };

        // Place building so its south (door) edge is at road_y - SETBACK - 1
        let bp_h   = bp_north.height();
        let bp_w_n = bp_north.width().min(inner_w);
        let north_y = road_y.saturating_sub(SETBACK + bp_h);

        if inner_x + bp_w_n < w && north_y + bp_h < road_y {
            stamp(&mut tiles, &mut tile_ids, bp_north, inner_x, north_y, false,  w, h);

            // path stub from door to road
            let door_col  = inner_x + bp_north.door_col().min(bp_w_n - 1);
            let door_row  = north_y + bp_h;     // row just south of blueprint
            for py in door_row..road_y {
                if py < h { tiles[py][door_col] = TileKind::Path; }
            }

            let lkind = if is_tavern { LocationKind::Pub } else { LocationKind::Home };
            locations.push(Location { kind: lkind, tile_x: door_col as i32, tile_y: (north_y + bp_h / 2) as i32 });
        }

        // ── South block ──────────────────────────────────────────────────────
        let bp_south: &blueprint::Blueprint = if is_tavern {
            &MARKET
        } else {
            BUILDINGS[rng.gen_range(0..BUILDINGS.len())]
        };

        let bp_h_s  = bp_south.height();
        let bp_w_s  = bp_south.width().min(inner_w);
        let south_y = road_y + ROAD_W + SETBACK;

        if inner_x + bp_w_s < w && south_y + bp_h_s < h {
            stamp(&mut tiles, &mut tile_ids, bp_south, inner_x, south_y, true, w, h);

            // path stub from door (now north-facing after flip) to road
            let door_col = inner_x + bp_south.door_col().min(bp_w_s - 1);
            let door_row_end = south_y;
            let road_south = road_y + ROAD_W;
            for py in road_south..door_row_end {
                if py < h { tiles[py][door_col] = TileKind::Path; }
            }

            locations.push(Location { kind: LocationKind::Home, tile_x: door_col as i32, tile_y: (south_y + bp_h_s / 2) as i32 });
        }
    }

    // ── Scatter wells along the road at cross-street intersections ────────────
    for &sx in street_xs.iter().skip(2).step_by(2) {
        let wx = sx.saturating_sub(1);
        let wy = road_y.saturating_sub(3);
        if wx + 3 < w && wy + 3 < road_y {
            stamp(&mut tiles, &mut tile_ids, &WELL, wx, wy, false, w, h);
        }
    }

    // ── Patch: make sure road itself is always walkable Path ─────────────────
    for x in 0..w {
        for dy in 0..ROAD_W {
            let ty = road_y + dy;
            if ty < h { tiles[ty][x] = TileKind::Path; }
        }
    }

    // ── Collect spawn points (road + path tiles) ──────────────────────────────
    let mut spawns = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if tiles[y][x].is_walkable() {
                spawns.push((x as i32, y as i32));
            }
        }
    }

    // Add standard locations if not already present
    locations.push(Location { kind: LocationKind::Common,    tile_x: (w / 2) as i32, tile_y: road_y as i32 });
    locations.push(Location { kind: LocationKind::Workplace, tile_x: (w / 4) as i32, tile_y: road_y as i32 });

    VillageLayout { tiles, tile_ids, spawns, locations }
}

//! Blueprint system for procedural village generation.
//!
//! A Blueprint is a rectangular 2-D stamp of tiles described as rows of chars:
//!   '.' = transparent — don't overwrite the underlying tile
//!   'G' = Grass
//!   'P' = Path  (walkable, road/doorstep colour)
//!   'F' = BuildingFloor
//!   'W' = BuildingWall  (not walkable)
//!   'A' = Bracken       (re-used as tilled farmland / crop patch)
//!   '~' = Water
//!
//! Entry (door) convention: blueprints are defined facing SOUTH (entrance row
//! is the last row).  The village generator flips them when placing on the
//! south side of the road so the door always faces the nearest road.

use crate::sim::world::TileKind;

// ─── Cell ────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Transparent,
    Grass,
    Path,
    Floor,
    Wall,
    Farmland,
    Water,
}

impl Cell {
    /// Map this blueprint cell to a TileKind, or None (transparent).
    pub fn to_tile(self) -> Option<TileKind> {
        match self {
            Cell::Transparent => None,
            Cell::Grass       => Some(TileKind::Grass),
            Cell::Path        => Some(TileKind::Path),
            Cell::Floor       => Some(TileKind::BuildingFloor),
            Cell::Wall        => Some(TileKind::BuildingWall),
            Cell::Farmland    => Some(TileKind::Bracken),
            Cell::Water       => Some(TileKind::Water),
        }
    }

    fn from_char(c: char) -> Self {
        match c {
            '.' => Cell::Transparent,
            'G' => Cell::Grass,
            'P' => Cell::Path,
            'F' => Cell::Floor,
            'W' => Cell::Wall,
            'A' => Cell::Farmland,
            '~' => Cell::Water,
            _   => Cell::Transparent,
        }
    }
}

// ─── Visual-override tile sentinel ────────────────────────────────────────────
/// (255,255) is out-of-range for the 64×64 tileset — used as "no override" sentinel
/// inside vis grids so the default tile_src() mapping is used for that cell.
pub const VIS_NONE: (u8, u8) = (255, 255);

// Shorthand for readability in long vis arrays
const N: (u8, u8) = VIS_NONE;
// Grass tile (1,1) — used at roof corners so they blend with surroundings
const GR: (u8, u8) = (1, 1);

// ─── Blueprint ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Blueprint {
    pub name: &'static str,
    /// Row-major grid.  rows[0] = northernmost row.
    /// Last row must contain exactly one 'P' cell = door facing south.
    pub rows: &'static [&'static str],
    /// Optional per-cell visual override: same dimensions as `rows`.
    /// Each `(col, row)` references a tile in the Sunnyside 16-px tileset.
    /// Use `VIS_NONE` (255,255) to keep the default tile_src() appearance.
    /// An empty slice (`&[]`) means no visual overrides at all.
    pub vis: &'static [&'static [(u8, u8)]],
}

impl Blueprint {
    pub fn width(&self) -> usize {
        self.rows[0].len()
    }

    pub fn height(&self) -> usize {
        self.rows.len()
    }

    /// Returns the cell at (col, row), row 0 = north.
    pub fn cell(&self, col: usize, row: usize) -> Cell {
        self.rows
            .get(row)
            .and_then(|r| r.chars().nth(col))
            .map(Cell::from_char)
            .unwrap_or(Cell::Transparent)
    }

    /// Visual tile override for (col, row).  Returns None when no override.
    pub fn vis_tile(&self, col: usize, row: usize) -> Option<(u8, u8)> {
        let t = *self.vis.get(row)?.get(col)?;
        if t == VIS_NONE { None } else { Some(t) }
    }

    /// Column of the door in the last (south-facing) row.
    pub fn door_col(&self) -> usize {
        let last = self.rows[self.height() - 1];
        last.chars()
            .position(|c| c == 'P')
            .unwrap_or(self.width() / 2)
    }
}

// ─── Blueprint library ────────────────────────────────────────────────────────
//
//  Door (P) is ALWAYS in the bottom row so it faces south toward the road.
//  Building walls are 1 cell thick.  Interiors are F.
//  Width is kept ≤ 10 to fit inside 12-tile-wide block slots with 1-tile margin.
//
//  Visual tile coordinates reference the Sunnyside 16-px tileset (64×64 grid):
//   Med red house:      rows 34-38, cols 32-34 (left=32, fill=33, right=34)
//   Small purple house: rows 41-46, cols 29-30
//

/// Cosy 6-wide cottage (RED tile palette, cols 32-34, rows 34-38 of tileset).
/// Centred in 6 tile slots with grass padding at outer edges.
pub const COTTAGE: Blueprint = Blueprint {
    name: "Cottage",
    rows: &[
        "WWWWWW",   // north wall + chimney zone
        "WWWWWW",   // roof body
        "WFFFFW",   // interior (roof from above)
        "WFFFFW",   // interior (front facade at south)
        "WWPWWW",   // south wall + door at col 2
        "..P...",   // doorstep path
    ],
    vis: &[
        &[GR,(32,34),(33,34),(34,34),GR,GR],   // chimney row
        &[GR,(32,35),(33,35),(34,35),GR,GR],   // main roof
        &[GR,(32,36),(33,36),(34,36),GR,GR],   // lower roof
        &[GR,(32,37),(33,37),(34,37),GR,GR],   // wall + window
        &[GR,(32,38),(33,38),(34,38),GR,GR],   // south eave
        &[N,N,N,N,N,N],                        // doorstep — natural
    ],
};

/// Narrow 6-wide cottage using the small PURPLE house (cols 29-30) centred in 6 tiles.
pub const SMALL_COTTAGE: Blueprint = Blueprint {
    name: "Small Cottage",
    rows: &[
        "..WW..",   // chimney zone (non-walkable, transparent edges)
        "..WW..",   // roof
        "..WW..",   // lower roof
        "..WW..",   // wall + window
        "..PW..",   // south entrance + wall
        "..P...",   // doorstep
    ],
    vis: &[
        &[GR,GR,(29,41),(30,41),GR,GR],    // chimney (purple)
        &[GR,GR,(29,42),(30,42),GR,GR],    // roof
        &[GR,GR,(29,43),(30,43),GR,GR],    // lower roof
        &[GR,GR,(29,44),(30,44),GR,GR],    // wall + window
        &[GR,GR,(29,45),(30,45),GR,GR],    // south eave
        &[N,N,N,N,N,N],                    // doorstep — natural
    ],
};

/// Wider 8-wide house (RED tile palette, cols 32-34, rows 34-38 — left/fill/right extended).
pub const HOUSE_WIDE: Blueprint = Blueprint {
    name: "Wide House",
    rows: &[
        "WWWWWWWW",
        "WWWWWWWW",
        "WFFFFFFW",
        "WFFFFFFW",
        "WWWPWWWW",
        "...P....",
    ],
    vis: &[
        &[GR,(32,34),(33,34),(33,34),(33,34),(33,34),(34,34),GR],  // chimney
        &[GR,(32,35),(33,35),(33,35),(33,35),(33,35),(34,35),GR],  // main roof
        &[GR,(32,36),(33,36),(33,36),(33,36),(33,36),(34,36),GR],  // lower roof
        &[GR,(32,37),(33,37),(33,37),(33,37),(33,37),(34,37),GR],  // wall + windows
        &[GR,(32,38),(33,38),(33,38),(33,38),(33,38),(34,38),GR],  // south eave
        &[N,N,N,N,N,N,N,N],                                        // doorstep
    ],
};

/// Two-room home — front room + back room.
pub const HOUSE_TWO_ROOM: Blueprint = Blueprint {
    name: "Two-Room House",
    rows: &[
        "WWWWWWW",
        "WFFFFW.",
        "WFFFFWW",
        "WFFFFW.",
        "WWWPWW.",
    ],
    vis: &[],
};

/// Large inn / tavern with wide entrance.
pub const TAVERN: Blueprint = Blueprint {
    name: "Tavern",
    rows: &[
        "WWWWWWWWWW",
        "WFFFFFFFFW",
        "WFFFFFFFFW",
        "WFFFFFFFFW",
        "WWWWWWWWWW",
        "...WWPPWW.",
    ],
    vis: &[],
};

/// Blacksmith — L-shaped with forge area.
pub const BLACKSMITH: Blueprint = Blueprint {
    name: "Blacksmith",
    rows: &[
        "WWWWWW..",
        "WFFFFWW.",
        "WFFFFFW.",
        "WFFFFFFW",
        "WWWPWWWW",
    ],
    vis: &[],
};

/// Farm plot — open-air farmland with small barn.
pub const FARM: Blueprint = Blueprint {
    name: "Farm",
    rows: &[
        "AAAAAAAAAA",
        "AAAAAAAAAA",
        "AAAAAAWWWW",
        "AAAAAAWFFW",
        "AAAAAAWFFW",
        "AAAAAAWPWW",
    ],
    vis: &[],
};

/// Small well / town square feature (3×3).
pub const WELL: Blueprint = Blueprint {
    name: "Well",
    rows: &[
        ".~.",
        "~P~",
        ".P.",
    ],
    vis: &[],
};

/// Market stall row — open sheltered area.
pub const MARKET: Blueprint = Blueprint {
    name: "Market",
    rows: &[
        "WWWWWWWWW",
        "WFFFFFFFW",
        ".PPPPPPP.",
    ],
    vis: &[],
};

/// Lookup table for all placeable buildings (not WELL — placed specially).
pub const BUILDINGS: &[&Blueprint] = &[
    &COTTAGE,
    &COTTAGE,         // doubled weight — most common
    &SMALL_COTTAGE,
    &SMALL_COTTAGE,   // doubled weight
    &HOUSE_WIDE,
    &HOUSE_TWO_ROOM,
    &BLACKSMITH,
    &FARM,
];

/// Stamps a blueprint onto a tile grid AND the visual tile-id grid.
/// `origin_x`, `origin_y` = top-left corner in tile coordinates.
/// `flip_v` = true → flip the blueprint vertically (door faces north).
pub fn stamp(
    tiles:    &mut Vec<Vec<TileKind>>,
    tile_ids: &mut Vec<Vec<Option<(u8, u8)>>>,
    bp:       &Blueprint,
    origin_x: usize,
    origin_y: usize,
    flip_v:   bool,
    map_w:    usize,
    map_h:    usize,
) {
    let bh = bp.height();
    let bw = bp.width();
    for row in 0..bh {
        let src_row = if flip_v { bh - 1 - row } else { row };
        for col in 0..bw {
            let tx = origin_x + col;
            let ty = origin_y + row;
            if tx >= map_w || ty >= map_h { continue; }

            // ── logic tile ──
            if let Some(tk) = bp.cell(col, src_row).to_tile() {
                tiles[ty][tx] = tk;
            }

            // ── visual override ──
            if !bp.vis.is_empty() {
                // When flipped, read vis rows in reverse so the house reads correctly.
                let vis_src_row = if flip_v {
                    bp.vis.len().saturating_sub(1 + row.min(bp.vis.len() - 1))
                } else {
                    row
                };
                if let Some(vis_row) = bp.vis.get(vis_src_row) {
                    if let Some(&(vc, vr)) = vis_row.get(col) {
                        if (vc, vr) != VIS_NONE {
                            tile_ids[ty][tx] = Some((vc, vr));
                        }
                    }
                }
            }
        }
    }
}

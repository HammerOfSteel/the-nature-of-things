//! POC-4: Kenney Tiny-Town 16 px tiles (2× → 32 px) + RPG top-down character sprites.
//!
//! Tilemap sheet `tiny_town.png`  (192 × 176, 12 cols × 11 rows of 16 × 16 px, no gap).
//!   tile_index = row * 12 + col   →   pixel (col * 16, row * 16) in sheet.
//!
//! Character sheets `char_m.png` / `char_f.png`  (128 × 256, 4 cols × 4 rows of 32 × 64 px).
//!   Row 0 = face south  Row 1 = face north  Row 2 = face east  Row 3 = face west.
//!   Col 0–3 = walk-cycle frames.

use macroquad::prelude::*;

use crate::constants::{
    MAP_WIDTH, MAP_HEIGHT, TILE_SIZE,
    SCREEN_HEIGHT, VIEWPORT_WIDTH,
};
use crate::sim::actor::Actor;
use crate::sim::world::{SimWorld, TileKind};

// ── constants ──────────────────────────────────────────────────────────────────
/// Screen pixels per tile (2× the 16 px source tiles).
pub const SPRITE_TS: f32 = 32.0;
const SHEET_TILE: f32   = 16.0;

/// Character source frame dimensions — sheet is 4 cols × 8 rows of 32×32 px.
/// Direction row mapping: south=row0 (y=0), west=row2 (y=64), east=row4 (y=128), north=row6 (y=192)
const CHAR_FW: f32  = 32.0;
const CHAR_FH: f32  = 32.0;   // 32 px tall, NOT 64
const CHAR_FPS: f32 = 6.0;    // walk-animation frames per second
const CHAR_FRAMES: u32 = 4;   // 4 walk frames per direction row

// ── texture bundle ─────────────────────────────────────────────────────────────
pub struct SpriteTextures {
    pub tileset: Texture2D,
    pub char_m:  Texture2D,
    pub char_f:  Texture2D,
}

impl SpriteTextures {
    pub async fn load() -> Self {
        let t = load_texture("assets/tiles/tiny_town.png").await
            .expect("assets/tiles/tiny_town.png missing");
        t.set_filter(FilterMode::Nearest);

        let m = load_texture("assets/chars/char_m.png").await
            .expect("assets/chars/char_m.png missing");
        m.set_filter(FilterMode::Nearest);

        let f = load_texture("assets/chars/char_f.png").await
            .expect("assets/chars/char_f.png missing");
        f.set_filter(FilterMode::Nearest);

        SpriteTextures { tileset: t, char_m: m, char_f: f }
    }
}

// ── tile helpers ───────────────────────────────────────────────────────────────

/// Cheap positional hash → [0, 1).
#[inline]
fn th(tx: i32, ty: i32) -> f32 {
    let h = tx.wrapping_mul(2_654_435_761_u32 as i32)
              .wrapping_add(ty.wrapping_mul(2_246_822_519_u32 as i32));
    (h as u32) as f32 / u32::MAX as f32
}

/// Kenney Tiny-Town has no water tiles — return a solid colour for water kinds.
/// Returns `None` for all other kinds (use spritesheet).
fn tile_solid_color(kind: TileKind) -> Option<Color> {
    match kind {
        TileKind::Water     => Some(Color::new(0.38, 0.58, 0.80, 1.0)),
        TileKind::DeepWater => Some(Color::new(0.20, 0.36, 0.65, 1.0)),
        _ => None,
    }
}

/// Returns up to 2 (sheet_col, sheet_row) layers for `kind` at tile (`tx`,`ty`).
/// Layers are composited bottom → top.  Water/DeepWater → use tile_solid_color instead.
///
/// Verified tile layout (from tools/out_tiny_town.png visual inspection):
///   Row 0 cols 0-2  → grass (plain green)
///   Row 0 cols 4-6  → transparent green tree overlays (for Bracken)
///   Row 1 cols 0-2  → warm tan floor (BuildingFloor / interior)
///   Row 2 cols 0-2  → sandy tan floor (Path / exterior)
///   Row 3 cols 0-2  → tan with grass edges (more floor/ground)
///   Row 4 cols 0-2  → grey/blue STONE WALL tiles (Stone terrain)
///   Row 4 cols 4-6  → red terracotta roof tiles (BuildingWall top-down)
///   Row 8 cols 0-2  → lighter grey stone (Mountain)
fn tile_layers(kind: TileKind, tx: i32, ty: i32) -> ([(u32, u32); 2], usize) {
    let h  = th(tx, ty);
    let h2 = th(tx ^ 0x5A, ty ^ 0xA5);

    const GRASS: [(u32, u32); 3] = [(0, 0), (1, 0), (2, 0)];
    let g = GRASS[(h * 3.0) as usize % 3];

    match kind {
        TileKind::Grass => ([g, (0, 0)], 1),
        // Exterior sandy path (row 2, cols 0-2)
        TileKind::Path => {
            const P: [(u32, u32); 3] = [(0, 2), (1, 2), (2, 2)];
            ([P[(h * 3.0) as usize % 3], (0, 0)], 1)
        }
        // Interior floor — row 1 cols 0-1 (warmer/darker tan than path)
        TileKind::BuildingFloor => {
            const F: [(u32, u32); 2] = [(0, 1), (1, 1)];
            ([F[(h * 2.0) as usize % 2], (0, 0)], 1)
        }
        // Red terracotta roof tiles viewed from above (row 4, cols 4-6)
        TileKind::BuildingWall => {
            const W: [(u32, u32); 3] = [(4, 4), (5, 4), (6, 4)];
            ([W[(h * 3.0) as usize % 3], (0, 0)], 1)
        }
        // Water/DeepWater → solid colour, handled in draw_world_sprites; dummy fallback
        TileKind::Water | TileKind::DeepWater => ([g, (0, 0)], 1),
        // Grass base + transparent green tree overlay (row 0, cols 4-6)
        TileKind::Bracken => {
            const OV: [(u32, u32); 3] = [(4, 0), (5, 0), (6, 0)];
            ([g, OV[(h2 * 3.0) as usize % 3]], 2)
        }
        // Grey/blue stone wall tiles (row 4, cols 0-2) — repurposed as stone terrain
        TileKind::Stone => {
            const ST: [(u32, u32); 3] = [(0, 4), (1, 4), (2, 4)];
            ([ST[(h * 3.0) as usize % 3], (0, 0)], 1)
        }
        // Lighter grey stone (row 8, cols 0-2) — mountain outcroppings
        TileKind::Mountain => {
            const MT: [(u32, u32); 3] = [(0, 8), (1, 8), (2, 8)];
            ([MT[(h * 3.0) as usize % 3], (0, 0)], 1)
        }
        // Farmland — reuse bracken base with different overlay
        TileKind::Farmland => ([g, (4, 0)], 2),
        // Fence — reuse stone
        TileKind::Fence => {
            const FN: [(u32, u32); 2] = [(0, 4), (1, 4)];
            ([FN[(h * 2.0) as usize % 2], (0, 0)], 1)
        }
        // Cobble — dark stone
        TileKind::Cobble => {
            const CB: [(u32, u32); 2] = [(0, 8), (2, 8)];
            ([CB[(h * 2.0) as usize % 2], (0, 0)], 1)
        }
    }
}

/// Draw a single tile from the Kenney spritesheet at screen position (`sx`, `sy`).
#[inline]
fn draw_tile_at(tex: &Texture2D, sc: u32, sr: u32, sx: f32, sy: f32, tint: Color) {
    draw_texture_ex(
        tex, sx, sy, tint,
        DrawTextureParams {
            dest_size: Some(vec2(SPRITE_TS, SPRITE_TS)),
            source: Some(Rect::new(
                sc as f32 * SHEET_TILE,
                sr as f32 * SHEET_TILE,
                SHEET_TILE, SHEET_TILE,
            )),
            ..Default::default()
        },
    );
}

// ── main render function ───────────────────────────────────────────────────────

pub fn draw_world_sprites(
    world:    &SimWorld,
    textures: &SpriteTextures,
    selected: Option<usize>,
    cam_x:   f32,
    cam_y:   f32,
) {
    let br   = world.weather.brightness();
    let tint = Color::new(br, br, br, 1.0);

    // ── tile pass ──────────────────────────────────────────────────────────────
    for ty in 0..MAP_HEIGHT {
        for tx in 0..MAP_WIDTH {
            let sx = tx as f32 * SPRITE_TS - cam_x;
            let sy = ty as f32 * SPRITE_TS - cam_y;
            if sx + SPRITE_TS < 0.0 || sx > VIEWPORT_WIDTH  { continue; }
            if sy + SPRITE_TS < 0.0 || sy > SCREEN_HEIGHT   { continue; }

            let kind = world.tiles[ty][tx];
            // Water has no sprite in this pack — fall back to a solid colour.
            if let Some(base_col) = tile_solid_color(kind) {
                let c = Color::new(base_col.r * br, base_col.g * br, base_col.b * br, 1.0);
                draw_rectangle(sx, sy, SPRITE_TS, SPRITE_TS, c);
                continue;
            }
            let (layers, n) = tile_layers(kind, tx as i32, ty as i32);
            for i in 0..n {
                draw_tile_at(&textures.tileset, layers[i].0, layers[i].1, sx, sy, tint);
            }
        }
    }

    // ── actor pass (Y-sorted) ──────────────────────────────────────────────────
    let mut order: Vec<usize> = (0..world.actors.len()).collect();
    order.sort_by_key(|&i| world.actors[i].tile_y);

    let t  = get_time() as f32;
    let nb = if world.clock.is_night() { 0.55_f32 } else { 1.0_f32 };

    for &ai in &order {
        let actor = &world.actors[ai];
        // Rescale from the 24-px grid used by pixel_x/y to SPRITE_TS
        let wx = actor.pixel_x / TILE_SIZE * SPRITE_TS;
        let wy = actor.pixel_y / TILE_SIZE * SPRITE_TS;
        let sx = wx - cam_x + SPRITE_TS * 0.5;
        let sy = wy - cam_y + SPRITE_TS * 0.5;

        if sx < -CHAR_FW      || sx > VIEWPORT_WIDTH + CHAR_FW  { continue; }
        if sy < -CHAR_FH      || sy > SCREEN_HEIGHT  + CHAR_FH  { continue; }

        draw_actor(textures, actor, ai, sx, sy, selected == Some(ai), t, nb);
    }
}

fn draw_actor(
    textures:  &SpriteTextures,
    actor:     &Actor,
    idx:       usize,
    sx:        f32,
    sy:        f32,
    selected:  bool,
    t:         f32,
    brightness: f32,
) {
    let tex   = if idx % 2 == 0 { &textures.char_m } else { &textures.char_f };
    let frame = ((t * CHAR_FPS) as u32 + idx as u32) % CHAR_FRAMES;

    // Sheet row from movement direction (2 sub-rows per direction, use first)
    let dx = actor.target_x - actor.tile_x;
    let dy = actor.target_y - actor.tile_y;
    let dir_y = if dy > 0      { 0.0 }    // south: row 0
                else if dy < 0 { 192.0 }  // north: row 6
                else if dx > 0 { 128.0 }  // east:  row 4
                else if dx < 0 { 64.0 }   // west:  row 2
                else           { 0.0 };   // stationary → south

    let src_x = frame as f32 * CHAR_FW;
    let src_y = dir_y;

    // Selection indicator
    if selected {
        draw_circle_lines(sx, sy + 2.0, 14.0, 2.0, YELLOW);
    }

    // Drop shadow
    draw_circle(sx, sy + 3.0, 5.0, Color::new(0.0, 0.0, 0.0, 0.22 * brightness));

    let tint = Color::new(brightness, brightness, brightness, 1.0);

    // Draw 32×32 sprite centred horizontally, feet at sy
    draw_texture_ex(
        tex,
        sx - CHAR_FW * 0.5,
        sy - CHAR_FH,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(CHAR_FW, CHAR_FH)),
            source:    Some(Rect::new(src_x, src_y, CHAR_FW, CHAR_FH)),
            ..Default::default()
        },
    );

    // Role-colour dot above sprite
    let (cr, cg, cb) = actor.role.color();
    draw_circle(
        sx,
        sy - CHAR_FH - 4.0,
        3.0,
        Color::new(cr, cg, cb, 1.0),
    );
}

// ── camera helpers (used by poc_sprite.rs) ─────────────────────────────────────
pub fn cam_max_x() -> f32 { (MAP_WIDTH  as f32 * SPRITE_TS - VIEWPORT_WIDTH).max(0.0) }
pub fn cam_max_y() -> f32 { (MAP_HEIGHT as f32 * SPRITE_TS - SCREEN_HEIGHT ).max(0.0) }

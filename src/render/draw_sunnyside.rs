//! POC-5: Sunnyside World 16 px tiles (2× → 32 px) + layered animated walk strips.
//!
//! Tileset `sunnyside.png`  (1024 × 1024, 64 cols × 64 rows of 16 × 16 px, no gap).
//!   tile_index = row * 64 + col   →   pixel (col * 16, row * 16) in sheet.
//!
//! Character walk strips `walk_*.png`  (768 × 64, 8 frames of 96 × 64 px each).
//! Character idle strips `idle_*.png`  (864 × 64, 9 frames of 96 × 64 px each).
//!   Base layer (body) + hair overlay layer are drawn composited per actor.
//!
//! Colour analysis reference:
//!   Grass  → (1,1)=65, (2,1)=66, (1,2)=129, (2,2)=130, (0,3)=192, (4,3)=196
//!   Water  → (4,1)=68, (7,2)=135, (8,2)=136
//!   Path   → (5,1)=69, (7,1)=71, (8,1)=72
//!   Floor  → (12,8)=524
//!   Wall   → (1,9)=577  (warm brown – TODO: verify in editor)
//!   Stone  → (19,10)=659
//!   Mtn    → (22,10)=662  (blue-grey – TODO: verify in editor)

use macroquad::prelude::*;

use crate::constants::{
    MAP_WIDTH, MAP_HEIGHT, TILE_SIZE,
    SCREEN_HEIGHT, VIEWPORT_WIDTH,
};
use crate::sim::actor::{Action, Actor};
use crate::sim::world::{SimWorld, TileKind};

// ── constants ──────────────────────────────────────────────────────────────────
pub const SPRITE_TS: f32 = 32.0;      // display pixels per tile (2× the 16 px source)
const SHEET_TILE: f32    = 16.0;

// Character source frame size (in strip pixels)
const CHAR_SRC_W: f32 = 96.0;
const CHAR_SRC_H: f32 = 64.0;
const CHAR_WALK_FRAMES: u32 = 8;
const CHAR_IDLE_FRAMES: u32 = 9;
// Display size: 2/3× of native 96×64 so characters look ~2×1.5 tiles at 32px grid
const CHAR_DST_W: f32 = 64.0;
const CHAR_DST_H: f32 = 48.0;
const CHAR_WALK_FPS: f32 = 8.0;
const CHAR_IDLE_FPS: f32 = 4.0;

// Hair style variants (index maps to walk_*/idle_* filenames)
const HAIR_STYLES: usize = 6;
const HAIR_NAMES: [&str; HAIR_STYLES] = [
    "bowlhair", "curlyhair", "longhair", "mophair", "shorthair", "spikeyhair",
];

// ── texture bundle ─────────────────────────────────────────────────────────────
pub struct SunnyTextures {
    pub tileset:     Texture2D,
    pub walk_base:   Texture2D,
    pub idle_base:   Texture2D,
    pub walk_hair:   [Texture2D; HAIR_STYLES],
    pub idle_hair:   [Texture2D; HAIR_STYLES],
}

async fn load_nn(path: &str) -> Texture2D {
    let t = load_texture(path).await
        .unwrap_or_else(|_| panic!("{path} missing"));
    t.set_filter(FilterMode::Nearest);
    t
}

impl SunnyTextures {
    pub async fn load() -> Self {
        let tileset   = load_nn("assets/tiles/sunnyside.png").await;
        let walk_base = load_nn("assets/chars/walk_base.png").await;
        let idle_base = load_nn("assets/chars/idle_base.png").await;

        // Macro-ish workaround: load each hair variant sequentially.
        let wh0 = load_nn(&format!("assets/chars/walk_{}.png", HAIR_NAMES[0])).await;
        let wh1 = load_nn(&format!("assets/chars/walk_{}.png", HAIR_NAMES[1])).await;
        let wh2 = load_nn(&format!("assets/chars/walk_{}.png", HAIR_NAMES[2])).await;
        let wh3 = load_nn(&format!("assets/chars/walk_{}.png", HAIR_NAMES[3])).await;
        let wh4 = load_nn(&format!("assets/chars/walk_{}.png", HAIR_NAMES[4])).await;
        let wh5 = load_nn(&format!("assets/chars/walk_{}.png", HAIR_NAMES[5])).await;

        let ih0 = load_nn(&format!("assets/chars/idle_{}.png", HAIR_NAMES[0])).await;
        let ih1 = load_nn(&format!("assets/chars/idle_{}.png", HAIR_NAMES[1])).await;
        let ih2 = load_nn(&format!("assets/chars/idle_{}.png", HAIR_NAMES[2])).await;
        let ih3 = load_nn(&format!("assets/chars/idle_{}.png", HAIR_NAMES[3])).await;
        let ih4 = load_nn(&format!("assets/chars/idle_{}.png", HAIR_NAMES[4])).await;
        let ih5 = load_nn(&format!("assets/chars/idle_{}.png", HAIR_NAMES[5])).await;

        SunnyTextures {
            tileset,
            walk_base,
            idle_base,
            walk_hair: [wh0, wh1, wh2, wh3, wh4, wh5],
            idle_hair: [ih0, ih1, ih2, ih3, ih4, ih5],
        }
    }
}

// ── tile helpers ───────────────────────────────────────────────────────────────

#[inline]
fn th(tx: i32, ty: i32) -> f32 {
    let h = tx.wrapping_mul(2_654_435_761_u32 as i32)
              .wrapping_add(ty.wrapping_mul(2_246_822_519_u32 as i32));
    (h as u32) as f32 / u32::MAX as f32
}

/// Returns (sheet_col, sheet_row) for the given TileKind.
/// Tile positions verified from tools/out_sunnyside.png visual inspection.
///
///   (1,1)=65  (2,1)=66  (1-6, 2-3) = grass variants (confirmed green)
///   (4,1)=68            = bright-blue WATER (confirmed)
///   (7,2)=135 (8,2)=136 = deeper water (dark blue)
///   (5,1)=69  (7,1)=71  (8,1)=72   = tan/sandy PATH
///   (3,1)=67  (10-13,3) = brown DIRT (alternate path variant)
///   (1,9)=577 (2,9)=578 = wood plank BuildingFloor (confirmed warm brown)
///   (1,9)=577 (4,9)=580 = wood BuildingWall (best available)
///   (1,15)=961           = grey cobblestone Stone  (TODO: verify col in editor)
///   (2,15)=962           = grey cobblestone Mountain
fn tile_src(kind: TileKind, tx: i32, ty: i32) -> (u32, u32) {
    let h = th(tx, ty);
    match kind {
        TileKind::Grass => {
            const G: [(u32, u32); 6] = [
                (1, 1), (2, 1), (1, 2), (2, 2), (1, 3), (2, 3),
            ];
            G[(h * 6.0) as usize % 6]
        }
        TileKind::Path => {
            // Sandy tan path tiles
            const P: [(u32, u32); 3] = [(5, 1), (7, 1), (8, 1)];
            P[(h * 3.0) as usize % 3]
        }
        // Bright blue water (confirmed from preview)
        TileKind::Water => (4, 1),
        // Deeper blue water
        TileKind::DeepWater => {
            const DW: [(u32, u32); 2] = [(7, 2), (8, 2)];
            DW[(h * 2.0) as usize % 2]
        }
        // Wood plank interior floor (row 9)
        TileKind::BuildingFloor => {
            const F: [(u32, u32); 2] = [(1, 9), (2, 9)];
            F[(h * 2.0) as usize % 2]
        }
        // Wooden wall — best available in this range (TODO: find actual wall tile)
        TileKind::BuildingWall  => {
            const W: [(u32, u32); 2] = [(3, 9), (4, 9)];
            W[(h * 2.0) as usize % 2]
        }
        // Darker grass for bracken / woodland patches
        TileKind::Bracken => {
            const B: [(u32, u32); 4] = [(3, 2), (4, 2), (5, 2), (6, 2)];
            B[(h * 4.0) as usize % 4]
        }
        // Grey cobblestone stone terrain (row 15)
        TileKind::Stone    => {
            const ST: [(u32, u32); 2] = [(1, 15), (2, 15)];
            ST[(h * 2.0) as usize % 2]
        }
        // Slightly different grey for mountain
        TileKind::Mountain => {
            const MT: [(u32, u32); 2] = [(3, 15), (4, 15)];
            MT[(h * 2.0) as usize % 2]
        }
        // Farmland — tilled soil, brownish (row 10 of tileset)
        TileKind::Farmland => {
            const FA: [(u32, u32); 2] = [(0, 10), (1, 10)];
            FA[(h * 2.0) as usize % 2]
        }
        // Fence — reuse narrow wall tiles (row 8 area)
        TileKind::Fence => (5, 9),
        // Cobble — grey courtyard (slightly different from mountain stone)
        TileKind::Cobble => {
            const CO: [(u32, u32); 2] = [(5, 15), (6, 15)];
            CO[(h * 2.0) as usize % 2]
        }
    }
}

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

pub fn draw_world_sunnyside(
    world:    &SimWorld,
    textures: &SunnyTextures,
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

            // Always draw the base terrain tile first so transparent pixels in
            // any visual override show grass/path underneath (no black holes).
            let (base_sc, base_sr) = tile_src(kind, tx as i32, ty as i32);
            draw_tile_at(&textures.tileset, base_sc, base_sr, sx, sy, tint);

            // Draw per-cell visual override on top if set (blueprint house tiles).
            if let Some((vc, vr)) = world.tile_ids[ty][tx] {
                draw_tile_at(&textures.tileset, vc as u32, vr as u32, sx, sy, tint);
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
        let wx = actor.pixel_x / TILE_SIZE * SPRITE_TS;
        let wy = actor.pixel_y / TILE_SIZE * SPRITE_TS;
        let sx = wx - cam_x + SPRITE_TS * 0.5;
        let sy = wy - cam_y + SPRITE_TS * 0.5;

        if sx < -(CHAR_DST_W + 4.0) || sx > VIEWPORT_WIDTH  + CHAR_DST_W { continue; }
        if sy < -(CHAR_DST_H + 4.0) || sy > SCREEN_HEIGHT   + CHAR_DST_H { continue; }

        draw_sunny_actor(textures, actor, ai, sx, sy, selected == Some(ai), t, nb);
    }
}

fn draw_sunny_actor(
    textures:   &SunnyTextures,
    actor:      &Actor,
    idx:        usize,
    sx:         f32,
    sy:         f32,
    selected:   bool,
    t:          f32,
    brightness: f32,
) {
    let is_walking = matches!(&actor.current_action, Action::Walking);
    let hair_idx   = idx % HAIR_STYLES;

    let (base_tex, hair_tex, n_frames, fps) = if is_walking {
        (&textures.walk_base, &textures.walk_hair[hair_idx],
         CHAR_WALK_FRAMES, CHAR_WALK_FPS)
    } else {
        (&textures.idle_base, &textures.idle_hair[hair_idx],
         CHAR_IDLE_FRAMES, CHAR_IDLE_FPS)
    };

    // Stagger frame offsets per actor so they don't all move in sync
    let frame = ((t * fps) as u32 + idx as u32 * 3) % n_frames;
    let src_x = frame as f32 * CHAR_SRC_W;

    if selected {
        draw_circle_lines(sx, sy + 4.0, 18.0, 2.0, YELLOW);
    }

    // Drop shadow
    draw_circle(sx, sy + 6.0, 8.0, Color::new(0.0, 0.0, 0.0, 0.20 * brightness));

    let tint = Color::new(brightness, brightness, brightness, 1.0);

    // Strip faces RIGHT by default; flip when moving left.
    let flip_x = actor.target_x < actor.tile_x;

    let params = DrawTextureParams {
        dest_size: Some(vec2(CHAR_DST_W, CHAR_DST_H)),
        source: Some(Rect::new(src_x, 0.0, CHAR_SRC_W, CHAR_SRC_H)),
        flip_x,
        ..Default::default()
    };

    // Feet centred on (sx, sy); sprite extends upward
    let draw_x = sx - CHAR_DST_W * 0.5;
    let draw_y = sy - CHAR_DST_H * 0.8;

    // Base body layer
    draw_texture_ex(base_tex, draw_x, draw_y, tint, params.clone());
    // Hair overlay layer
    draw_texture_ex(hair_tex, draw_x, draw_y, tint, params);

    // Role colour dot above sprite
    let (cr, cg, cb) = actor.role.color();
    draw_circle(
        sx + CHAR_DST_W * 0.5 - 4.0,
        draw_y - 4.0,
        3.0,
        Color::new(cr, cg, cb, 1.0),
    );
}

// ── camera helpers ─────────────────────────────────────────────────────────────
pub fn cam_max_x() -> f32 { (MAP_WIDTH  as f32 * SPRITE_TS - VIEWPORT_WIDTH).max(0.0) }
pub fn cam_max_y() -> f32 { (MAP_HEIGHT as f32 * SPRITE_TS - SCREEN_HEIGHT ).max(0.0) }

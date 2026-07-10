//! Little Dreamyland pack renderer — 16 px tiles × 3 = 48 px display.
//!
//! Tileset layout (confirmed from visual inspection):
//!   ground.png  336×176  (21×11 tiles at 16 px)
//!     Grass  : col 3  row 0  (solid flat green)
//!     Path   : col 16 row 1  (tan/dirt)
//!     Water  : col 13 row 1  (blue — rgb≈89,168,210)
//!     Darker : col  5 row 0  (shaded grass / bracken)
//!
//!   house_ts.png  608×304
//!     Red house front-view  : src rect (0, 0, 48, 80)   — 3×5 source tiles
//!     Green house front-view: src rect (0,160, 48, 64)  — 3×4 source tiles
//!
//!   nature.png  336×192
//!     Single round tree : src rect (0, 0, 32, 48)
//!     Double tree cluster: src rect (96, 0, 64, 64)
//!
//!   bunny_run.png  384×192  — 8 frames × 4 rows, 48×48 per frame
//!     Row 0 = South (toward viewer), Row 1 = West, Row 2 = East, Row 3 = North

use macroquad::prelude::*;
use crate::constants::{MAP_WIDTH, MAP_HEIGHT, TILE_SIZE, SCREEN_HEIGHT, VIEWPORT_WIDTH};
use crate::sim::actor::{Actor, Action};
use crate::sim::world::{SimWorld, TileKind};

// ── constants ─────────────────────────────────────────────────────────────────
pub const TS: f32 = 48.0;   // display tile size (16 source × 3)
const ST: f32    = 16.0;    // source tile size

// Bunny
const BUN_SW: f32    = 48.0;   // source frame width
const BUN_SH: f32    = 48.0;   // source frame height
const BUN_FRAMES: u32 = 8;
const BUN_DST: f32   = 52.0;   // display size (slightly larger than 1 tile)

// House display sizes (3× source)
const H_RED_SW:   f32 = 48.0;  // red house source w
const H_RED_SH:   f32 = 80.0;  // red house source h
const H_GREEN_SW: f32 = 48.0;
const H_GREEN_SH: f32 = 64.0;

// Tree display (2× source)
const TREE_SW: f32 = 32.0;
const TREE_SH: f32 = 48.0;
const TREE_SCALE: f32 = 2.5;

// ── texture bundle ─────────────────────────────────────────────────────────────
pub struct DreamyTextures {
    pub ground:   Texture2D,
    pub house_ts: Texture2D,
    pub nature:   Texture2D,
    pub bunny:    Texture2D,
}

async fn load_nn(path: &str) -> Texture2D {
    let t = load_texture(path).await.unwrap_or_else(|e| panic!("Cannot load {path}: {e}"));
    t.set_filter(FilterMode::Nearest);
    t
}

impl DreamyTextures {
    pub async fn load() -> Self {
        DreamyTextures {
            ground:   load_nn("assets/dreamyland/ground.png").await,
            house_ts: load_nn("assets/dreamyland/house_ts.png").await,
            nature:   load_nn("assets/dreamyland/nature.png").await,
            bunny:    load_nn("assets/dreamyland/bunny_run.png").await,
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

#[inline]
fn th(tx: i32, ty: i32) -> f32 {
    let h = (tx as u32).wrapping_mul(374_761_393)
        .wrapping_add((ty as u32).wrapping_mul(1_103_515_245));
    (h >> 8) as f32 / (u32::MAX >> 8) as f32
}

fn draw_tile(tex: &Texture2D, sc: u32, sr: u32, sx: f32, sy: f32, tint: Color) {
    draw_texture_ex(tex, sx, sy, tint, DrawTextureParams {
        dest_size: Some(vec2(TS, TS)),
        source:    Some(Rect::new(sc as f32 * ST, sr as f32 * ST, ST, ST)),
        ..Default::default()
    });
}

fn draw_sprite(tex: &Texture2D, src: Rect, dx: f32, dy: f32, dw: f32, dh: f32,
               tint: Color, flip_x: bool) {
    draw_texture_ex(tex, dx, dy, tint, DrawTextureParams {
        dest_size: Some(vec2(dw, dh)),
        source:    Some(src),
        flip_x,
        ..Default::default()
    });
}

/// Ground tile (col, row) for the given TileKind.
fn ground_src(kind: TileKind, tx: i32, ty: i32) -> (u32, u32) {
    let h = th(tx, ty);
    match kind {
        TileKind::Grass => {
            const G: [(u32,u32); 4] = [(3,0),(2,0),(4,0),(3,0)];
            G[(h * 4.0) as usize % 4]
        }
        TileKind::Path | TileKind::BuildingFloor => (16, 1),
        TileKind::Water | TileKind::DeepWater    => (13, 1),
        TileKind::Bracken                        => (5, 0),
        _                                        => (3, 0),
    }
}

fn bunny_dir(actor: &Actor) -> u32 {
    let dx = actor.target_x - actor.tile_x;
    let dy = actor.target_y - actor.tile_y;
    if dx.abs() >= dy.abs() {
        if dx > 0 { 2 } else { 1 }   // East=2, West=1
    } else {
        if dy > 0 { 0 } else { 3 }   // South=0, North=3
    }
}

// ── draw items (Y-sorted) ─────────────────────────────────────────────────────

enum DItem {
    Building { tx: usize, ty: usize, width: usize, green: bool },
    Tree     { tx: usize, ty: usize },
    Actor    { idx: usize },
}

struct DrawEntry {
    sort_y: f32,
    item:   DItem,
}

// ── main renderer ─────────────────────────────────────────────────────────────

pub fn draw_world_dreamyland(
    world:    &SimWorld,
    textures: &DreamyTextures,
    selected: Option<usize>,
    cam_x:   f32,
    cam_y:   f32,
) {
    let br   = world.weather.brightness();
    let tint = Color::new(br, br, br, 1.0);
    let t    = get_time() as f32;

    // ── Pass 1: ground tiles ─────────────────────────────────────────────────
    for ty in 0..MAP_HEIGHT {
        for tx in 0..MAP_WIDTH {
            let sx = tx as f32 * TS - cam_x;
            let sy = ty as f32 * TS - cam_y;
            if sx + TS < 0.0 || sx > VIEWPORT_WIDTH { continue; }
            if sy + TS < 0.0 || sy > SCREEN_HEIGHT  { continue; }
            let kind = world.tiles[ty][tx];
            let (sc, sr) = ground_src(kind, tx as i32, ty as i32);
            draw_tile(&textures.ground, sc, sr, sx, sy, tint);
        }
    }

    // ── Collect elevated items for Y-sort ────────────────────────────────────
    let mut entries: Vec<DrawEntry> = Vec::new();
    let mut used = vec![[false; MAP_WIDTH]; MAP_HEIGHT];

    for ty in 0..MAP_HEIGHT {
        for tx in 0..MAP_WIDTH {
            match world.tiles[ty][tx] {
                TileKind::BuildingWall => {
                    if used[ty][tx] { continue; }
                    // Only the front row of a building (nothing walkable below)
                    let below = if ty + 1 < MAP_HEIGHT { world.tiles[ty+1][tx] } else { TileKind::Grass };
                    let is_front = !matches!(below, TileKind::BuildingWall | TileKind::BuildingFloor);
                    if !is_front { continue; }
                    // Only the left edge of a horizontal run
                    let left = if tx > 0 { world.tiles[ty][tx-1] } else { TileKind::Grass };
                    if matches!(left, TileKind::BuildingWall) { continue; }
                    // Measure run width
                    let mut width = 1;
                    while tx + width < MAP_WIDTH &&
                          matches!(world.tiles[ty][tx+width], TileKind::BuildingWall) {
                        width += 1;
                    }
                    for i in 0..width { used[ty][tx+i] = true; }
                    // Alternate red/green by position
                    let green = ((tx + ty) / 3) % 2 == 1;
                    entries.push(DrawEntry {
                        sort_y: ty as f32,
                        item: DItem::Building { tx, ty, width, green },
                    });
                }
                TileKind::Bracken => {
                    entries.push(DrawEntry {
                        sort_y: ty as f32,
                        item: DItem::Tree { tx, ty },
                    });
                }
                _ => {}
            }
        }
    }

    // Actors
    for (idx, actor) in world.actors.iter().enumerate() {
        entries.push(DrawEntry {
            sort_y: actor.pixel_y / TILE_SIZE,
            item: DItem::Actor { idx },
        });
    }

    entries.sort_by(|a, b| a.sort_y.partial_cmp(&b.sort_y).unwrap_or(std::cmp::Ordering::Equal));

    // ── Pass 2: draw Y-sorted items ──────────────────────────────────────────
    for entry in &entries {
        match &entry.item {

            DItem::Building { tx, ty, width, green } => {
                let sx = *tx as f32 * TS - cam_x;
                let sy = *ty as f32 * TS - cam_y;
                let build_w = *width as f32 * TS;

                // Scale house sprite to cover the building's full width
                let (src_w, src_h, src_y_off) = if *green {
                    (H_GREEN_SW, H_GREEN_SH, 160.0_f32)
                } else {
                    (H_RED_SW, H_RED_SH, 0.0_f32)
                };
                let scale    = build_w / src_w;
                let dst_h    = src_h * scale;
                // Anchor: sprite base aligns with bottom of front tile
                let dst_y    = sy + TS - dst_h;

                if sy + TS > 0.0 && dst_y < SCREEN_HEIGHT && sx < VIEWPORT_WIDTH && sx + build_w > 0.0 {
                    draw_sprite(&textures.house_ts,
                        Rect::new(0.0, src_y_off, src_w, src_h),
                        sx, dst_y, build_w, dst_h, tint, false);
                }
            }

            DItem::Tree { tx, ty } => {
                let sx = *tx as f32 * TS - cam_x;
                let sy = *ty as f32 * TS - cam_y;
                let dw = TREE_SW * TREE_SCALE;
                let dh = TREE_SH * TREE_SCALE;
                let dx = sx + (TS - dw) * 0.5;
                let dy = sy + TS - dh;
                if sy + TS > 0.0 && dy < SCREEN_HEIGHT && sx < VIEWPORT_WIDTH && sx + TS > 0.0 {
                    draw_sprite(&textures.nature,
                        Rect::new(0.0, 0.0, TREE_SW, TREE_SH),
                        dx, dy, dw, dh, tint, false);
                }
            }

            DItem::Actor { idx } => {
                let actor = &world.actors[*idx];
                let wx = actor.pixel_x / TILE_SIZE * TS;
                let wy = actor.pixel_y / TILE_SIZE * TS;
                let sx = wx - cam_x + TS * 0.5;
                let sy = wy - cam_y + TS * 0.5;
                if sx < -(BUN_DST + 4.0) || sx > VIEWPORT_WIDTH + BUN_DST { continue; }
                if sy < -(BUN_DST + 4.0) || sy > SCREEN_HEIGHT + BUN_DST  { continue; }
                draw_bunny(textures, actor, *idx, sx, sy, selected == Some(*idx), t, br);
            }
        }
    }
}

fn draw_bunny(
    textures: &DreamyTextures,
    actor:    &Actor,
    idx:      usize,
    sx: f32, sy: f32,
    selected: bool,
    t: f32,
    br: f32,
) {
    let is_walking = matches!(actor.current_action, Action::Walking);
    let fps        = if is_walking { 8.0 } else { 2.0 };
    let frame      = ((t * fps) as u32 + idx as u32 * 3) % BUN_FRAMES;
    let dir_row    = if is_walking { bunny_dir(actor) } else { 0 };

    if selected {
        draw_circle_lines(sx, sy, BUN_DST * 0.6, 2.0, YELLOW);
    }
    // Shadow
    draw_circle(sx, sy + BUN_DST * 0.3, BUN_DST * 0.25,
        Color::new(0.0, 0.0, 0.0, 0.18 * br));

    let tint = Color::new(br, br, br, 1.0);
    draw_texture_ex(
        &textures.bunny,
        sx - BUN_DST * 0.5,
        sy - BUN_DST * 0.8,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(BUN_DST, BUN_DST)),
            source:    Some(Rect::new(
                frame as f32 * BUN_SW,
                dir_row as f32 * BUN_SH,
                BUN_SW, BUN_SH,
            )),
            ..Default::default()
        },
    );

    // Role colour dot
    let (cr, cg, cb) = actor.role.color();
    draw_circle(
        sx + BUN_DST * 0.35,
        sy - BUN_DST * 0.85,
        2.5,
        Color::new(cr, cg, cb, 1.0),
    );
}

// ── camera helpers ─────────────────────────────────────────────────────────────
pub fn cam_max_x() -> f32 { (MAP_WIDTH  as f32 * TS - VIEWPORT_WIDTH).max(0.0) }
pub fn cam_max_y() -> f32 { (MAP_HEIGHT as f32 * TS - SCREEN_HEIGHT ).max(0.0) }

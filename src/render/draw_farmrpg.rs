//! Farm RPG 16×16 Tiny pack renderer — 16 px tiles × 3 = 48 px display.
//!
//! Asset layout (confirmed):
//!   tileset.png  192×320  spring tileset (12×20 at 16 px)
//!     Grass interior : col 8, row 0
//!     Path/road      : col 2, row 1  (tan-ish)
//!     Water          : col 4, row 12 (blue)  — approximate, verify visually
//!     Stone          : col 1, row 15
//!
//!   house.png   224×112  two assembled house sprites:
//!     Barn (left)   : crop(0,   0, 80, 112)  — large barn
//!     Cottage(right): crop(128, 0, 96, 112)  — smaller cottage
//!
//!   tree.png    160×48  5 maple tree variants at 32×48 each
//!
//!   walk.png    192×96  — 12 frames × 4 rows at 16×24
//!     Row 0 = South, Row 1 = North, Row 2 = West, Row 3 = East
//!
//!   chicken.png 64×32   — 2 frames at 32×32
//!   cow.png     128×96  — 2 frames × 2 rows at 64×48

use macroquad::prelude::*;
use crate::constants::{MAP_WIDTH, MAP_HEIGHT, TILE_SIZE, SCREEN_HEIGHT, VIEWPORT_WIDTH};
use crate::sim::actor::{Actor, Action};
use crate::sim::world::{SimWorld, TileKind};

pub const TS: f32 = 48.0;
const ST: f32    = 16.0;

// Walk sheet
const WALK_SW:     f32 = 16.0;
const WALK_SH:     f32 = 24.0;
const WALK_FRAMES: u32 = 12;
const WALK_SCALE:  f32 = 3.0; // 16→48, 24→72
const WALK_DST_W:  f32 = WALK_SW * WALK_SCALE;
const WALK_DST_H:  f32 = WALK_SH * WALK_SCALE;

// House
const BARN_SX:  f32 = 0.0;
const BARN_SW:  f32 = 80.0;
const BARN_SH:  f32 = 112.0;
const COTT_SX:  f32 = 128.0;
const COTT_SW:  f32 = 96.0;
const COTT_SH:  f32 = 112.0;

// Tree: each variant is 32×48 in source
const TREE_SW: f32  = 32.0;
const TREE_SH: f32  = 48.0;
const TREE_SCALE: f32 = 2.5;

// Animals
const CHK_SW: f32 = 32.0;
const CHK_SH: f32 = 32.0;
const COW_SW: f32 = 64.0;
const COW_SH: f32 = 48.0;

// ── texture bundle ─────────────────────────────────────────────────────────────
pub struct FarmTextures {
    pub tileset: Texture2D,
    pub house:   Texture2D,
    pub tree:    Texture2D,
    pub chicken: Texture2D,
    pub cow:     Texture2D,
    pub walk:    Texture2D,
}

async fn load_nn(path: &str) -> Texture2D {
    let t = load_texture(path).await.unwrap_or_else(|e| panic!("Cannot load {path}: {e}"));
    t.set_filter(FilterMode::Nearest);
    t
}

impl FarmTextures {
    pub async fn load() -> Self {
        FarmTextures {
            tileset: load_nn("assets/farmrpg/tileset.png").await,
            house:   load_nn("assets/farmrpg/house.png").await,
            tree:    load_nn("assets/farmrpg/tree.png").await,
            chicken: load_nn("assets/farmrpg/chicken.png").await,
            cow:     load_nn("assets/farmrpg/cow.png").await,
            walk:    load_nn("assets/farmrpg/walk.png").await,
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

fn tileset_src(kind: TileKind, tx: i32, ty: i32) -> (u32, u32) {
    let h = th(tx, ty);
    match kind {
        TileKind::Grass => {
            const G: [(u32,u32); 4] = [(8,0),(9,0),(8,1),(9,1)];
            G[(h * 4.0) as usize % 4]
        }
        TileKind::Path | TileKind::BuildingFloor => (2, 1),
        TileKind::Water                          => (4, 12),
        TileKind::DeepWater                      => (3, 12),
        TileKind::Bracken                        => (6, 4),  // leafy tile
        TileKind::Stone                          => (1, 15),
        TileKind::Mountain                       => (2, 15),
        TileKind::BuildingWall                   => (8, 0),  // reuse grass under roof
        _                                        => (8, 0),
    }
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

fn walk_dir(actor: &Actor) -> u32 {
    let dx = actor.target_x - actor.tile_x;
    let dy = actor.target_y - actor.tile_y;
    if dx.abs() >= dy.abs() {
        if dx > 0 { 3 } else { 2 }  // East=3, West=2
    } else {
        if dy > 0 { 0 } else { 1 }  // South=0, North=1
    }
}

// ── draw items ────────────────────────────────────────────────────────────────

enum FItem {
    Building { tx: usize, ty: usize, width: usize, is_barn: bool },
    Tree     { tx: usize, ty: usize, variant: u32 },
    Animal   { tx: usize, ty: usize, is_cow: bool },
    Actor    { idx: usize },
}
struct FEntry { sort_y: f32, item: FItem }

// ── main renderer ─────────────────────────────────────────────────────────────

pub fn draw_world_farmrpg(
    world:    &SimWorld,
    textures: &FarmTextures,
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
            let (sc, sr) = tileset_src(kind, tx as i32, ty as i32);
            draw_tile(&textures.tileset, sc, sr, sx, sy, tint);
        }
    }

    // ── Collect elevated + actors ────────────────────────────────────────────
    let mut entries: Vec<FEntry> = Vec::new();
    let mut used = vec![[false; MAP_WIDTH]; MAP_HEIGHT];

    for ty in 0..MAP_HEIGHT {
        for tx in 0..MAP_WIDTH {
            let h = th(tx as i32, ty as i32);
            match world.tiles[ty][tx] {
                TileKind::BuildingWall => {
                    if used[ty][tx] { continue; }
                    let below = if ty+1 < MAP_HEIGHT { world.tiles[ty+1][tx] } else { TileKind::Grass };
                    let is_front = !matches!(below, TileKind::BuildingWall | TileKind::BuildingFloor);
                    if !is_front { continue; }
                    let left = if tx > 0 { world.tiles[ty][tx-1] } else { TileKind::Grass };
                    if matches!(left, TileKind::BuildingWall) { continue; }
                    let mut width = 1;
                    while tx + width < MAP_WIDTH &&
                          matches!(world.tiles[ty][tx+width], TileKind::BuildingWall) {
                        width += 1;
                    }
                    for i in 0..width { used[ty][tx+i] = true; }
                    let is_barn = (tx + ty) % 3 != 0;
                    entries.push(FEntry { sort_y: ty as f32, item: FItem::Building { tx, ty, width, is_barn } });
                }
                TileKind::Bracken => {
                    let variant = (h * 5.0) as u32 % 5;
                    entries.push(FEntry { sort_y: ty as f32, item: FItem::Tree { tx, ty, variant } });
                    // Scatter farm animals near trees
                    if h > 0.75 {
                        let is_cow = h > 0.88;
                        entries.push(FEntry {
                            sort_y: ty as f32 + 0.5,
                            item: FItem::Animal { tx, ty, is_cow },
                        });
                    }
                }
                _ => {}
            }
        }
    }

    for (idx, actor) in world.actors.iter().enumerate() {
        entries.push(FEntry {
            sort_y: actor.pixel_y / TILE_SIZE,
            item: FItem::Actor { idx },
        });
    }

    entries.sort_by(|a, b| a.sort_y.partial_cmp(&b.sort_y).unwrap_or(std::cmp::Ordering::Equal));

    // ── Pass 2: draw Y-sorted ────────────────────────────────────────────────
    for entry in &entries {
        match &entry.item {

            FItem::Building { tx, ty, width, is_barn } => {
                let sx = *tx as f32 * TS - cam_x;
                let sy = *ty as f32 * TS - cam_y;
                let (s_x, s_w, s_h) = if *is_barn {
                    (BARN_SX, BARN_SW, BARN_SH)
                } else {
                    (COTT_SX, COTT_SW, COTT_SH)
                };
                let build_w = *width as f32 * TS;
                let scale   = build_w / s_w;
                let dst_h   = s_h * scale;
                let dst_y   = sy + TS - dst_h;
                if sy + TS > 0.0 && dst_y < SCREEN_HEIGHT && sx < VIEWPORT_WIDTH && sx + build_w > 0.0 {
                    draw_sprite(&textures.house,
                        Rect::new(s_x, 0.0, s_w, s_h),
                        sx, dst_y, build_w, dst_h, tint, false);
                }
            }

            FItem::Tree { tx, ty, variant } => {
                let sx = *tx as f32 * TS - cam_x;
                let sy = *ty as f32 * TS - cam_y;
                let dw = TREE_SW * TREE_SCALE;
                let dh = TREE_SH * TREE_SCALE;
                let dx = sx + (TS - dw) * 0.5;
                let dy = sy + TS - dh;
                if dy < SCREEN_HEIGHT && dx + dw > 0.0 && dx < VIEWPORT_WIDTH {
                    draw_sprite(&textures.tree,
                        Rect::new(*variant as f32 * TREE_SW, 0.0, TREE_SW, TREE_SH),
                        dx, dy, dw, dh, tint, false);
                }
            }

            FItem::Animal { tx, ty, is_cow } => {
                let sx = *tx as f32 * TS - cam_x + TS * 0.1;
                let sy = *ty as f32 * TS - cam_y + TS * 0.25;
                let frame = ((get_time() as f32 * 1.5 + th(*tx as i32, *ty as i32) * 5.0) as u32) % 2;
                if *is_cow {
                    draw_sprite(&textures.cow,
                        Rect::new(frame as f32 * COW_SW, 0.0, COW_SW, COW_SH),
                        sx, sy, TS * 1.2, TS * 0.9, tint, false);
                } else {
                    draw_sprite(&textures.chicken,
                        Rect::new(frame as f32 * CHK_SW, 0.0, CHK_SW, CHK_SH),
                        sx, sy, TS * 0.65, TS * 0.65, tint, false);
                }
            }

            FItem::Actor { idx } => {
                let actor = &world.actors[*idx];
                let wx = actor.pixel_x / TILE_SIZE * TS;
                let wy = actor.pixel_y / TILE_SIZE * TS;
                let sx = wx - cam_x + TS * 0.5;
                let sy = wy - cam_y + TS * 0.5;
                if sx < -(WALK_DST_W + 4.0) || sx > VIEWPORT_WIDTH + WALK_DST_W { continue; }
                if sy < -(WALK_DST_H + 4.0) || sy > SCREEN_HEIGHT + WALK_DST_H  { continue; }
                draw_farmer(textures, actor, *idx, sx, sy,
                    selected == Some(*idx), t, br);
            }
        }
    }
}

fn draw_farmer(
    textures: &FarmTextures,
    actor:    &Actor,
    idx:      usize,
    sx: f32, sy: f32,
    selected: bool,
    t: f32,
    br: f32,
) {
    let is_walking = matches!(actor.current_action, Action::Walking);
    let fps        = if is_walking { 8.0 } else { 2.0 };
    let frame      = ((t * fps) as u32 + idx as u32 * 3) % WALK_FRAMES;
    let dir_row    = if is_walking { walk_dir(actor) } else { 0 };

    if selected {
        draw_circle_lines(sx, sy, WALK_DST_W * 0.65, 2.0, YELLOW);
    }
    draw_circle(sx, sy + WALK_DST_H * 0.2, WALK_DST_W * 0.32,
        Color::new(0.0, 0.0, 0.0, 0.18 * br));

    let tint = Color::new(br, br, br, 1.0);
    draw_texture_ex(
        &textures.walk,
        sx - WALK_DST_W * 0.5,
        sy - WALK_DST_H * 0.8,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(WALK_DST_W, WALK_DST_H)),
            source: Some(Rect::new(
                frame   as f32 * WALK_SW,
                dir_row as f32 * WALK_SH,
                WALK_SW, WALK_SH,
            )),
            ..Default::default()
        },
    );

    let (cr, cg, cb) = actor.role.color();
    draw_circle(
        sx + WALK_DST_W * 0.4,
        sy - WALK_DST_H * 0.85,
        2.5,
        Color::new(cr, cg, cb, 1.0),
    );
}

// ── camera helpers ─────────────────────────────────────────────────────────────
pub fn cam_max_x() -> f32 { (MAP_WIDTH  as f32 * TS - VIEWPORT_WIDTH).max(0.0) }
pub fn cam_max_y() -> f32 { (MAP_HEIGHT as f32 * TS - SCREEN_HEIGHT ).max(0.0) }

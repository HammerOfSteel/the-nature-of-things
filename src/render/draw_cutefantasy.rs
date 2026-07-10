//! Cute Fantasy Free pack renderer — 16 px tiles × 3 = 48 px display.
//!
//! Asset layout (confirmed):
//!   grass.png   16×16  single tile fill
//!   path.png    16×16  single tile fill
//!   water.png   16×16  single tile fill
//!   house.png   96×128  complete assembled front-view house (6×8 source tiles)
//!   tree.png    64×80   oak tree sprite
//!   pig.png     64×64   pig idle sprite
//!   player.png  192×320  walk sheet — 6 cols × 10 rows, 32×32 per frame
//!     Row 0 = South/Down (toward viewer)
//!     Row 1 = South variant / walk left
//!     Row 2 = North/Up (away)
//!     Row 3 = Walk right/east

use macroquad::prelude::*;
use crate::constants::{MAP_WIDTH, MAP_HEIGHT, TILE_SIZE, SCREEN_HEIGHT, VIEWPORT_WIDTH};
use crate::sim::actor::{Actor, Action};
use crate::sim::world::{SimWorld, TileKind};

pub const TS: f32 = 48.0;

// Player sprite
const PLAYER_SW:     f32 = 32.0;
const PLAYER_SH:     f32 = 32.0;
const PLAYER_COLS:   u32 = 6;
const PLAYER_DST:    f32 = 48.0; // display at 1.5×
const PLAYER_DST_H:  f32 = 48.0;

// House display: scale to ~3 tiles wide
const HOUSE_SW: f32 = 96.0;
const HOUSE_SH: f32 = 128.0;

// Tree display
const TREE_SW: f32 = 64.0;
const TREE_SH: f32 = 80.0;
const TREE_SCALE: f32 = 2.0;

// Pig
const PIG_SW: f32 = 32.0; // use only left half (2 frames?)
const PIG_SH: f32 = 32.0;

// ── texture bundle ─────────────────────────────────────────────────────────────
pub struct FantasyTextures {
    pub grass:   Texture2D,
    pub path:    Texture2D,
    pub water:   Texture2D,
    pub house:   Texture2D,
    pub tree:    Texture2D,
    pub pig:     Texture2D,
    pub player:  Texture2D,
}

async fn load_nn(path: &str) -> Texture2D {
    let t = load_texture(path).await.unwrap_or_else(|e| panic!("Cannot load {path}: {e}"));
    t.set_filter(FilterMode::Nearest);
    t
}

impl FantasyTextures {
    pub async fn load() -> Self {
        FantasyTextures {
            grass:  load_nn("assets/cutefantasy/grass.png").await,
            path:   load_nn("assets/cutefantasy/path.png").await,
            water:  load_nn("assets/cutefantasy/water.png").await,
            house:  load_nn("assets/cutefantasy/house.png").await,
            tree:   load_nn("assets/cutefantasy/tree.png").await,
            pig:    load_nn("assets/cutefantasy/pig.png").await,
            player: load_nn("assets/cutefantasy/player.png").await,
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

fn ground_tex<'a>(kind: TileKind, textures: &'a FantasyTextures) -> &'a Texture2D {
    match kind {
        TileKind::Path | TileKind::BuildingFloor => &textures.path,
        TileKind::Water | TileKind::DeepWater    => &textures.water,
        _                                        => &textures.grass,
    }
}

fn draw_tile(tex: &Texture2D, sx: f32, sy: f32, tint: Color) {
    draw_texture_ex(tex, sx, sy, tint, DrawTextureParams {
        dest_size: Some(vec2(TS, TS)),
        source:    Some(Rect::new(0.0, 0.0, 16.0, 16.0)),
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

fn player_dir_row(actor: &Actor) -> u32 {
    let dx = actor.target_x - actor.tile_x;
    let dy = actor.target_y - actor.tile_y;
    if dx.abs() >= dy.abs() {
        0 // use south-row + flip for both east/west
    } else if dy > 0 {
        0 // south
    } else {
        2 // north (row 2 is back view)
    }
}

// ── draw items ────────────────────────────────────────────────────────────────

enum CItem {
    Building { tx: usize, ty: usize, width: usize },
    Tree     { tx: usize, ty: usize },
    Pig      { tx: usize, ty: usize },
    Actor    { idx: usize },
}
struct CEntry { sort_y: f32, item: CItem }

// ── main renderer ─────────────────────────────────────────────────────────────

pub fn draw_world_cutefantasy(
    world:    &SimWorld,
    textures: &FantasyTextures,
    selected: Option<usize>,
    cam_x:   f32,
    cam_y:   f32,
) {
    let br   = world.weather.brightness();
    let tint = Color::new(br, br, br, 1.0);
    let t    = get_time() as f32;

    // ── Pass 1: ground ───────────────────────────────────────────────────────
    for ty in 0..MAP_HEIGHT {
        for tx in 0..MAP_WIDTH {
            let sx = tx as f32 * TS - cam_x;
            let sy = ty as f32 * TS - cam_y;
            if sx + TS < 0.0 || sx > VIEWPORT_WIDTH { continue; }
            if sy + TS < 0.0 || sy > SCREEN_HEIGHT  { continue; }
            let kind  = world.tiles[ty][tx];
            let gtex  = ground_tex(kind, textures);
            draw_tile(gtex, sx, sy, tint);
        }
    }

    // ── Collect elevated + actors ────────────────────────────────────────────
    let mut entries: Vec<CEntry> = Vec::new();
    let mut used = vec![[false; MAP_WIDTH]; MAP_HEIGHT];

    for ty in 0..MAP_HEIGHT {
        for tx in 0..MAP_WIDTH {
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
                    entries.push(CEntry { sort_y: ty as f32, item: CItem::Building { tx, ty, width } });
                }
                TileKind::Bracken => {
                    entries.push(CEntry { sort_y: ty as f32, item: CItem::Tree { tx, ty } });
                    // Place a pig near every 5th bracken tile
                    if th(tx as i32, ty as i32) > 0.82 {
                        entries.push(CEntry {
                            sort_y: ty as f32 + 0.5,
                            item: CItem::Pig { tx, ty },
                        });
                    }
                }
                _ => {}
            }
        }
    }

    for (idx, actor) in world.actors.iter().enumerate() {
        entries.push(CEntry {
            sort_y: actor.pixel_y / TILE_SIZE,
            item: CItem::Actor { idx },
        });
    }

    entries.sort_by(|a, b| a.sort_y.partial_cmp(&b.sort_y).unwrap_or(std::cmp::Ordering::Equal));

    // ── Pass 2: draw Y-sorted ────────────────────────────────────────────────
    for entry in &entries {
        match &entry.item {

            CItem::Building { tx, ty, width } => {
                let sx = *tx as f32 * TS - cam_x;
                let sy = *ty as f32 * TS - cam_y;
                let build_w = *width as f32 * TS;
                let scale   = build_w / HOUSE_SW;
                let dst_h   = HOUSE_SH * scale;
                let dst_y   = sy + TS - dst_h;
                if sy + TS > 0.0 && dst_y < SCREEN_HEIGHT && sx < VIEWPORT_WIDTH && sx + build_w > 0.0 {
                    draw_sprite(&textures.house,
                        Rect::new(0.0, 0.0, HOUSE_SW, HOUSE_SH),
                        sx, dst_y, build_w, dst_h, tint, false);
                }
            }

            CItem::Tree { tx, ty } => {
                let sx = *tx as f32 * TS - cam_x;
                let sy = *ty as f32 * TS - cam_y;
                let dw = TREE_SW * TREE_SCALE;
                let dh = TREE_SH * TREE_SCALE;
                let dx = sx + (TS - dw) * 0.5;
                let dy = sy + TS - dh;
                if dy < SCREEN_HEIGHT && dx + dw > 0.0 && dx < VIEWPORT_WIDTH {
                    draw_sprite(&textures.tree,
                        Rect::new(0.0, 0.0, TREE_SW, TREE_SH),
                        dx, dy, dw, dh, tint, false);
                }
            }

            CItem::Pig { tx, ty } => {
                let sx = *tx as f32 * TS - cam_x + TS * 0.3;
                let sy = *ty as f32 * TS - cam_y + TS * 0.2;
                let h  = th(*tx as i32, *ty as i32);
                let frame = ((get_time() as f32 * 2.0 + h * 8.0) as u32) % 2;
                draw_sprite(&textures.pig,
                    Rect::new(frame as f32 * PIG_SW, 0.0, PIG_SW, PIG_SH),
                    sx, sy, TS * 0.7, TS * 0.7, tint, false);
            }

            CItem::Actor { idx } => {
                let actor = &world.actors[*idx];
                let wx = actor.pixel_x / TILE_SIZE * TS;
                let wy = actor.pixel_y / TILE_SIZE * TS;
                let sx = wx - cam_x + TS * 0.5;
                let sy = wy - cam_y + TS * 0.5;
                if sx < -(PLAYER_DST+4.0) || sx > VIEWPORT_WIDTH+PLAYER_DST { continue; }
                if sy < -(PLAYER_DST_H+4.0) || sy > SCREEN_HEIGHT+PLAYER_DST_H { continue; }
                draw_player(textures, actor, *idx, sx, sy,
                    selected == Some(*idx), t, br);
            }
        }
    }
}

fn draw_player(
    textures: &FantasyTextures,
    actor:    &Actor,
    idx:      usize,
    sx: f32, sy: f32,
    selected: bool,
    t: f32,
    br: f32,
) {
    let is_walking = matches!(actor.current_action, Action::Walking);
    let fps        = if is_walking { 6.0 } else { 1.5 };
    let frame      = ((t * fps) as u32 + idx as u32 * 2) % PLAYER_COLS;
    let row        = player_dir_row(actor);
    // Flip for west movement (row 0 faces south, flip gives a "leftward" lean)
    let flip_x     = actor.target_x < actor.tile_x;

    if selected {
        draw_circle_lines(sx, sy, PLAYER_DST * 0.6, 2.0, YELLOW);
    }
    draw_circle(sx, sy + PLAYER_DST_H * 0.25, PLAYER_DST * 0.28,
        Color::new(0.0, 0.0, 0.0, 0.18 * br));

    let tint = Color::new(br, br, br, 1.0);
    draw_texture_ex(
        &textures.player,
        sx - PLAYER_DST * 0.5,
        sy - PLAYER_DST_H * 0.85,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(PLAYER_DST, PLAYER_DST_H)),
            source: Some(Rect::new(
                frame as f32 * PLAYER_SW,
                row   as f32 * PLAYER_SH,
                PLAYER_SW, PLAYER_SH,
            )),
            flip_x,
            ..Default::default()
        },
    );

    let (cr, cg, cb) = actor.role.color();
    draw_circle(
        sx + PLAYER_DST * 0.35,
        sy - PLAYER_DST_H * 0.90,
        2.5,
        Color::new(cr, cg, cb, 1.0),
    );
}

// ── camera helpers ─────────────────────────────────────────────────────────────
pub fn cam_max_x() -> f32 { (MAP_WIDTH  as f32 * TS - VIEWPORT_WIDTH).max(0.0) }
pub fn cam_max_y() -> f32 { (MAP_HEIGHT as f32 * TS - SCREEN_HEIGHT ).max(0.0) }

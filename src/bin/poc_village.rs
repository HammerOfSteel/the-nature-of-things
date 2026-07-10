/// poc_village — Procedural village with blueprint generation + player movement.
///
/// Controls
///   WASD / Arrow keys  — move player
///   Mouse drag / Arrow  — pan camera (hold Shift for fast pan)
///   Tab                — cycle selected NPC
///   Space              — pause NPC simulation
///   M                  — toggle minimap
///   R                  — regenerate village with new seed
///   [ / ]              — slow / speed NPC tick rate
///   Escape             — deselect
///
/// Run:  cargo run --bin poc_village [seed]
use macroquad::prelude::*;
use ::rand::rngs::StdRng;
use ::rand::SeedableRng;
use ::rand::Rng;

use the_nature_of_things::constants::*;
use the_nature_of_things::sim::world::{SimWorld, TileKind};
use the_nature_of_things::sim::systems::tick;
use the_nature_of_things::render::draw_sunnyside::{
    SunnyTextures, SPRITE_TS,
    draw_world_sunnyside, cam_max_x, cam_max_y,
};
use the_nature_of_things::render::draw::{draw_minimap, draw_relationships};
use the_nature_of_things::render::ui::{draw_panel, draw_controls, draw_pause_overlay};

// ── Player ────────────────────────────────────────────────────────────────────

const PLAYER_SPEED: f32 = 3.5;  // tiles per second

#[derive(PartialEq, Eq, Clone, Copy)]
enum Dir { North, South, East, West }

struct Player {
    /// Current tile position (logical)
    tx: i32,
    ty: i32,
    /// Sub-tile smooth pixel position
    px: f32,
    py: f32,
    /// Movement target (may equal current position = idle)
    target_tx: i32,
    target_ty: i32,
    facing: Dir,
    /// Time spent on current walk cycle (for animation frame)
    walk_t: f32,
}

impl Player {
    fn new(tx: i32, ty: i32) -> Self {
        Player {
            tx, ty,
            px: tx as f32 * TILE_SIZE,
            py: ty as f32 * TILE_SIZE,
            target_tx: tx,
            target_ty: ty,
            facing: Dir::South,
            walk_t: 0.0,
        }
    }

    fn is_moving(&self) -> bool {
        self.tx != self.target_tx || self.ty != self.target_ty
    }

    /// Attempt to step in a direction; only commits if destination is walkable.
    fn try_move(&mut self, dx: i32, dy: i32, tiles: &[Vec<TileKind>]) {
        if self.is_moving() { return; }   // still animating previous step
        let ntx = self.tx + dx;
        let nty = self.ty + dy;
        if ntx < 0 || nty < 0 || ntx >= MAP_WIDTH as i32 || nty >= MAP_HEIGHT as i32 { return; }
        if !tiles[nty as usize][ntx as usize].is_walkable() { return; }

        self.target_tx = ntx;
        self.target_ty = nty;
        self.facing = match (dx, dy) {
            ( 0, -1) => Dir::North,
            ( 0,  1) => Dir::South,
            ( 1,  0) => Dir::East,
            (-1,  0) => Dir::West,
            _        => self.facing,
        };
    }

    /// Advance smooth position toward target; returns true when step completes.
    fn update(&mut self, dt: f32) {
        let target_px = self.target_tx as f32 * TILE_SIZE;
        let target_py = self.target_ty as f32 * TILE_SIZE;
        let speed = PLAYER_SPEED * TILE_SIZE;

        let dx = target_px - self.px;
        let dy = target_py - self.py;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist < 0.5 {
            self.px = target_px;
            self.py = target_py;
            self.tx = self.target_tx;
            self.ty = self.target_ty;
        } else {
            let step = (speed * dt).min(dist);
            self.px += dx / dist * step;
            self.py += dy / dist * step;
        }

        if self.is_moving() { self.walk_t += dt; } else { self.walk_t = 0.0; }
    }
}

// ── GameState ─────────────────────────────────────────────────────────────────

struct GameState {
    world:            SimWorld,
    rng:              StdRng,
    player:           Player,
    cam_x:            f32,
    cam_y:            f32,
    selected_actor:   Option<usize>,
    tick_accumulator: f64,
    tick_interval:    f64,
    paused:           bool,
    show_minimap:     bool,
    seed:             u64,
}

impl GameState {
    fn new(seed: u64) -> Self {
        let world = SimWorld::generate_village(seed);

        // Spawn player at the first path tile near the centre of the map
        let (px, py) = find_road_spawn(&world);
        let player = Player::new(px, py);

        // Centre camera on player
        let cx = (px as f32 * SPRITE_TS - VIEWPORT_WIDTH  * 0.5).max(0.0).min(cam_max_x());
        let cy = (py as f32 * SPRITE_TS - SCREEN_HEIGHT   * 0.5).max(0.0).min(cam_max_y());

        GameState {
            world,
            rng: StdRng::seed_from_u64(seed ^ 0xCAFE_BABE),
            player,
            cam_x: cx,
            cam_y: cy,
            selected_actor:   None,
            tick_accumulator: 0.0,
            tick_interval:    DEFAULT_TICK_INTERVAL,
            paused:           false,
            show_minimap:     true,
            seed,
        }
    }

    fn regenerate(&mut self) {
        self.seed = self.rng.gen();
        *self = GameState::new(self.seed);
    }
}

fn find_road_spawn(world: &SimWorld) -> (i32, i32) {
    // Prefer a path tile near horizontal centre of the map
    let cx = MAP_WIDTH as i32 / 2;
    let cy = MAP_HEIGHT as i32 / 2;
    for r in 0..20i32 {
        for dx in -r..=r {
            for dy in -r..=r {
                let tx = cx + dx; let ty = cy + dy;
                if tx < 0 || ty < 0 { continue; }
                if tx >= MAP_WIDTH as i32 || ty >= MAP_HEIGHT as i32 { continue; }
                if world.tiles[ty as usize][tx as usize] == TileKind::Path {
                    return (tx, ty);
                }
            }
        }
    }
    (cx, cy)
}

// ── Draw helpers ──────────────────────────────────────────────────────────────

fn draw_player(textures: &SunnyTextures, player: &Player, cam_x: f32, cam_y: f32) {
    use the_nature_of_things::render::draw_sunnyside::SPRITE_TS;

    let wx = player.px / TILE_SIZE * SPRITE_TS;
    let wy = player.py / TILE_SIZE * SPRITE_TS;
    let sx = wx - cam_x + SPRITE_TS * 0.5;
    let sy = wy - cam_y + SPRITE_TS * 0.5;

    const FW: f32 = 96.0; const FH: f32 = 64.0;
    const DW: f32 = 64.0; const DH: f32 = 48.0;

    let is_walking = player.is_moving();
    let fps: f32   = 8.0;
    let n_frames: u32 = if is_walking { 8 } else { 9 };

    let t = get_time() as f32;
    let frame = if is_walking {
        ((t * fps) as u32) % n_frames
    } else {
        ((t * 4.0) as u32) % n_frames
    };
    let src_x = frame as f32 * FW;

    // Selection ring
    draw_circle_lines(sx, sy + 4.0, 18.0, 2.0, SKYBLUE);

    // Drop shadow
    draw_circle(sx, sy + 6.0, 8.0, Color::new(0.0, 0.0, 0.0, 0.22));

    let flip_x = player.facing == Dir::West;

    let base_tex = if is_walking { &textures.walk_base } else { &textures.idle_base };
    // Hair: use index 5 (spikeyhair) for the player to distinguish them
    let hair_tex = if is_walking { &textures.walk_hair[5] } else { &textures.idle_hair[5] };

    let params = DrawTextureParams {
        dest_size: Some(vec2(DW, DH)),
        source: Some(Rect::new(src_x, 0.0, FW, FH)),
        flip_x,
        ..Default::default()
    };

    let draw_x = sx - DW * 0.5;
    let draw_y = sy - DH * 0.8;
    draw_texture_ex(base_tex, draw_x, draw_y, WHITE, params.clone());
    draw_texture_ex(hair_tex, draw_x, draw_y, WHITE, params);

    // "YOU" label
    draw_text("YOU", sx - 10.0, draw_y - 8.0, 12.0, SKYBLUE);
}

fn draw_hud(world: &SimWorld, player: &Player, seed: u64) {
    let tx = player.tx; let ty = player.ty;
    let tile = world.tile_at(tx, ty);
    let info = format!(
        "Village seed: {}   Player ({},{}) [{:?}]   Day {}  {}   {}",
        seed, tx, ty, tile,
        world.clock.day, world.clock.time_label(),
        world.weather.label(),
    );
    draw_text(&info, 12.0, SCREEN_HEIGHT - 18.0, 16.0, WHITE);
}

// ── Window configuration ──────────────────────────────────────────────────────

fn window_conf() -> Conf {
    Conf {
        window_title:     "The Nature of Things — Village".to_owned(),
        window_width:     SCREEN_WIDTH  as i32,
        window_height:    SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

// ── Main loop ─────────────────────────────────────────────────────────────────

#[macroquad::main(window_conf)]
async fn main() {
    let textures = SunnyTextures::load().await;

    let seed: u64 = std::env::args().nth(1)
        .and_then(|s| s.parse().ok()).unwrap_or(42);
    let mut gs = GameState::new(seed);

    loop {
        let dt     = get_frame_time().min(0.05);
        let dt_f64 = dt as f64;

        // ── Player input ──────────────────────────────────────────────────────
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)    { gs.player.try_move( 0, -1, &gs.world.tiles); }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)  { gs.player.try_move( 0,  1, &gs.world.tiles); }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)  { gs.player.try_move(-1,  0, &gs.world.tiles); }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { gs.player.try_move( 1,  0, &gs.world.tiles); }
        gs.player.update(dt);

        // ── Camera: follow player ─────────────────────────────────────────────
        let target_cx = (gs.player.px / TILE_SIZE * SPRITE_TS - VIEWPORT_WIDTH  * 0.5)
            .clamp(0.0, cam_max_x());
        let target_cy = (gs.player.py / TILE_SIZE * SPRITE_TS - SCREEN_HEIGHT   * 0.5)
            .clamp(0.0, cam_max_y());
        // Smooth camera lerp
        gs.cam_x += (target_cx - gs.cam_x) * (dt * 6.0).min(1.0);
        gs.cam_y += (target_cy - gs.cam_y) * (dt * 6.0).min(1.0);

        // ── Misc keyboard ─────────────────────────────────────────────────────
        if is_key_pressed(KeyCode::Space)  { gs.paused = !gs.paused; }
        if is_key_pressed(KeyCode::Escape) { gs.selected_actor = None; }
        if is_key_pressed(KeyCode::M)      { gs.show_minimap = !gs.show_minimap; }
        if is_key_pressed(KeyCode::R)      { gs.regenerate(); }

        if is_key_pressed(KeyCode::RightBracket) {
            gs.tick_interval = (gs.tick_interval * 0.6).max(0.03);
        }
        if is_key_pressed(KeyCode::LeftBracket) {
            gs.tick_interval = (gs.tick_interval * 1.6).min(1.0);
        }

        if is_key_pressed(KeyCode::Tab) {
            let n = gs.world.actors.len();
            gs.selected_actor = Some(match gs.selected_actor {
                None    => 0,
                Some(i) => (i + 1) % n,
            });
        }

        // ── NPC simulation tick ───────────────────────────────────────────────
        if !gs.paused {
            gs.tick_accumulator += dt_f64;
            while gs.tick_accumulator >= gs.tick_interval {
                tick(&mut gs.world, &mut gs.rng);
                gs.tick_accumulator -= gs.tick_interval;
            }
        }

        // ── Render ────────────────────────────────────────────────────────────
        clear_background(BLACK);

        // Tile map + NPC sprites
        draw_world_sunnyside(&gs.world, &textures, gs.selected_actor, gs.cam_x, gs.cam_y);

        // Player character (drawn on top of NPCs at its Y depth)
        draw_player(&textures, &gs.player, gs.cam_x, gs.cam_y);

        // UI panel (right side)
        draw_panel(&gs.world, gs.selected_actor);

        // Controls hint
        draw_controls();

        // Minimap
        if gs.show_minimap {
            draw_minimap(&gs.world, gs.cam_x, gs.cam_y);
        }

        // Relationship lines (selected NPC)
        if let Some(id) = gs.selected_actor {
            draw_relationships(&gs.world.actors, Some(id), gs.cam_x, gs.cam_y, &gs.world.clock);
        }

        // Pause overlay
        if gs.paused { draw_pause_overlay(); }

        // Bottom HUD
        draw_hud(&gs.world, &gs.player, gs.seed);

        next_frame().await;
    }
}

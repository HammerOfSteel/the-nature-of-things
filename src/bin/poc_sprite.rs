/// POC-4: Kenney Tiny-Town 16 px sprite tiles + RPG top-down characters.
/// Run with: cargo run --bin poc_sprite
use macroquad::prelude::*;
use ::rand::rngs::StdRng;
use ::rand::SeedableRng;

use the_nature_of_things::constants::*;
use the_nature_of_things::sim::world::{SimWorld, GlobalEvent};
use the_nature_of_things::sim::systems::tick;
use the_nature_of_things::render::draw_sprites::{
    SpriteTextures, SPRITE_TS,
    draw_world_sprites, cam_max_x, cam_max_y,
};
use the_nature_of_things::render::draw::{
    draw_weather_overlay, update_actor_positions,
    draw_minimap, draw_relationships,
};
use the_nature_of_things::render::ui::{draw_panel, draw_controls, draw_pause_overlay};

fn window_conf() -> Conf {
    Conf {
        window_title:    "The Nature of Things — Sprite POC (Kenney Tiles)".to_owned(),
        window_width:    SCREEN_WIDTH  as i32,
        window_height:   SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

struct GameState {
    world:            SimWorld,
    rng:              StdRng,
    cam_x:            f32,
    cam_y:            f32,
    selected_actor:   Option<usize>,
    tick_accumulator: f64,
    tick_interval:    f64,
    paused:           bool,
    show_minimap:     bool,
}

impl GameState {
    fn new(seed: u64) -> Self {
        let world = SimWorld::generate(seed);
        let cx = (MAP_WIDTH  as f32 * SPRITE_TS * 0.5 - VIEWPORT_WIDTH  * 0.5).max(0.0);
        let cy = (MAP_HEIGHT as f32 * SPRITE_TS * 0.55 - SCREEN_HEIGHT  * 0.5).max(0.0);
        GameState {
            world,
            rng: StdRng::seed_from_u64(seed ^ 0xDEAD_BEEF),
            cam_x: cx.min(cam_max_x()),
            cam_y: cy.min(cam_max_y()),
            selected_actor: None,
            tick_accumulator: 0.0,
            tick_interval: DEFAULT_TICK_INTERVAL,
            paused: false,
            show_minimap: true,
        }
    }
}

/// Pick the nearest actor within ~16 px of the click (sprite-tile coordinate space).
fn actor_at_screen(
    actors: &[the_nature_of_things::sim::actor::Actor],
    mx: f32, my: f32,
    cam_x: f32, cam_y: f32,
) -> Option<usize> {
    let mut best = None;
    let mut best_d = 18.0_f32;
    for a in actors {
        let wx = a.pixel_x / TILE_SIZE * SPRITE_TS;
        let wy = a.pixel_y / TILE_SIZE * SPRITE_TS;
        let sx = wx - cam_x + SPRITE_TS * 0.5;
        let sy = wy - cam_y + SPRITE_TS * 0.5;
        let d = ((mx - sx).powi(2) + (my - sy).powi(2)).sqrt();
        if d < best_d { best_d = d; best = Some(a.id); }
    }
    best
}

#[macroquad::main(window_conf)]
async fn main() {
    // ── Load textures (async, must happen before game loop) ───────────────────
    let textures = SpriteTextures::load().await;

    let seed: u64 = std::env::args().nth(1)
        .and_then(|s| s.parse().ok()).unwrap_or(42);
    let mut gs = GameState::new(seed);

    loop {
        let dt     = get_frame_time();
        let dt_f64 = dt as f64;

        // ── Input ─────────────────────────────────────────────────────────────
        if is_key_pressed(KeyCode::Space)  { gs.paused = !gs.paused; }
        if is_key_pressed(KeyCode::Escape) { gs.selected_actor = None; }
        if is_key_pressed(KeyCode::M)      { gs.show_minimap = !gs.show_minimap; }

        if is_key_pressed(KeyCode::RightBracket) {
            gs.tick_interval = (gs.tick_interval * 0.6).max(0.03);
        }
        if is_key_pressed(KeyCode::LeftBracket) {
            gs.tick_interval = (gs.tick_interval * 1.6).min(1.0);
        }

        if is_key_pressed(KeyCode::P) { gs.world.inject_event(GlobalEvent::PitClosure); }
        if is_key_pressed(KeyCode::E) { gs.world.inject_event(GlobalEvent::Eisteddfod); }
        if is_key_pressed(KeyCode::H) { gs.world.inject_event(GlobalEvent::HardWinter); }
        if is_key_pressed(KeyCode::B) {
            let id = (get_time() as usize) % gs.world.actors.len();
            gs.world.inject_event(GlobalEvent::Bereavement { actor_id: id });
        }

        if is_key_pressed(KeyCode::Tab) {
            let n = gs.world.actors.len();
            gs.selected_actor = Some(match gs.selected_actor {
                None    => 0,
                Some(i) => (i + 1) % n,
            });
            if let Some(id) = gs.selected_actor {
                if let Some(a) = gs.world.actors.iter().find(|a| a.id == id) {
                    let wx = a.pixel_x / TILE_SIZE * SPRITE_TS;
                    let wy = a.pixel_y / TILE_SIZE * SPRITE_TS;
                    gs.cam_x = (wx - VIEWPORT_WIDTH  * 0.5).clamp(0.0, cam_max_x());
                    gs.cam_y = (wy - SCREEN_HEIGHT   * 0.5).clamp(0.0, cam_max_y());
                }
            }
        }

        let pan = CAM_SPEED * dt;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)  {
            gs.cam_x = (gs.cam_x - pan).max(0.0);
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            gs.cam_x = (gs.cam_x + pan).min(cam_max_x());
        }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)    {
            gs.cam_y = (gs.cam_y - pan).max(0.0);
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)  {
            gs.cam_y = (gs.cam_y + pan).min(cam_max_y());
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx < VIEWPORT_WIDTH {
                gs.selected_actor =
                    actor_at_screen(&gs.world.actors, mx, my, gs.cam_x, gs.cam_y);
            }
        }

        // ── Sim tick ──────────────────────────────────────────────────────────
        if !gs.paused {
            gs.tick_accumulator += dt_f64;
            while gs.tick_accumulator >= gs.tick_interval {
                gs.tick_accumulator -= gs.tick_interval;
                tick(&mut gs.world, &mut gs.rng);
            }
        }
        update_actor_positions(&mut gs.world.actors, dt);

        // ── Render ────────────────────────────────────────────────────────────
        clear_background(Color::new(0.06, 0.08, 0.06, 1.0));

        draw_world_sprites(&gs.world, &textures, gs.selected_actor, gs.cam_x, gs.cam_y);
        draw_weather_overlay(gs.world.weather, gs.world.clock.season);
        draw_relationships(
            &gs.world.actors, gs.selected_actor, gs.cam_x, gs.cam_y, &gs.world.clock,
        );

        draw_panel(&gs.world, gs.selected_actor);
        draw_controls();
        if gs.paused         { draw_pause_overlay(); }
        if gs.show_minimap   {
            // Minimap uses TILE_SIZE co-ordinates internally; cam here is in SPRITE_TS space.
            // Scale camera back to TILE_SIZE space for the minimap.
            let mc_x = gs.cam_x / SPRITE_TS * TILE_SIZE;
            let mc_y = gs.cam_y / SPRITE_TS * TILE_SIZE;
            draw_minimap(&gs.world, mc_x, mc_y);
        }

        let label = format!(
            "POC-4: Kenney Sprites  FPS:{:.0}  Day {}  {}  [WASD pan  Tab select  [/] speed  Space pause]",
            get_fps(), gs.world.clock.day, gs.world.weather.label(),
        );
        draw_text(&label, 4.0, 12.0, 10.0, Color::new(0.80, 0.80, 0.85, 0.85));

        next_frame().await;
    }
}

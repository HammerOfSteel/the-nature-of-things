use macroquad::prelude::*;
use ::rand::rngs::StdRng;
use ::rand::SeedableRng;

mod constants;
mod sim;
mod render;

use constants::*;
use sim::world::{SimWorld, GlobalEvent};
use sim::systems::tick;
use render::draw::{draw_tiles, draw_location_labels, draw_actors,
    update_actor_positions, actor_at_screen, draw_weather_overlay};
use render::ui::{draw_panel, draw_controls, draw_pause_overlay};

// ─── Window configuration ─────────────────────────────────────────────────────

fn window_conf() -> Conf {
    Conf {
        window_title: "The Nature of Things".to_owned(),
        window_width:  SCREEN_WIDTH  as i32,
        window_height: SCREEN_HEIGHT as i32,
        window_resizable: false,
        ..Default::default()
    }
}

// ─── Game state ───────────────────────────────────────────────────────────────

struct GameState {
    world: SimWorld,
    rng: StdRng,
    cam_x: f32,
    cam_y: f32,
    selected_actor: Option<usize>,
    tick_accumulator: f64,
    tick_interval: f64,
    paused: bool,
}

impl GameState {
    fn new(seed: u64) -> Self {
        let world = SimWorld::generate(seed);

        // Start camera centered on the High Street area
        let cx = (MAP_WIDTH  as f32 * TILE_SIZE * 0.5 - VIEWPORT_WIDTH  * 0.5).max(0.0);
        let cy = (MAP_HEIGHT as f32 * TILE_SIZE * 0.55 - SCREEN_HEIGHT * 0.5).max(0.0);

        GameState {
            world,
            rng: StdRng::seed_from_u64(seed ^ 0xDEAD_BEEF),
            cam_x: cx,
            cam_y: cy,
            selected_actor: None,
            tick_accumulator: 0.0,
            tick_interval: DEFAULT_TICK_INTERVAL,
            paused: false,
        }
    }

    fn cam_max_x(&self) -> f32 {
        (MAP_WIDTH  as f32 * TILE_SIZE - VIEWPORT_WIDTH).max(0.0)
    }
    fn cam_max_y(&self) -> f32 {
        (MAP_HEIGHT as f32 * TILE_SIZE - SCREEN_HEIGHT).max(0.0)
    }
}

// ─── Entry point ─────────────────────────────────────────────────────────────

#[macroquad::main(window_conf)]
async fn main() {
    // Parse seed from environment (default 42)
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    let mut gs = GameState::new(seed);

    loop {
        let dt    = get_frame_time();
        let dt_f64 = dt as f64;

        // ── Input ────────────────────────────────────────────────────────────

        // Pause / unpause
        if is_key_pressed(KeyCode::Space) {
            gs.paused = !gs.paused;
        }

        // Deselect
        if is_key_pressed(KeyCode::Escape) {
            gs.selected_actor = None;
        }

        // Speed controls
        if is_key_pressed(KeyCode::RightBracket) {
            gs.tick_interval = (gs.tick_interval * 0.6).max(0.03);
        }
        if is_key_pressed(KeyCode::LeftBracket) {
            gs.tick_interval = (gs.tick_interval * 1.6).min(1.0);
        }

        // ── Global event triggers ──────────────────────────────────────────
        if is_key_pressed(KeyCode::P) {
            gs.world.inject_event(GlobalEvent::PitClosure);
        }
        if is_key_pressed(KeyCode::E) {
            gs.world.inject_event(GlobalEvent::Eisteddfod);
        }
        if is_key_pressed(KeyCode::H) {
            gs.world.inject_event(GlobalEvent::HardWinter);
        }
        if is_key_pressed(KeyCode::B) {
            // Bereavement: pick a random actor
            let id = (get_time() as usize) % gs.world.actors.len();
            gs.world.inject_event(GlobalEvent::Bereavement { actor_id: id });
        }

        // Tab: cycle through actors
        if is_key_pressed(KeyCode::Tab) {
            let n = gs.world.actors.len();
            gs.selected_actor = Some(match gs.selected_actor {
                None     => 0,
                Some(id) => (id + 1) % n,
            });
            // Snap camera to selected actor
            if let Some(id) = gs.selected_actor {
                if let Some(a) = gs.world.actors.iter().find(|a| a.id == id) {
                    gs.cam_x = (a.pixel_x - VIEWPORT_WIDTH  * 0.5).clamp(0.0, gs.cam_max_x());
                    gs.cam_y = (a.pixel_y - SCREEN_HEIGHT   * 0.5).clamp(0.0, gs.cam_max_y());
                }
            }
        }

        // Camera pan (WASD + Arrow keys)
        let pan = CAM_SPEED * dt;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            gs.cam_x = (gs.cam_x - pan).max(0.0);
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            gs.cam_x = (gs.cam_x + pan).min(gs.cam_max_x());
        }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            gs.cam_y = (gs.cam_y - pan).max(0.0);
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            gs.cam_y = (gs.cam_y + pan).min(gs.cam_max_y());
        }

        // Camera: follow selected actor when near edge of viewport
        if let Some(id) = gs.selected_actor {
            if let Some(actor) = gs.world.actors.iter().find(|a| a.id == id) {
                let ax = actor.pixel_x - gs.cam_x;
                let ay = actor.pixel_y - gs.cam_y;
                let margin = 80.0;
                let nudge = 60.0 * dt;
                if ax < margin          { gs.cam_x = (gs.cam_x - nudge).max(0.0); }
                if ax > VIEWPORT_WIDTH - margin - TILE_SIZE {
                    gs.cam_x = (gs.cam_x + nudge).min(gs.cam_max_x());
                }
                if ay < margin          { gs.cam_y = (gs.cam_y - nudge).max(0.0); }
                if ay > SCREEN_HEIGHT  - margin - TILE_SIZE {
                    gs.cam_y = (gs.cam_y + nudge).min(gs.cam_max_y());
                }
            }
        }

        // Actor selection via left click
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx < VIEWPORT_WIDTH {
                gs.selected_actor =
                    actor_at_screen(&gs.world.actors, mx, my, gs.cam_x, gs.cam_y);
            }
        }

        // ── Simulation tick ───────────────────────────────────────────────────

        if !gs.paused {
            gs.tick_accumulator += dt_f64;
            while gs.tick_accumulator >= gs.tick_interval {
                gs.tick_accumulator -= gs.tick_interval;
                tick(&mut gs.world, &mut gs.rng);
            }
        }

        // ── Smooth actor visual positions ─────────────────────────────────────

        update_actor_positions(&mut gs.world.actors, dt);

        // ── Render ────────────────────────────────────────────────────────────

        clear_background(Color::new(0.05, 0.05, 0.08, 1.0));

        draw_tiles(&gs.world, gs.cam_x, gs.cam_y);
        draw_weather_overlay(gs.world.weather, gs.world.clock.season);
        draw_location_labels(&gs.world, gs.cam_x, gs.cam_y);
        draw_actors(&gs.world.actors, gs.selected_actor, gs.cam_x, gs.cam_y, &gs.world.clock);

        // UI panel (drawn on top, over the viewport boundary)
        draw_panel(&gs.world, gs.selected_actor);
        draw_controls();

        if gs.paused {
            draw_pause_overlay();
        }

        // FPS / tick rate info (top-left, outside viewport)
        let fps_str = format!("FPS:{:.0}  {:.0}ms/tick  Day {}  {}",
            get_fps(), gs.tick_interval * 1000.0,
            gs.world.clock.day, gs.world.weather.label());
        draw_text(&fps_str, 4.0, 12.0, 10.0, Color::new(0.50, 0.50, 0.55, 0.80));

        next_frame().await;
    }
}

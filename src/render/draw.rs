use macroquad::prelude::*;
use crate::constants::*;
use crate::sim::actor::{Actor, Action};
use crate::sim::world::{SimWorld, TileKind, WorldClock};

// ─── Colour helpers ───────────────────────────────────────────────────────────

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::new(r, g, b, 1.0)
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        1.0,
    )
}

/// Night-time darkening: returns a [0..1] dim factor (1 = full brightness)
fn daylight(clock: &WorldClock) -> f32 {
    let t = clock.time_of_day;
    // Bright between 0.25 (dawn) and 0.80 (dusk), dark otherwise
    if t < 0.20 {
        0.25 + t * 2.5 // dawn ramp up
    } else if t < 0.80 {
        1.0
    } else {
        1.0 - (t - 0.80) * 3.5 // dusk ramp down, min 0.25
    }.clamp(0.25, 1.0)
}

fn dim(c: Color, factor: f32) -> Color {
    Color::new(c.r * factor, c.g * factor, c.b * factor, 1.0)
}

// ─── Tile rendering ───────────────────────────────────────────────────────────

pub fn draw_tiles(world: &SimWorld, cam_x: f32, cam_y: f32) {
    let light = daylight(&world.clock);
    let t = get_time() as f32; // for water animation

    let x_start = (cam_x / TILE_SIZE) as i32 - 1;
    let y_start = (cam_y / TILE_SIZE) as i32 - 1;
    let x_end   = x_start + (VIEWPORT_WIDTH  / TILE_SIZE) as i32 + 2;
    let y_end   = y_start + (SCREEN_HEIGHT   / TILE_SIZE) as i32 + 2;

    for ty in y_start..=y_end {
        if ty < 0 || ty >= MAP_HEIGHT as i32 { continue; }
        for tx in x_start..=x_end {
            if tx < 0 || tx >= MAP_WIDTH as i32 { continue; }

            let tile  = world.tiles[ty as usize][tx as usize];
            let noise = world.tile_noise[ty as usize][tx as usize];

            let (br, bg, bb) = tile.base_color();

            // Water: slight animated shimmer
            let color = if tile == TileKind::Water || tile == TileKind::DeepWater {
                let wave = (t * 1.8 + tx as f32 * 0.3 + ty as f32 * 0.2).sin() * 0.04;
                rgb(br + noise + wave, bg + noise * 0.8 + wave, bb + noise * 0.5)
            } else {
                rgb(br + noise, bg + noise * 0.9, bb + noise * 0.8)
            };

            let screen_x = tx as f32 * TILE_SIZE - cam_x;
            let screen_y = ty as f32 * TILE_SIZE - cam_y;

            draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, dim(color, light));

            // Subtle grid lines for buildings to define walls
            if tile == TileKind::BuildingWall {
                draw_rectangle_lines(screen_x, screen_y, TILE_SIZE, TILE_SIZE, 1.0,
                    dim(rgb(0.20, 0.18, 0.16), light));
            }
        }
    }
}

// ─── Location labels ─────────────────────────────────────────────────────────

pub fn draw_location_labels(world: &SimWorld, cam_x: f32, cam_y: f32) {
    let light = daylight(&world.clock);
    let label_color = Color::new(1.0, 1.0, 0.85, 0.65 * light);

    for loc in &world.locations {
        let sx = loc.tile_x as f32 * TILE_SIZE - cam_x - 10.0;
        let sy = loc.tile_y as f32 * TILE_SIZE - cam_y - 6.0;
        if sx < -80.0 || sx > VIEWPORT_WIDTH + 80.0 { continue; }
        if sy < -20.0 || sy > SCREEN_HEIGHT + 20.0  { continue; }
        draw_text(loc.kind.label(), sx, sy, 11.0, label_color);
    }
}

// ─── Actor rendering ─────────────────────────────────────────────────────────

pub fn draw_actors(actors: &[Actor], selected: Option<usize>, cam_x: f32, cam_y: f32,
                   clock: &WorldClock) {
    let light = daylight(clock);
    let t = get_time() as f32;

    for actor in actors {
        let sx = actor.pixel_x - cam_x + TILE_SIZE * 0.5;
        let sy = actor.pixel_y - cam_y + TILE_SIZE * 0.5;

        if sx < -24.0 || sx > VIEWPORT_WIDTH + 24.0 { continue; }
        if sy < -24.0 || sy > SCREEN_HEIGHT  + 24.0 { continue; }

        let (cr, cg, cb) = actor.role.color();
        let body_color   = dim(rgb(cr, cg, cb), light);
        let dark_color   = dim(rgb(cr * 0.55, cg * 0.55, cb * 0.55), light);

        // Idle bob: a gentle vertical sine oscillation
        let bob = match actor.current_action {
            Action::Walking  => (t * 12.0 + actor.id as f32).sin() * 1.5,
            Action::Singing  => (t * 8.0  + actor.id as f32).sin() * 2.5,
            _                => (t * 2.5  + actor.id as f32).sin() * 0.8,
        };

        let by = sy + bob;

        // Shadow
        draw_ellipse(sx, sy + ACTOR_BODY_H * 0.4, 5.0, 2.5,
            0.0, Color::new(0.0, 0.0, 0.0, 0.25 * light));

        // Body (rectangle)
        draw_rectangle(
            sx - ACTOR_BODY_W * 0.5,
            by - ACTOR_BODY_H * 0.5,
            ACTOR_BODY_W, ACTOR_BODY_H,
            body_color,
        );

        // Head (circle)
        let head_y = by - ACTOR_BODY_H * 0.5 - ACTOR_HEAD_R;
        draw_circle(sx, head_y, ACTOR_HEAD_R, body_color);

        // Eyes (two tiny white dots)
        draw_circle(sx - 1.4, head_y - 0.6, 1.0, Color::new(1.0, 1.0, 1.0, 0.9 * light));
        draw_circle(sx + 1.4, head_y - 0.6, 1.0, Color::new(1.0, 1.0, 1.0, 0.9 * light));

        // Dark outline on body bottom edge (ground shadow)
        draw_rectangle(
            sx - ACTOR_BODY_W * 0.5,
            by + ACTOR_BODY_H * 0.5 - 2.0,
            ACTOR_BODY_W, 2.0,
            dark_color,
        );

        // Emotion indicator: small coloured dot above head when a node is urgent
        if let Some((node_idx, val)) = actor.emotion() {
            let (er, eg, eb) = NODE_COLORS[node_idx];
            let alpha = ((val - 0.62) * 2.8).min(1.0) * light;
            draw_circle(sx + 7.0, head_y - ACTOR_HEAD_R - 2.0, 3.2,
                Color::new(er, eg, eb, alpha));
        }

        // Selection ring
        if selected == Some(actor.id) {
            draw_circle_lines(sx, by - ACTOR_BODY_H * 0.1,
                SELECTION_RING_R, 1.5, YELLOW);
            // Name above
            let name_x = sx - actor.name.len() as f32 * 2.8;
            draw_text(&actor.name, name_x, head_y - ACTOR_HEAD_R - 4.0, 12.0,
                Color::new(1.0, 1.0, 0.7, 0.90));
        }
    }
}

// ─── Smooth actor pixel position update ──────────────────────────────────────

pub fn update_actor_positions(actors: &mut [Actor], dt: f32) {
    let speed = 10.0 * dt;
    for actor in actors.iter_mut() {
        let tx = actor.tile_x as f32 * TILE_SIZE;
        let ty = actor.tile_y as f32 * TILE_SIZE;
        actor.pixel_x += (tx - actor.pixel_x) * speed.min(1.0);
        actor.pixel_y += (ty - actor.pixel_y) * speed.min(1.0);
    }
}

// ─── Hit test: which actor is nearest to a screen click? ─────────────────────

pub fn actor_at_screen(actors: &[Actor], sx: f32, sy: f32, cam_x: f32, cam_y: f32)
    -> Option<usize>
{
    let wx = sx + cam_x;
    let wy = sy + cam_y;
    let mut best: Option<(f32, usize)> = None;

    for actor in actors {
        let ax = actor.pixel_x + TILE_SIZE * 0.5;
        let ay = actor.pixel_y + TILE_SIZE * 0.5;
        let dist = ((wx - ax).powi(2) + (wy - ay).powi(2)).sqrt();
        if dist < 16.0 {
            if best.map_or(true, |(d, _)| dist < d) {
                best = Some((dist, actor.id));
            }
        }
    }

    best.map(|(_, id)| id)
}

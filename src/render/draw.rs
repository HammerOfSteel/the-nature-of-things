use macroquad::prelude::*;
use crate::constants::*;
use crate::sim::actor::{Actor, Action, Role};
use crate::sim::world::{SimWorld, TileKind, WorldClock, Weather, Season};

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

/// Apply season-specific colour tint to outdoor tiles.
fn season_tint(season: Season, tile: TileKind, r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let outdoor = matches!(tile,
        TileKind::Grass | TileKind::Bracken | TileKind::Stone
        | TileKind::Path | TileKind::Mountain);
    if !outdoor { return (r, g, b); }
    match season {
        Season::Spring => (r * 0.95, g * 1.07, b * 0.95),  // fresh, green
        Season::Summer => (r * 1.05, g * 1.02, b * 0.88),  // warm, bright
        Season::Autumn => (r * 1.20, g * 0.86, b * 0.70),  // ochre/russet
        Season::Winter => (r * 0.78, g * 0.83, b * 0.98),  // cold, blue-grey
    }
}

/// Simple hash for per-tile snow/variation.
fn tile_hash(x: i32, y: i32) -> f32 {
    let h = (x as u32).wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(1_103_515_245));
    (h >> 8) as f32 / (u32::MAX >> 8) as f32
}

// ─── Tile rendering ───────────────────────────────────────────────────────────

pub fn draw_tiles(world: &SimWorld, cam_x: f32, cam_y: f32) {
    let light   = daylight(&world.clock) * world.weather.brightness();
    let season  = world.clock.season;
    let t       = get_time() as f32;
    let is_winter = matches!(season, Season::Winter);

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
            let (br, bg, bb) = season_tint(season, tile, br, bg, bb);

            // Water: animated shimmer
            let color = if tile == TileKind::Water || tile == TileKind::DeepWater {
                let wave = (t * 1.8 + tx as f32 * 0.3 + ty as f32 * 0.2).sin() * 0.04;
                rgb(br + noise + wave, bg + noise * 0.8 + wave, bb + noise * 0.5)
            } else {
                rgb((br + noise).clamp(0.0, 1.0),
                    (bg + noise * 0.9).clamp(0.0, 1.0),
                    (bb + noise * 0.8).clamp(0.0, 1.0))
            };

            let screen_x = tx as f32 * TILE_SIZE - cam_x;
            let screen_y = ty as f32 * TILE_SIZE - cam_y;

            draw_rectangle(screen_x, screen_y, TILE_SIZE, TILE_SIZE, dim(color, light));

            // Winter snow patches on outdoor tiles
            if is_winter {
                let outdoor = matches!(tile, TileKind::Grass | TileKind::Path | TileKind::Bracken);
                if outdoor && tile_hash(tx, ty) > 0.55 {
                    let snow_alpha = 0.25 + tile_hash(tx + 1, ty) * 0.20;
                    draw_rectangle(screen_x + 1.0, screen_y + 1.0,
                        TILE_SIZE - 2.0, TILE_SIZE - 2.0,
                        Color::new(0.88, 0.90, 0.94, snow_alpha));
                }
            }

            // Building wall edge lines
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

/// Draw a small role-specific flourish at (sx, by) with given light factor.
fn draw_role_flourish(role: Role, sx: f32, by: f32, light: f32) {
    let dark = Color::new(0.12, 0.10, 0.08, 0.85 * light);
    let gold  = Color::new(0.88, 0.72, 0.20, 0.90 * light);
    let white = Color::new(0.90, 0.90, 0.95, 0.85 * light);

    match role {
        // Miner: pickaxe — short handle + angled head
        Role::Miner => {
            draw_line(sx + 4.0, by - 2.0, sx + 9.0, by + 4.0, 1.5, dark);
            draw_line(sx + 7.0, by - 1.0, sx + 10.0, by + 2.5, 2.5, dark);
        }
        // Teacher: book — thin rectangle + spine line
        Role::Teacher => {
            draw_rectangle(sx + 4.0, by - 4.0, 6.0, 8.0, white);
            draw_line(sx + 4.5, by - 4.0, sx + 4.5, by + 4.0, 1.0, dark);
        }
        // Shopkeeper: coin — small gold circle
        Role::Shopkeeper => {
            draw_circle(sx + 7.0, by, 3.2, gold);
            draw_circle_lines(sx + 7.0, by, 3.2, 0.8, dark);
        }
        // Musician: note — circle + stem + flag
        Role::Musician => {
            draw_circle(sx + 6.5, by + 3.0, 2.2, dark);
            draw_line(sx + 8.5, by + 3.0, sx + 8.5, by - 3.0, 1.2, dark);
            draw_line(sx + 8.5, by - 3.0, sx + 12.0, by - 1.5, 1.0, dark);
        }
        // Elder: walking stick — curved top handled as two lines
        Role::Elder => {
            draw_line(sx + 5.0, by + 5.0, sx + 5.0, by - 6.0, 1.5, dark);
            draw_line(sx + 5.0, by - 6.0, sx + 7.5, by - 8.0, 1.5, dark);
        }
        // Child: ball — small bouncy dot at feet
        Role::Child => {
            draw_circle(sx + 7.0, by + 5.0, 2.5, Color::new(0.85, 0.35, 0.35, 0.75 * light));
        }
        // NewArrival: pack — small rectangle on back
        Role::NewArrival => {
            draw_rectangle(sx - ACTOR_BODY_W * 0.5 - 4.5, by - 3.0, 4.0, 6.0, dark);
            draw_rectangle(sx - ACTOR_BODY_W * 0.5 - 4.5, by - 3.0, 4.0, 1.5, white);
        }
    }
}

pub fn draw_actors(actors: &[Actor], selected: Option<usize>, cam_x: f32, cam_y: f32,
                   clock: &WorldClock) {
    let light = daylight(clock);
    let t = get_time() as f32;

    // ── Crowd clustering: count actors per tile ────────────────────────────
    let mut tile_counts: std::collections::HashMap<(i32, i32), u8> =
        std::collections::HashMap::new();
    for a in actors {
        *tile_counts.entry((a.tile_x, a.tile_y)).or_insert(0) += 1;
    }

    for actor in actors {
        let sx = actor.pixel_x - cam_x + TILE_SIZE * 0.5;
        let sy = actor.pixel_y - cam_y + TILE_SIZE * 0.5;

        if sx < -24.0 || sx > VIEWPORT_WIDTH + 24.0 { continue; }
        if sy < -24.0 || sy > SCREEN_HEIGHT  + 24.0 { continue; }

        let (cr, cg, cb) = actor.role.color();
        let body_color   = dim(rgb(cr, cg, cb), light);
        let dark_color   = dim(rgb(cr * 0.55, cg * 0.55, cb * 0.55), light);

        // Size modifier: Elder slightly bigger, Child slightly smaller
        let size_f = match actor.role { Role::Elder => 1.18, Role::Child => 0.78, _ => 1.0 };
        let bw = ACTOR_BODY_W * size_f;
        let bh = ACTOR_BODY_H * size_f;
        let hr = ACTOR_HEAD_R * size_f;

        // Idle bob
        let bob = match actor.current_action {
            Action::Walking  => (t * 12.0 + actor.id as f32).sin() * 1.5,
            Action::Singing  => (t * 8.0  + actor.id as f32).sin() * 2.5,
            _                => (t * 2.5  + actor.id as f32).sin() * 0.8,
        };

        let by = sy + bob;

        // Shadow
        draw_ellipse(sx, sy + bh * 0.4, 5.0 * size_f, 2.5,
            0.0, Color::new(0.0, 0.0, 0.0, 0.25 * light));

        // Body
        draw_rectangle(sx - bw * 0.5, by - bh * 0.5, bw, bh, body_color);

        // Head
        let head_y = by - bh * 0.5 - hr;
        draw_circle(sx, head_y, hr, body_color);

        // Eyes
        draw_circle(sx - 1.4, head_y - 0.6, 0.9, Color::new(1.0, 1.0, 1.0, 0.9 * light));
        draw_circle(sx + 1.4, head_y - 0.6, 0.9, Color::new(1.0, 1.0, 1.0, 0.9 * light));

        // Ground shadow line
        draw_rectangle(sx - bw * 0.5, by + bh * 0.5 - 2.0, bw, 2.0, dark_color);

        // Role flourish
        draw_role_flourish(actor.role, sx, by, light);

        // Crowd badge: if 3+ actors share this tile, draw a small number
        let count = tile_counts.get(&(actor.tile_x, actor.tile_y)).copied().unwrap_or(0);
        if count >= 3 && selected != Some(actor.id) {
            draw_text(&format!("{}", count), sx - 3.0, head_y - hr - 6.0, 9.0,
                Color::new(1.0, 0.90, 0.50, 0.75 * light));
        }

        // Emotion indicator
        if let Some((node_idx, val)) = actor.emotion() {
            let (er, eg, eb) = NODE_COLORS[node_idx];
            let alpha = ((val - 0.62) * 2.8).min(1.0) * light;
            draw_circle(sx + hr + 3.0, head_y - hr - 2.0, 3.2,
                Color::new(er, eg, eb, alpha));
        }

        // Selection ring + name
        if selected == Some(actor.id) {
            draw_circle_lines(sx, by - bh * 0.1, SELECTION_RING_R * size_f, 1.5, YELLOW);
            let name_x = sx - actor.name.len() as f32 * 2.8;
            draw_text(&actor.name, name_x, head_y - hr - 4.0, 12.0,
                Color::new(1.0, 1.0, 0.7, 0.90));
        }
    }
}

// ─── Relationship lines ───────────────────────────────────────────────────────

/// Draw faint lines from the selected actor to all actors they have a
/// strong bond with (|weight| > 0.55).  Positive = warm, negative = cold.
pub fn draw_relationships(actors: &[Actor], selected: Option<usize>,
                          cam_x: f32, cam_y: f32, clock: &WorldClock) {
    let id = match selected { Some(id) => id, None => return };
    let light = daylight(clock);

    let src = match actors.iter().find(|a| a.id == id) { Some(a) => a, None => return };
    let sx = src.pixel_x - cam_x + TILE_SIZE * 0.5;
    let sy = src.pixel_y - cam_y + TILE_SIZE * 0.5;

    for (partner_id, weight) in &src.relationships {
        if weight.abs() < 0.55 { continue; }
        let partner = match actors.iter().find(|a| a.id == *partner_id) {
            Some(p) => p, None => continue,
        };
        let px = partner.pixel_x - cam_x + TILE_SIZE * 0.5;
        let py = partner.pixel_y - cam_y + TILE_SIZE * 0.5;

        let alpha = (weight.abs() - 0.55) * 1.8 * light;
        let color = if *weight > 0.0 {
            Color::new(0.55, 0.90, 0.55, alpha.min(0.55)) // warm green
        } else {
            Color::new(0.90, 0.45, 0.45, alpha.min(0.45)) // cold red
        };
        draw_line(sx, sy, px, py, 1.0, color);
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

// ─── Weather overlay (drawn after tiles, before actors) ───────────────────────

pub fn draw_weather_overlay(weather: Weather, season: Season) {
    let t = get_time() as f32;

    match weather {
        Weather::Rain => {
            // Falling rain streaks
            for i in 0..140usize {
                let x = (i as f32 * 31.7 + t * 210.0).rem_euclid(VIEWPORT_WIDTH + 40.0) - 20.0;
                let y = (i as f32 * 53.1 + t * 290.0).rem_euclid(SCREEN_HEIGHT  + 40.0) - 20.0;
                draw_line(x, y, x + 1.8, y + 7.0, 1.0,
                    Color::new(0.68, 0.80, 0.92, 0.38));
            }
            draw_rectangle(0.0, 0.0, VIEWPORT_WIDTH, SCREEN_HEIGHT,
                Color::new(0.08, 0.10, 0.18, 0.12));
        }
        Weather::Fog => {
            // Slow-drifting fog patches
            for i in 0..6usize {
                let fx = (i as f32 * 190.0 + t * 18.0).rem_euclid(VIEWPORT_WIDTH + 300.0) - 150.0;
                let fy = (i as f32 * 97.0  + t *  8.0).rem_euclid(SCREEN_HEIGHT  + 100.0) - 50.0;
                draw_rectangle(fx, fy, 400.0, 130.0,
                    Color::new(0.72, 0.76, 0.78, 0.10));
            }
            draw_rectangle(0.0, 0.0, VIEWPORT_WIDTH, SCREEN_HEIGHT,
                Color::new(0.65, 0.70, 0.72, 0.13));
        }
        Weather::Overcast => {
            draw_rectangle(0.0, 0.0, VIEWPORT_WIDTH, SCREEN_HEIGHT,
                Color::new(0.05, 0.05, 0.10, 0.10));
        }
        Weather::Sunny => {}
    }

    // Winter snow flurries (season overlay, always on in winter regardless of weather)
    if matches!(season, Season::Winter) {
        for i in 0..70usize {
            let x = (i as f32 * 43.1 + t * 45.0 + (t * 0.6).sin() * 15.0)
                .rem_euclid(VIEWPORT_WIDTH);
            let y = (i as f32 * 71.3 + t * 62.0).rem_euclid(SCREEN_HEIGHT);
            draw_circle(x, y, 1.4, Color::new(0.90, 0.93, 0.96, 0.65));
        }
    }

    // Autumn falling leaves
    if matches!(season, Season::Autumn) {
        for i in 0..30usize {
            let x = (i as f32 * 83.7 + t * 30.0 + (t * 0.4 + i as f32).sin() * 20.0)
                .rem_euclid(VIEWPORT_WIDTH);
            let y = (i as f32 * 59.3 + t * 48.0).rem_euclid(SCREEN_HEIGHT);
            draw_rectangle(x, y, 3.0, 3.0,
                Color::new(0.72 + (i as f32 * 0.02).sin() * 0.15,
                           0.42, 0.15, 0.55));
        }
    }
}

// ─── Mini-map ─────────────────────────────────────────────────────────────────

const MM_X: f32 = 8.0;          // bottom-left corner x
const MM_Y: f32 = SCREEN_HEIGHT - 8.0 - 100.0; // bottom-left corner y
const MM_W: f32 = 150.0;
const MM_H: f32 = 100.0;
const MM_TILE_W: f32 = MM_W / MAP_WIDTH  as f32;
const MM_TILE_H: f32 = MM_H / MAP_HEIGHT as f32;

/// Compact tile colour for mini-map (no noise, just type).
fn mm_tile_color(tile: TileKind) -> Color {
    let (r, g, b) = tile.base_color();
    Color::new(r, g, b, 1.0)
}

pub fn draw_minimap(world: &SimWorld, cam_x: f32, cam_y: f32) {
    // Background
    draw_rectangle(MM_X - 2.0, MM_Y - 2.0, MM_W + 4.0, MM_H + 4.0,
        Color::new(0.04, 0.04, 0.06, 0.92));
    draw_rectangle_lines(MM_X - 2.0, MM_Y - 2.0, MM_W + 4.0, MM_H + 4.0, 1.0,
        Color::new(0.35, 0.35, 0.42, 1.0));

    // Tiles — drawn as solid rectangles scaled to mini size
    for ty in 0..MAP_HEIGHT {
        for tx in 0..MAP_WIDTH {
            let tile = world.tiles[ty][tx];
            let mx = MM_X + tx as f32 * MM_TILE_W;
            let my = MM_Y + ty as f32 * MM_TILE_H;
            draw_rectangle(mx, my, MM_TILE_W.ceil(), MM_TILE_H.ceil(), mm_tile_color(tile));
        }
    }

    // Actor dots
    for actor in &world.actors {
        let mx = MM_X + actor.tile_x as f32 * MM_TILE_W + MM_TILE_W * 0.5;
        let my = MM_Y + actor.tile_y as f32 * MM_TILE_H + MM_TILE_H * 0.5;
        let (r, g, b) = actor.role.color();
        draw_circle(mx, my, 1.5, Color::new(r, g, b, 1.0));
    }

    // Viewport rectangle
    let vp_x = MM_X + cam_x / (MAP_WIDTH  as f32 * TILE_SIZE) * MM_W;
    let vp_y = MM_Y + cam_y / (MAP_HEIGHT as f32 * TILE_SIZE) * MM_H;
    let vp_w = VIEWPORT_WIDTH  / (MAP_WIDTH  as f32 * TILE_SIZE) * MM_W;
    let vp_h = SCREEN_HEIGHT   / (MAP_HEIGHT as f32 * TILE_SIZE) * MM_H;
    draw_rectangle_lines(vp_x, vp_y, vp_w, vp_h, 1.0,
        Color::new(1.0, 1.0, 0.6, 0.80));

    draw_text("M: hide map", MM_X, MM_Y + MM_H + 10.0, 9.0,
        Color::new(0.45, 0.45, 0.50, 0.80));
}

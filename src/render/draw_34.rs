/// LTTP-style 3/4 view renderer.
///
/// Two-pass approach:
///   Pass 1 – flat ground tiles (water, grass, path, floor) drawn back-to-front
///   Pass 2 – elevated objects (buildings, mountains, trees) + actors, Y-sorted
///             so that front faces of closer objects correctly overlap far objects.
use macroquad::prelude::*;
use crate::constants::*;
use crate::sim::actor::{Actor, Action, Role};
use crate::sim::world::{SimWorld, TileKind, WorldClock, Season};

// Height of front face for each elevated tile type (screen pixels)
const WALL_H:  f32 = 13.0;
const CLIFF_H: f32 = 8.0;
const BUSH_H:  f32 = 9.0;
const STONE_H: f32 = 5.0;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn rgb(r: f32, g: f32, b: f32) -> Color { Color::new(r, g, b, 1.0) }
fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color { Color::new(r, g, b, a) }
fn dim(c: Color, f: f32) -> Color { Color::new(c.r * f, c.g * f, c.b * f, c.a) }

fn daylight(clock: &WorldClock) -> f32 {
    let t = clock.time_of_day;
    if t < 0.20      { 0.25 + t * 2.5 }
    else if t < 0.80 { 1.0 }
    else             { 1.0 - (t - 0.80) * 3.5 }
        .clamp(0.25, 1.0)
}

fn is_ground(tile: TileKind) -> bool {
    matches!(tile, TileKind::DeepWater | TileKind::Water
             | TileKind::Grass | TileKind::Path | TileKind::BuildingFloor)
}

fn th(x: i32, y: i32) -> f32 {
    let h = (x as u32).wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(1_103_515_245));
    (h >> 8) as f32 / (u32::MAX >> 8) as f32
}

// ─── Pass 1: Ground tiles ─────────────────────────────────────────────────────

fn draw_ground(tile: TileKind, sx: f32, sy: f32, noise: f32,
               light: f32, t: f32, tx: i32, ty: i32) {
    let ts = TILE_SIZE;
    match tile {
        TileKind::DeepWater => {
            draw_rectangle(sx, sy, ts, ts, dim(rgb(0.10, 0.28, 0.48), light));
            let w = (t * 1.3 + tx as f32 * 0.5 + ty as f32 * 0.4).sin() * 0.06;
            draw_rectangle(sx, sy, ts, ts, rgba(0.18 + w, 0.42 + w, 0.68, 0.25));
        }
        TileKind::Water => {
            let base = dim(rgb(0.22 + noise * 0.05, 0.52 + noise * 0.05, 0.75), light);
            draw_rectangle(sx, sy, ts, ts, base);
            // Wave shimmer lines
            let wave = (t * 2.0 + tx as f32 * 0.6 + ty as f32 * 0.4).sin();
            if wave > 0.55 {
                draw_line(sx + 4.0, sy + ts * 0.38,
                          sx + ts - 4.0, sy + ts * 0.38,
                          0.9, rgba(0.75, 0.90, 0.98, 0.40 * light));
            }
            if wave < -0.55 {
                draw_line(sx + 2.0, sy + ts * 0.66,
                          sx + ts - 2.0, sy + ts * 0.66,
                          0.9, rgba(0.75, 0.90, 0.98, 0.30 * light));
            }
        }
        TileKind::Grass => {
            let g = rgb(0.35 + noise * 0.06, 0.60 + noise * 0.09, 0.26 + noise * 0.04);
            draw_rectangle(sx, sy, ts, ts, dim(g, light));
            // Grass tufts
            let h = th(tx, ty);
            if h > 0.62 {
                let bx = sx + h * 13.0;
                let by_ = sy + th(tx + 1, ty) * 11.0 + 4.0;
                let gc = dim(rgb(0.26, 0.48, 0.20), light);
                draw_rectangle(bx, by_, 1.5, 4.0, gc);
                draw_rectangle(bx + 3.5, by_ + 1.5, 1.5, 3.0, gc);
                draw_rectangle(bx + 7.0, by_, 1.5, 5.0, gc);
            }
            // Small flower
            if th(tx + 3, ty) > 0.87 {
                let fx = sx + th(tx, ty + 2) * 15.0 + 3.0;
                let fy = sy + th(tx + 2, ty) * 14.0 + 4.0;
                draw_circle(fx, fy, 1.5, dim(rgb(0.88, 0.78, 0.25), light));
            }
        }
        TileKind::Path => {
            let base = dim(rgb(0.70 + noise * 0.05, 0.60 + noise * 0.04, 0.44), light);
            draw_rectangle(sx, sy, ts, ts, base);
            // Stone brick mortar lines
            let m = dim(rgb(0.56, 0.48, 0.34), light * 0.8);
            draw_line(sx, sy + ts * 0.5, sx + ts, sy + ts * 0.5, 0.8, m);
            let off = if (ty % 2) == 0 { 0.0 } else { ts * 0.5 };
            draw_line(sx + off,          sy,           sx + off,          sy + ts * 0.5, 0.8, m);
            draw_line(sx + off + ts*0.5, sy + ts * 0.5, sx + off + ts*0.5, sy + ts,       0.8, m);
        }
        TileKind::BuildingFloor => {
            draw_rectangle(sx, sy, ts, ts, dim(rgb(0.32, 0.26, 0.18), light));
            // Wooden floor planks
            let pc = dim(rgb(0.25, 0.20, 0.14), light * 0.75);
            for i in 1..4 {
                draw_line(sx, sy + ts * i as f32 * 0.25,
                          sx + ts, sy + ts * i as f32 * 0.25, 0.7, pc);
            }
        }
        _ => {
            let (r, g, b) = tile.base_color();
            draw_rectangle(sx, sy, ts, ts, dim(rgb(r, g, b), light));
        }
    }
}

// ─── Pass 2: Elevated tiles ───────────────────────────────────────────────────

fn draw_elevated(tile: TileKind, sx: f32, sy: f32, noise: f32,
                 light: f32, _t: f32, tx: i32, ty: i32) {
    let ts = TILE_SIZE;
    match tile {
        TileKind::BuildingWall => {
            // ── Roof (top face) ──────────────────────────────────────────────
            let roof_mid  = rgb(0.52, 0.32, 0.20);
            let roof_hi   = rgb(0.65, 0.44, 0.28);
            let roof_dark = rgb(0.38, 0.22, 0.13);
            draw_rectangle(sx, sy, ts, ts, dim(roof_mid, light));
            // Ridge band
            draw_rectangle(sx + 2.0, sy + ts * 0.28, ts - 4.0, ts * 0.32,
                dim(roof_hi, light));
            // Eaves shadow at bottom
            draw_rectangle(sx, sy + ts * 0.80, ts, ts * 0.20, dim(roof_dark, light));
            // Roof border
            draw_rectangle_lines(sx, sy, ts, ts, 1.0,
                dim(rgb(0.28, 0.18, 0.10), light * 0.6));

            // Chimney (probabilistic)
            if th(tx, ty) > 0.55 {
                let cx = sx + ts * 0.68;
                draw_rectangle(cx, sy - 5.0, 5.0, 8.0,
                    dim(rgb(0.42, 0.35, 0.28), light));
                draw_rectangle(cx - 1.0, sy - 6.5, 7.0, 2.0,
                    dim(rgb(0.25, 0.20, 0.16), light));
                // Smoke hint
                draw_circle(cx + 2.5, sy - 9.0, 2.0,
                    rgba(0.65, 0.65, 0.68, 0.25 * light));
            }

            // ── Front wall face ──────────────────────────────────────────────
            let wall   = rgb(0.84, 0.78, 0.65);
            let shadow = rgb(0.60, 0.56, 0.46); // under eave shadow
            draw_rectangle(sx, sy + ts, ts, WALL_H, dim(wall, light));
            draw_rectangle(sx, sy + ts, ts, 2.5, dim(shadow, light));
            // Horizontal mortar line
            draw_line(sx, sy + ts + WALL_H * 0.50,
                      sx + ts, sy + ts + WALL_H * 0.50,
                      0.5, dim(rgb(0.68, 0.62, 0.52), light * 0.55));

            // Windows (only if tile is wide enough to not be a corner stub)
            let win_size = 4.5;
            let win_y    = sy + ts + 2.5;
            let glass    = dim(rgb(0.72, 0.86, 0.92), light);
            let frame    = dim(rgb(0.28, 0.20, 0.12), light);
            // Left window
            draw_rectangle(sx + 2.5, win_y, win_size, win_size, glass);
            draw_rectangle_lines(sx + 2.5, win_y, win_size, win_size, 0.8, frame);
            draw_line(sx + 2.5 + win_size * 0.5, win_y,
                      sx + 2.5 + win_size * 0.5, win_y + win_size, 0.5, frame);
            // Right window
            draw_rectangle(sx + ts - 7.5, win_y, win_size, win_size, glass);
            draw_rectangle_lines(sx + ts - 7.5, win_y, win_size, win_size, 0.8, frame);
            draw_line(sx + ts - 7.5 + win_size * 0.5, win_y,
                      sx + ts - 7.5 + win_size * 0.5, win_y + win_size, 0.5, frame);

            // Door (probabilistic — not every wall tile is a front face)
            if th(tx + 7, ty) > 0.68 {
                let dx = sx + ts * 0.5 - 2.5;
                let dy = sy + ts + WALL_H - 7.5;
                draw_rectangle(dx, dy, 5.0, 8.0,
                    dim(rgb(0.30, 0.18, 0.10), light)); // dark wood
                draw_rectangle_lines(dx, dy, 5.0, 8.0, 0.7, frame);
                draw_circle(dx + 4.0, dy + 4.5, 0.8, // knob
                    dim(rgb(0.78, 0.65, 0.25), light));
            }

            // Bottom ledge (base of wall)
            draw_rectangle(sx - 1.0, sy + ts + WALL_H - 1.5, ts + 2.0, 2.0,
                dim(rgb(0.50, 0.45, 0.35), light));
        }

        TileKind::Mountain => {
            // Top face
            let rock = rgb(0.54 + noise * 0.06, 0.52 + noise * 0.04, 0.50);
            draw_rectangle(sx, sy, ts, ts, dim(rock, light));
            // Snow cap highlight
            let snow_r = 5.0 + noise * 3.0;
            draw_circle(sx + ts * 0.45 + noise * 4.0, sy + ts * 0.28,
                snow_r, dim(rgb(0.78, 0.78, 0.82), light));
            draw_circle(sx + ts * 0.45 + noise * 4.0, sy + ts * 0.28,
                snow_r * 0.55, dim(rgb(0.90, 0.90, 0.94), light));
            // Rocky bump
            draw_circle(sx + ts * 0.68, sy + ts * 0.58, 3.0,
                dim(rgb(0.40, 0.38, 0.36), light));

            // Cliff front face
            let cliff = rgb(0.30, 0.28, 0.26);
            draw_rectangle(sx, sy + ts, ts, CLIFF_H, dim(cliff, light));
            // Crack lines
            draw_line(sx + ts * 0.22, sy + ts,
                      sx + ts * 0.18, sy + ts + CLIFF_H, 0.7,
                      dim(rgb(0.20, 0.18, 0.16), light * 0.55));
            draw_line(sx + ts * 0.62, sy + ts,
                      sx + ts * 0.68, sy + ts + CLIFF_H, 0.7,
                      dim(rgb(0.20, 0.18, 0.16), light * 0.55));
            // Lighter highlight strip at very bottom of cliff
            draw_rectangle(sx, sy + ts + CLIFF_H - 1.5, ts, 1.5,
                dim(rgb(0.42, 0.38, 0.34), light * 0.6));
        }

        TileKind::Stone => {
            let s = rgb(0.46 + noise * 0.08, 0.44 + noise * 0.06, 0.42);
            draw_rectangle(sx, sy, ts, ts, dim(s, light));
            draw_circle(sx + ts * 0.38, sy + ts * 0.38, 4.5,
                dim(rgb(0.60, 0.58, 0.55), light)); // highlight
            draw_circle(sx + ts * 0.65, sy + ts * 0.60, 2.8,
                dim(rgb(0.36, 0.34, 0.32), light)); // shadow

            // Stone front face (short)
            draw_rectangle(sx, sy + ts, ts, STONE_H,
                dim(rgb(0.32, 0.30, 0.28), light));
            draw_line(sx + ts * 0.40, sy + ts, sx + ts * 0.38, sy + ts + STONE_H,
                0.6, dim(rgb(0.22, 0.20, 0.18), light * 0.5));
        }

        TileKind::Bracken => {
            // Ground beneath canopy
            draw_rectangle(sx, sy, ts, ts,
                dim(rgb(0.32 + noise * 0.04, 0.54 + noise * 0.06, 0.24), light));

            // Canopy — layered circles for tree/shrub look
            let cx = sx + ts * 0.50;
            let cy = sy + ts * 0.42;
            let r  = ts * 0.36 + noise * 3.0;
            draw_circle(cx, cy, r + 2.0, dim(rgb(0.15, 0.32, 0.12), light)); // dark outer
            draw_circle(cx, cy, r,        dim(rgb(0.22 + noise*0.06, 0.44 + noise*0.08, 0.18), light));
            draw_circle(cx - ts*0.10, cy - ts*0.10, r * 0.58,
                dim(rgb(0.34, 0.60, 0.24), light)); // mid highlight
            draw_circle(cx - ts*0.16, cy - ts*0.16, r * 0.30,
                dim(rgb(0.45, 0.72, 0.30), light)); // bright top

            // Trunk (front face below tile)
            let trunk_x = sx + ts * 0.5 - 2.5;
            draw_rectangle(trunk_x, sy + ts, 5.0, BUSH_H,
                dim(rgb(0.30, 0.18, 0.08), light));
            // Root flare
            draw_rectangle(trunk_x - 1.5, sy + ts + BUSH_H - 2.0, 8.0, 2.0,
                dim(rgb(0.26, 0.15, 0.06), light * 0.7));
        }

        _ => {
            let (r, g, b) = tile.base_color();
            draw_rectangle(sx, sy, ts, ts, dim(rgb(r, g, b), light));
        }
    }
}

// ─── Actor rendering (3/4 view — taller, more front-facing) ──────────────────

fn draw_actor_34(actor: &Actor, sx: f32, sy: f32, light: f32,
                 t: f32, selected: Option<usize>) {
    let (cr, cg, cb) = actor.role.color();
    let body  = dim(rgb(cr, cg, cb), light);
    let dark  = dim(rgb(cr * 0.50, cg * 0.50, cb * 0.50), light);
    let pants = dim(rgb(cr * 0.45, cg * 0.42, cb * 0.38), light);

    let sf = match actor.role { Role::Elder => 1.15, Role::Child => 0.80, _ => 1.0 };
    let bw = 8.0 * sf;
    let bh = 14.0 * sf;  // taller in 3/4 view
    let hr = 5.0 * sf;

    let bob = match actor.current_action {
        Action::Walking => (t * 12.0 + actor.id as f32).sin() * 1.5,
        Action::Singing => (t * 8.0  + actor.id as f32).sin() * 2.5,
        _               => (t * 2.5  + actor.id as f32).sin() * 0.8,
    };
    let by = sy + bob;

    // Ground shadow (elongated for 3/4 feel)
    draw_ellipse(sx + 2.0, sy + bh * 0.30, 5.5 * sf, 2.0,
        0.0, Color::new(0.0, 0.0, 0.0, 0.28 * light));

    // Legs (visible below body in 3/4)
    let leg_top = by + bh * 0.25;
    draw_rectangle(sx - bw * 0.22 - 1.0, leg_top, 2.5 * sf, bh * 0.42, pants);
    draw_rectangle(sx + bw * 0.06,        leg_top, 2.5 * sf, bh * 0.42, pants);

    // Body (torso)
    draw_rectangle(sx - bw * 0.5, by - bh * 0.18, bw, bh * 0.52, body);

    // Head
    let head_y = by - bh * 0.18 - hr;
    draw_circle(sx, head_y, hr, body);

    // Eyes with pupils
    draw_circle(sx - 1.7, head_y + 0.5, 1.2, Color::new(1.0, 1.0, 1.0, 0.9 * light));
    draw_circle(sx + 1.7, head_y + 0.5, 1.2, Color::new(1.0, 1.0, 1.0, 0.9 * light));
    draw_circle(sx - 1.7, head_y + 0.5, 0.55, Color::new(0.08, 0.08, 0.10, 0.95 * light));
    draw_circle(sx + 1.7, head_y + 0.5, 0.55, Color::new(0.08, 0.08, 0.10, 0.95 * light));

    // Ground shadow line below feet
    draw_rectangle(sx - bw * 0.5, by + bh * 0.25 + bh * 0.42 - 1.5,
        bw, 1.5, dark);

    // Role flourish (same as draw.rs version)
    draw_role_34(actor.role, sx, by, light);

    // Emotion dot
    if let Some((node_idx, val)) = actor.emotion() {
        let (er, eg, eb) = NODE_COLORS[node_idx];
        let alpha = ((val - 0.62) * 2.8).min(1.0) * light;
        draw_circle(sx + hr + 3.0, head_y - hr - 1.0, 3.0,
            Color::new(er, eg, eb, alpha));
    }

    // Selection ring + name
    if selected == Some(actor.id) {
        draw_circle_lines(sx, by, 13.0 * sf, 1.5, YELLOW);
        let name_x = sx - actor.name.len() as f32 * 2.8;
        draw_text(&actor.name, name_x, head_y - hr - 3.0, 12.0,
            Color::new(1.0, 1.0, 0.7, 0.90));
    }
}

fn draw_role_34(role: Role, sx: f32, by: f32, light: f32) {
    let dark = Color::new(0.12, 0.10, 0.08, 0.85 * light);
    let gold  = Color::new(0.88, 0.72, 0.20, 0.90 * light);
    let white = Color::new(0.90, 0.90, 0.95, 0.85 * light);
    match role {
        Role::Miner      => {
            draw_line(sx + 4.0, by - 2.0, sx + 9.0, by + 4.0, 1.5, dark);
            draw_line(sx + 7.0, by - 1.0, sx + 10.5, by + 2.5, 2.5, dark);
        }
        Role::Teacher    => {
            draw_rectangle(sx + 4.0, by - 4.0, 6.0, 8.0, white);
            draw_line(sx + 4.5, by - 4.0, sx + 4.5, by + 4.0, 1.0, dark);
        }
        Role::Shopkeeper => {
            draw_circle(sx + 7.5, by, 3.2, gold);
            draw_circle_lines(sx + 7.5, by, 3.2, 0.8, dark);
        }
        Role::Musician   => {
            draw_circle(sx + 7.0, by + 3.0, 2.2, dark);
            draw_line(sx + 9.0, by + 3.0, sx + 9.0, by - 3.5, 1.2, dark);
            draw_line(sx + 9.0, by - 3.5, sx + 12.5, by - 2.0, 1.0, dark);
        }
        Role::Elder      => {
            draw_line(sx + 5.5, by + 5.0, sx + 5.5, by - 7.0, 1.5, dark);
            draw_line(sx + 5.5, by - 7.0, sx + 8.0, by - 9.0, 1.5, dark);
        }
        Role::Child      => {
            draw_circle(sx + 7.5, by + 5.5, 2.5,
                Color::new(0.85, 0.35, 0.35, 0.75 * light));
        }
        Role::NewArrival => {
            draw_rectangle(sx - 8.5, by - 3.5, 4.0, 6.0, dark);
            draw_rectangle(sx - 8.5, by - 3.5, 4.0, 1.5, white);
        }
    }
}

// ─── Location labels (same style, slightly offset for front-face depth) ───────

pub fn draw_location_labels_34(world: &SimWorld, cam_x: f32, cam_y: f32) {
    let light = daylight(&world.clock);
    let color = Color::new(1.0, 1.0, 0.85, 0.65 * light);
    for loc in &world.locations {
        let sx = loc.tile_x as f32 * TILE_SIZE - cam_x - 10.0;
        let sy = loc.tile_y as f32 * TILE_SIZE - cam_y - 6.0;
        if sx < -80.0 || sx > VIEWPORT_WIDTH + 80.0 { continue; }
        if sy < -20.0 || sy > SCREEN_HEIGHT  + 20.0 { continue; }
        draw_text(loc.kind.label(), sx, sy, 11.0, color);
    }
}

// ─── Main draw entry point ────────────────────────────────────────────────────

pub fn draw_world_34(world: &SimWorld, selected: Option<usize>,
                     cam_x: f32, cam_y: f32) {
    let t     = get_time() as f32;
    let light = daylight(&world.clock) * world.weather.brightness();

    let x0 = (cam_x / TILE_SIZE) as i32 - 1;
    let y0 = (cam_y / TILE_SIZE) as i32 - 1;
    let x1 = x0 + (VIEWPORT_WIDTH / TILE_SIZE) as i32 + 2;
    let y1 = y0 + (SCREEN_HEIGHT  / TILE_SIZE) as i32 + 4; // extra rows for front faces

    // ── Pass 1: Ground ───────────────────────────────────────────────────────
    for ty in y0..=y1 {
        if ty < 0 || ty >= MAP_HEIGHT as i32 { continue; }
        for tx in x0..=x1 {
            if tx < 0 || tx >= MAP_WIDTH as i32 { continue; }
            let tile = world.tiles[ty as usize][tx as usize];
            if is_ground(tile) {
                let sx = tx as f32 * TILE_SIZE - cam_x;
                let sy = ty as f32 * TILE_SIZE - cam_y;
                let n  = world.tile_noise[ty as usize][tx as usize];
                draw_ground(tile, sx, sy, n, light, t, tx, ty);
            }
        }
    }

    // ── Pass 2: Elevated tiles + actors, back-to-front ───────────────────────
    for ty in y0..=y1 {
        if ty < 0 || ty >= MAP_HEIGHT as i32 { continue; }

        // Elevated tiles in this row
        for tx in x0..=x1 {
            if tx < 0 || tx >= MAP_WIDTH as i32 { continue; }
            let tile = world.tiles[ty as usize][tx as usize];
            if !is_ground(tile) {
                let sx = tx as f32 * TILE_SIZE - cam_x;
                let sy = ty as f32 * TILE_SIZE - cam_y;
                if sx < -50.0 || sx > VIEWPORT_WIDTH + 50.0 { continue; }
                if sy < -50.0 || sy > SCREEN_HEIGHT  + 50.0 { continue; }
                let n = world.tile_noise[ty as usize][tx as usize];
                draw_elevated(tile, sx, sy, n, light, t, tx, ty);
            }
        }

        // Actors whose tile row matches
        for actor in &world.actors {
            if actor.tile_y != ty { continue; }
            let sx = actor.pixel_x - cam_x + TILE_SIZE * 0.5;
            let sy = actor.pixel_y - cam_y + TILE_SIZE * 0.5;
            if sx < -24.0 || sx > VIEWPORT_WIDTH + 24.0 { continue; }
            if sy < -24.0 || sy > SCREEN_HEIGHT  + 24.0 { continue; }
            draw_actor_34(actor, sx, sy, light, t, selected);
        }
    }

    draw_location_labels_34(world, cam_x, cam_y);
}

// ─── Hit test (same geometry) ─────────────────────────────────────────────────

pub fn actor_at_screen_34(actors: &[Actor], sx: f32, sy: f32,
                           cam_x: f32, cam_y: f32) -> Option<usize> {
    let wx = sx + cam_x;
    let wy = sy + cam_y;
    let mut best: Option<(f32, usize)> = None;
    for actor in actors {
        let ax = actor.pixel_x + TILE_SIZE * 0.5;
        let ay = actor.pixel_y + TILE_SIZE * 0.5;
        let d  = ((wx - ax).powi(2) + (wy - ay).powi(2)).sqrt();
        if d < 18.0 {
            if best.map_or(true, |(bd, _)| d < bd) {
                best = Some((d, actor.id));
            }
        }
    }
    best.map(|(_, id)| id)
}

/// Enhanced top-down renderer.
///
/// Same overhead perspective as the main view but with richer procedural art:
/// buildings show a pitched roof with ridge/eaves/chimney, trees render as
/// layered canopy circles, paths show stone-brick mortar lines, water has
/// animated ripple arcs, and actors get crisper detail.
use macroquad::prelude::*;
use crate::constants::*;
use crate::sim::actor::{Actor, Action, Role};
use crate::sim::world::{SimWorld, TileKind, WorldClock, Season};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn rgb(r: f32, g: f32, b: f32) -> Color { Color::new(r, g, b, 1.0) }
fn dim(c: Color, f: f32) -> Color { Color::new(c.r * f, c.g * f, c.b * f, c.a) }

fn daylight(clock: &WorldClock) -> f32 {
    let t = clock.time_of_day;
    if t < 0.20      { 0.25 + t * 2.5 }
    else if t < 0.80 { 1.0 }
    else             { 1.0 - (t - 0.80) * 3.5 }
        .clamp(0.25, 1.0)
}

fn th(x: i32, y: i32) -> f32 {
    let h = (x as u32).wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(1_103_515_245));
    (h >> 8) as f32 / (u32::MAX >> 8) as f32
}

// ─── Individual tile drawing ──────────────────────────────────────────────────

fn draw_tile_enhanced(tile: TileKind, sx: f32, sy: f32, noise: f32,
                      light: f32, t: f32, tx: i32, ty: i32) {
    let ts = TILE_SIZE;
    match tile {
        TileKind::DeepWater => {
            draw_rectangle(sx, sy, ts, ts, dim(rgb(0.10, 0.28, 0.48), light));
            let w = (t * 1.2 + tx as f32 * 0.4 + ty as f32 * 0.3).sin() * 0.05;
            let wc = Color::new(0.20 + w, 0.44 + w, 0.66, 0.28);
            draw_rectangle(sx, sy, ts, ts, wc);
        }
        TileKind::Water => {
            let base = rgb(0.20 + noise * 0.05, 0.50 + noise * 0.06, 0.72);
            draw_rectangle(sx, sy, ts, ts, dim(base, light));
            // Animated ripple arcs
            let phase = t * 0.9 + (tx * 7 + ty * 5) as f32 * 0.4;
            let r = (phase.sin() * 0.5 + 0.5) * ts * 0.35 + ts * 0.1;
            draw_circle_lines(sx + ts * 0.5, sy + ts * 0.5, r, 0.7,
                Color::new(0.70, 0.88, 0.96, 0.30 * light));
        }
        TileKind::Grass => {
            let g = rgb(0.34 + noise * 0.07, 0.58 + noise * 0.09, 0.25 + noise * 0.04);
            draw_rectangle(sx, sy, ts, ts, dim(g, light));
            // Grass blade clusters
            let h = th(tx, ty);
            if h > 0.60 {
                let gc = dim(rgb(0.25, 0.48, 0.19), light);
                let bx = sx + h * 14.0;
                let by_ = sy + th(tx + 1, ty) * 12.0 + 3.0;
                draw_rectangle(bx, by_, 1.5, 5.0, gc);
                draw_rectangle(bx + 4.0, by_ + 1.0, 1.5, 4.0, gc);
                draw_rectangle(bx + 8.0, by_ - 0.5, 1.5, 5.5, gc);
            }
            // Flower
            if th(tx + 3, ty) > 0.88 {
                let fx = sx + th(tx, ty + 2) * 15.0 + 4.0;
                let fy = sy + th(tx + 2, ty) * 14.0 + 4.0;
                draw_circle(fx, fy, 2.0, dim(rgb(0.90, 0.80, 0.28), light));
                draw_circle(fx, fy, 0.8, dim(rgb(1.0, 0.95, 0.60), light));
            }
        }
        TileKind::Path => {
            let base = rgb(0.70 + noise * 0.05, 0.58 + noise * 0.04, 0.42);
            draw_rectangle(sx, sy, ts, ts, dim(base, light));
            let m = dim(rgb(0.54, 0.44, 0.30), light * 0.75);
            // Mortar grid
            draw_line(sx, sy + ts * 0.5, sx + ts, sy + ts * 0.5, 0.8, m);
            let off = if ty % 2 == 0 { 0.0 } else { ts * 0.5 };
            draw_line(sx + off,          sy,           sx + off,          sy + ts * 0.5, 0.8, m);
            draw_line(sx + off + ts*0.5, sy + ts * 0.5, sx + off + ts*0.5, sy + ts,       0.8, m);
        }
        TileKind::Bracken => {
            // Base ground
            draw_rectangle(sx, sy, ts, ts,
                dim(rgb(0.30 + noise*0.04, 0.50 + noise*0.06, 0.22), light));
            // Canopy (overhead tree view)
            let cx = sx + ts * 0.50;
            let cy = sy + ts * 0.48;
            let r  = ts * 0.38 + noise * 2.5;
            draw_circle(cx, cy, r + 2.0, dim(rgb(0.14, 0.30, 0.10), light));
            draw_circle(cx, cy, r,        dim(rgb(0.22 + noise*0.05, 0.44 + noise*0.07, 0.16), light));
            draw_circle(cx - ts*0.12, cy - ts*0.12, r * 0.55,
                dim(rgb(0.35, 0.60, 0.24), light));
            draw_circle(cx - ts*0.18, cy - ts*0.18, r * 0.28,
                dim(rgb(0.50, 0.75, 0.32), light));
        }
        TileKind::Stone => {
            let s = rgb(0.44 + noise * 0.08, 0.42 + noise * 0.05, 0.40);
            draw_rectangle(sx, sy, ts, ts, dim(s, light));
            draw_circle(sx + ts * 0.35, sy + ts * 0.35, 5.0,
                dim(rgb(0.60, 0.58, 0.54), light));
            draw_circle(sx + ts * 0.65, sy + ts * 0.60, 3.0,
                dim(rgb(0.32, 0.30, 0.28), light));
        }
        TileKind::Mountain => {
            let m = rgb(0.30 + noise * 0.05, 0.28 + noise * 0.04, 0.32);
            draw_rectangle(sx, sy, ts, ts, dim(m, light));
            // Snow peak from above
            draw_circle(sx + ts * 0.50 + noise * 3.0, sy + ts * 0.42,
                5.0 + noise * 2.5, dim(rgb(0.82, 0.84, 0.88), light));
            draw_circle(sx + ts * 0.50 + noise * 3.0, sy + ts * 0.42,
                2.5, dim(rgb(0.92, 0.94, 0.96), light));
            // Shadow side
            draw_triangle(
                vec2(sx + ts * 0.75, sy + ts * 0.30),
                vec2(sx + ts * 0.95, sy + ts * 0.85),
                vec2(sx + ts * 0.50, sy + ts * 0.85),
                dim(rgb(0.20, 0.18, 0.22), light * 0.6));
        }
        TileKind::BuildingFloor => {
            draw_rectangle(sx, sy, ts, ts, dim(rgb(0.34, 0.27, 0.19), light));
            let pc = dim(rgb(0.26, 0.20, 0.13), light * 0.7);
            for i in 1..4 {
                draw_line(sx, sy + ts * i as f32 * 0.25,
                          sx + ts, sy + ts * i as f32 * 0.25, 0.7, pc);
            }
        }
        TileKind::BuildingWall => {
            // Roof visible from directly above (LTTP-ish)
            let roof = rgb(0.48 + noise * 0.04, 0.29 + noise * 0.03, 0.17);
            draw_rectangle(sx, sy, ts, ts, dim(roof, light));
            // Ridge line (slightly lighter band across center)
            let ridge = rgb(0.62, 0.40, 0.25);
            draw_rectangle(sx + 1.5, sy + ts * 0.25, ts - 3.0, ts * 0.40,
                dim(ridge, light));
            // Eaves (darker at edges)
            let eave = rgb(0.32, 0.18, 0.10);
            draw_rectangle(sx,             sy,             ts,   2.0, dim(eave, light));
            draw_rectangle(sx,             sy + ts - 2.0,  ts,   2.0, dim(eave, light));
            draw_rectangle(sx,             sy,             2.0,  ts,   dim(eave, light));
            draw_rectangle(sx + ts - 2.0,  sy,             2.0,  ts,   dim(eave, light));
            // Chimney (small square on top)
            if th(tx, ty) > 0.55 {
                draw_rectangle(sx + ts * 0.65, sy + ts * 0.12, 4.5, 5.5,
                    dim(rgb(0.38, 0.30, 0.22), light));
                draw_rectangle(sx + ts * 0.64, sy + ts * 0.10, 6.0, 1.5,
                    dim(rgb(0.22, 0.18, 0.14), light));
            }
            // Two tiny windows visible from above (just dots)
            let wc = dim(rgb(0.70, 0.85, 0.92), light);
            draw_rectangle(sx + 3.0, sy + ts * 0.60, 4.0, 3.0, wc);
            draw_rectangle(sx + ts - 7.0, sy + ts * 0.60, 4.0, 3.0, wc);
        }
        TileKind::Farmland => {
            // Tilled soil — dark brown rows
            draw_rectangle(sx, sy, ts, ts, dim(rgb(0.42, 0.28, 0.15), light));
            let row_c = dim(rgb(0.30, 0.20, 0.10), light * 0.7);
            for i in 1..4 {
                draw_line(sx, sy + ts * i as f32 * 0.25,
                          sx + ts, sy + ts * i as f32 * 0.25, 1.0, row_c);
            }
        }
        TileKind::Fence => {
            // Wooden fence posts
            draw_rectangle(sx, sy, ts, ts, dim(rgb(0.28, 0.50, 0.26), light));
            let pc = dim(rgb(0.60, 0.44, 0.24), light);
            draw_rectangle(sx + ts * 0.15, sy + ts * 0.20, 3.0, ts * 0.60, pc);
            draw_rectangle(sx + ts * 0.55, sy + ts * 0.20, 3.0, ts * 0.60, pc);
            draw_rectangle(sx + ts * 0.10, sy + ts * 0.35, ts * 0.80, 2.0, pc);
            draw_rectangle(sx + ts * 0.10, sy + ts * 0.55, ts * 0.80, 2.0, pc);
        }
        TileKind::Cobble => {
            // Stone cobble courtyard
            let s = rgb(0.44 + noise * 0.06, 0.44 + noise * 0.04, 0.48 + noise * 0.04);
            draw_rectangle(sx, sy, ts, ts, dim(s, light));
        }
    }
}

// ─── Actor rendering (enhanced, same overhead angle) ─────────────────────────

fn draw_actor_enhanced(actor: &Actor, sx: f32, sy: f32, light: f32,
                       t: f32, selected: Option<usize>) {
    let (cr, cg, cb) = actor.role.color();
    let body  = dim(rgb(cr, cg, cb), light);
    let dark  = dim(rgb(cr * 0.50, cg * 0.50, cb * 0.50), light);

    let sf = match actor.role { Role::Elder => 1.18, Role::Child => 0.78, _ => 1.0 };
    let bw = ACTOR_BODY_W * sf;
    let bh = ACTOR_BODY_H * sf;
    let hr = ACTOR_HEAD_R * sf;

    let bob = match actor.current_action {
        Action::Walking => (t * 12.0 + actor.id as f32).sin() * 1.5,
        Action::Singing => (t * 8.0  + actor.id as f32).sin() * 2.5,
        _               => (t * 2.5  + actor.id as f32).sin() * 0.8,
    };
    let by = sy + bob;

    draw_ellipse(sx, sy + bh * 0.40, 5.0 * sf, 2.5,
        0.0, Color::new(0.0, 0.0, 0.0, 0.26 * light));
    draw_rectangle(sx - bw * 0.5, by - bh * 0.5, bw, bh, body);
    let head_y = by - bh * 0.5 - hr;
    draw_circle(sx, head_y, hr, body);
    draw_circle(sx - 1.5, head_y - 0.5, 1.0, Color::new(1.0, 1.0, 1.0, 0.9 * light));
    draw_circle(sx + 1.5, head_y - 0.5, 1.0, Color::new(1.0, 1.0, 1.0, 0.9 * light));
    draw_rectangle(sx - bw * 0.5, by + bh * 0.5 - 2.0, bw, 2.0, dark);

    // Role glyphs
    let gold  = Color::new(0.88, 0.72, 0.20, 0.90 * light);
    let white = Color::new(0.90, 0.90, 0.95, 0.85 * light);
    match actor.role {
        Role::Miner      => { draw_line(sx+4.0, by-2.0, sx+9.0, by+4.0, 1.5, dark); }
        Role::Teacher    => { draw_rectangle(sx+4.0, by-4.0, 6.0, 8.0, white);
                              draw_line(sx+4.5, by-4.0, sx+4.5, by+4.0, 1.0, dark); }
        Role::Shopkeeper => { draw_circle(sx+7.5, by, 3.2, gold); }
        Role::Musician   => { draw_circle(sx+7.0, by+3.0, 2.2, dark);
                              draw_line(sx+9.0, by+3.0, sx+9.0, by-3.5, 1.2, dark); }
        Role::Elder      => { draw_line(sx+5.5, by+5.0, sx+5.5, by-7.0, 1.5, dark); }
        Role::Child      => { draw_circle(sx+7.5, by+5.5, 2.5,
                                  Color::new(0.85, 0.35, 0.35, 0.75 * light)); }
        Role::NewArrival => { draw_rectangle(sx-8.5, by-3.5, 4.0, 6.0, dark); }
    }

    if let Some((ni, val)) = actor.emotion() {
        let (er, eg, eb) = NODE_COLORS[ni];
        let alpha = ((val - 0.62) * 2.8).min(1.0) * light;
        draw_circle(sx + hr + 3.0, head_y - hr - 2.0, 3.2,
            Color::new(er, eg, eb, alpha));
    }

    if selected == Some(actor.id) {
        draw_circle_lines(sx, by - bh * 0.1, SELECTION_RING_R * sf, 1.5, YELLOW);
        let nx = sx - actor.name.len() as f32 * 2.8;
        draw_text(&actor.name, nx, head_y - hr - 4.0, 12.0,
            Color::new(1.0, 1.0, 0.7, 0.90));
    }
}

// ─── Main draw entry point ────────────────────────────────────────────────────

pub fn draw_world_enhanced(world: &SimWorld, selected: Option<usize>,
                           cam_x: f32, cam_y: f32) {
    let t     = get_time() as f32;
    let light = daylight(&world.clock) * world.weather.brightness();
    let season = world.clock.season;

    let x0 = (cam_x / TILE_SIZE) as i32 - 1;
    let y0 = (cam_y / TILE_SIZE) as i32 - 1;
    let x1 = x0 + (VIEWPORT_WIDTH / TILE_SIZE) as i32 + 2;
    let y1 = y0 + (SCREEN_HEIGHT  / TILE_SIZE) as i32 + 2;

    for ty in y0..=y1 {
        if ty < 0 || ty >= MAP_HEIGHT as i32 { continue; }
        for tx in x0..=x1 {
            if tx < 0 || tx >= MAP_WIDTH as i32 { continue; }
            let tile = world.tiles[ty as usize][tx as usize];
            let sx   = tx as f32 * TILE_SIZE - cam_x;
            let sy   = ty as f32 * TILE_SIZE - cam_y;
            let n    = world.tile_noise[ty as usize][tx as usize];
            draw_tile_enhanced(tile, sx, sy, n, light, t, tx, ty);

            // Winter snow patches
            if matches!(season, Season::Winter)
                && matches!(tile, TileKind::Grass | TileKind::Path | TileKind::Bracken)
                && th(tx, ty) > 0.55
            {
                let a = 0.22 + th(tx + 1, ty) * 0.22;
                draw_rectangle(sx + 1.0, sy + 1.0, TILE_SIZE - 2.0, TILE_SIZE - 2.0,
                    Color::new(0.88, 0.90, 0.94, a));
            }
        }
    }

    // Actors
    let act_light = daylight(&world.clock);
    for actor in &world.actors {
        let sx = actor.pixel_x - cam_x + TILE_SIZE * 0.5;
        let sy = actor.pixel_y - cam_y + TILE_SIZE * 0.5;
        if sx < -24.0 || sx > VIEWPORT_WIDTH + 24.0 { continue; }
        if sy < -24.0 || sy > SCREEN_HEIGHT  + 24.0 { continue; }
        draw_actor_enhanced(actor, sx, sy, act_light, t, selected);
    }

    // Location labels
    let lc = Color::new(1.0, 1.0, 0.85, 0.65 * light);
    for loc in &world.locations {
        let sx = loc.tile_x as f32 * TILE_SIZE - cam_x - 10.0;
        let sy = loc.tile_y as f32 * TILE_SIZE - cam_y - 6.0;
        if sx < -80.0 || sx > VIEWPORT_WIDTH + 80.0 { continue; }
        if sy < -20.0 || sy > SCREEN_HEIGHT  + 20.0 { continue; }
        draw_text(loc.kind.label(), sx, sy, 11.0, lc);
    }
}

pub fn actor_at_screen_enh(actors: &[Actor], sx: f32, sy: f32,
                            cam_x: f32, cam_y: f32) -> Option<usize> {
    let wx = sx + cam_x;
    let wy = sy + cam_y;
    let mut best: Option<(f32, usize)> = None;
    for actor in actors {
        let ax = actor.pixel_x + TILE_SIZE * 0.5;
        let ay = actor.pixel_y + TILE_SIZE * 0.5;
        let d  = ((wx - ax).powi(2) + (wy - ay).powi(2)).sqrt();
        if d < 16.0 {
            if best.map_or(true, |(bd, _)| d < bd) {
                best = Some((d, actor.id));
            }
        }
    }
    best.map(|(_, id)| id)
}

/// Macroquad-3D renderer.
///
/// Uses macroquad's built-in Camera3D + draw_cube / draw_sphere API.
/// No extra crate needed. Each tile becomes a 3D box of varying height.
/// The world coords map to 3D:  tile (tx, ty) → world (tx, 0..h, ty).
///
/// Camera is positioned above and slightly in front to give a ~45° angle
/// that tracks the 2D cam_x/cam_y pan.
use macroquad::prelude::*;
use crate::constants::*;
use crate::sim::actor::{Actor, Role};
use crate::sim::world::{SimWorld, TileKind, WorldClock};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn rgb(r: f32, g: f32, b: f32) -> Color { Color::new(r, g, b, 1.0) }

fn daylight(clock: &WorldClock) -> f32 {
    let t = clock.time_of_day;
    if t < 0.20      { 0.25 + t * 2.5 }
    else if t < 0.80 { 1.0 }
    else             { 1.0 - (t - 0.80) * 3.5 }
        .clamp(0.25, 1.0)
}

// Tile → 3D colour and box height
fn tile_3d(tile: TileKind, noise: f32) -> (Color, f32) {
    let n = noise;
    match tile {
        TileKind::DeepWater    => (rgb(0.12, 0.30, 0.55), 0.05),
        TileKind::Water        => (rgb(0.22 + n*0.04, 0.52 + n*0.04, 0.75), 0.08),
        TileKind::Grass        => (rgb(0.30 + n*0.08, 0.55 + n*0.10, 0.24 + n*0.05), 0.12),
        TileKind::Path         => (rgb(0.68 + n*0.04, 0.56 + n*0.03, 0.40), 0.10),
        TileKind::Bracken      => (rgb(0.20 + n*0.04, 0.40 + n*0.06, 0.16 + n*0.03), 0.55),
        TileKind::Stone        => (rgb(0.42 + n*0.06, 0.40 + n*0.04, 0.38), 0.50),
        TileKind::BuildingFloor=> (rgb(0.34, 0.26, 0.18), 0.12),
        TileKind::BuildingWall => (rgb(0.52 + n*0.04, 0.32 + n*0.03, 0.20), 1.60),
        TileKind::Mountain     => (rgb(0.38 + n*0.06, 0.36 + n*0.04, 0.40 + n*0.04), 2.20),
    }
}

// ─── Main draw entry point ────────────────────────────────────────────────────

pub fn draw_world_3d(world: &SimWorld, selected: Option<usize>,
                     cam_x: f32, cam_y: f32) {
    let light = daylight(&world.clock) * world.weather.brightness();

    // Camera tracks the 2D pan position
    // Convert 2D pixel cam_x/cam_y → tile-space centre of viewport
    let vp_tiles_w = VIEWPORT_WIDTH  / TILE_SIZE;
    let vp_tiles_h = SCREEN_HEIGHT   / TILE_SIZE;
    let cx = cam_x / TILE_SIZE + vp_tiles_w * 0.5;
    let cz = cam_y / TILE_SIZE + vp_tiles_h * 0.5;
    let cam_h    = 28.0;
    let cam_tilt = 16.0;

    set_camera(&Camera3D {
        position:   vec3(cx, cam_h, cz - cam_tilt),
        target:     vec3(cx, 0.0,   cz),
        up:         vec3(0.0, 1.0,  0.0),
        fovy:       45.0_f32.to_radians(),
        projection: Projection::Perspective,
        ..Default::default()
    });

    // Draw visible tiles
    let x0 = (cam_x / TILE_SIZE) as i32 - 1;
    let y0 = (cam_y / TILE_SIZE) as i32 - 1;
    let x1 = x0 + (VIEWPORT_WIDTH / TILE_SIZE) as i32 + 2;
    let y1 = y0 + (SCREEN_HEIGHT  / TILE_SIZE) as i32 + 2;

    for ty in y0..=y1 {
        if ty < 0 || ty >= MAP_HEIGHT as i32 { continue; }
        for tx in x0..=x1 {
            if tx < 0 || tx >= MAP_WIDTH as i32 { continue; }
            let tile  = world.tiles[ty as usize][tx as usize];
            let noise = world.tile_noise[ty as usize][tx as usize];
            let (color, h) = tile_3d(tile, noise);
            let dim_color = Color::new(
                color.r * light, color.g * light, color.b * light, 1.0);

            // Box: centred at (tx+0.5, h/2, ty+0.5)
            draw_cube(
                vec3(tx as f32 + 0.5, h * 0.5, ty as f32 + 0.5),
                vec3(0.98, h, 0.98),
                None,
                dim_color,
            );

            // Tree canopy on bracken tiles
            if tile == TileKind::Bracken {
                let canopy_c = Color::new(
                    0.22 * light, 0.48 * light, 0.18 * light, 1.0);
                draw_sphere(
                    vec3(tx as f32 + 0.5, h + 0.55, ty as f32 + 0.5),
                    0.52,
                    None,
                    canopy_c,
                );
            }
        }
    }

    // Actors as coloured spheres with a cylinder body
    for actor in &world.actors {
        let ax = actor.pixel_x / TILE_SIZE + 0.5;
        let az = actor.pixel_y / TILE_SIZE + 0.5;
        let (cr, cg, cb) = actor.role.color();
        let body_col = Color::new(cr * light, cg * light, cb * light, 1.0);

        // Body (slim box)
        let body_h = match actor.role { Role::Elder => 0.70, Role::Child => 0.42, _ => 0.56 };
        draw_cube(
            vec3(ax, body_h * 0.5 + 0.12, az),
            vec3(0.30, body_h, 0.22),
            None,
            body_col,
        );
        // Head (sphere)
        draw_sphere(
            vec3(ax, body_h + 0.12 + 0.20, az),
            0.22,
            None,
            body_col,
        );

        // Selection ring (white sphere overlay, slightly larger)
        if selected == Some(actor.id) {
            draw_sphere(
                vec3(ax, body_h + 0.12 + 0.20, az),
                0.28,
                None,
                Color::new(1.0, 1.0, 0.5, 0.35),
            );
        }
    }

    // Back to 2D for the UI panel
    set_default_camera();
}

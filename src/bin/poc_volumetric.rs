/// POC: Volumetric holographic display.
///
/// Renders the village world as a glowing neon voxel field — dark background,
/// transparent fill cubes, bright wireframe edges, sweeping scan line, and a
/// slow auto-orbiting camera.
///
/// Controls:
///   R         — toggle auto-rotate
///   A / D     — orbit left / right
///   W / S     — raise / lower camera
///   scroll    — zoom in / out
use macroquad::prelude::*;

use the_nature_of_things::constants::{MAP_WIDTH, MAP_HEIGHT};
use the_nature_of_things::sim::world::{SimWorld, TileKind};

fn window_conf() -> Conf {
    Conf {
        window_title:     "The Nature of Things — Volumetric Display".to_owned(),
        window_width:     1280,
        window_height:    800,
        window_resizable: false,
        ..Default::default()
    }
}

// ─── Holographic colour palette ───────────────────────────────────────────────
//
// Returns (fill_rgba, wire_rgba, box_height).
// Fill is low-alpha so distant geometry glows through.
// Wire is near-opaque for crisp voxel edges.
fn vol_color(tile: TileKind, noise: f32) -> (Color, Color, f32) {
    let n = noise;
    match tile {
        TileKind::DeepWater => (
            Color::new(0.00, 0.20, 0.70, 0.12),
            Color::new(0.10, 0.45, 1.00, 0.55),
            0.06,
        ),
        TileKind::Water => (
            Color::new(0.00, 0.30, 0.80 + n * 0.05, 0.16),
            Color::new(0.20, 0.60, 1.00, 0.75),
            0.10,
        ),
        TileKind::Grass => (
            Color::new(0.00, 0.40 + n * 0.10, 0.18 + n * 0.05, 0.09),
            Color::new(0.08, 0.65 + n * 0.15, 0.30, 0.42),
            0.14,
        ),
        TileKind::Path => (
            Color::new(0.55 + n * 0.05, 0.50, 0.12, 0.17),
            Color::new(0.95, 0.82, 0.35, 0.72),
            0.11,
        ),
        TileKind::Bracken => (
            Color::new(0.00, 0.52 + n * 0.08, 0.12, 0.20),
            Color::new(0.12, 0.88 + n * 0.08, 0.32, 0.82),
            0.60,
        ),
        TileKind::Stone => (
            Color::new(0.18, 0.32 + n * 0.05, 0.48, 0.18),
            Color::new(0.42, 0.62, 0.88, 0.68),
            0.50,
        ),
        TileKind::BuildingFloor => (
            Color::new(0.08, 0.42, 0.62, 0.22),
            Color::new(0.28, 0.72, 1.00, 0.82),
            0.14,
        ),
        TileKind::BuildingWall => (
            Color::new(0.04, 0.52, 0.82, 0.28),
            Color::new(0.22, 0.88, 1.00, 1.00),
            1.60,
        ),
        TileKind::Mountain => (
            Color::new(0.28, 0.42, 0.72, 0.20),
            Color::new(0.52, 0.78, 1.00, 0.85),
            2.20,
        ),
        TileKind::Farmland => (
            Color::new(0.18, 0.38, 0.08 + n * 0.05, 0.16),
            Color::new(0.38, 0.68, 0.18, 0.62),
            0.60,
        ),
        TileKind::Fence => (
            Color::new(0.32, 0.48, 0.58, 0.22),
            Color::new(0.58, 0.78, 0.88, 0.78),
            1.20,
        ),
        TileKind::Cobble => (
            Color::new(0.22, 0.38, 0.62, 0.20),
            Color::new(0.42, 0.68, 1.00, 0.75),
            0.70,
        ),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let world = SimWorld::generate_village(42);

    let map_cx = MAP_WIDTH  as f32 * 0.5;
    let map_cz = MAP_HEIGHT as f32 * 0.5;
    let center  = vec3(map_cx, 0.0, map_cz);

    let mut auto_rotate   = true;
    let mut manual_angle: f32 = 0.25;
    let mut cam_height:   f32 = 55.0;
    let mut orbit_radius: f32 = 65.0;

    loop {
        let dt = get_frame_time();

        // ── Input ─────────────────────────────────────────────────────────
        if is_key_pressed(KeyCode::R)      { auto_rotate = !auto_rotate; }
        if is_key_pressed(KeyCode::Escape) { break; }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)  { manual_angle -= dt * 0.8; }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { manual_angle += dt * 0.8; }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)    { cam_height = (cam_height - dt * 20.0).max(14.0); }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)  { cam_height = (cam_height + dt * 20.0).min(95.0); }
        let (_, scroll) = mouse_wheel();
        orbit_radius = (orbit_radius - scroll * 3.0).clamp(20.0, 110.0);

        let angle = if auto_rotate {
            get_time() as f32 * 0.10 + manual_angle
        } else {
            manual_angle
        };

        let cam_pos = center + vec3(
            angle.cos() * orbit_radius,
            cam_height,
            angle.sin() * orbit_radius,
        );

        // Scan line — sweeps slowly up through the world height range
        let scan_y: f32 = (get_time() as f32 * 0.6).sin() * 1.2 + 1.1;

        // ── Render ────────────────────────────────────────────────────────
        clear_background(Color::new(0.010, 0.018, 0.055, 1.0));

        set_camera(&Camera3D {
            position:   cam_pos,
            target:     center,
            up:         vec3(0.0, 1.0, 0.0),
            fovy:       38.0_f32.to_radians(),
            projection: Projection::Perspective,
            ..Default::default()
        });

        // ── Base grid lines (every 5 tiles) ───────────────────────────────
        let grid_dim  = Color::new(0.04, 0.10, 0.22, 0.45);
        let grid_brt  = Color::new(0.08, 0.20, 0.40, 0.60);
        let mw = MAP_WIDTH  as i32;
        let mh = MAP_HEIGHT as i32;
        for ix in (0..=mw).step_by(5) {
            let c = if ix % 10 == 0 { grid_brt } else { grid_dim };
            draw_line_3d(vec3(ix as f32, 0.01, 0.0),  vec3(ix as f32, 0.01, mh as f32), c);
        }
        for iz in (0..=mh).step_by(5) {
            let c = if iz % 10 == 0 { grid_brt } else { grid_dim };
            draw_line_3d(vec3(0.0, 0.01, iz as f32), vec3(mw as f32, 0.01, iz as f32), c);
        }

        // ── Voxel tiles ───────────────────────────────────────────────────
        for ty in 0..MAP_HEIGHT {
            for tx in 0..MAP_WIDTH {
                let tile  = world.tiles[ty][tx];
                let noise = world.tile_noise[ty][tx];
                let (fill, wire, h) = vol_color(tile, noise);

                let pos = vec3(tx as f32 + 0.5, h * 0.5, ty as f32 + 0.5);
                let sz  = vec3(0.94, h, 0.94);

                // Brighten wireframe near the scan plane
                let wire_final = {
                    let dist = (h - scan_y).abs();
                    if dist < 0.35 {
                        let boost = 1.0 - dist / 0.35;
                        Color::new(
                            (wire.r + boost * 0.6).min(1.0),
                            (wire.g + boost * 0.6).min(1.0),
                            (wire.b + boost * 0.4).min(1.0),
                            1.0,
                        )
                    } else {
                        wire
                    }
                };

                // Fill cube (only for taller geometry — saves draw calls for flat terrain)
                if h >= 0.35 {
                    draw_cube(pos, sz, None, fill);
                }
                draw_cube_wires(pos, sz, wire_final);

                // Tree canopy wireframe sphere on bracken
                if tile == TileKind::Bracken {
                    let sp_col = Color::new(0.10, 0.92, 0.38, 0.60);
                    draw_sphere_wires(
                        vec3(pos.x, h + 0.50, pos.z),
                        0.40,
                        None,
                        sp_col,
                    );
                }
            }
        }

        // ── HUD ───────────────────────────────────────────────────────────
        set_default_camera();
        let hud = Color::new(0.30, 0.72, 1.00, 0.80);
        let rotate_label = if auto_rotate { "orbit: ON" } else { "orbit: OFF" };
        draw_text(
            &format!(
                "VOLUMETRIC  |  R: {}  |  A/D: rotate  |  W/S: tilt  |  scroll: zoom",
                rotate_label
            ),
            12.0, 22.0, 17.0, hud,
        );

        next_frame().await;
    }
}

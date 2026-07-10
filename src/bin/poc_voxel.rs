/// High-resolution South Wales valley — volumetric voxel display.
///
/// World: 128 × 32 × 128 voxels at 0.40 m³ each.
/// Rendering: one draw_mesh call per 16³ chunk (~128 calls).
///
/// Camera modes:
///   Tab          — toggle Orbit ↔ Fly-through
///   Orbit:  A/D rotate · W/S tilt · scroll zoom · R auto-orbit
///   Fly:    WASD forward/back/strafe · Q/E up/down · mouse look
///           Shift held = 3× speed
/// Other:   SPACE regen world · Esc quit
use macroquad::prelude::*;

use the_nature_of_things::voxel::generate_wales_valley;
use the_nature_of_things::render::draw_voxel::ChunkRenderer;
use the_nature_of_things::voxel::mesher::VS;

const WORLD_X: usize = 512;
const WORLD_Y: usize = 128;
const WORLD_Z: usize = 512;

#[derive(PartialEq, Clone, Copy)]
enum CamMode { Orbit, Fly }

fn window_conf() -> macroquad::conf::Conf {
    macroquad::conf::Conf {
        miniquad_conf: Conf {
            window_title:     "South Wales Valley — Volumetric Display".to_owned(),
            window_width:     1280,
            window_height:    800,
            window_resizable: false,
            ..Default::default()
        },
        // Each 64³ chunk mesh can have up to ~33k vertices / ~50k indices.
        // macroquad defaults (10k verts / 5k indices) are far too small.
        draw_call_vertex_capacity: 65536,
        draw_call_index_capacity:  98304,  // 65536 × 1.5 for triangles
        ..Default::default()
    }
}

// Fly-camera helpers
fn fly_forward(yaw: f32, pitch: f32) -> Vec3 {
    vec3(yaw.sin() * pitch.cos(), pitch.sin(), yaw.cos() * pitch.cos())
}
fn fly_right(yaw: f32) -> Vec3 {
    vec3(yaw.cos(), 0.0, -yaw.sin())
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut seed: u32 = 42;

    let mut world    = generate_wales_valley(WORLD_X, WORLD_Y, WORLD_Z, seed);
    let mut renderer = ChunkRenderer::new(&world);

    let scene_cx = WORLD_X as f32 * VS * 0.5;
    let scene_cz = WORLD_Z as f32 * VS * 0.5;
    let center   = vec3(scene_cx, 0.0, scene_cz);

    // ── Orbit state ───────────────────────────────────────────────────
    let mut auto_rotate  = true;
    let mut angle: f32   = 0.25;
    let mut cam_h: f32   = 30.0;
    let mut radius: f32  = 50.0;

    // ── Fly state ─────────────────────────────────────────────────────
    let mut cam_mode     = CamMode::Orbit;
    // Start fly camera at a good vantage point (above the terrace row)
    let mut fly_pos      = vec3(scene_cx, 6.0, scene_cz - 8.0);
    let mut fly_yaw: f32 = 0.0;   // looking toward +Z
    let mut fly_pitch: f32 = -0.15;

    loop {
        let dt = get_frame_time();

        // ── Global keys ───────────────────────────────────────────────
        if is_key_pressed(KeyCode::Escape) {
            if cam_mode == CamMode::Fly {
                // Exit fly mode first
                cam_mode = CamMode::Orbit;
                set_cursor_grab(false);
                show_mouse(true);
            } else {
                break;
            }
        }

        if is_key_pressed(KeyCode::Tab) {
            cam_mode = match cam_mode {
                CamMode::Orbit => {
                    // Initialise fly camera from current orbit position
                    let orbit_pos = center + vec3(angle.cos() * radius, cam_h, angle.sin() * radius);
                    let dir = (center - orbit_pos).normalize();
                    fly_pitch = dir.y.asin();
                    fly_yaw   = dir.x.atan2(dir.z);
                    fly_pos   = orbit_pos;
                    set_cursor_grab(true);
                    show_mouse(false);
                    CamMode::Fly
                }
                CamMode::Fly => {
                    set_cursor_grab(false);
                    show_mouse(true);
                    CamMode::Orbit
                }
            };
        }

        if is_key_pressed(KeyCode::Space) {
            seed = seed.wrapping_add(1);
            world    = generate_wales_valley(WORLD_X, WORLD_Y, WORLD_Z, seed);
            renderer = ChunkRenderer::new(&world);
        }

        // ── Mode-specific input ───────────────────────────────────────
        let cam_pos;
        let cam_target;
        let cam_up = vec3(0.0, 1.0, 0.0);

        match cam_mode {
            // ── Orbit ─────────────────────────────────────────────────
            CamMode::Orbit => {
                if is_key_pressed(KeyCode::R) { auto_rotate = !auto_rotate; }
                if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)  { angle  -= dt * 0.70; }
                if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { angle  += dt * 0.70; }
                if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)    { cam_h   = (cam_h  - dt * 18.0).max(1.0); }
                if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)  { cam_h   = (cam_h  + dt * 18.0).min(80.0); }
                let (_, scroll) = mouse_wheel();
                radius = (radius - scroll * 2.5).clamp(4.0, 120.0);
                if auto_rotate { angle += dt * 0.06; }

                cam_pos    = center + vec3(angle.cos() * radius, cam_h, angle.sin() * radius);
                cam_target = center;
            }

            // ── Fly-through ───────────────────────────────────────────
            CamMode::Fly => {
                let md = mouse_delta_position();
                fly_yaw   += md.x * 0.004;
                fly_pitch  = (fly_pitch - md.y * 0.004).clamp(-1.48, 1.48);

                let fwd   = fly_forward(fly_yaw, fly_pitch);
                let right = fly_right(fly_yaw);
                let speed = dt * if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) { 20.0 } else { 7.0 };

                if is_key_down(KeyCode::W) { fly_pos += fwd   * speed; }
                if is_key_down(KeyCode::S) { fly_pos -= fwd   * speed; }
                if is_key_down(KeyCode::A) { fly_pos -= right * speed; }
                if is_key_down(KeyCode::D) { fly_pos += right * speed; }
                if is_key_down(KeyCode::Q) || is_key_down(KeyCode::Down)  { fly_pos.y -= speed; }
                if is_key_down(KeyCode::E) || is_key_down(KeyCode::Up)    { fly_pos.y += speed; }

                cam_pos    = fly_pos;
                cam_target = fly_pos + fwd;
            }
        }

        renderer.cam = cam_pos;

        // ── Render ────────────────────────────────────────────────────
        clear_background(Color::new(0.010, 0.016, 0.048, 1.0));

        set_camera(&Camera3D {
            position:   cam_pos,
            target:     cam_target,
            up:         cam_up,
            fovy:       70.0_f32.to_radians(),
            projection: Projection::Perspective,
            ..Default::default()
        });

        // Base grid
        let gc = Color::new(0.04, 0.10, 0.22, 0.30);
        let gb = Color::new(0.08, 0.20, 0.40, 0.45);
        let mw = WORLD_X as f32 * VS;
        let md = WORLD_Z as f32 * VS;
        for ix in (0..=WORLD_X).step_by(8) {
            let c = if ix % 32 == 0 { gb } else { gc };
            draw_line_3d(vec3(ix as f32 * VS, 0.01, 0.0), vec3(ix as f32 * VS, 0.01, md), c);
        }
        for iz in (0..=WORLD_Z).step_by(8) {
            let c = if iz % 32 == 0 { gb } else { gc };
            draw_line_3d(vec3(0.0, 0.01, iz as f32 * VS), vec3(mw, 0.01, iz as f32 * VS), c);
        }

        renderer.draw(&world);

        // ── HUD ───────────────────────────────────────────────────────
        set_default_camera();
        let hc  = Color::new(0.30, 0.72, 1.00, 0.80);
        let fps = get_fps();
        let mode_str = match cam_mode {
            CamMode::Orbit => format!("ORBIT  R:auto-rotate A/D:spin W/S:tilt scroll:zoom"),
            CamMode::Fly   => format!("FLY    WASD:move  Q/E:↕  mouse:look  Shift:fast"),
        };
        draw_text(
            &format!("{fps} fps | {} ktris | seed:{seed} | Tab:toggle | {mode_str} | SPACE:regen",
                renderer.triangle_count() / 1000),
            10.0, 22.0, 15.0, hc,
        );
        if cam_mode == CamMode::Fly {
            let p = cam_pos / VS;
            draw_text(&format!("pos  x:{:.0}  y:{:.0}  z:{:.0}", p.x, p.y, p.z),
                10.0, 42.0, 14.0, Color::new(0.60, 0.80, 0.60, 0.70));
        }

        next_frame().await;
    }
}

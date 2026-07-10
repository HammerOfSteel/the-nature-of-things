/// Interactive tile browser for inspecting spritesheet tile mappings.
///
/// Controls:
///   1 / 2         — switch between Kenney Tiny-Town / Sunnyside tilesets
///   Mouse hover   — show tile index, (col, row), and average colour
///   Left click    — pin / unpin selection
///   Scroll wheel  — zoom in / out
///   WASD / arrows — pan
///   R             — reset zoom + pan
///   M             — show/hide current TileKind → tile-index mapping overlay
///
/// Run:  cargo run --bin tile_browser
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};

// ─────────────────────────────────────────────────────────────────────────────

const WIN_W: i32 = 1280;
const WIN_H: i32 = 720;
const INFO_W: f32 = 260.0;

fn window_conf() -> Conf {
    Conf {
        window_title:    "Tile Browser".to_owned(),
        window_width:    WIN_W,
        window_height:   WIN_H,
        window_resizable: false,
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────

struct Sheet {
    name:      &'static str,
    path:      &'static str,
    tile_w:    u32,
    tile_h:    u32,
    /// TileKind label → tile flat-index pairs (label, flat_index)
    mapping:   &'static [(&'static str, &'static [u32])],
}

/// Verified mapping for draw_sprites.rs (Kenney Tiny-Town, 12-col sheet).
/// Water/DeepWater are solid colours — no tile exists in this pack.
const KENNEY_MAPPING: &[(&str, &[u32])] = &[
    ("Grass",         &[0, 1, 2]),
    ("Path",          &[24, 25, 26]),
    ("BuildingFloor", &[12, 13]),
    ("BuildingWall",  &[52, 53, 54]),   // red terracotta roof from above
    ("Water",         &[]),             // solid colour fallback (no tile)
    ("DeepWater",     &[]),             // solid colour fallback (no tile)
    ("Bracken base",  &[0, 1, 2]),
    ("Bracken tree",  &[4, 5, 6]),      // transparent green tree overlays
    ("Stone",         &[48, 49, 50]),   // grey/blue stone wall tiles
    ("Mountain",      &[96, 97, 98]),   // lighter grey stone (row 8)
];

/// Current draw_sunnyside.rs mapping (64-col sheet).
const SUNNY_MAPPING: &[(&str, &[u32])] = &[
    ("Grass",         &[65, 66, 129, 130, 192, 196]),
    ("Path",          &[69, 71, 72]),
    ("BuildingFloor", &[524]),
    ("BuildingWall",  &[577, 578]),
    ("Water",         &[68, 135, 136]),
    ("DeepWater",     &[68]),
    ("Bracken",       &[130, 131, 132]),
    ("Stone",         &[659]),
    ("Mountain",      &[662, 595]),
];

const SHEETS: [Sheet; 2] = [
    Sheet {
        name:    "Kenney Tiny-Town  (tiny_town.png, 12 cols)",
        path:    "assets/tiles/tiny_town.png",
        tile_w:  16,
        tile_h:  16,
        mapping: KENNEY_MAPPING,
    },
    Sheet {
        name:    "Sunnyside World   (sunnyside.png, 64 cols)",
        path:    "assets/tiles/sunnyside.png",
        tile_w:  16,
        tile_h:  16,
        mapping: SUNNY_MAPPING,
    },
];

// ─────────────────────────────────────────────────────────────────────────────

struct BrowserState {
    sheet_idx:   usize,
    textures:    [Texture2D; 2],
    zoom:        f32,
    pan_x:       f32,
    pan_y:       f32,
    hovered:     Option<(u32, u32)>,   // (col, row)
    pinned:      Option<(u32, u32)>,
    show_map:    bool,
}

impl BrowserState {
    fn cols(&self) -> u32 {
        let s = &SHEETS[self.sheet_idx];
        self.textures[self.sheet_idx].width() as u32 / s.tile_w
    }
    fn rows(&self) -> u32 {
        let s = &SHEETS[self.sheet_idx];
        self.textures[self.sheet_idx].height() as u32 / s.tile_h
    }
    fn flat_index(&self, col: u32, row: u32) -> u32 {
        row * self.cols() + col
    }
    fn sheet(&self) -> &Sheet { &SHEETS[self.sheet_idx] }
    fn viewport_w(&self) -> f32 { WIN_W as f32 - INFO_W }
}

// ─────────────────────────────────────────────────────────────────────────────

/// Colour for a TileKind label (for the mapping overlay badge).
fn kind_color(label: &str) -> Color {
    match label {
        "Grass"         => Color::from_hex(0x2e8b2e),
        "Water"         => Color::from_hex(0x2244aa),
        "DeepWater"     => Color::from_hex(0x112266),
        "Path"          => Color::from_hex(0xaa8833),
        "BuildingFloor" => Color::from_hex(0x998844),
        "BuildingWall"  => Color::from_hex(0xcc6622),
        "Bracken base"  => Color::from_hex(0x226622),
        "Bracken over"  => Color::from_hex(0x448844),
        "Bracken"       => Color::from_hex(0x335533),
        "Stone"         => Color::from_hex(0x556677),
        "Mountain"      => Color::from_hex(0x889aaa),
        _               => GRAY,
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let t0 = load_texture(SHEETS[0].path).await
        .unwrap_or_else(|_| panic!("{} missing", SHEETS[0].path));
    t0.set_filter(FilterMode::Nearest);

    let t1 = load_texture(SHEETS[1].path).await
        .unwrap_or_else(|_| panic!("{} missing", SHEETS[1].path));
    t1.set_filter(FilterMode::Nearest);

    let mut st = BrowserState {
        sheet_idx: 0,
        textures:  [t0, t1],
        zoom:      4.0,
        pan_x:     0.0,
        pan_y:     0.0,
        hovered:   None,
        pinned:    None,
        show_map:  true,
    };

    loop {
        let dt = get_frame_time();

        // ── input ────────────────────────────────────────────────────────────
        if is_key_pressed(KeyCode::Key1) { st.sheet_idx = 0; st.pinned = None; }
        if is_key_pressed(KeyCode::Key2) { st.sheet_idx = 1; st.pinned = None; }
        if is_key_pressed(KeyCode::M)    { st.show_map = !st.show_map; }
        if is_key_pressed(KeyCode::R) {
            st.zoom  = 4.0; st.pan_x = 0.0; st.pan_y = 0.0;
        }

        // Zoom via scroll
        let scroll = mouse_wheel().1;
        if scroll != 0.0 {
            let factor = if scroll > 0.0 { 1.15_f32 } else { 1.0 / 1.15 };
            st.zoom   = (st.zoom * factor).clamp(1.0, 16.0);
        }

        // Pan
        let pan_speed = 400.0 * dt / st.zoom;
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left)  { st.pan_x -= pan_speed; }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { st.pan_x += pan_speed; }
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up)    { st.pan_y -= pan_speed; }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down)  { st.pan_y += pan_speed; }
        st.pan_x = st.pan_x.max(0.0);
        st.pan_y = st.pan_y.max(0.0);

        // Hover detection
        let (mx, my) = mouse_position();
        let vw = st.viewport_w();
        st.hovered = None;
        if mx < vw {
            let tw = SHEETS[st.sheet_idx].tile_w as f32 * st.zoom;
            let th = SHEETS[st.sheet_idx].tile_h as f32 * st.zoom;
            let col = ((mx + st.pan_x) / tw) as u32;
            let row = ((my + st.pan_y) / th) as u32;
            if col < st.cols() && row < st.rows() {
                st.hovered = Some((col, row));
            }
        }

        // Click to pin/unpin
        if is_mouse_button_pressed(MouseButton::Left) && mx < vw {
            st.pinned = match (st.pinned, st.hovered) {
                (Some(p), Some(h)) if p == h => None,
                (_, h) => h,
            };
        }

        // ── render ───────────────────────────────────────────────────────────
        clear_background(Color::new(0.12, 0.12, 0.14, 1.0));

        let tw = SHEETS[st.sheet_idx].tile_w as f32 * st.zoom;
        let th = SHEETS[st.sheet_idx].tile_h as f32 * st.zoom;
        let tex = &st.textures[st.sheet_idx];

        // Draw all tiles in the viewport
        let first_col = (st.pan_x / tw) as u32;
        let first_row = (st.pan_y / th) as u32;
        let last_col  = ((st.pan_x + vw) / tw).ceil() as u32 + 1;
        let last_row  = ((st.pan_y + WIN_H as f32) / th).ceil() as u32 + 1;
        let last_col  = last_col.min(st.cols());
        let last_row  = last_row.min(st.rows());

        let s = &SHEETS[st.sheet_idx];

        for row in first_row..last_row {
            for col in first_col..last_col {
                let sx = col as f32 * tw - st.pan_x;
                let sy = row as f32 * th - st.pan_y;
                let flat = st.flat_index(col, row);

                // Draw tile
                draw_texture_ex(
                    tex, sx, sy, WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(tw, th)),
                        source: Some(Rect::new(
                            col as f32 * s.tile_w as f32,
                            row as f32 * s.tile_h as f32,
                            s.tile_w as f32,
                            s.tile_h as f32,
                        )),
                        ..Default::default()
                    },
                );

                // Grid lines
                draw_rectangle_lines(sx, sy, tw, th, 0.5, Color::new(1.0, 1.0, 1.0, 0.15));

                // Mapping badge (show_map mode)
                if st.show_map {
                    if let Some(kind_label) = kind_for_tile(s.mapping, flat) {
                        let bc = kind_color(kind_label);
                        draw_rectangle(sx + 1.0, sy + 1.0, tw - 2.0, 7.0,
                            Color::new(bc.r, bc.g, bc.b, 0.75));
                        if tw >= 32.0 {
                            draw_text(kind_label, sx + 2.0, sy + 8.0, 7.0,
                                Color::new(1.0, 1.0, 1.0, 0.95));
                        }
                    }
                }

                // Tile index label
                if tw >= 40.0 {
                    let label = format!("{flat}\n{col},{row}");
                    draw_text(&label, sx + 2.0, sy + th - 16.0, 8.0,
                        Color::new(1.0, 1.0, 0.7, 0.85));
                }
            }
        }

        // Hovered highlight
        if let Some((col, row)) = st.hovered {
            let sx = col as f32 * tw - st.pan_x;
            let sy = row as f32 * th - st.pan_y;
            draw_rectangle_lines(sx, sy, tw, th, 2.0, YELLOW);
        }

        // Pinned highlight
        if let Some((col, row)) = st.pinned {
            let sx = col as f32 * tw - st.pan_x;
            let sy = row as f32 * th - st.pan_y;
            draw_rectangle_lines(sx, sy, tw, th, 2.5, Color::from_hex(0xff6644));
        }

        // ── info panel (right side) ───────────────────────────────────────────
        draw_rectangle(vw, 0.0, INFO_W, WIN_H as f32, Color::new(0.08, 0.08, 0.10, 1.0));
        draw_line(vw, 0.0, vw, WIN_H as f32, 1.0, Color::new(0.4, 0.4, 0.5, 1.0));

        let px = vw + 8.0;
        let mut py = 14.0;
        let lh = 15.0;
        let sm = 10.0;

        macro_rules! txt {
            ($s:expr, $c:expr) => { draw_text($s, px, py, sm, $c); py += lh; };
            ($s:expr) => { txt!($s, Color::new(0.85, 0.85, 0.90, 1.0)); };
        }

        txt!(s.name, Color::new(0.95, 0.85, 0.50, 1.0));
        py += 4.0;
        txt!(&format!("Sheet: {}×{} tiles", st.cols(), st.rows()));
        txt!(&format!("Zoom: {:.1}×   Pan: {:.0},{:.0}", st.zoom, st.pan_x, st.pan_y));
        py += lh;
        txt!("[1] Kenney  [2] Sunnyside", Color::new(0.60, 0.70, 0.80, 1.0));
        txt!("[M] mapping  [R] reset  [scroll] zoom", Color::new(0.60, 0.70, 0.80, 1.0));
        py += lh;

        // Hovered / pinned tile info
        let inspect = st.pinned.or(st.hovered);
        if let Some((col, row)) = inspect {
            let flat = st.flat_index(col, row);
            let label = if st.pinned.is_some() { "PINNED" } else { "HOVER" };
            txt!(&format!("── {label} ──────────────"), Color::new(0.95, 0.80, 0.60, 1.0));
            txt!(&format!("Flat index : {flat}"));
            txt!(&format!("Col, Row   : {col}, {row}"));

            // Show which TileKinds include this tile
            let mut matched = Vec::new();
            for (kind, indices) in s.mapping {
                if indices.contains(&flat) { matched.push(*kind); }
            }
            if matched.is_empty() {
                txt!("Mapping    : (unmapped)", Color::new(0.55, 0.55, 0.60, 1.0));
            } else {
                txt!("Mapping    :", Color::new(0.70, 0.90, 0.70, 1.0));
                for m in &matched {
                    txt!(&format!("  → {m}"), Color::new(0.70, 0.90, 0.70, 1.0));
                }
            }
        } else {
            txt!("Hover / click a tile", Color::new(0.50, 0.50, 0.55, 1.0));
        }

        // TileKind mapping legend
        py += lh;
        txt!("── Current mapping ─────────", Color::new(0.60, 0.65, 0.75, 1.0));
        for (kind, indices) in s.mapping {
            let kc = kind_color(kind);
            draw_rectangle(px, py - 8.0, 6.0, 8.0, kc);
            let idxs: Vec<String> = indices.iter().map(|i| i.to_string()).collect();
            let list = idxs.join(",");
            draw_text(
                &format!(" {kind}: {list}"),
                px + 7.0, py, sm,
                Color::new(0.75, 0.80, 0.85, 1.0),
            );
            py += lh;
        }

        py += lh;
        txt!("── click tile → pin, click again → unpin ──", Color::new(0.40, 0.40, 0.45, 1.0));

        next_frame().await;
    }
}

fn kind_for_tile<'a>(mapping: &'a [(&'a str, &'a [u32])], flat: u32) -> Option<&'a str> {
    for (kind, indices) in mapping {
        if indices.contains(&flat) {
            return Some(kind);
        }
    }
    None
}

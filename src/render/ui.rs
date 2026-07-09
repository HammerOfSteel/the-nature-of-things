use macroquad::prelude::*;
use crate::constants::*;
use crate::sim::actor::{Actor, Role};
use crate::sim::world::SimWorld;

const PANEL_X: f32 = VIEWPORT_WIDTH;
const PAD: f32 = 10.0;

fn panel_bg() -> Color { Color::new(0.07, 0.07, 0.10, 1.0) }
fn separator() -> Color { Color::new(0.22, 0.22, 0.28, 1.0) }
fn text_dim()  -> Color { Color::new(0.55, 0.55, 0.62, 1.0) }
fn text_main() -> Color { Color::new(0.88, 0.88, 0.92, 1.0) }
fn text_hi()   -> Color { Color::new(1.00, 0.95, 0.65, 1.0) }

/// Draw the entire right-side UI panel.
pub fn draw_panel(world: &SimWorld, selected: Option<usize>) {
    // Panel background
    draw_rectangle(PANEL_X, 0.0, UI_PANEL_WIDTH, SCREEN_HEIGHT, panel_bg());
    draw_line(PANEL_X, 0.0, PANEL_X, SCREEN_HEIGHT, 1.5, separator());

    let mut cursor_y = PAD + 14.0;

    // ── World clock ──────────────────────────────────────────────────────────
    draw_text("THE NATURE OF THINGS", PANEL_X + PAD, cursor_y, 13.0, text_hi());
    cursor_y += 18.0;
    draw_line(PANEL_X + PAD, cursor_y, PANEL_X + UI_PANEL_WIDTH - PAD, cursor_y, 1.0, separator());
    cursor_y += 10.0;

    let clock_str = format!(
        "Day {}  {}  {}",
        world.clock.day,
        world.clock.time_label(),
        world.clock.season.label(),
    );
    draw_text(&clock_str, PANEL_X + PAD, cursor_y, 12.0, text_main());
    cursor_y += 14.0;

    // TOD bar (24h strip)
    let bar_w = UI_PANEL_WIDTH - PAD * 2.0;
    draw_rectangle(PANEL_X + PAD, cursor_y, bar_w, 5.0,
        Color::new(0.15, 0.15, 0.20, 1.0));
    let marker_x = PANEL_X + PAD + world.clock.time_of_day * bar_w;
    draw_rectangle(marker_x - 1.5, cursor_y - 1.0, 3.0, 7.0, text_hi());
    cursor_y += 14.0;

    draw_line(PANEL_X + PAD, cursor_y, PANEL_X + UI_PANEL_WIDTH - PAD, cursor_y, 1.0, separator());
    cursor_y += 8.0;

    // ── Actor panel ──────────────────────────────────────────────────────────
    if let Some(id) = selected {
        if let Some(actor) = world.actors.iter().find(|a| a.id == id) {
            cursor_y = draw_actor_panel(actor, cursor_y);
        }
    } else {
        draw_text("Click an actor to inspect", PANEL_X + PAD, cursor_y, 11.0, text_dim());
        cursor_y += 18.0;
    }

    draw_line(PANEL_X + PAD, cursor_y, PANEL_X + UI_PANEL_WIDTH - PAD, cursor_y, 1.0, separator());
    cursor_y += 8.0;

    // ── Population summary ───────────────────────────────────────────────────
    draw_text("VALLEY", PANEL_X + PAD, cursor_y, 11.0, text_dim());
    cursor_y += 14.0;
    draw_text(&format!("{} souls  |  {}",
        world.actors.len(), world.weather.label()),
        PANEL_X + PAD, cursor_y, 11.0, text_main());
    cursor_y += 14.0;

    // Role breakdown (compact two-column list)
    let roles = [Role::Miner, Role::Teacher, Role::Shopkeeper, Role::Musician,
                 Role::Elder, Role::Child, Role::NewArrival];
    for (i, role) in roles.iter().enumerate() {
        let count = world.actors.iter().filter(|a| a.role == *role).count();
        if count == 0 { continue; }
        let col_x = PANEL_X + PAD + (i % 2) as f32 * 138.0;
        if i % 2 == 0 && i > 0 { cursor_y += 12.0; }
        if i % 2 == 0 {
            draw_text(&format!("{}x {}", count, role.display_name()),
                col_x, cursor_y, 10.0, text_dim());
        } else {
            draw_text(&format!("  {}x {}", count, role.display_name()),
                col_x - 138.0 + 8.0, cursor_y + 12.0, 10.0, text_dim());
            cursor_y += 0.0; // will advance below
        }
    }
    cursor_y += 14.0;

    draw_line(PANEL_X + PAD, cursor_y, PANEL_X + UI_PANEL_WIDTH - PAD, cursor_y, 1.0, separator());
    cursor_y += 8.0;

    // ── Chronicle ────────────────────────────────────────────────────────────
    draw_text("CHRONICLE", PANEL_X + PAD, cursor_y, 11.0, text_dim());
    cursor_y += 14.0;

    let chronicle_entries: Vec<&String> = world.chronicle.iter().rev().collect();
    let available_h = SCREEN_HEIGHT - cursor_y - PAD;
    let line_h = 12.0;
    let max_lines = (available_h / line_h) as usize;

    for entry in chronicle_entries.iter().take(max_lines) {
        let wrapped = wrap_text(entry, 35);
        for line in wrapped {
            if cursor_y + line_h > SCREEN_HEIGHT - PAD { break; }
            draw_text(&line, PANEL_X + PAD, cursor_y, 10.5, text_dim());
            cursor_y += line_h;
        }
    }
}

fn draw_actor_panel(actor: &Actor, mut y: f32) -> f32 {
    let x = PANEL_X + PAD;

    // Name & role
    let (cr, cg, cb) = actor.role.color();
    draw_text(&actor.name, x, y, 14.0, Color::new(cr, cg, cb, 1.0));
    y += 16.0;
    draw_text(actor.role.display_name(), x, y, 11.0, text_dim());
    y += 14.0;

    // Current action
    draw_text(&format!("↳ {}", actor.current_action.label()), x, y, 11.0, text_main());
    y += 16.0;

    // Node graph bars
    let bar_max_w = UI_PANEL_WIDTH - PAD * 2.0 - 72.0;
    let bar_h = 7.0;
    let bar_gap = 9.5;

    for i in 0..NODE_COUNT {
        let val = actor.node_graph.values[i];
        let (nr, ng, nb) = NODE_COLORS[i];
        let bar_color = Color::new(nr, ng, nb, 0.90);
        let bg_color  = Color::new(0.14, 0.14, 0.18, 1.0);
        let label     = NODE_NAMES[i];

        draw_text(label, x, y + bar_h - 1.0, 10.0, text_dim());
        let label_w = 72.0;
        let bar_x   = x + label_w;

        draw_rectangle(bar_x, y, bar_max_w, bar_h, bg_color);
        draw_rectangle(bar_x, y, bar_max_w * val, bar_h, bar_color);

        // Value text
        let pct = format!("{:.0}", val * 100.0);
        draw_text(&pct, bar_x + bar_max_w + 3.0, y + bar_h - 1.0, 9.5, text_dim());

        y += bar_gap;
    }
    y += 6.0;

    // Memory (last 4 events)
    if !actor.memory.is_empty() {
        draw_text("MEMORIES", x, y, 10.0, text_dim());
        y += 12.0;
        for event in actor.memory.iter().rev().take(4) {
            let wrapped = wrap_text(&event.text, 34);
            for line in wrapped {
                draw_text(&line, x, y, 9.5, Color::new(0.60, 0.58, 0.68, 1.0));
                y += 11.0;
            }
            y += 2.0;
        }
    }

    y
}

// ─── Controls legend ─────────────────────────────────────────────────────────

pub fn draw_controls() {
    let y = SCREEN_HEIGHT - 13.0;
    draw_text(
        "WASD: pan  |  Tab: cycle  |  Click: select  |  M: map  |  [/]: speed  |  Space: pause  |  P/E/H/B: events",
        5.0, y, 10.0, Color::new(0.45, 0.45, 0.50, 1.0),
    );
}

// ─── Pause overlay ───────────────────────────────────────────────────────────

pub fn draw_pause_overlay() {
    draw_rectangle(0.0, 0.0, VIEWPORT_WIDTH, SCREEN_HEIGHT,
        Color::new(0.0, 0.0, 0.0, 0.35));
    draw_text("PAUSED", VIEWPORT_WIDTH * 0.5 - 28.0, SCREEN_HEIGHT * 0.5, 28.0,
        Color::new(1.0, 0.95, 0.65, 0.90));
}

// ─── Simple word-wrap ────────────────────────────────────────────────────────

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if current.len() + 1 + word.len() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() { lines.push(current); }
    lines
}

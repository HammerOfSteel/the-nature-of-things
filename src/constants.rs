// Tile & map
pub const TILE_SIZE: f32 = 24.0;
pub const MAP_WIDTH: usize = 90;
pub const MAP_HEIGHT: usize = 60;

// Screen
pub const SCREEN_WIDTH: f32 = 1280.0;
pub const SCREEN_HEIGHT: f32 = 720.0;
pub const UI_PANEL_WIDTH: f32 = 300.0;
pub const VIEWPORT_WIDTH: f32 = SCREEN_WIDTH - UI_PANEL_WIDTH;

// Simulation
pub const NODE_COUNT: usize = 10;
pub const PROPAGATION_DT: f32 = 0.06;
pub const DEFAULT_TICK_INTERVAL: f64 = 0.18; // seconds between sim ticks
pub const TICKS_PER_DAY: u32 = 360;
pub const ACTOR_COUNT: usize = 28;
pub const CONTAGION_RANGE: i32 = 4; // tiles

// Actor rendering
pub const ACTOR_BODY_W: f32 = 7.0;
pub const ACTOR_BODY_H: f32 = 11.0;
pub const ACTOR_HEAD_R: f32 = 4.5;
pub const SELECTION_RING_R: f32 = 11.0;

// Chronicle
pub const MAX_CHRONICLE: usize = 60;
pub const MAX_MEMORY: usize = 20;

// Camera pan speed (px/s)
pub const CAM_SPEED: f32 = 280.0;

// Node display colors (r, g, b)
pub const NODE_COLORS: [(f32, f32, f32); NODE_COUNT] = [
    (0.95, 0.55, 0.15), // Hunger   — orange
    (0.35, 0.55, 0.85), // Fatigue  — blue
    (0.90, 0.25, 0.25), // Stress   — red
    (0.95, 0.85, 0.20), // Social   — yellow
    (0.95, 0.95, 0.50), // Joy      — bright yellow
    (0.35, 0.25, 0.65), // Grief    — deep purple
    (0.30, 0.80, 0.40), // Purpose  — green
    (0.90, 0.60, 0.20), // Belonging— amber
    (0.25, 0.85, 0.85), // Curiosity— cyan
    (0.70, 0.15, 0.15), // Resentmt — dark red
];

pub const NODE_NAMES: [&str; NODE_COUNT] = [
    "Hunger", "Fatigue", "Stress", "Social", "Joy",
    "Grief", "Purpose", "Belonging", "Curiosity", "Resentment",
];

use std::collections::VecDeque;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use crate::constants::*;
use super::actor::{Actor, Role};

// ─── Tile ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileKind {
    DeepWater,
    Water,
    Grass,
    Path,
    Bracken,
    Stone,
    BuildingFloor,
    BuildingWall,
    Mountain,
    // Village-gen additions
    Farmland,   // tilled soil / crop patch
    Fence,      // low fence (not walkable)
    Cobble,     // stone courtyard / square
}

impl TileKind {
    /// Base RGB colour for this tile
    pub fn base_color(self) -> (f32, f32, f32) {
        match self {
            TileKind::DeepWater    => (0.12, 0.30, 0.45),
            TileKind::Water        => (0.18, 0.42, 0.60),
            TileKind::Grass        => (0.28, 0.50, 0.26),
            TileKind::Path         => (0.60, 0.48, 0.32),
            TileKind::Bracken      => (0.50, 0.38, 0.20),
            TileKind::Stone        => (0.38, 0.38, 0.42),
            TileKind::BuildingFloor=> (0.48, 0.44, 0.40),
            TileKind::BuildingWall => (0.55, 0.50, 0.46),
            TileKind::Mountain     => (0.28, 0.26, 0.30),
            TileKind::Farmland     => (0.55, 0.38, 0.20),
            TileKind::Fence        => (0.50, 0.42, 0.30),
            TileKind::Cobble       => (0.45, 0.45, 0.48),
        }
    }

    pub fn is_walkable(self) -> bool {
        !matches!(self,
            TileKind::DeepWater | TileKind::BuildingWall |
            TileKind::Mountain  | TileKind::Fence
        )
    }
}

// ─── Location ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocationKind {
    Home,
    Pub,
    Common,
    River,
    Mountain,
    Bakery,
    ChoirHall,
    Library,
    Workplace,
}

impl LocationKind {
    pub fn label(self) -> &'static str {
        match self {
            LocationKind::Home      => "Home",
            LocationKind::Pub       => "The Red Dragon",
            LocationKind::Common    => "The Common",
            LocationKind::River     => "The River",
            LocationKind::Mountain  => "The Mountain",
            LocationKind::Bakery    => "The Bakery",
            LocationKind::ChoirHall => "Choir Hall",
            LocationKind::Library   => "Library",
            LocationKind::Workplace => "Workplace",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Location {
    pub kind: LocationKind,
    pub tile_x: i32,
    pub tile_y: i32,
}

// ─── Weather ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weather { Sunny, Overcast, Rain, Fog }

impl Weather {
    pub fn label(self) -> &'static str {
        match self {
            Weather::Sunny    => "☀ Sunny",
            Weather::Overcast => "☁ Overcast",
            Weather::Rain     => "⛈ Rain",
            Weather::Fog      => "🌫 Fog",
        }
    }
    /// Visual brightness multiplier (applied on top of daylight)
    pub fn brightness(self) -> f32 {
        match self {
            Weather::Sunny    => 1.10,
            Weather::Overcast => 0.80,
            Weather::Rain     => 0.68,
            Weather::Fog      => 0.74,
        }
    }
}

// ─── WorldClock ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub enum Season { Spring, Summer, Autumn, Winter }

impl Season {
    pub fn label(self) -> &'static str {
        match self { Season::Spring => "Spring", Season::Summer => "Summer",
                     Season::Autumn => "Autumn", Season::Winter => "Winter" }
    }
}

#[derive(Clone, Debug)]
pub struct WorldClock {
    pub tick: u64,
    pub day: u32,
    /// 0.0 = midnight, 0.25 = dawn, 0.5 = noon, 0.75 = dusk, 1.0 = midnight
    pub time_of_day: f32,
    pub season: Season,
}

impl WorldClock {
    pub fn new() -> Self {
        WorldClock { tick: 0, day: 1, time_of_day: 0.28, season: Season::Spring }
    }

    pub fn advance(&mut self) {
        self.tick += 1;
        self.time_of_day = (self.tick as f32 / TICKS_PER_DAY as f32).fract();
        if self.tick % TICKS_PER_DAY as u64 == 0 {
            self.day += 1;
            self.season = match (self.day / 28) % 4 {
                0 => Season::Spring,
                1 => Season::Summer,
                2 => Season::Autumn,
                _ => Season::Winter,
            };
        }
    }

    pub fn time_label(&self) -> &'static str {
        match (self.time_of_day * 8.0) as u32 {
            0 | 7 => "Night",
            1     => "Dawn",
            2 | 3 => "Morning",
            4     => "Noon",
            5     => "Afternoon",
            6     => "Dusk",
            _     => "Night",
        }
    }

    pub fn is_night(&self) -> bool {
        self.time_of_day < 0.2 || self.time_of_day > 0.85
    }
}

// ─── Noise helpers (no external crate) ────────────────────────────────────────

fn hash2d(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(1_103_515_245))
        .wrapping_add(seed);
    h ^= h >> 13;
    h = h.wrapping_mul(1_274_126_177);
    h ^= h >> 16;
    (h as f32) / (u32::MAX as f32)
}

fn smooth_noise(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let u = xf * xf * (3.0 - 2.0 * xf);
    let v = yf * yf * (3.0 - 2.0 * yf);
    let a = hash2d(xi,     yi,     seed);
    let b = hash2d(xi + 1, yi,     seed);
    let c = hash2d(xi,     yi + 1, seed);
    let d = hash2d(xi + 1, yi + 1, seed);
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

pub fn fbm(x: f32, y: f32, seed: u32, octaves: u32) -> f32 {
    let mut val = 0.0f32;
    let mut amp = 0.5f32;
    let mut freq = 1.0f32;
    for i in 0..octaves {
        val  += amp * smooth_noise(x * freq, y * freq, seed.wrapping_add(i * 7919));
        amp  *= 0.5;
        freq *= 2.0;
    }
    val
}

// ─── Global Events ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum GlobalEvent {
    PitClosure,
    Eisteddfod,
    HardWinter,
    Bereavement { actor_id: usize },
}

impl GlobalEvent {
    pub fn label(&self) -> &'static str {
        match self {
            GlobalEvent::PitClosure       => "THE PIT HAS CLOSED",
            GlobalEvent::Eisteddfod       => "EISTEDDFOD",
            GlobalEvent::HardWinter       => "HARD WINTER",
            GlobalEvent::Bereavement {..} => "BEREAVEMENT",
        }
    }
}

// ─── SimWorld ─────────────────────────────────────────────────────────────────

pub struct SimWorld {
    pub tiles: Vec<Vec<TileKind>>,
    /// Per-cell tileset override: Some((col, row)) draws that exact tile from the
    /// Sunnyside tileset instead of the default tile_src() mapping.
    pub tile_ids: Vec<Vec<Option<(u8, u8)>>>,
    /// Pre-computed per-tile noise variation (added to base colour at render time)
    pub tile_noise: Vec<Vec<f32>>,
    pub locations: Vec<Location>,
    pub actors: Vec<Actor>,
    pub clock: WorldClock,
    pub chronicle: VecDeque<String>,
    pub pending_events: VecDeque<GlobalEvent>,
    pub weather: Weather,
    pub weather_timer: u32,
    pub seed: u64,
}

impl SimWorld {
    pub fn generate(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let s32 = seed as u32;

        // ── Build heightmap ──
        let mut elevation = vec![vec![0.0f32; MAP_WIDTH]; MAP_HEIGHT];
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let nx = x as f32 / MAP_WIDTH  as f32;
                let ny = y as f32 / MAP_HEIGHT as f32;

                // Valley shape: high at north/south edges, low in middle band
                let valley_center = 0.50;
                let dist = (ny - valley_center).abs() * 2.2;
                let valley = (dist * dist).min(1.0);

                // FBM noise for natural variation
                let noise = fbm(nx * 5.0, ny * 5.0, s32, 4);

                elevation[y][x] = (valley * 0.72 + noise * 0.28).clamp(0.0, 1.0);
            }
        }

        // ── Classify tiles ──
        let mut tiles = vec![vec![TileKind::Grass; MAP_WIDTH]; MAP_HEIGHT];
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                let e = elevation[y][x];
                tiles[y][x] = if e < 0.08 {
                    TileKind::DeepWater
                } else if e < 0.16 {
                    TileKind::Water
                } else if e < 0.28 {
                    TileKind::Grass
                } else if e < 0.50 {
                    TileKind::Grass
                } else if e < 0.68 {
                    TileKind::Bracken
                } else if e < 0.82 {
                    TileKind::Stone
                } else {
                    TileKind::Mountain
                };
            }
        }

        // ── Carve river along the valley floor (centre row, sinuous) ──
        let river_y_base = (MAP_HEIGHT as f32 * 0.50) as i32;
        for x in 0..MAP_WIDTH {
            let meander = (smooth_noise(x as f32 * 0.12, 0.5, s32 + 200) * 4.0) as i32 - 2;
            let ry = (river_y_base + meander).clamp(1, MAP_HEIGHT as i32 - 2);
            for dy in -1i32..=1 {
                let ty = (ry + dy) as usize;
                tiles[ty][x] = if dy == 0 { TileKind::DeepWater } else { TileKind::Water };
            }
        }

        // ── High Street: horizontal path just south of river ──
        let street_y = (river_y_base + 5).clamp(0, MAP_HEIGHT as i32 - 1) as usize;
        for x in 5..MAP_WIDTH - 5 {
            if tiles[street_y][x] == TileKind::Grass || tiles[street_y][x] == TileKind::Bracken {
                tiles[street_y][x] = TileKind::Path;
            }
        }
        // Second terrace row north of river
        let terrace_y = (river_y_base - 5).clamp(0, MAP_HEIGHT as i32 - 1) as usize;
        for x in 5..MAP_WIDTH - 5 {
            if tiles[terrace_y][x] == TileKind::Grass || tiles[terrace_y][x] == TileKind::Bracken {
                tiles[terrace_y][x] = TileKind::Path;
            }
        }

        // ── Place key buildings along the High Street ──
        let building_row = street_y + 1;
        let buildings_x = [
            12usize,  // Pub
            22usize,  // Bakery
            35usize,  // Choir Hall
            50usize,  // Library
            65usize,  // Workplace
        ];
        for &bx in &buildings_x {
            for dy in 0usize..3 {
                for dx in 0usize..4 {
                    let tx = (bx + dx).min(MAP_WIDTH - 1);
                    let ty = (building_row + dy).min(MAP_HEIGHT - 1);
                    tiles[ty][tx] = if dy == 0 || dy == 2 || dx == 0 || dx == 3 {
                        TileKind::BuildingWall
                    } else {
                        TileKind::BuildingFloor
                    };
                }
            }
        }

        // ── Terrace houses north of river ──
        let house_row = terrace_y - 2;
        for slot in 0..12usize {
            let hx = 6 + slot * 7;
            if hx + 3 >= MAP_WIDTH { break; }
            for dy in 0usize..3 {
                for dx in 0usize..4 {
                    let tx = (hx + dx).min(MAP_WIDTH - 1);
                    let ty = (house_row.saturating_sub(dy)).min(MAP_HEIGHT - 1);
                    tiles[ty][tx] = if dy == 2 || dx == 0 || dx == 3 {
                        TileKind::BuildingWall
                    } else {
                        TileKind::BuildingFloor
                    };
                }
            }
        }

        // ── Pre-compute per-tile noise variation (±0.15) ──
        let mut tile_noise = vec![vec![0.0f32; MAP_WIDTH]; MAP_HEIGHT];
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                tile_noise[y][x] = (smooth_noise(x as f32 * 0.8, y as f32 * 0.8, s32 + 500) - 0.5) * 0.20;
            }
        }

        // ── Named locations ──
        let river_x_mid = MAP_WIDTH / 2;
        let river_y_mid = river_y_base as usize;

        let locations = vec![
            Location { kind: LocationKind::River,     tile_x: river_x_mid as i32, tile_y: river_y_mid as i32 },
            Location { kind: LocationKind::Common,    tile_x: (MAP_WIDTH / 2) as i32, tile_y: street_y as i32 + 3 },
            Location { kind: LocationKind::Mountain,  tile_x: (MAP_WIDTH / 2) as i32, tile_y: 4 },
            Location { kind: LocationKind::Pub,       tile_x: buildings_x[0] as i32 + 1, tile_y: building_row as i32 + 1 },
            Location { kind: LocationKind::Bakery,    tile_x: buildings_x[1] as i32 + 1, tile_y: building_row as i32 + 1 },
            Location { kind: LocationKind::ChoirHall, tile_x: buildings_x[2] as i32 + 1, tile_y: building_row as i32 + 1 },
            Location { kind: LocationKind::Library,   tile_x: buildings_x[3] as i32 + 1, tile_y: building_row as i32 + 1 },
            Location { kind: LocationKind::Workplace, tile_x: buildings_x[4] as i32 + 1, tile_y: building_row as i32 + 1 },
        ];

        // ── Spawn actors ──
        let welsh_first: &[&str] = &[
            "Rhys","Dylan","Seren","Anwen","Gareth","Bronwen","Iwan","Catrin",
            "Emyr","Ffion","Huw","Megan","Llywelyn","Nia","Owen","Gwen",
            "Caerwyn","Nerys","Dafydd","Lowri","Bethan","Gethin","Elan","Tomos",
            "Carys","Alun","Eirwen","Prys",
        ];
        let welsh_last: &[&str] = &[
            "Morgan","Evans","Williams","Jones","Davies","Thomas","Hughes",
            "Price","Lloyd","Rees","Griffiths","Bowen","Lewis","Prosser",
        ];
        let roles = [
            Role::Miner, Role::Miner, Role::Miner,
            Role::Teacher, Role::Teacher,
            Role::Shopkeeper, Role::Shopkeeper,
            Role::Musician, Role::Musician,
            Role::Elder, Role::Elder,
            Role::Child, Role::Child, Role::Child,
            Role::NewArrival,
        ];

        let mut actors: Vec<Actor> = Vec::new();
        for id in 0..ACTOR_COUNT {
            let first = welsh_first[id % welsh_first.len()];
            let last  = welsh_last[rng.gen_range(0..welsh_last.len())];
            let name  = format!("{} {}", first, last);
            let role  = roles[id % roles.len()];

            // Spawn on a walkable tile in the lower half (between river and south slope)
            let (tx, ty) = find_spawn_tile(&tiles, &mut rng, street_y + 2, street_y + 12);

            let mut actor = Actor::new(id, name, role, tx, ty);
            actor.home_x = tx;
            actor.home_y = ty;
            actors.push(actor);
        }

        // ── Assign basic relationships (nearby actors start with positive weight) ──
        for i in 0..actors.len() {
            for j in 0..actors.len() {
                if i == j { continue; }
                let dx = (actors[i].tile_x - actors[j].tile_x).abs();
                let dy = (actors[i].tile_y - actors[j].tile_y).abs();
                if dx + dy < 8 {
                    let weight = rng.gen_range(0.1f32..0.5f32);
                    actors[i].relationships.push((j, weight));
                }
            }
        }

        SimWorld {
            tiles,
            tile_ids: vec![vec![None; MAP_WIDTH]; MAP_HEIGHT],
            tile_noise,
            locations,
            actors,
            clock: WorldClock::new(),
            chronicle: VecDeque::with_capacity(MAX_CHRONICLE),
            pending_events: VecDeque::new(),
            weather: Weather::Sunny,
            weather_timer: 90,
            seed,
        }
    }

    pub fn inject_event(&mut self, event: GlobalEvent) {
        self.pending_events.push_back(event);
    }

    pub fn find_location(&self, kind: LocationKind) -> Option<&Location> {
        self.locations.iter().find(|l| l.kind == kind)
    }

    pub fn log(&mut self, entry: String) {
        if self.chronicle.len() >= MAX_CHRONICLE {
            self.chronicle.pop_front();
        }
        self.chronicle.push_back(entry);
    }

    pub fn tile_at(&self, x: i32, y: i32) -> TileKind {
        if x < 0 || y < 0 || x >= MAP_WIDTH as i32 || y >= MAP_HEIGHT as i32 {
            return TileKind::Mountain;
        }
        self.tiles[y as usize][x as usize]
    }

    /// Procedurally generate a village using the blueprint system.
    pub fn generate_village(seed: u64) -> Self {
        use crate::sim::village_gen;

        let layout = village_gen::generate(seed);
        let mut rng = StdRng::seed_from_u64(seed ^ 0xBEEF_F00D);
        let s32 = seed as u32;

        // Per-tile noise
        let mut tile_noise = vec![vec![0.0f32; MAP_WIDTH]; MAP_HEIGHT];
        for y in 0..MAP_HEIGHT {
            for x in 0..MAP_WIDTH {
                tile_noise[y][x] = (smooth_noise(x as f32 * 0.8, y as f32 * 0.8, s32 + 500) - 0.5) * 0.15;
            }
        }

        // Spawn actors at walkable path/grass tiles
        let welsh_first: &[&str] = &[
            "Rhys","Dylan","Seren","Anwen","Gareth","Bronwen","Iwan","Catrin",
            "Emyr","Ffion","Huw","Megan","Llywelyn","Nia","Owen","Gwen",
            "Caerwyn","Nerys","Dafydd","Lowri","Bethan","Gethin","Elan","Tomos",
            "Carys","Alun","Eirwen","Prys",
        ];
        let welsh_last: &[&str] = &[
            "Morgan","Evans","Williams","Jones","Davies","Thomas","Hughes",
            "Price","Lloyd","Rees","Griffiths","Bowen","Lewis","Prosser",
        ];
        let roles = [
            Role::Miner, Role::Miner,
            Role::Teacher, Role::Shopkeeper,
            Role::Musician, Role::Elder,
            Role::Child, Role::Child,
            Role::NewArrival, Role::Shopkeeper,
        ];

        let spawns = &layout.spawns;
        let road_spawns: Vec<&(i32, i32)> = spawns.iter()
            .filter(|&&(x, y)| {
                let tk = layout.tiles[y as usize][x as usize];
                tk == TileKind::Path
            })
            .collect();

        let mut actors: Vec<Actor> = Vec::new();
        for id in 0..ACTOR_COUNT {
            let first = welsh_first[id % welsh_first.len()];
            let last  = welsh_last[rng.gen_range(0..welsh_last.len())];
            let name  = format!("{} {}", first, last);
            let role  = roles[id % roles.len()];

            let &(tx, ty) = if road_spawns.is_empty() {
                spawns.get(id % spawns.len().max(1)).unwrap_or(&(5, MAP_HEIGHT as i32 / 2))
            } else {
                road_spawns[rng.gen_range(0..road_spawns.len())]
            };

            let mut actor = Actor::new(id, name, role, tx, ty);
            actor.home_x = tx;
            actor.home_y = ty;
            actors.push(actor);
        }

        // Basic nearby relationships
        for i in 0..actors.len() {
            for j in 0..actors.len() {
                if i == j { continue; }
                let dx = (actors[i].tile_x - actors[j].tile_x).abs();
                let dy = (actors[i].tile_y - actors[j].tile_y).abs();
                if dx + dy < 12 {
                    actors[i].relationships.push((j, rng.gen_range(0.1f32..0.5f32)));
                }
            }
        }

        SimWorld {
            tiles:     layout.tiles,
            tile_ids:  layout.tile_ids,
            tile_noise,
            locations: layout.locations,
            actors,
            clock:     WorldClock::new(),
            chronicle: VecDeque::with_capacity(MAX_CHRONICLE),
            pending_events: VecDeque::new(),
            weather:       Weather::Sunny,
            weather_timer: 90,
            seed,
        }
    }
}

fn find_spawn_tile(
    tiles: &[Vec<TileKind>],
    rng: &mut StdRng,
    y_min: usize,
    y_max: usize,
) -> (i32, i32) {
    let y_max = y_max.min(MAP_HEIGHT - 1);
    for _ in 0..200 {
        let x = rng.gen_range(5..MAP_WIDTH - 5);
        let y = rng.gen_range(y_min..=y_max);
        if tiles[y][x].is_walkable() {
            return (x as i32, y as i32);
        }
    }
    (MAP_WIDTH as i32 / 2, (y_min + y_max) as i32 / 2)
}

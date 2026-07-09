use std::collections::VecDeque;
use crate::constants::*;

// ─── Node IDs ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeId {
    Hunger     = 0,
    Fatigue    = 1,
    Stress     = 2,
    Social     = 3,
    Joy        = 4,
    Grief      = 5,
    Purpose    = 6,
    Belonging  = 7,
    Curiosity  = 8,
    Resentment = 9,
}

impl NodeId {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Hunger,
            1 => Self::Fatigue,
            2 => Self::Stress,
            3 => Self::Social,
            4 => Self::Joy,
            5 => Self::Grief,
            6 => Self::Purpose,
            7 => Self::Belonging,
            8 => Self::Curiosity,
            9 => Self::Resentment,
            _ => Self::Hunger,
        }
    }
}

// ─── NodeGraph ────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct NodeGraph {
    pub values: [f32; NODE_COUNT],
    /// edges[from][to] = signed weight; positive = excitatory, negative = inhibitory
    pub edges: [[f32; NODE_COUNT]; NODE_COUNT],
    /// natural drift rate per tick (positive = rises, negative = decays)
    pub drift: [f32; NODE_COUNT],
    /// threshold: excess above this value propagates to connected nodes
    pub thresholds: [f32; NODE_COUNT],
}

impl NodeGraph {
    pub fn new_for_role(role: Role) -> Self {
        let mut ng = NodeGraph {
            values: [0.30; NODE_COUNT],
            edges: [[0.0; NODE_COUNT]; NODE_COUNT],
            drift: [0.0; NODE_COUNT],
            thresholds: [0.50; NODE_COUNT],
        };

        // ── Universal edges (from GDD) ──
        ng.set_edge(NodeId::Stress,      NodeId::Social,      0.40);
        ng.set_edge(NodeId::Stress,      NodeId::Grief,       0.30);
        ng.set_edge(NodeId::Social,      NodeId::Resentment,  0.50);
        ng.set_edge(NodeId::Joy,         NodeId::Purpose,     0.60);
        ng.set_edge(NodeId::Grief,       NodeId::Purpose,    -0.40);
        ng.set_edge(NodeId::Belonging,   NodeId::Joy,         0.30);
        ng.set_edge(NodeId::Curiosity,   NodeId::Joy,         0.30);
        ng.set_edge(NodeId::Resentment,  NodeId::Stress,      0.20);
        ng.set_edge(NodeId::Purpose,     NodeId::Stress,     -0.30);

        // ── Universal drift rates ──
        ng.drift[NodeId::Hunger     as usize] =  0.008;  // rises naturally
        ng.drift[NodeId::Fatigue    as usize] =  0.005;  // rises naturally
        ng.drift[NodeId::Joy        as usize] = -0.012;  // decays
        ng.drift[NodeId::Grief      as usize] = -0.002;  // decays very slowly
        ng.drift[NodeId::Social     as usize] =  0.006;  // rises (loneliness)
        ng.drift[NodeId::Curiosity  as usize] =  0.004;  // rises
        ng.drift[NodeId::Stress     as usize] = -0.003;  // self-regulates down
        ng.drift[NodeId::Resentment as usize] = -0.002;  // decays slowly

        // ── Role-specific starting values ──
        match role {
            Role::Miner => {
                ng.values[NodeId::Belonging as usize] = 0.72;
                ng.values[NodeId::Purpose   as usize] = 0.65;
                ng.values[NodeId::Grief     as usize] = 0.42; // pit-closure weight
            }
            Role::Teacher => {
                ng.values[NodeId::Curiosity as usize] = 0.70;
                ng.values[NodeId::Purpose   as usize] = 0.72;
                ng.values[NodeId::Social    as usize] = 0.38;
            }
            Role::Shopkeeper => {
                ng.values[NodeId::Social    as usize] = 0.28;
                ng.values[NodeId::Belonging as usize] = 0.60;
            }
            Role::Musician => {
                ng.values[NodeId::Joy       as usize] = 0.68;
                ng.values[NodeId::Belonging as usize] = 0.65;
                ng.drift[NodeId::Joy        as usize] = -0.007; // joy decays slower
            }
            Role::Elder => {
                ng.values[NodeId::Belonging as usize] = 0.88;
                ng.values[NodeId::Grief     as usize] = 0.52;
                ng.values[NodeId::Purpose   as usize] = 0.50;
                for d in ng.drift.iter_mut() { *d *= 0.65; } // slower drift
            }
            Role::Child => {
                ng.values[NodeId::Curiosity as usize] = 0.82;
                ng.values[NodeId::Joy       as usize] = 0.75;
                ng.values[NodeId::Resentment as usize] = 0.05;
                ng.drift[NodeId::Curiosity  as usize] =  0.012;
                ng.drift[NodeId::Joy        as usize] = -0.020; // spikes and drops fast
            }
            Role::NewArrival => {
                ng.values[NodeId::Belonging as usize] = 0.08;
                ng.values[NodeId::Curiosity as usize] = 0.62;
                ng.values[NodeId::Social    as usize] = 0.55;
            }
        }

        ng
    }

    fn set_edge(&mut self, from: NodeId, to: NodeId, w: f32) {
        self.edges[from as usize][to as usize] = w;
    }

    /// Returns the node with the highest urgency (deviation from 0.5 baseline).
    pub fn dominant_node(&self) -> NodeId {
        let mut best = 0;
        let mut best_score = 0.0f32;
        for i in 0..NODE_COUNT {
            let score = (self.values[i] - 0.35).abs();
            if score > best_score {
                best_score = score;
                best = i;
            }
        }
        NodeId::from_index(best)
    }
}

// ─── Role ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Miner,
    Teacher,
    Shopkeeper,
    Musician,
    Elder,
    Child,
    NewArrival,
}

impl Role {
    pub fn display_name(self) -> &'static str {
        match self {
            Role::Miner      => "Miner",
            Role::Teacher    => "Teacher",
            Role::Shopkeeper => "Shopkeeper",
            Role::Musician   => "Musician",
            Role::Elder      => "Elder",
            Role::Child      => "Child",
            Role::NewArrival => "New Arrival",
        }
    }

    /// Base body colour (r, g, b)
    pub fn color(self) -> (f32, f32, f32) {
        match self {
            Role::Miner      => (0.90, 0.50, 0.18),
            Role::Teacher    => (0.60, 0.38, 0.80),
            Role::Shopkeeper => (0.28, 0.72, 0.50),
            Role::Musician   => (0.92, 0.72, 0.18),
            Role::Elder      => (0.72, 0.72, 0.72),
            Role::Child      => (0.92, 0.38, 0.52),
            Role::NewArrival => (0.45, 0.82, 0.92),
        }
    }
}

// ─── Action ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum Action {
    Idle,
    Eating,
    Sleeping,
    Socialising,
    Isolating,
    Creating,
    Helping,
    Grieving,
    Singing,
    Walking,
}

impl Action {
    pub fn label(&self) -> &'static str {
        match self {
            Action::Idle       => "idle",
            Action::Eating     => "eating",
            Action::Sleeping   => "sleeping",
            Action::Socialising=> "socialising",
            Action::Isolating  => "seeking solitude",
            Action::Creating   => "creating",
            Action::Helping    => "helping someone",
            Action::Grieving   => "grieving",
            Action::Singing    => "singing",
            Action::Walking    => "walking",
        }
    }
}

// ─── Memory ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct MemoryEvent {
    pub day: u32,
    pub text: String,
}

// ─── Actor ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Actor {
    pub id: usize,
    pub name: String,
    pub role: Role,

    // Logical grid position
    pub tile_x: i32,
    pub tile_y: i32,

    // Smooth visual position (lerped toward tile_x/y each frame)
    pub pixel_x: f32,
    pub pixel_y: f32,

    // Movement target
    pub target_x: i32,
    pub target_y: i32,

    pub node_graph: NodeGraph,
    pub relationships: Vec<(usize, f32)>, // (actor_id, weight -1..1)
    pub memory: VecDeque<MemoryEvent>,
    pub current_action: Action,

    pub home_x: i32,
    pub home_y: i32,

    /// Ticks remaining before picking a new action
    pub action_cooldown: i32,
}

impl Actor {
    pub fn new(id: usize, name: String, role: Role, tile_x: i32, tile_y: i32) -> Self {
        let ng = NodeGraph::new_for_role(role);
        let px = tile_x as f32 * TILE_SIZE;
        let py = tile_y as f32 * TILE_SIZE;
        Actor {
            id,
            name,
            role,
            tile_x,
            tile_y,
            pixel_x: px,
            pixel_y: py,
            target_x: tile_x,
            target_y: tile_y,
            node_graph: ng,
            relationships: Vec::new(),
            memory: VecDeque::with_capacity(MAX_MEMORY),
            current_action: Action::Idle,
            home_x: tile_x,
            home_y: tile_y,
            action_cooldown: 0,
        }
    }

    pub fn at_target(&self) -> bool {
        self.tile_x == self.target_x && self.tile_y == self.target_y
    }

    pub fn push_memory(&mut self, day: u32, text: String) {
        if self.memory.len() >= MAX_MEMORY {
            self.memory.pop_front();
        }
        self.memory.push_back(MemoryEvent { day, text });
    }
}

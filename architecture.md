# Architecture: The Nature of Things

> A pure-code, headless-first, ECS-driven simulation architecture.

---

## 1. Guiding Principles

1. **Simulation is sovereign.** The world model runs independently of any renderer. A full day of simulation can complete in a terminal with zero graphical output.
2. **ECS as the data model.** Entities, Components, and Systems are the only organisational primitive. No inheritance hierarchies.
3. **Math, not assets.** Terrain, structures, and visuals are computed from noise functions and signed distance fields — not loaded from files.
4. **Strict layer separation.** Three layers exist: `sim`, `render`, and `app`. They communicate in one direction only: `sim` → `render`. The simulation never imports rendering code.
5. **Compiled, fast, local.** Targeting Apple Silicon (ARM64). The simulation tick budget is microseconds, not milliseconds.

---

## 2. Recommended Stack

| Concern | Choice | Rationale |
|---|---|---|
| Language | **Rust** | Zero-cost abstractions, fearless concurrency, excellent ARM64 performance, no GC pauses mid-tick. |
| ECS Framework | **Bevy ECS** (headless, no `bevy_render`) | Purpose-built ECS, schedules, systems, and queries with minimal boilerplate. Can be used without any window or renderer. |
| Math | `glam` (bundled with Bevy) | SIMD-accelerated vector/matrix math for node propagation. |
| Noise | `noise-rs` | Simplex/Perlin for terrain and procedural generation. |
| Serialisation | `serde` + `ron` | Human-readable save states; easy to inspect in editor. |
| Terminal Output | `crossterm` + `ratatui` | Rich TUI for headless simulation observation during Phase 1. |
| Test Runner | `cargo test` | Standard. Simulation logic is pure functions — trivially testable. |
| Build | `cargo` | Single command builds. No external build system. |

> **Alternative:** If Rust is ruled out, a TypeScript + custom ECS loop (inspired by `bitECS`) with Node.js as the headless runtime is a viable fallback. The architecture described below maps directly to that model.

---

## 3. Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│                        APP LAYER                        │
│   main.rs — wires sim + render, handles time loop      │
└────────────────┬────────────────────────────────────────┘
                 │ read-only snapshot / events
┌────────────────▼────────────────────────────────────────┐
│                      RENDER LAYER                       │
│   (Phase 2+) bevy_render / ratatui TUI / Raylib        │
│   Reads World snapshots. Never writes sim state.       │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│                       SIM LAYER                         │
│   Pure ECS world. No rendering imports.                 │
│   Schedules: Tick → Propagate → Act → Age → Events     │
└─────────────────────────────────────────────────────────┘
```

The sim layer exposes a single public interface:
- `SimWorld::tick(dt)` — advances the world by one step.
- `SimWorld::snapshot() -> WorldSnapshot` — serialisable read-only view of current state.
- `SimWorld::inject_event(event: GlobalEvent)` — externally triggers a global simulation event.

The render layer consumes `WorldSnapshot`. It never holds mutable references to sim state.

---

## 4. ECS Design

### 4.1 Components

```rust
// Identity
struct ActorId(u64);
struct ActorName(String);
struct ActorRole(Role);       // Miner, Teacher, Elder, Child, etc.

// Position
struct GridPosition { x: i32, y: i32 }
struct Location(LocationId);  // Which named location the actor is in

// Behavioral Automata — the node graph
struct NodeGraph {
    nodes: [f32; NODE_COUNT],  // indexed by NodeId enum
    edges: EdgeMatrix,         // sparse adjacency + weight matrix
}

// Relationships
struct RelationshipRegister {
    entries: Vec<(ActorId, f32)>,  // (other_id, weight -1.0..1.0)
}

// Memory
struct MemoryBuffer {
    events: RingBuffer<MemoryEvent>,
}

// Action
struct CurrentAction(Option<Action>);
struct ActionCooldown(f32);

// World / environment
struct TimeOfDay(f32);         // 0.0..1.0
struct Season(SeasonKind);
struct Location { id: LocationId, position: Vec2, capacity: u32 }
struct Weather(WeatherKind);
```

### 4.2 Systems Schedule

Systems run in this fixed order each tick:

```
Schedule: SimTick
├── [1] environmental_input_system
│       Reads TimeOfDay, Season, Weather → injects deltas into NodeGraph
│
├── [2] node_decay_system
│       Applies per-node natural drift rates (hunger rises, fatigue rises, joy decays)
│
├── [3] intra_actor_propagation_system
│       For each actor: runs cross-node propagation pass on their NodeGraph
│       Uses edge matrix weights; clamps results to [0.0, 1.0]
│
├── [4] inter_actor_contagion_system
│       For actors sharing a Location: applies social propagation (joy, stress, grief contagion)
│       Weighted by RelationshipRegister entries
│
├── [5] action_selection_system
│       Reads NodeGraph urgency scores → selects Action from available tagged actions at Location
│       Writes CurrentAction
│
├── [6] action_execution_system
│       Executes CurrentAction → modifies NodeGraph (satisfaction deltas)
│       Updates RelationshipRegister on social interactions
│
├── [7] memory_recording_system
│       Detects threshold crossings → appends MemoryEvent to MemoryBuffer
│
├── [8] event_broadcast_system
│       Reads pending GlobalEvents → applies wide-area node delta injections
│
└── [9] clock_advance_system
        Advances TimeOfDay, Season; triggers day/season change events
```

### 4.3 Resources (World-Singletons)

```rust
struct WorldClock { tick: u64, day: u32, season: SeasonKind }
struct PendingEvents(Vec<GlobalEvent>)
struct TownMap { locations: HashMap<LocationId, Location>, graph: RoadGraph }
struct SimConfig { tick_rate_hz: f32, propagation_dt: f32, ... }
```

---

## 5. The NodeGraph in Detail

```rust
const NODE_COUNT: usize = 10;

#[repr(usize)]
enum NodeId {
    Hunger    = 0,
    Fatigue   = 1,
    Stress    = 2,
    Social    = 3,
    Joy       = 4,
    Grief     = 5,
    Purpose   = 6,
    Belonging = 7,
    Curiosity = 8,
    Resentment= 9,
}

struct EdgeMatrix([[f32; NODE_COUNT]; NODE_COUNT]);
// edge[from][to] = signed weight. Positive = excitatory. Negative = inhibitory.
// Sparse: most entries are 0.0.
```

**Propagation pass (per actor, per tick):**

```
for each source_node s:
    if nodes[s] > THRESHOLD[s]:
        excess = nodes[s] - THRESHOLD[s]
        for each edge (s → t) with weight w:
            delta = excess * w * propagation_dt
            nodes[t] = clamp(nodes[t] + delta, 0.0, 1.0)
```

Edge weights are initialised per role archetype and can drift slightly over an actor's lifetime (slow character development).

---

## 6. Procedural World Generation Pipeline

```
Seed (u64)
  │
  ▼
[1] Heightmap           noise::Simplex → valley shape array
  │
  ▼
[2] Road Graph          WFC / L-system → high street + terraced rows
  │
  ▼
[3] Parcel Assignment   Place location types (pub, bakery, homes, common, mountain)
  │
  ▼
[4] Actor Seeding       Spawn N actors with role distribution, assign homes
  │
  ▼
[5] Relationship Init   Assign initial relationship weights (neighbours closer, strangers neutral)
  │
  ▼
[6] NodeGraph Init      Per-role baseline values + small random noise
```

All steps are deterministic given the same seed. Save state = seed + tick count + any injected events log.

---

## 7. Headless Operation

The sim layer has zero dependency on any windowing or rendering crate. A headless binary runs the full simulation loop and outputs to stdout or a TUI:

```rust
// crates/sim — pure simulation, no render deps
// crates/app — wires everything, feature-gated rendering

[features]
default = ["headless"]
headless = []
render = ["bevy_render", "bevy_winit", ...]
```

Running headless:
```bash
cargo run --features headless
```

Running with renderer (Phase 2+):
```bash
cargo run --features render
```

---

## 8. Crate Structure

```
the_nature_of_things/
├── Cargo.toml               # workspace root
├── crates/
│   ├── sim/                 # Pure simulation: ECS world, systems, components
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── components/
│   │   │   ├── systems/
│   │   │   ├── resources/
│   │   │   ├── events.rs
│   │   │   └── world_gen.rs
│   │   └── Cargo.toml
│   │
│   ├── render/              # Rendering layer (Phase 2+, feature-gated)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tui.rs       # Phase 1: ratatui terminal renderer
│   │   │   └── pixel/       # Phase 2: pixel art renderer
│   │   └── Cargo.toml
│   │
│   └── app/                 # Binary entry point — wires sim + render
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
│
├── assets/                  # Generated assets (shaders, fonts) — no static sprites
├── saves/                   # Serialised world snapshots
└── tests/                   # Integration tests for simulation invariants
```

---

## 9. Testing Strategy

- **Unit tests** (`sim` crate): Each system is a pure function testable in isolation. Assert that propagation rules produce expected node deltas.
- **Invariant tests**: After N ticks, assert global invariants hold (e.g., no node value exceeds 1.0, actors do not teleport, relationship weights stay in range).
- **Snapshot regression tests**: Seed a world, run 1000 ticks, assert the serialised snapshot matches a known-good fixture. Detects accidental behavioural changes.
- **Headless integration test**: Run a full simulation day in CI. Assert no panics, no dead actors (unless age system is implemented), chronicle log is non-empty.

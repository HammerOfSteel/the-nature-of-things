# The Nature of Things

> *A cozy, emergent simulation of a South Wales valley town — rendered as a high-resolution volumetric voxel world.*

---

## What Is This?

*The Nature of Things* is a simulation game set in a procedurally generated small town in the South Wales valleys. The player observes, not controls. A community of autonomous actors lives, works, grieves, sings in the chapel, and drinks in the pub — driven entirely by the **Behavioral Automata** engine.

There is no authored story. Everything emerges from math.

The world is rendered as a **3D holographic voxel field**: thousands of tiny coloured cubes with glowing wireframe edges on a dark background. Resolution is high enough to model the inside of a terraced house room by room — beds, bookshelves, fireplaces, kitchen ranges. Characters are procedural voxel assemblies with unique faces, role-based clothing, and size variation by age.

---

## Setting

A steep V-shaped glacial valley in the South Wales coalfield. A river along the floor. Victorian terraced houses climbing both hillsides. A nonconformist chapel. A working men's club. A derelict colliery headframe on the hillside. Sheep on the upper moorland. Ravens on the rooftops.

Every building is hollow and furnished. Every room is navigable.

---

## Key Features

- **Behavioral Automata AI** — Ten emotional/psychological nodes per actor (Stress, Hunger, Social, Purpose, Belonging, Joy, Grief, Energy, Suspicion, Hope). States propagate by weight on every tick — no scripted behavior trees, no if/else logic.
- **Community Contagion** — A single singer in the pub ripples joy outward through the crowd.
- **Volumetric Voxel World** — High-resolution 3D voxels (0.4 m³ per voxel). Chunk-based, surface-only rendering. Interior spaces fully modelled with furniture.
- **Procedural Everything** — Town layout, terrain, buildings, interiors, and characters all generated from noise functions and grammar rules.
- **South Wales DNA** — Biome layers from valley floor to moorland ridge. Cultural specifics encoded in the sim: solidarity in hardship, chapel Sundays, rugby match moods, economic anxiety.
- **Headless-First** — The simulation world model runs as a pure data process, fully decoupled from the renderer.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (stable) |
| Window + 2D UI | macroquad 0.4 |
| 3D / Voxel rendering | macroquad Camera3D → wgpu instancing (Phase 6) |
| Noise / procgen | Inline FBM (no extra crate at POC scale) |
| Serialisation | serde + ron (planned) |
| Build | cargo |

Target hardware: Apple Silicon M4. Goal: 60 fps at 512×512×48 voxel world with full simulation running.

No Unity. No Godot. No proprietary editors. 100% VS Code + `cargo`.

---

## Repository Structure

```
the_nature_of_things/
├── src/
│   ├── sim/           # Headless simulation core
│   ├── voxel/         # Voxel world, chunk grid, procgen (in progress)
│   ├── render/        # Renderers (volumetric primary + legacy POCs)
│   └── bin/
│       ├── poc_voxel.rs      # ← primary: high-res volumetric world
│       ├── poc_volumetric.rs # holographic tile-height render
│       ├── poc_village.rs    # 2D Sunnyside tileset
│       ├── poc_3d.rs         # macroquad 3D camera
│       └── poc_2d.rs         # classic top-down
├── tools/             # Python asset analysis scripts
├── gdd.md             # Game Design Document
├── architecture.md    # Technical architecture
├── overview.md        # Vision and setting
└── todo.md            # Implementation plan
```

---

## Getting Started

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run the primary volumetric voxel POC
cargo run --bin poc_voxel

# Controls:  SPACE new world  |  R auto-orbit  |  A/D rotate  |  W/S tilt  |  scroll zoom

# Run the original 2D simulation
cargo run

# Run all POCs in background
cargo run --bin poc_voxel      # high-res voxel world  ← current direction
cargo run --bin poc_volumetric # holographic tile-height
cargo run --bin poc_village    # 2D Sunnyside village
cargo run --bin poc_3d         # 3D camera sim
```

---

## Current Status

| Phase | Status |
|---|---|
| Phase 1: Behavioral Automata sim | ✅ Complete |
| Phase 2: Simulation depth (pathfinding, events, lifecycle) | ✅ Complete |
| Phase 3: Visual POCs (2D, 3/4, 3D, village) | ✅ Complete |
| **Phase 6: Volumetric voxel foundation** | 🚧 In progress |
| Phase 7: South Wales landscape | 📋 Planned |
| Phase 8: Interior detail (rooms, furniture) | 📋 Planned |
| Phase 9: Voxel characters (adults, children, animals) | 📋 Planned |
| Phase 10: Sim × voxel integration | 📋 Planned |
| Phase 11: Performance & GPU instancing | 📋 Planned |

See [todo.md](todo.md) for the full implementation plan.

---

## Design Philosophy

1. **Emergence over authorship.** The most interesting stories are the ones the system didn't know it was telling.
2. **Separation of concerns is non-negotiable.** The math runs first. The visuals are a window onto math that already happened.
3. **The setting is not decoration.** The South Wales valleys carry specific economic, cultural, and emotional weight. The mechanics reflect that honestly.
4. **Resolution enables imagination.** A voxel world detailed enough to show a cup on a kitchen table is detailed enough to make you believe someone actually lives there.

---

## License

MIT

---

## Key Features

- **Behavioral Automata AI** — Each actor's psychology is modelled as a weighted node graph. On every tick, internal states propagate across nodes (stress bleeds into social need; grief suppresses purpose; belonging provides baseline joy). Behavior emerges from the live state of the graph with no hardcoded if/else logic.
- **Community Contagion** — Joy, stress, and grief spread between actors sharing a space. A single singer in a pub ripples outward.
- **Headless-First Simulation** — The world model runs as a pure terminal process, completely decoupled from any renderer.
- **Procedural Everything** — Town layout, terrain, actors, and visuals are generated from code (noise functions, WFC, SDFs). No static sprite sheets.
- **Welsh Valley Setting** — Systemic biases encode cultural specifics: solidarity in hardship, the weight of economic history, singing as community ritual.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust |
| ECS Framework | Bevy ECS (headless) |
| Math | `glam` (SIMD) |
| Noise / Procgen | `noise-rs` |
| Terminal UI | `ratatui` + `crossterm` |
| Serialisation | `serde` + `ron` |
| Build | `cargo` |

Targets: Apple Silicon (ARM64), macOS. No Unity. No Godot. Pure VS Code + terminal.

---

## Repository Structure

```
the_nature_of_things/
├── crates/
│   ├── sim/        # Pure simulation — ECS world, systems, Behavioral Automata
│   ├── render/     # Rendering layer (Phase 2+, feature-gated)
│   └── app/        # Binary entry point
├── saves/          # Serialised world snapshots
├── tests/          # Simulation invariant & regression tests
├── gdd.md          # Game Design Document
├── architecture.md # Technical Architecture
└── todo.md         # Implementation Plan
```

---

## Getting Started

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify
rustc --version  # 1.78+
cargo --version
```

### Build & Run (Headless Simulation)

```bash
# Clone
git clone https://github.com/HammerOfSteel/the-nature-of-things.git
cd the-nature-of-things

# Run headless simulation loop with TUI output
cargo run -p app --features headless

# Run all tests
cargo test --workspace

# Run simulation with a fixed seed (reproducible world)
cargo run -p app --features headless -- --seed 42

# Fast-forward N days and dump chronicle log
cargo run -p app --features headless -- --seed 42 --days 30 --dump-chronicle
```

### Output

In headless mode the terminal shows:
- A live node graph readout for a focused actor
- The rolling **Chronicle** — significant events across the entire town
- Current world clock, season, and weather
- Actor population summary with dominant behavioral states

---

## Documents

| File | Contents |
|---|---|
| [gdd.md](gdd.md) | Full game design: Behavioral Automata nodes, propagation rules, action system, Welsh valley world design |
| [architecture.md](architecture.md) | ECS architecture, crate structure, system schedule, procedural generation pipeline |
| [todo.md](todo.md) | Prioritised implementation plan, Phase 1 through Phase 4 |

---

## Design Philosophy

The simulation is built on three convictions:

1. **Emergence over authorship.** The most interesting stories are the ones the system didn't know it was telling.
2. **Separation of concerns is non-negotiable.** The math runs first. The visuals are a window onto math that already happened.
3. **The setting is not decoration.** The South Wales valleys carry specific economic, cultural, and emotional weight. The mechanics reflect that honestly.

---

## Status

Early development. Phase 1 (headless simulation) in progress. See [todo.md](todo.md).

---

## License

MIT

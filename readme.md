# The Nature of Things

> *A cozy, emergent slice-of-life simulation of a Welsh valley town. No quests. No win state. Just lives unfolding.*

---

## What Is This?

*The Nature of Things* is a simulation game set in a procedurally generated small town in the South Wales valleys. The player observes — they do not control. A community of actors lives, works, grieves, sings, and endures, driven entirely by an emergent AI system called the **Behavioral Automata**.

There is no authored story. Everything that happens emerges from math.

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

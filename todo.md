# Implementation Plan: The Nature of Things

Phases are strictly ordered. No phase begins until the previous one produces verifiable terminal output or passing tests.

---

## Phase 1 — Headless Behavioral Automata (Terminal Only)

> Goal: A fully running simulation with no graphics, no window, no game loop beyond a CLI. Actors live in your terminal.

### 1.1 Project Scaffold
- [ ] `cargo new` workspace with three crates: `sim`, `render`, `app`
- [ ] Add `bevy_ecs`, `glam`, `serde`, `ron`, `ratatui`, `crossterm`, `clap` to appropriate `Cargo.toml`s
- [ ] Feature-flag `render` behind a Cargo feature; default is `headless`
- [ ] Verify `cargo build --features headless` compiles cleanly with zero warnings

### 1.2 Core ECS Components
- [ ] Define `NodeId` enum (10 nodes as per GDD)
- [ ] Implement `NodeGraph` component with `[f32; NODE_COUNT]` values and `EdgeMatrix`
- [ ] Implement `ActorId`, `ActorName`, `ActorRole` components
- [ ] Implement `GridPosition` and `Location` components
- [ ] Implement `RelationshipRegister` component (Vec of `(ActorId, f32)`)
- [ ] Implement `MemoryBuffer` component with fixed-capacity ring buffer
- [ ] Implement `CurrentAction` and `ActionCooldown` components

### 1.3 World Resources
- [ ] Implement `WorldClock` resource (tick, day, season)
- [ ] Implement `PendingEvents` resource
- [ ] Implement `SimConfig` resource (tick rate, propagation dt, thresholds)
- [ ] Implement `TownMap` resource with `HashMap<LocationId, Location>`

### 1.4 Simulation Systems (in schedule order)
- [ ] `environmental_input_system` — time-of-day/season → node deltas
- [ ] `node_decay_system` — per-node drift rates (hunger rises, joy decays, etc.)
- [ ] `intra_actor_propagation_system` — cross-node edge propagation pass
- [ ] `inter_actor_contagion_system` — social propagation between co-located actors
- [ ] `action_selection_system` — urgency scoring → action selection
- [ ] `action_execution_system` — execute action → satisfaction deltas
- [ ] `memory_recording_system` — threshold crossings → memory events
- [ ] `event_broadcast_system` — consume `PendingEvents` → wide-area deltas
- [ ] `clock_advance_system` — tick time forward, trigger day/season events

### 1.5 Role Archetypes
- [ ] Define `Role` enum: Miner, Teacher, Shopkeeper, Musician, Elder, Child, NewArrival
- [ ] Implement per-role baseline `NodeGraph` initialisation (values + edge weights)
- [ ] Implement per-role action tag access list

### 1.6 Headless World Generation (Seeded)
- [ ] Implement stub `TownMap` generator with fixed set of named locations
- [ ] Implement actor spawner: N actors, role distribution, assigned home locations
- [ ] Implement relationship initialiser: neighbours start with positive weight, strangers neutral
- [ ] Make generation fully deterministic from `u64` seed

### 1.7 CLI & Terminal Output
- [ ] `clap`-based CLI: `--seed`, `--days`, `--tick-rate`, `--focus-actor`, `--dump-chronicle`
- [ ] Implement `ratatui` TUI layout:
  - Panel 1: World clock, season, weather
  - Panel 2: Live node graph for focused actor (bar chart per node)
  - Panel 3: Rolling Chronicle log (last N memory events, all actors)
  - Panel 4: Population summary table (actor, role, dominant node, current action)
- [ ] `--dump-chronicle` flag: run N days headless, write chronicle to stdout as plain text, exit

### 1.8 Phase 1 Tests
- [ ] Unit test: propagation pass produces correct node deltas for known edge matrix
- [ ] Unit test: `node_decay_system` clamps values to `[0.0, 1.0]`
- [ ] Unit test: `action_selection_system` selects highest-urgency action
- [ ] Unit test: seeded world generation is deterministic (run twice, compare)
- [ ] Integration test: run 100 ticks headless, assert no panics, chronicle is non-empty
- [ ] Invariant test: after 1000 ticks, all node values in `[0.0, 1.0]`, no NaN

**Phase 1 exit criterion:** `cargo run -p app -- --seed 42 --days 7 --dump-chronicle` prints a coherent 7-day chronicle of a small town to stdout with no errors.

---

## Phase 2 — Procedural World Generation

> Goal: Replace stub town map with a fully procedurally generated valley.

- [ ] Integrate `noise-rs`; implement Simplex heightmap → valley shape
- [ ] Implement WFC road/parcel layout (high street, terraced rows, common, mountain path)
- [ ] Assign location types to parcels based on layout rules
- [ ] Generate actor population scaled to town size
- [ ] Generate Welsh first names and place-derived surnames from embedded corpus
- [ ] Validate: seeded world looks meaningfully different from another seed
- [ ] Regression test: snapshot test on `--seed 1` world at tick 0 matches fixture

---

## Phase 3 — Global Events & Emergent Narrative

> Goal: The world can change. Communities respond as systems.

- [ ] Implement `GlobalEvent` enum: PitClosure, Eisteddfod, HardWinter, NewFamilyArrives, Bereavement
- [ ] Implement `event_broadcast_system` wide-area propagation for each event type
- [ ] Add event scheduler: events can be triggered on a schedule or randomly with configurable probability
- [ ] Verify solidarity cascade: PitClosure → elevated community Belonging/Grief response
- [ ] Chronicle system: tag memory events with source (spontaneous vs. triggered by global event)
- [ ] CLI: `--inject-event PitClosure@day=10` flag for reproducible testing
- [ ] Relationship drift: add long-term weight decay/growth based on interaction history

---

## Phase 4 — Terminal Visualisation Polish

> Goal: The TUI becomes genuinely readable and expressive. This is the "playable" headless build.

- [ ] Actor focus cycling: keyboard nav to step through actors
- [ ] Relationship web view: display relationship weights for focused actor
- [ ] Memory timeline: per-actor view of life events in chronological order
- [ ] World event log with causal tagging (event → downstream effects)
- [ ] Export: `--export-html` generates a static chronicle page
- [ ] Performance: profile tick loop; target <1ms per tick for 100 actors on M4

---

## Phase 5 — Rendering Layer (Pixel Art)

> Goal: Attach a visual window to the running simulation. Sim code is not modified.

- [ ] Add `render` feature to workspace; gate all render imports behind it
- [ ] Implement `WorldSnapshot` serialisation from `sim` crate (read-only view)
- [ ] Choose renderer: Bevy render pipeline or Raylib bindings
- [ ] Implement top-down tile renderer: each `Location` maps to screen region
- [ ] Actor sprites: SDF-based procedural character rendering (no static sheets)
- [ ] Terrain: noise-driven shader for hillside, river, common
- [ ] Animate actors: simple walk cycles driven by `CurrentAction`
- [ ] Overlay: toggle node graph visualisation on click
- [ ] Verify: `cargo run --features render` opens window; `cargo run` (headless) still works

---

## Phase 6 — Polish & Release

- [ ] Soundtrack: generative ambient music responding to community state (joy level → brightness, grief → minor keys)
- [ ] Accessibility: configurable tick speed, font size, colour contrast modes
- [ ] Save/load: serialise full world state to `.ron` file via `serde`
- [ ] Seed sharing: printable seed code that recreates a specific world
- [ ] Itch.io build: macOS ARM64 binary + web export (if WASM target is viable)
- [ ] Write postmortem / devlog

---

## Running the Implementation

At any phase, the single source of truth is:

```bash
cargo test --workspace          # all tests pass
cargo run -p app -- --seed 1    # world runs without errors
```

Never commit code that breaks either of these.

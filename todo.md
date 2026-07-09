# Implementation Plan: The Nature of Things

> Stack: Rust · macroquad · plain `Vec<Actor>` (no ECS framework yet) · inline noise

---

## ✅ Phase 1 — Working Prototype (DONE)

> Shipped: top-down rendered sim with behavioral automata running in real time.

- [x] Single-crate project (`Cargo.toml` + `src/`) with `macroquad` and `rand`
- [x] `NodeId` enum (10 nodes), `NodeGraph` with edge matrix, per-role baselines
- [x] `Actor` struct: position, relationships, memory ring buffer, action state
- [x] `SimWorld`: seeded proc-gen valley (noise heightmap → tile classification → road/building placement)
- [x] Welsh name corpus; role distribution across 28 actors
- [x] All sim systems: decay → propagation → contagion → movement → arrival/action → memory
- [x] `WorldClock` with time-of-day and season cycling
- [x] macroquad renderer: noise-varied tile colours, actor body/head shapes, bobbing animation, night dimming
- [x] Right panel: live node graph bars, current action, memory log, Chronicle
- [x] Controls: WASD/arrow pan, click to select, `[`/`]` speed, `Space` pause, `Esc` deselect
- [x] `cargo build` compiles clean; `cargo run [seed]` launches window

---

## 🔧 Phase 2 — Simulation Depth (current)

> Goal: The town feels alive. Events cascade. Relationships evolve. Actors feel distinct.

### 2.1 Pathfinding
- [ ] Replace straight-line step with simple A* or BFS on the tile grid
- [ ] Actors should walk *along* roads, not through buildings
- [ ] Add a path cache so actors don't recalculate every tick

### 2.2 Relationship Drift
- [ ] Each social interaction (Pub, Choir, helping) nudges relationship weight +
- [ ] Stress contagion exposure nudges relationship weight −
- [ ] Cap drift at ±1.0; apply a slow natural decay toward neutral
- [ ] Chronicle entry when a relationship crosses +0.7 ("friendship") or −0.5 ("friction")

### 2.3 Global Events System
- [ ] Define `GlobalEvent` enum: `PitClosure`, `Eisteddfod`, `HardWinter`, `NewFamilyArrives`, `Bereavement(actor_id)`
- [ ] `PendingEvents` queue in `SimWorld`; processed at start of each tick
- [ ] Per-event wide-area node delta rules:
  - `PitClosure` → all Miners: Grief +0.4, Stress +0.3; community: Belonging +0.1 (solidarity cascade)
  - `Eisteddfod` → all: Joy +0.25, Belonging +0.15; Musician role: Joy +0.4
  - `HardWinter` → all: Hunger drift ×1.8, Fatigue drift ×1.5 for 30 ticks
  - `Bereavement` → nearby actors: Grief +0.3; Elder actors: Grief +0.5
- [ ] Event scheduler: configurable probability per day; some events are one-off, some repeating
- [ ] Chronicle logs event trigger and notable downstream reactions
- [ ] Keyboard shortcut to manually inject events (e.g. `P` = PitClosure, `E` = Eisteddfod) for testing

### 2.4 Actor Lifecycle
- [ ] Actors slowly age (elder threshold after N days)
- [ ] Sleep behaviour: actors prefer home during night ticks; node graph shows rest benefit
- [ ] "Idle wander" fallback: when no node is urgent, actor drifts toward Common or home

### 2.5 Simulation Tests
- [ ] Unit test: propagation pass produces correct deltas for a known edge matrix
- [ ] Unit test: all node values stay in `[0.0, 1.0]` after 1000 ticks (no NaN, no overflow)
- [ ] Unit test: seeded world gen is deterministic (two runs produce identical tile grids)
- [ ] Integration test: run 100 ticks, assert chronicle is non-empty, no panics

---

## 🎨 Phase 3 — Visual & Audio Polish

> Goal: The valley looks and sounds like somewhere real.

### 3.1 Terrain Visual Depth
- [ ] Season colour palettes: spring (bright greens), summer (warm yellows), autumn (ochres/reds), winter (desaturated + snow overlay)
- [ ] Weather system: `Overcast`, `Rain`, `Fog`, `Sunny`; affects tile brightness and node drift rates
- [ ] Rain particle effect (simple falling dots)
- [ ] Animated water shimmer already in; add river-flow direction arrows

### 3.2 Actor Visual Improvements
- [ ] Role-specific visual flourish: Miner carries a dark tool shape, Musician has a note glyph, Elder slightly larger
- [ ] Emotion indicator: small coloured circle above head showing dominant node (red=stress, yellow=joy, purple=grief)
- [ ] Crowd clustering: when 3+ actors at same location, draw a loose cluster glyph
- [ ] Relationship lines: when two actors with weight > 0.7 share a space, draw a faint connecting line

### 3.3 Generative Ambient Sound
- [ ] Add `kira` or `rodio` crate for audio
- [ ] Procedural drone layer: pitch/tempo driven by community average Joy + Belonging
- [ ] Grief spike triggers minor-key shift; Eisteddfod triggers choir-like harmonic swell
- [ ] Rain/wind ambience tied to weather state

---

## 💾 Phase 4 — Save, Share & Export

- [ ] Add `serde` + `ron`; derive `Serialize/Deserialize` on `SimWorld`, `Actor`, `NodeGraph`
- [ ] `S` key: save world snapshot to `saves/<seed>_day<N>.ron`
- [ ] `L` key: load most recent save
- [ ] `--export-chronicle` CLI flag: run N days headless, write chronicle to `output/chronicle.txt`
- [ ] Seed display in UI; `C` key copies seed to clipboard

---

## 🚀 Phase 5 — Release

- [ ] `cargo run --release` profile tuned; target 120fps on M4 with 50 actors
- [ ] WASM build investigation: `cargo build --target wasm32-unknown-unknown`
- [ ] macOS ARM64 binary bundled with `cargo bundle` or manual `.app` wrapper
- [ ] Itch.io page with screenshots and GIF of chronicle filling up
- [ ] Write devlog post

---

## Run at any time

```bash
source ~/.cargo/env
cargo run          # seed 42 (default)
cargo run -- 1337  # custom seed
cargo test         # invariant tests (once written)
```

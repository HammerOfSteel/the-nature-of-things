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

## ✅ Phase 2 — Simulation Depth (DONE)

> Goal: The town feels alive. Events cascade. Relationships evolve. Actors feel distinct.

### 2.1 Pathfinding
- [x] Replace straight-line step with BFS on the tile grid
- [x] Actors walk along roads, not through buildings

### 2.2 Relationship Drift
- [x] Social interactions nudge relationship weight +/−
- [x] Stress contagion exposure nudges relationship weight −
- [x] Cap drift at ±1.0; slow natural decay toward neutral
- [x] Chronicle entries for friendship/friction thresholds

### 2.3 Global Events System
- [x] `GlobalEvent` enum: `PitClosure`, `Eisteddfod`, `HardWinter`, `Bereavement(actor_id)`
- [x] `PendingEvents` queue; processed at start of each tick
- [x] Per-event cascade rules implemented
- [x] Chronicle logs event trigger and notable downstream reactions
- [x] Keyboard shortcuts: `P` / `E` / `H` / `B`

### 2.4 Actor Lifecycle
- [x] Emotion indicator dot above actor head (dominant node colour)
- [x] `action_cooldown` prevents action spam
- [x] Idle wander fallback toward Common or home
- [ ] Integration test: run 100 ticks, assert chronicle is non-empty, no panics

---

## 🔧 Phase 3 — Visual & Audio Polish (in progress)

> Goal: The valley looks and sounds like somewhere real.

### 3.1 Terrain Visual Depth
- [x] Season colour palettes: spring (greens), summer (warm), autumn (ochre), winter (cool + snow overlay)
- [x] Weather system: `Overcast`, `Rain`, `Fog`, `Sunny` — affects tile brightness + node drift
- [x] Rain particle overlay (animated streaks)
- [x] Autumn falling-leaf overlay
- [x] Winter snow flurry overlay + snow patches on ground tiles
- [x] Fog drifting-patch overlay

### 3.2 Actor Visual Improvements
- [x] Emotion indicator: small coloured dot above head (red=stress, yellow=joy, purple=grief)
- [x] Role-specific visual flourish: pickaxe (Miner), book (Teacher), coin (Shopkeeper), note (Musician), stick (Elder), ball (Child), pack (NewArrival)
- [x] Elder slightly larger, Child slightly smaller
- [x] Crowd clustering badge: number shown when 3+ actors share a tile
- [x] Relationship lines: faint green/red lines from selected actor to bonded actors

### 3.3 Navigation
- [x] `Tab` key cycles through actors and snaps camera
- [x] Mini-map overlay (`M` key toggle) — tile overview + actor dots + viewport rect

### 3.4 Generative Ambient Sound
- [ ] `rodio` crate for audio
- [ ] Procedural drone layer driven by community Joy + Belonging
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

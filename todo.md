# Implementation Plan: The Nature of Things

> Stack: Rust · macroquad · wgpu (Phase 6+) · inline FBM noise · no ECS framework yet
>
> **Active direction: high-resolution volumetric voxel world set in a South Wales valley town.**

---

## ✅ Phase 1 — Behavioral Automata Prototype (DONE)

- [x] Single-crate project, macroquad + rand
- [x] `NodeGraph` (10 nodes), per-role baselines, weighted propagation
- [x] `Actor` struct: position, relationships, memory ring buffer, action state
- [x] `SimWorld`: seeded proc-gen valley (noise heightmap → tile classification)
- [x] Welsh name corpus; role distribution; sim systems: decay → propagation → contagion → movement → arrival/action → memory
- [x] `WorldClock` (time-of-day + seasons), weather system
- [x] macroquad renderer: tile colours, actor shapes, bobbing, night dimming, UI panel

---

## ✅ Phase 2 — Simulation Depth (DONE)

- [x] BFS pathfinding on tile grid (actors walk roads, not through walls)
- [x] Relationship drift from interactions and stress contagion
- [x] `GlobalEvent` system: `PitClosure`, `Eisteddfod`, `HardWinter`, `Bereavement`
- [x] Emotion indicator dot, role flourish icons, crowd clustering badge
- [x] Tab-cycle actor focus, minimap overlay, season colour palettes, weather particles

---

## ✅ Phase 3 — Visual POC Suite (DONE)

Four parallel renderers sharing the same sim: `poc_2d`, `poc_34`, `poc_3d`, `poc_village`.
Visual direction **selected: volumetric 3D voxel world** (`poc_voxel`).

---

## ✅ Phase 6 — Volumetric Engine Foundation (DONE)

> World: 128×32×128 voxels at VS=0.40 m³ each. Rendering: ~128 `draw_mesh` calls (one per 16³ chunk).

### 6.1 Chunk-Based World
- [x] `Vox` enum — 30+ material types with holographic face colours (`src/voxel/vox.rs`)
- [x] `VoxelWorld` flat-array storage with `get/set` and `chunk_iter` (`src/voxel/world.rs`)
- [x] World dimensions: 128×32×128 voxels (8×2×8 = 128 chunks of 16³)

### 6.2 Face-Culled Chunk Mesh Extraction
- [x] `build_chunk_mesh` — one exposed quad per face, directional light multipliers (`src/voxel/mesher.rs`)
- [x] `ChunkRenderer` — dirty flags, back-to-front sort, `draw_mesh` per chunk (`src/render/draw_voxel.rs`)
- [x] Safety cap: 60k vertex guard at TOP of voxel closure (break was wrong — only exited face loop)
- [x] Fix geometry() overflow: macroquad default `draw_call_vertex_capacity=10k` — raised to 65536 via `macroquad::conf::Conf`

### 6.3 South Wales Valley Generator
- [x] V-valley profile with FBM detail, biome layers by elevation (`src/voxel/gen.rs`)
- [x] River water fill in valley trough, gravel shore
- [x] Hedgerow trees (trunk + layered canopy) with occupancy-aware placement
- [x] Two terrace rows of hollow 2-storey houses with interior furniture
- [x] Derelict colliery feature (engine house, headframe, coal tip)
- [x] Occupancy grid: building footprints marked before tree placement — no trees inside houses
- [x] **4× resolution upgrade**: VS 0.40→0.10, CHUNK 16→64, world 128→512, same 128 chunks
- [x] **Detailed houses**: depth=24, wall_height=32, two rooms per floor, door 3×8, windows 4×6
- [x] **Detailed furniture**: bed 6×10 frame+mattress, bookshelf 4×8 with shelves, fireplace with surround+mantle+coal

### 6.4 Camera & Controls
- [x] **Orbit mode**: auto-rotate, A/D spin, W/S tilt, scroll zoom
- [x] **Fly-through mode** (Tab toggle): WASD+Q/E movement, mouse look, Shift=3× speed
- [x] HUD: fps, ktris, seed, current mode, voxel position in fly mode
- [x] Space to regenerate with new seed; Esc exits fly / quits

### 6.5 Remaining Engine Improvements (deferred to Phase 11)
- [ ] Frustum culling: skip chunks outside camera frustum
- [ ] Ambient occlusion: per-vertex AO baked into mesh at extract time
- [ ] Greedy mesh merging: coplanar same-type face merging to reduce triangle count further

---

## 📋 Phase 7 — South Wales Valley Landscape

> Goal: Terrain that unmistakably reads as the South Wales coalfield valleys.

---

## 📋 Phase 7.0 — Component-Based Building & Settlement Grammar ← implement first

> Every building is assembled from typed components (walls, windows, doors, floors, ceilings, stairs, roofs)
> rather than hand-stamped voxel shapes. Components compose into rooms, rooms into floors, floors into buildings.
> Buildings are placed by a settlement layout algorithm that generates streets, blocks, and plots first —
> so trees and terrain always clear correctly around the built environment.

### Building Primitives
Each primitive carries its material and orientation; stamping it into the world is a single call.

| Primitive    | Description |
|-------------|-------------|
| `Wall`      | Solid panel; material = Brick / Render / Stone / Plank |
| `Window`    | Glass inset in a Wall, defines an opening voxels are left clear |
| `Door`      | Traversable gap in a Wall; carries `DoorState` (open/closed) |
| `Floor`     | Horizontal surface separating storeys |
| `Ceiling`   | Underside of the floor above (same slab, different face tint) |
| `Stairs`    | Navigable stepped run connecting two Floor levels |
| `RoofPanel` | Pitched or flat cap sealing the top of a building |

### Room Schema
```rust
Room {
    role:        RoomRole,      // FrontParlour | Kitchen | Bedroom | Pub | Chapel | ...
    bounds:      Aabb,          // voxel AABB
    components:  Vec<Component>,// walls, windows, doors around the perimeter
    furniture:   Vec<Furniture>,// from a per-RoomRole library
    connections: Vec<DoorRef>,  // doors leading to adjacent rooms / outside
}
```

### Floor Schema
```rust
Floor {
    level:  u8,          // 0 = ground, 1 = first, ...
    rooms:  Vec<Room>,   // packed within building footprint
    plan:   FloorPlan,   // Linear | L-shape | U-shape
}
```

### Building Schema
```rust
Building {
    kind:      BuildingKind, // Terrace | DetachedHouse | Chapel | Pub | School | Colliery
    floors:    Vec<Floor>,
    roof:      RoofKind,     // Slate | Flat | Gabled
    facade:    FacadeStyle,  // Render | Brick | Stone
    footprint: Rect,         // voxel XZ footprint
    entrance:  IVec3,        // front-door voxel
}
```

### Settlement Layout Algorithm
```
1.  Generate the street network (main road + side streets + back lanes)
2.  Mark street voxels in the occupancy grid
3.  Subdivide each inter-street block into building plots (5–9 voxels wide)
4.  Assign each plot a BuildingKind from a probability table
5.  Instantiate a Building for each plot using the schema above
6.  Mark all building footprints + garden clearance in the occupancy grid
7.  Stamp all buildings, then roads, then trees in remaining clear space
```

### Tasks
- [x] Define `Component`, `Room`, `Floor`, `Building` structs in `src/voxel/building.rs` *(done)*
- [x] Define `RoomRole`, `BuildingKind`, `FacadeStyle`, `RoofKind` enums in `src/voxel/building.rs` *(done)*
- [x] Implement `build_terrace_house(ox, oy, oz, width, face_south, seed) -> Building` *(done)*
- [x] Implement `stamp_building(world, &Building)` — fills voxels from schema *(done)*
- [x] Furniture library: `Furniture::fireplace/bed/table_and_chairs/bookshelf/pub_bar/pew_row` *(done)*
- [x] Migrate `stamp_terrace_house` in `gen.rs` to use `build_terrace_house` + `stamp_building` *(done)*
- [x] Occupancy grid: building footprints + colliery block trees before any stamping *(done)*
- [x] Street network: main cobble road (16 vox wide) + side streets every 80 vox *(done)*
- [ ] Block subdivision: street cells → plot rects → `Building` placements
- [ ] Plot probability table: road-front → terrace; mid-slope → detached; valley bottom → pub/shop
- [ ] Houses must not float: flatten terrain slab under each plot footprint before stamping

---

### 🎯 Next up: Phase 7.6 — Organic Street Network & Neighbourhood Variety

---

### 7.1 Valley Terrain Generator
- [x] V-shaped valley with FBM detail noise, biome layers by elevation fraction *(done)*
- [x] Valley floor: gravel/mud → shore → meadow/grass *(done)*
- [x] Mid slope: terrace housing strips + bracken/heather *(done)*
- [x] Ridge: slate and stone *(done)*
- [ ] Proper cwm (bowl) hollows carved into upper slopes
- [ ] Seasonal snow accumulation on high faces

### 7.2 Biome Material Layers
- [ ] Voxel material selected from `(elevation, slope, moisture)` lookup
- [ ] Materials: River Gravel, Mud, Meadow Grass, Garden Soil, Slate Path, Cobble Road, Bracken, Heather, Bare Rock, Scree
- [ ] Snow accumulation on top-facing voxels above 80% max elevation in winter

### 7.3 River System
- [x] River water fill along valley trough *(done)*
- [x] Gravel shore 1 voxel above river level *(done)*
- [ ] Seasonal level variation
- [ ] Shallow fords where roads cross

### 7.4 Vegetation
- [x] Hedgerow trees (trunk + layered canopy) with occupancy-aware placement *(done)*
- [ ] Bracken patches on mid-slope
- [ ] Heather on upper slope
- [ ] Garden plants: leeks, cabbages, runner beans in back gardens
- [ ] Sheep on moorland

### 7.5 Derelict Colliery Feature
- [x] Stone engine house (roofless walls) *(done)*
- [x] Headframe tower *(done)*
- [x] Coal tip mound *(done)*
- [ ] Tramway track leading to drift entrance
- [ ] Rust and ruin details: broken windows, collapsed roof section

---

## 📋 Phase 7.6 — Organic Street Network & Neighbourhood Variety

> Current: two fixed terrace bands at constant Z, straight streets only.
> Goal: an organic road graph with curves, junctions, and varied neighbourhood types.
> One row per side of a straight street is the right density — add variety through topology.

### Street Network Graph
- [ ] Replace fixed `nz_band` with a `StreetGraph { nodes: Vec<IVec2>, edges: Vec<(usize, usize)> }`
- [ ] Graph generation: start from valley floor road → branch side streets every 60–100 vox → dead-end cul-de-sacs
- [ ] Road segments follow terrain contours (step the centre line to nearest flat Y, then pave)
- [ ] Road types: main street (16 vox, cobble), side lane (8 vox), back alley (4 vox, dirt/mud)
- [ ] Curves: road segments bend with ~20° variance per 40-vox run (interpolate from waypoints)
- [ ] Intersections: T-junction and 4-way; curb-stone corner detail voxels

### Neighbourhood Types (per street segment)
Each segment is tagged; house placement and density follow the tag:

| Tag | Description |
|-----|-------------|
| `TerracedRow` | Houses 20–32 vox wide packed tight, one row each side |
| `MixedResidential` | Detached + semi-detached, 4–8 vox gap between, gardens |
| `VillageCore` | Pub, chapel, school, corner shop around a small green/square |
| `Farmstead` | 1–2 isolated large houses with barn, yard, animal pens |
| `OpenHillside` | No houses; trees, heather, sheep pens |

- [ ] Implement `SettlementZone` enum and `zone_for_segment()` based on elevation + distance from valley floor
- [ ] `TerracedRow` → existing `build_terrace_house()` rows
- [ ] `VillageCore` → pub (`build_pub()`), chapel, small green of grass voxels
- [ ] `Farmstead` → wide detached house + barn (new `build_barn()` template)

### Plot & Block Assignment
- [ ] For each road segment: walk one-voxel band on each side, subdivide into plots by width
- [ ] Plot = `(x0, z0, width, depth, side)` rect; store in `BlockLayout`
- [ ] Fill plots with buildings from neighbourhood-type table
- [ ] Mark all plots in occupancy grid before tree/vegetation pass

### Terrain Adaptation
- [ ] Detect slope under each road segment; if slope > threshold, add retaining wall voxels
- [ ] `flatten_plot(world, hmap, x, z, w, d)` — fill low spots with stone, cut high spots to flat
- [ ] Flat-bottom pockets allowed on hillside: road can cut across the slope horizontally

---

## 📋 Phase 7.7 — Blueprint System & Modular Preview

> Same modular philosophy as the house component system, scaled up to the whole map.
> Every geographic/built feature is a `Blueprint` that can be:
>   (a) previewed in isolation on flat test terrain, and
>   (b) referenced by the procedural generator or a YAML map file.
>
> Like lego: you can hold up a single brick, or assemble a whole town.

### Blueprint Hierarchy
```
MapBlueprint
  └─ RegionBlueprint (valley / moorland / river crossing / coastal)
       └─ SettlementBlueprint  (TownCentre / TerracedStreet / HighStreet / Village)
            └─ NeighbourhoodBlueprint  (TerracedRow / MixedResidential / VillageCore / ...)
                 └─ StreetSegmentBlueprint (straight / curved / cul-de-sac / T-junction)
                      └─ PlotBlueprint     (one building + garden + path to road)
                           └─ BuildingBlueprint (uses existing building.rs grammar)
```

### 7.7.1 Blueprint Trait
- [ ] Define `Blueprint` trait in `src/voxel/blueprint.rs`:
  ```rust
  trait Blueprint {
      fn name(&self) -> &str;
      fn bounding_box(&self) -> IVec3;           // max voxel extent (X,Y,Z)
      fn stamp(&self, world: &mut VoxelWorld, origin: IVec3, seed: u64);
  }
  ```
- [ ] Blanket `previewable()` helper: wraps any Blueprint in a flat 256×128×256 test world with a grass base
- [ ] Every struct in `building.rs`, `gen.rs` neighbourhood code eventually implements `Blueprint`

### 7.7.2 Preview Runner (`src/bin/preview.rs`)
- [ ] New binary: `cargo run --bin preview -- <BlueprintName> [seed]`
- [ ] Enumerates a `BlueprintRegistry` (simple match arm for now) and stamps the named blueprint at world centre
- [ ] Opens the same macroquad window with fly-cam — no full world, just the preview slab
- [ ] Examples:
  - `cargo run --bin preview -- TerracedStreet` — one straight row of 8 houses
  - `cargo run --bin preview -- VillageCore` — pub + chapel + school + green
  - `cargo run --bin preview -- HighStreet` — 200-vox commercial strip
  - `cargo run --bin preview -- TerracedHouse` — single house
  - `cargo run --bin preview -- WelshValleyRegion` — full valley in mini-map

### 7.7.3 Neighbourhood Blueprints
- [ ] `TerracedRow` — N×terrace house on each side, 16-vox cobble road between, pavements
- [ ] `MixedResidential` — mix of detached + semi, 8-vox paths, small front gardens
- [ ] `VillageCore` — pub + chapel + war memorial + bench + green square; can work on flat or slight slope
- [ ] `HighStreet` — commercial strip: chippy, mart, corner shop, café, betting shop, newsagent, office above
- [ ] `SchoolYard` — schoolhouse + fenced tarmac yard + bike sheds + playing field
- [ ] `IndustrialYard` — workshop, delivery yard, stacked pallets, chain-link fence
- [ ] `FarmComplex` — farmhouse + two barns + hay bales + pen + muddy track

### 7.7.4 Commercial Buildings (new `building.rs` templates)
- [ ] `build_chippy()` — narrow shopfront, extractor duct, serving counter + fryer inside
- [ ] `build_corner_shop()` — wider ground floor, large windows, flat roof, flat upstairs
- [ ] `build_pub()` — two-storey, frosted glass, sign bracket, snug + bar + cellar
- [ ] `build_chapel()` — tall gable, pointed windows, small vestry, pews inside
- [ ] `build_school()` — long, redbrick, arched windows, belfry, cloakrooms
- [ ] `build_office_block()` — 3–4 storey, strip windows, flat roof, fire escape

### 7.7.5 YAML / RON Map Format
- [ ] `MapSpec` struct (serde-deserializable):
  ```rust
  MapSpec {
      seed: u64,
      terrain: TerrainSpec,       // valley / flat / coastal / custom heightmap
      settlements: Vec<SettlementSpec> {
          name, centre_xz, kind, orientation_deg, blueprints: Vec<BlueprintRef>
      }
  }
  ```
- [ ] `cargo run --bin poc_voxel -- --map maps/sunnyside.ron` — load map from file
- [ ] `S` key in-game saves current world as `world_<seed>.ron`
- [ ] `L` key loads most recent saved map
- [ ] Example starter map files: `maps/default_valley.ron`, `maps/highstreet_preview.ron`
- [ ] Validation: missing blueprints print a helpful error, not a panic

### 7.7.6 Procedural Composition
- [ ] Procedural generator draws from `BlueprintRegistry` when placing neighbourhoods
- [ ] Each settlement zone maps to one or more `NeighbourhoodBlueprint` entries
- [ ] `weight` on each entry allows bias tuning (e.g. 80% TerracedRow, 20% MixedResidential in valley)
- [ ] Blueprint selection seeded per street segment — same seed always produces same neighbourhood

---

## 📋 Phase 8 — Building Generation & Interiors

> Goal: Every building is hollow, furnished, and identifiable by type from inside and out.

### 8.1 Building Templates
- [ ] Define `BuildingTemplate` grammar: exterior footprint, height, roof type, facade details
- [ ] **Welsh terraced house** (2–3 storeys): slate-grey roof, white render walls, bay window front, lean-to scullery back, small walled garden
- [ ] **Nonconformist chapel**: narrow plan, tall pointed windows, prominent gable, double wooden door
- [ ] **Working men's club / pub**: wider plan, frosted glass windows, sign bracket over door, cellar steps
- [ ] **Corner shop**: ground floor shopfront with large windows, upstairs flat
- [ ] **Victorian schoolhouse**: redbrick, tall arched windows, belfry

### 8.2 Interior Room Layout
- [ ] Per-building room grid: place rooms on a 2-voxel-walled plan with 1-voxel door openings
- [ ] Stairs as a diagonal voxel ramp connecting floors
- [ ] Terrace house rooms: front parlour, kitchen, back scullery; bedrooms upstairs
- [ ] Pub rooms: public bar, snug, back corridor, cellar

### 8.3 Furniture Voxel Library
- [ ] Bed (iron frame + mattress + pillow) — 3×2×2 voxels
- [ ] Armchair — 2×2×2 voxels
- [ ] Kitchen table + 4 chairs — 3×3×2 voxels
- [ ] Kitchen range (cast-iron) — 3×2×3 voxels
- [ ] Dresser with plates — 3×1×3 voxels
- [ ] Bookshelf — 2×1×3 voxels with book-spine detail
- [ ] Fireplace + mantle — 3×1×4 voxels
- [ ] Pub bar counter — 5×1×2 voxels
- [ ] Church pew — 4×1×1 voxels (rows of them)
- [ ] Harmonium / piano — 3×2×3 voxels

### 8.4 Lighting Hints (Volumetric Glow)
- [ ] Fireplace voxel emits warm orange tint to nearby voxel wire colors at night
- [ ] Window voxels on lit interiors emit soft yellow bloom outward
- [ ] Street lamp posts on cobble roads; small warm pool of lit grid at base

---

## 📋 Phase 9 — Voxel Characters & Creatures

> Goal: Every inhabitant is a unique, legible voxel figure navigating indoor and outdoor spaces.

### 9.1 Voxel Character Assembly
- [ ] Base body: head (3×3×3), torso (3×3×4), two arms (1×1×3), two legs (1×1×3)
- [ ] Procedural face: eye colour, skin tone, hair colour/style (1-voxel hair blocks on head top/sides)
- [ ] Clothing: torso and leg voxel colours assigned by role (miner=dark grey, teacher=navy, shopkeeper=brown, elder=black, child=bright)
- [ ] Age sizes: Child = 0.65× scale; Adult = 1.0×; Elder = 0.95× with slight hunch (offset torso)
- [ ] Animation: walk cycle via voxel offset (arms/legs swing on bone pivot)

### 9.2 Age, Growth & Families
- [ ] Every actor has `age: u32` (sim-days) and `life_stage: Child | Teen | Adult | Elder`
- [ ] Children born when two bonded adults share a house node for sufficient ticks
- [ ] Growth: child advances life stage at age thresholds (simulated years)
- [ ] Physical voxel model scales and updates on stage transition
- [ ] Family relationships tracked (parent/child/sibling) in `Actor.relationships`
- [ ] Elders decline in Energy/Purpose over time; eventual death logged in Chronicle

### 9.3 Animals
- [ ] **Sheep**: squat body (3×3×2), head (2×2×2), stumpy legs — white/grey wool voxels; flock on upper slope
- [ ] **Dog**: low body, pointed head — accompanies specific actors on walks
- [ ] **Raven / jackdaw**: small (2×2×1), black, perches on rooftops and posts — hops about
- [ ] **Fox**: slim, orange-red — only active at night, sniffs around gardens and bins

### 9.4 Character Pathfinding in Voxel Space
- [ ] 3D A* over voxel grid: step onto any voxel with Air above and solid below
- [ ] Door voxels navigable (treated as walkable gap in wall)
- [ ] Stairs navigated by treating each step as adjacent
- [ ] Indoor destinations: bed, armchair, bar stool, pew — actor moves to within 1 voxel of furniture

---

## 📋 Phase 10 — Simulation × Voxel World Integration

> Goal: The Behavioral Automata drives character movement through the physical voxel world.

- [ ] Replace `TileKind` 2D world with `VoxelWorld` as the sim's spatial substrate
- [ ] Actor `pixel_x/y` replaced with `vox_x, vox_y, vox_z` world position
- [ ] Node graph targets mapped to building types: `Social` → pub/club, `Purpose` → workplace, `Belonging` → home
- [ ] Indoor-aware action selection: if actor is at pub, can trigger `Singing`, `Drinking`, `Arguing` actions using pub furniture positions
- [ ] Time-of-day routing: actors go home at dusk, sleep in their bed voxel, wake and have breakfast at kitchen table
- [ ] Community events in specific locations: Eisteddfod in chapel, rugby match at pitch, market stalls on cobble square

---

## 📋 Phase 11 — Performance, Polish & Release

### 11.1 Rendering Performance
- [ ] Profile on M4: identify CPU/GPU bottlenecks with `cargo flamegraph`
- [ ] Greedy mesh rebuild <2 ms per chunk; target 500k voxels at 60 fps release build
- [ ] Consider wgpu for explicit GPU instancing if macroquad bottlenecks
- [ ] Shader: simple directional light + AO baked into mesh

### 11.2 Procedural Variety
- [ ] 5+ distinct house interior layouts (procedurally varied per seed)
- [ ] Street furniture: post boxes, bus shelters, phone boxes, war memorial
- [ ] Weather effects in voxel world: rain streaks as particle voxels, snow accumulation on top-facing surfaces

### 11.3 Sound
- [ ] `rodio` crate: procedural ambient drone tuned to community Joy + Belonging level
- [ ] Rain ambience tied to weather state
- [ ] Distant singing (Eisteddfod event)

### 11.4 Save / Export
- [ ] `serde` + `ron`: serialize `VoxelWorld` + `SimWorld` snapshot
- [ ] `S` to save, `L` to load most recent snapshot
- [ ] `--export` CLI flag: run N days headless, dump chronicle to file

### 11.5 Release
- [ ] `cargo run --release` at 60 fps with full town
- [ ] macOS ARM64 `.app` bundle
- [ ] Itch.io page with GIF of a life unfolding

---

## 📋 Phase 12 — Large-Scale Map Generation (Minecraft-Scale)

> Goal: effectively infinite world via chunk streaming. Camera can explore for minutes without hitting an edge.
> Near-term target: 2km × 2km region (20000×20000 voxels at VS=0.10). Eventually: no hard limit.

### 12.1 Chunk Streaming
- [ ] Decouple world size from render distance: world = infinite seed space, render = loaded region
- [ ] `ChunkCache { loaded: HashMap<(i32,i32,i32), Chunk>, queue: VecDeque<ChunkCoord> }`
- [ ] Load radius: keep N=6 chunks in each direction around camera position (sphere of ~800 chunks)
- [ ] On camera move: add newly visible chunks to generate queue, remove far chunks from cache
- [ ] Generate chunks on demand: `generate_chunk(cx, cy, cz, seed)` — deterministic from coords alone
- [ ] Background generation: spawn a rayon thread pool task per new chunk; mark dirty when done

### 12.2 Infinite Terrain Generation
- [ ] Move from `generate_wales_valley(wx,wy,wz,seed)` to `generate_region_chunk(cx,cz,seed) -> Chunk`
- [ ] Global heightmap via multi-octave FBM sampled at chunk coordinates (not pre-computed array)
- [ ] Region biome assigned per 256-chunk macro-cell: valley / open moorland / river crossing / coastal cliff
- [ ] Valley corridors connect neighbouring valley macro-cells (rivers flow between them)
- [ ] Macro-scale features: ridge lines, cwm bowls, river network — all from seeded noise

### 12.3 Settlement Distribution
- [ ] `settlement_grid(seed, cx, cz)` — returns settlement centre coords every ~300–800 voxels
- [ ] Each settlement = `SettlementType { Valley Town | Hilltop Farm | River Crossing | Isolated Chapel }`
- [ ] Street graph generated per settlement at load time from settlement type + local terrain
- [ ] LOD: distant chunks (>8 chunks) rendered as low-detail (only top surface, no building interiors)

### 12.4 Persistence
- [ ] Player modifications saved as a diff layer per chunk (sparse `HashMap<IVec3, Vox>`)
- [ ] `serde_json` + per-chunk `.chunk` files; loaded/unloaded with chunk streaming
- [ ] Long-term: sim actors persist in a global chronicle independent of render region

### 12.5 Navigation & Minimap
- [ ] Minimap updated in real time from loaded chunk top surfaces
- [ ] World map view (M key): macro-scale 2D overview of loaded + generated region biomes
- [ ] Road network visible on world map; settlement markers

---

## Run at any time

```bash
cargo run --bin poc_voxel      # high-res voxel world (primary)
cargo run --bin poc_volumetric # earlier holographic POC
cargo run --bin poc_village    # 2D Sunnyside village
cargo run                      # original 2D simulation

# Controls in poc_voxel:
#   SPACE — new world seed
#   R     — toggle auto-orbit
#   A/D   — rotate camera
#   W/S   — tilt up/down
#   scroll— zoom
```

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

## 🎨 Phase 3.5 — Visual Direction POCs (in progress)

> Three separate renderers sharing the same simulation. Pick one direction to carry forward.
> Run with: `cargo run --bin poc_2d` | `cargo run --bin poc_34` | `cargo run --bin poc_3d`

### POC-2D: Enhanced Top-Down
- [ ] Procedural building rooftops (terracotta ridge, chimney, eaves)
- [ ] Trees as layered canopy circles + trunk
- [ ] Stone-brick path pattern + grass blade details
- [ ] Animated water ripples

### POC-34: LTTP-Style 3/4 View (most like Zelda LTTP)
- [ ] Two-pass renderer: ground first, then elevated objects Y-sorted back-to-front
- [ ] Buildings: terracotta roof (top face) + plaster wall + windows + door (front face)
- [ ] Mountains: rocky top + cliff face with crack lines
- [ ] Trees: layered canopy + visible trunk below
- [ ] Actors taller and more front-facing (legs visible, pupils in eyes)

### POC-3D: Macroquad 3D Top-Down
- [ ] Camera3D perspective from above at angle (no new crate needed — macroquad 3D API)
- [ ] Ground tiles as flat cubes coloured by type
- [ ] Buildings as taller coloured boxes
- [ ] Trees as cylinder trunk + sphere canopy
- [ ] Actors as capsule shapes

---

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

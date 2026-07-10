# Project Overview: The Nature of Things

## 1. Core Concept

A living, procedurally generated simulation of a small South Wales valley town — rendered as a high-resolution volumetric 3D holographic display. The player observes a community of autonomous inhabitants going about their lives: working, arguing, singing in the chapel, drinking in the pub, tending their gardens.

There is no authored story. Everything emerges from math.

The world is built from small 3D voxels — much finer-grained than Minecraft — with enough resolution to model the inside of a terraced house room-by-room: beds, bookshelves, fireplaces, kitchen ranges, back gardens with cabbages and leeks. The simulation and world can be arbitrarily detailed because every object, character, and landscape feature is generated procedurally from code.

---

## 2. The Setting: A South Wales Valley Town

The simulation is grounded in a specific place and culture. Key landscape features:

- **The valley itself** — the classic steep V-shaped glacial valley of the South Wales coalfield. A river runs along the flat valley floor. Terraced houses climb both hillsides in long parallel rows.
- **Biome layers** (valley floor → ridge):
  - River gravel / flood-plain meadow
  - Terrace housing rows with small back gardens
  - Mid-slope bracken and scrub
  - Upper-slope heather moorland and sheep pasture
  - Rocky ridge / cwm edges
- **Buildings** (procedurally generated):
  - Victorian terraced houses (2–3 storeys, slate roof, bay windows, small front step)
  - Nonconformist chapels (tall narrow windows, prominent gable end, harmonium inside)
  - Working men's club / pub (bar, fireplace, piano, pool table in back)
  - Corner shops (post office, general stores)
  - Primary school (Victorian redbrick)
  - Rugby club with pitch
  - Derelict colliery: ruined engine house, overgrown headframe, coal-tip now grassed over
- **Interior spaces** (built voxel-by-voxel at high resolution):
  - Terraced house rooms: front parlour (dresser, armchairs, clock), kitchen (range, sink, table), scullery, upstairs bedrooms (iron bed frame, washstand, books on shelf)
  - Pub: long bar with stools, snug booths, dartboard, fireplace with mantle
  - Chapel: rows of pews, wooden pulpit, harmonium at front
- **Flora and fauna**:
  - Sheep on upper slopes; dogs in gardens; ravens and jackdaws on rooftops
  - Foxes at night
  - Leeks, runner beans, cabbages in gardens
  - Oak, rowan and birch in hedgerows; heather on moor

---

## 3. The "Behavioral Automata" Engine

Each actor's inner life is modelled as a weighted node graph, not hardcoded if/else logic.

- **Nodes** (10 per actor): Stress, Hunger, Social, Purpose, Belonging, Joy, Grief, Energy, Suspicion, Hope
- **Propagation**: On every sim tick, node values bleed into adjacent nodes by weight. Grief suppresses Joy and Purpose; Belonging amplifies Joy; Stress cascades into Hunger and Social.
- **Action Selection**: The highest-weight actionable node drives the actor's next move. No scripted behavior trees.
- **Community Contagion**: Emotional states spread between actors sharing a space — a singer in the pub ripples outward.
- **Events**: `Pit Closure`, `Eisteddfod`, `Hard Winter`, `Bereavement`, `Rugby Win`, `New Arrival` inject shocks that cascade through the whole community.

---

## 4. Visual Direction: Volumetric Holographic Display

The chosen renderer is a **high-resolution 3D voxel world** rendered in a holographic/neon style:

- Each voxel is small (≤ 0.4 world units). A standard terraced house is ~40×30×24 voxels.
- **Only surface voxels are drawn** (solid voxels with at least one air neighbour) — keeping draw calls tight regardless of world size.
- The holographic aesthetic: dark navy background, low-alpha transparent fill cubes, bright neon wireframe edges, sweep-scan highlighting plane.
- Buildings are **hollow** — you can look through windows and open doors into furnished rooms.
- Characters are **voxel assemblies** — procedurally built from typed body-part blocks (torso, head, limbs) with variant face features, clothing colour by role, and size variation by age.

**Performance target**: 60 fps at 512×512×48 voxels on Apple M4, using chunk-based dirty-mesh rebuild and GPU instanced rendering.

---

## 5. Technical Architecture

```
the_nature_of_things/
├── src/
│   ├── sim/           # Headless simulation (world, actors, systems, events)
│   ├── voxel/         # Voxel world: chunk grid, generation, surface extraction
│   ├── render/        # Renderers: volumetric (primary), 3D, 2D (legacy POCs)
│   └── bin/
│       ├── poc_voxel.rs      # ← current focus: volumetric voxel world
│       ├── poc_volumetric.rs # earlier holographic render
│       ├── poc_village.rs    # 2D Sunnyside tileset village
│       ├── poc_3d.rs         # macroquad Camera3D
│       └── poc_2d.rs         # classic top-down
├── tools/             # Python asset-preview scripts
├── gdd.md
├── architecture.md
├── readme.md
├── overview.md        # ← this file
└── todo.md
```

**Stack**:
| Layer | Technology |
|---|---|
| Language | Rust (stable) |
| Window / 2D | macroquad 0.4 |
| 3D rendering | macroquad Camera3D → wgpu instancing (Phase 6+) |
| Noise / procgen | Inline FBM (no crate needed at POC scale) |
| Serialisation | serde + ron (planned) |
| Build | cargo |

**Constraints**: No Unity/Godot GUI. No proprietary editors. 100% code-driven from VS Code. Must compile with `cargo build`; simulation must be able to run headless.

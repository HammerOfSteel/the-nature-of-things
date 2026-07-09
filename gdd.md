# Game Design Document: The Nature of Things

> *"Dyn a ddysg hyd fedd."*
> A person learns until the grave. — Welsh proverb

---

## 1. Vision Statement

*The Nature of Things* is a cozy, emergent slice-of-life simulation set in a procedurally generated small town in the South Wales valleys. The player is less a protagonist and more an observer — a quiet god watching a community breathe, argue, fall in love, mourn, and persist. There are no quests. There is no win state. There is only time passing and lives unfolding.

The game's central promise: **every actor lives a life that surprises even the system that generated it.**

---

## 2. Setting: The Valley

The town — Cwm Newydd ("New Valley") — is procedurally generated on each new save. It sits in a steep-sided glacial valley between hills of bracken and slate. Key environmental features:

- **The Terraced Rows** — tight rows of stone cottages climbing the hillside, each with a small garden.
- **The High Street** — a single road with a bakery, a pharmacy, a pub (*The Red Dragon*), a library, and a market.
- **The Common** — an open green space at the valley floor used for fêtes, football, and gossip.
- **The Mountain** — the hill above town. Characters go there alone when they need to think.
- **The River Afon** — runs through the valley floor. Fishing spot, meeting place, backdrop to grief.

Welsh valley culture is encoded into the simulation as systemic biases, not cutscenes or dialogue trees:
- Communities gossip. Information propagates.
- Singing and music are stress relievers with community multipliers.
- Hardship (economic downturn, bereavement) triggers solidarity cascades.
- Pride in place is a persistent baseline value that resists erosion.

---

## 3. The Behavioral Automata System

### 3.1 Concept

Each actor's internal state is modelled as a **weighted directed graph** — a small network of interconnected nodes. On every simulation tick, energy flows between nodes according to mathematical propagation rules. Behavior emerges from the current state of the graph without any hardcoded if/else branches.

This is inspired by **Neural Cellular Automata (NCA)** and **Conway's Game of Life**: simple local rules producing complex global behavior.

### 3.2 The State Nodes

Each actor has the following nodes. Values are normalised floats in the range `[0.0, 1.0]`.

| Node ID | Name | Description |
|---|---|---|
| `N_HUNGER` | Hunger | Physical need to eat. Rises over time, drops on consumption. |
| `N_FATIGUE` | Fatigue | Physical tiredness. Rises with activity, drops with rest. |
| `N_STRESS` | Stress | Accumulated psychological pressure from unmet needs and negative events. |
| `N_SOCIAL` | Social Need | Desire for human connection. Rises with isolation, drops with interaction. |
| `N_JOY` | Joy | Positive affect. Temporary peak node — decays rapidly without reinforcement. |
| `N_GRIEF` | Grief | Persistent negative affect following loss events. Decays very slowly. |
| `N_PURPOSE` | Sense of Purpose | Satisfaction from meaningful activity (work, hobby, family). |
| `N_BELONGING` | Belonging | Connection to community and place. Slow to change. Cultural bedrock node. |
| `N_CURIOSITY` | Curiosity | Drive to explore, learn, or engage with novelty. |
| `N_RESENTMENT` | Resentment | Accumulated social friction. Builds from ignored social need and repeated conflict. |

### 3.3 Propagation Rules

On each tick, for every actor, the following update pass runs:

**Step 1 — Environmental Input:** External signals (time of day, weather, nearby actors, location) are read and inject delta values into relevant nodes.

**Step 2 — Node Decay/Rise:** Each node has a natural drift rate pulling it toward a baseline. Hunger rises. Fatigue rises with exertion. Joy decays. Grief decays very slowly.

**Step 3 — Cross-Node Propagation:** This is the core of the system. Edges between nodes carry weights. If a node exceeds a threshold, it bleeds into connected nodes.

Example propagation rules:

```
N_STRESS  --[0.4]--> N_SOCIAL       // High stress drives social seeking
N_STRESS  --[0.3]--> N_GRIEF        // Chronic stress opens grief channel
N_SOCIAL  --[0.5]--> N_RESENTMENT   // Unmet social need builds resentment if above threshold
N_JOY     --[0.6]--> N_PURPOSE      // Joy reinforces sense of purpose
N_GRIEF   --[-0.4]--> N_PURPOSE     // Grief suppresses purpose
N_BELONGING --[0.3]--> N_JOY        // Belonging provides baseline joy
N_CURIOSITY --[0.3]--> N_JOY        // Curiosity satisfied generates joy
N_RESENTMENT --[0.2]--> N_STRESS    // Resentment feeds back into stress (loop risk)
N_PURPOSE --[-0.3]--> N_STRESS      // Strong purpose acts as a stress buffer
```

**Step 4 — Behavioral Output:** The actor's current highest-priority node (computed via weighted urgency scoring) determines the **Action Category** selected this tick.

### 3.4 Action Categories

Actions are not hardcoded. They are tagged with node affinities. The actor selects the highest-affinity action available in their current location that satisfies the dominant node.

| Category | Triggered By | Example Actions |
|---|---|---|
| `ACT_EAT` | `N_HUNGER` high | Go to kitchen, go to bakery, share meal |
| `ACT_SLEEP` | `N_FATIGUE` high | Go home, nap on common, sleep at desk |
| `ACT_SEEK_SOCIAL` | `N_SOCIAL` high | Visit neighbour, go to pub, wave at passerby |
| `ACT_ISOLATE` | `N_STRESS` + `N_RESENTMENT` high | Walk to mountain, sit by river, stay indoors |
| `ACT_CREATIVE` | `N_CURIOSITY` + `N_PURPOSE` high | Paint, garden, play instrument, write |
| `ACT_HELP` | `N_BELONGING` high + `N_STRESS` low | Help neighbour, volunteer, share food |
| `ACT_GRIEVE` | `N_GRIEF` high | Visit grave, sit quietly, cry alone |
| `ACT_SING` | `N_SOCIAL` + `N_JOY` mid-range | Join choir, hum while working, sing in pub |

### 3.5 Inter-Actor Propagation (Community Contagion)

When two actors share a location, a secondary propagation step occurs between their graphs:

- **Joy is mildly contagious.** Proximity to a high-joy actor slightly raises others' `N_JOY`.
- **Grief commands respect.** Proximity to high-grief actors suppresses `N_JOY` and raises `N_BELONGING` (solidarity response).
- **Stress is somewhat contagious.** Shared proximity with high-stress actors raises `N_STRESS` in others at a low rate.
- **Singing triggers a community resonance multiplier.** `ACT_SING` in a shared space boosts `N_JOY` and `N_BELONGING` for all actors in range by a significant delta.

---

## 4. Actor Lifecycle

### 4.1 Roles

Each actor is initialised with a **role** that sets baseline node values and provides access to certain action tags:

- `Miner / Former Miner` — high baseline `N_PURPOSE` tied to physical work, high `N_BELONGING`, elevated `N_GRIEF` if pit closed.
- `Teacher` — high `N_CURIOSITY`, high `N_PURPOSE`, moderate `N_SOCIAL`.
- `Shopkeeper` — high `N_SOCIAL`, moderate `N_BELONGING`.
- `Musician / Choir Member` — `N_JOY` spikes from singing, strong `N_BELONGING`.
- `Elder` — slower drift rates, high `N_BELONGING`, elevated `N_GRIEF` baseline, strong community propagation radius.
- `Child` — high `N_CURIOSITY`, rapid `N_JOY` spikes and drops, low `N_RESENTMENT` capacity.
- `New Arrival` — low `N_BELONGING` at start, moderate `N_CURIOSITY`, the integration arc emerges naturally.

### 4.2 Relationships

Actors track a relationship register — a list of other actor IDs with a relationship weight `[-1.0, 1.0]`. This weight modifies inter-actor propagation:

- Positive weight: joy/belonging contagion amplified between them.
- Negative weight (resentment built up): stress contagion amplified, social interactions suppressed.

Relationships drift slowly toward neutral unless reinforced by shared events.

### 4.3 Memory Events

Actors store a small ring buffer of **memory events** — significant node threshold crossings. These influence narrative generation but do not alter the propagation rules. They are the raw material for emergent storytelling.

Example memory events: `LOST_JOB`, `SHARED_MEAL`, `HEARD_BAD_NEWS`, `SAW_SUNRISE_ON_MOUNTAIN`, `SANG_WITH_CROWD`.

---

## 5. World Simulation

### 5.1 Time

The world runs on a **tick-based clock**. One game day = configurable number of ticks. Seasons cycle. Time of day affects environmental inputs (e.g., nighttime suppresses `N_SOCIAL` activity, raises `N_FATIGUE`).

### 5.2 Economic Pressure (Global Events)

The simulation supports global event injection — things that affect the entire community's node graphs simultaneously:

- `EVENT_PIT_CLOSURE` — raises `N_GRIEF` and `N_STRESS` across all miners; triggers solidarity cascade.
- `EVENT_EISTEDDFOD` — a cultural festival; boosts `N_JOY` and `N_BELONGING` community-wide.
- `EVENT_HARD_WINTER` — raises `N_HUNGER` and `N_FATIGUE` drift rates.
- `EVENT_NEW_FAMILY_ARRIVES` — introduces actors with low `N_BELONGING`; tests community's solidarity response.

### 5.3 Procedural Generation

- **Town layout:** WFC (Wave Function Collapse) or noise-based road/parcel generation.
- **Terrain:** Simplex noise heightmap → valley shape → building placement rules.
- **Actor names:** Welsh name corpus for first names; place-derived surnames.
- **Visual style:** Top-down or side-view pixel art rendered via procedural shaders and SDF geometry. No static sprite sheets.

---

## 6. Player Role

The player has no avatar. They are an observer with optional light-touch tools:

- **Chronicle View:** A scrolling log of significant memory events across all actors — the town's living history.
- **Actor Focus:** Click any actor to open their node graph as a live visualisation.
- **Time Controls:** Pause, play, fast-forward.
- **Event Injection (optional, toggleable):** Manually trigger global events to observe systemic responses.

The game is not about winning. It is about watching ordinary lives accumulate into something that feels like meaning.

---

## 7. Tone & Feel

- **Cozy but honest.** The valley is beautiful. Life is also hard. Both are true simultaneously.
- **No irony.** The Welsh valley setting is treated with genuine affection, not as aesthetic shorthand.
- **Quietly political.** The economic history of the South Wales valleys (coal, closures, community resilience) is present in the mechanics without being didactic.
- **Emergent narrative over authored story.** There are no scripted story beats. Everything that happens, happens because the math said so.

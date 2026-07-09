use std::collections::{VecDeque, HashMap};
use rand::rngs::StdRng;
use rand::Rng;
use crate::constants::*;
use super::actor::{Action, NodeId, Role};
use super::world::{GlobalEvent, LocationKind, SimWorld, TileKind, Weather};

// ─── Main tick entry point ────────────────────────────────────────────────────

pub fn tick(world: &mut SimWorld, rng: &mut StdRng) {
    world.clock.advance();

    update_weather(world, rng);
    process_events(world);
    decay_nodes(world);
    propagate_nodes(world);
    apply_contagion(world);
    move_actors(world);
    resolve_arrivals(world, rng);
    update_relationships(world);
    record_events(world);
}

// ─── 1. Node decay / natural drift ───────────────────────────────────────────

fn decay_nodes(world: &mut SimWorld) {
    let is_night = world.clock.is_night();
    for actor in &mut world.actors {
        let ng = &mut actor.node_graph;
        for i in 0..NODE_COUNT {
            let mut rate = ng.drift[i];
            // Night modifier: fatigue rises faster, social need rises slower
            if is_night {
                if i == NodeId::Fatigue as usize  { rate *= 2.0; }
                if i == NodeId::Social  as usize  { rate *= 0.4; }
            }
            ng.values[i] = (ng.values[i] + rate).clamp(0.0, 1.0);
        }
    }
}

// ─── 2. Intra-actor propagation ───────────────────────────────────────────────

fn propagate_nodes(world: &mut SimWorld) {
    for actor in &mut world.actors {
        let ng = &mut actor.node_graph;
        let mut deltas = [0.0f32; NODE_COUNT];

        for src in 0..NODE_COUNT {
            let threshold = ng.thresholds[src];
            if ng.values[src] > threshold {
                let excess = ng.values[src] - threshold;
                for dst in 0..NODE_COUNT {
                    let w = ng.edges[src][dst];
                    if w != 0.0 {
                        deltas[dst] += excess * w * PROPAGATION_DT;
                    }
                }
            }
        }

        for i in 0..NODE_COUNT {
            ng.values[i] = (ng.values[i] + deltas[i]).clamp(0.0, 1.0);
        }
    }
}

// ─── 3. Inter-actor contagion ─────────────────────────────────────────────────

fn apply_contagion(world: &mut SimWorld) {
    let n = world.actors.len();
    let mut deltas = vec![[0.0f32; NODE_COUNT]; n];

    // Read pass — no mutation
    for i in 0..n {
        for j in 0..n {
            if i == j { continue; }
            let dist = (world.actors[i].tile_x - world.actors[j].tile_x).abs()
                     + (world.actors[i].tile_y - world.actors[j].tile_y).abs();
            if dist > CONTAGION_RANGE { continue; }

            let inv_dist = 1.0 / (dist as f32 + 1.0);
            let joy     = world.actors[i].node_graph.values[NodeId::Joy      as usize];
            let stress  = world.actors[i].node_graph.values[NodeId::Stress   as usize];
            let grief   = world.actors[i].node_graph.values[NodeId::Grief    as usize];

            // Relationship weight (default 0 for strangers)
            let rel = world.actors[j].relationships.iter()
                .find(|(id, _)| *id == i)
                .map(|(_, w)| *w)
                .unwrap_or(0.0);
            let amp = 1.0 + rel.max(0.0);

            deltas[j][NodeId::Joy    as usize] += joy    * 0.012 * inv_dist * amp;
            deltas[j][NodeId::Stress as usize] += stress * 0.008 * inv_dist;
            if grief > 0.5 {
                deltas[j][NodeId::Belonging as usize] += 0.006 * inv_dist;
            }
        }
    }

    // Write pass
    for i in 0..n {
        for node in 0..NODE_COUNT {
            world.actors[i].node_graph.values[node] =
                (world.actors[i].node_graph.values[node] + deltas[i][node]).clamp(0.0, 1.0);
        }
    }
}

// ─── 4. Movement (BFS pathfinding) ───────────────────────────────────────────

fn move_actors(world: &mut SimWorld) {
    // Snapshot what each actor needs to move toward (avoids borrow conflict)
    let moves: Vec<Option<(i32, i32)>> = world.actors.iter().map(|actor| {
        if actor.at_target() { return None; }
        Some(bfs_next_step(
            &world.tiles,
            actor.tile_x, actor.tile_y,
            actor.target_x, actor.target_y,
        ))
    }).collect();

    for (actor, next) in world.actors.iter_mut().zip(moves.iter()) {
        if let Some((nx, ny)) = next {
            actor.tile_x = *nx;
            actor.tile_y = *ny;
            actor.current_action = Action::Walking;
        }
    }
}

/// BFS from (sx,sy) toward (tx,ty) on the tile grid.
/// Returns the next tile position to step to, or current pos if already there / no path.
fn bfs_next_step(
    tiles: &[Vec<TileKind>],
    sx: i32, sy: i32,
    tx: i32, ty: i32,
) -> (i32, i32) {
    if sx == tx && sy == ty { return (sx, sy); }

    // If target is blocked, find nearest walkable tile near it
    let (tx, ty) = walkable_near(tiles, tx, ty);

    let mut queue: VecDeque<(i32, i32)> = VecDeque::new();
    let mut prev: HashMap<(i32, i32), (i32, i32)> = HashMap::new();

    queue.push_back((sx, sy));
    prev.insert((sx, sy), (sx, sy));

    const DIRS: [(i32, i32); 4] = [(1,0),(-1,0),(0,1),(0,-1)];

    while let Some((cx, cy)) = queue.pop_front() {
        if cx == tx && cy == ty {
            // Reconstruct: walk back to find first step after start
            let mut cur = (cx, cy);
            loop {
                let p = prev[&cur];
                if p == (sx, sy) { return cur; }
                if p == cur { break; }
                cur = p;
            }
            return (tx, ty);
        }
        if prev.len() > 1500 { break; }

        for (dx, dy) in DIRS {
            let nx = cx + dx;
            let ny = cy + dy;
            if nx < 0 || ny < 0 || nx >= MAP_WIDTH as i32 || ny >= MAP_HEIGHT as i32 { continue; }
            if !tiles[ny as usize][nx as usize].is_walkable() { continue; }
            if prev.contains_key(&(nx, ny)) { continue; }
            prev.insert((nx, ny), (cx, cy));
            queue.push_back((nx, ny));
        }
    }

    // No path: greedy fallback (step to any adjacent walkable tile toward target)
    let mut best = (sx, sy);
    let mut best_dist = i32::MAX;
    for (dx, dy) in DIRS {
        let nx = sx + dx;
        let ny = sy + dy;
        if nx < 0 || ny < 0 || nx >= MAP_WIDTH as i32 || ny >= MAP_HEIGHT as i32 { continue; }
        if !tiles[ny as usize][nx as usize].is_walkable() { continue; }
        let dist = (tx - nx).abs() + (ty - ny).abs();
        if dist < best_dist { best_dist = dist; best = (nx, ny); }
    }
    best
}

/// Returns (tx, ty) if walkable, else the nearest walkable tile within 3 steps.
fn walkable_near(tiles: &[Vec<TileKind>], tx: i32, ty: i32) -> (i32, i32) {
    if tx >= 0 && ty >= 0 && tx < MAP_WIDTH as i32 && ty < MAP_HEIGHT as i32
       && tiles[ty as usize][tx as usize].is_walkable() {
        return (tx, ty);
    }
    for r in 1..=3i32 {
        for dx in -r..=r {
            for dy in -r..=r {
                if dx.abs() != r && dy.abs() != r { continue; }
                let nx = tx + dx; let ny = ty + dy;
                if nx < 0 || ny < 0 || nx >= MAP_WIDTH as i32 || ny >= MAP_HEIGHT as i32 { continue; }
                if tiles[ny as usize][nx as usize].is_walkable() { return (nx, ny); }
            }
        }
    }
    (tx, ty)
}

// ─── 5. Arrival resolution & action selection ─────────────────────────────────

fn resolve_arrivals(world: &mut SimWorld, rng: &mut StdRng) {
    let n = world.actors.len();

    // Collect actions to apply, to avoid borrow conflicts
    struct ActionResult {
        action: Action,
        target_x: i32,
        target_y: i32,
        satisfaction: [(usize, f32); 2], // (node_idx, delta)
        chronicle_entry: Option<String>,
        cooldown: i32,
    }

    let mut results: Vec<Option<ActionResult>> = (0..n).map(|_| None).collect();

    for i in 0..n {
        let actor = &world.actors[i];

        // Decrement cooldown
        if actor.action_cooldown > 0 { continue; }

        // Only select new action when at target
        if !actor.at_target() { continue; }

        let dominant = actor.node_graph.dominant_node();
        let day = world.clock.day;
        let name = actor.name.clone();
        let home_x = actor.home_x;
        let home_y = actor.home_y;

        let (action, target_x, target_y, satisfaction, label) =
            choose_action(dominant, actor, &world.locations, home_x, home_y, rng);

        let entry = if rng.gen_bool(0.3) {
            Some(format!("Day {} {} — {} {}.", day, world.clock.time_label(), name, label))
        } else {
            None
        };

        results[i] = Some(ActionResult {
            action,
            target_x,
            target_y,
            satisfaction,
            chronicle_entry: entry,
            cooldown: rng.gen_range(3..10),
        });
    }

    // Apply results
    for (i, result) in results.into_iter().enumerate() {
        if let Some(r) = result {
            world.actors[i].current_action = r.action;
            world.actors[i].target_x = r.target_x;
            world.actors[i].target_y = r.target_y;
            world.actors[i].action_cooldown = r.cooldown;
            for (node, delta) in r.satisfaction {
                if node < NODE_COUNT {
                    let v = &mut world.actors[i].node_graph.values[node];
                    *v = (*v + delta).clamp(0.0, 1.0);
                }
            }
            if let Some(entry) = r.chronicle_entry {
                world.actors[i].push_memory(world.clock.day, entry.clone());
                world.log(entry);
            }
        }
        // Always tick down cooldown
        if world.actors[i].action_cooldown > 0 {
            world.actors[i].action_cooldown -= 1;
        }
    }
}

fn choose_action(
    dominant: NodeId,
    _actor: &crate::sim::actor::Actor,
    locations: &[crate::sim::world::Location],
    home_x: i32,
    home_y: i32,
    rng: &mut StdRng,
) -> (Action, i32, i32, [(usize, f32); 2], String) {
    // Helper: find a location of given kind, return its tile coords
    let loc = |kind: LocationKind| -> (i32, i32) {
        locations.iter()
            .find(|l| l.kind == kind)
            .map(|l| (l.tile_x, l.tile_y))
            .unwrap_or((home_x, home_y))
    };

    let j = |rng: &mut StdRng| rng.gen_range(-2i32..=2);

    match dominant {
        NodeId::Hunger => {
            let (tx, ty) = loc(LocationKind::Bakery);
            let (jx, jy) = (j(rng), j(rng));
            (Action::Eating, tx + jx, ty + jy,
             [(NodeId::Hunger as usize, -0.55), (NodeId::Joy as usize, 0.10)],
             "sits down for a meal at the bakery".to_string())
        }
        NodeId::Fatigue => {
            let (jx, jy) = (j(rng), j(rng));
            (Action::Sleeping, home_x + jx, home_y + jy,
             [(NodeId::Fatigue as usize, -0.50), (NodeId::Stress as usize, -0.05)],
             "goes home to rest".to_string())
        }
        NodeId::Stress | NodeId::Grief => {
            let go_mountain = rng.gen_bool(0.5);
            let (jx, jy) = (j(rng), j(rng));
            if go_mountain {
                let (tx, ty) = loc(LocationKind::Mountain);
                (Action::Isolating, tx + jx, ty + jy,
                 [(NodeId::Stress as usize, -0.20), (NodeId::Grief as usize, -0.05)],
                 "walks alone to the mountain".to_string())
            } else {
                let (tx, ty) = loc(LocationKind::River);
                (Action::Grieving, tx + jx, ty + jy,
                 [(NodeId::Grief as usize, -0.10), (NodeId::Stress as usize, -0.08)],
                 "sits quietly by the river".to_string())
            }
        }
        NodeId::Social | NodeId::Resentment => {
            let (tx, ty) = loc(LocationKind::Pub);
            let (jx, jy) = (j(rng), j(rng));
            (Action::Socialising, tx + jx, ty + jy,
             [(NodeId::Social as usize, -0.30), (NodeId::Joy as usize, 0.15)],
             "heads to The Red Dragon for company".to_string())
        }
        NodeId::Joy | NodeId::Belonging => {
            let go_choir = rng.gen_bool(0.4);
            let (jx, jy) = (j(rng), j(rng));
            if go_choir {
                let (tx, ty) = loc(LocationKind::ChoirHall);
                (Action::Singing, tx + jx, ty + jy,
                 [(NodeId::Joy as usize, 0.28), (NodeId::Belonging as usize, 0.12)],
                 "joins the choir".to_string())
            } else {
                let (tx, ty) = loc(LocationKind::Common);
                (Action::Socialising, tx + jx, ty + jy,
                 [(NodeId::Social as usize, -0.15), (NodeId::Belonging as usize, 0.10)],
                 "wanders onto the Common".to_string())
            }
        }
        NodeId::Curiosity | NodeId::Purpose => {
            let (tx, ty) = loc(LocationKind::Library);
            let (jx, jy) = (j(rng), j(rng));
            (Action::Creating, tx + jx, ty + jy,
             [(NodeId::Curiosity as usize, -0.25), (NodeId::Purpose as usize, 0.12)],
             "spends time at the library".to_string())
        }
    }
}

// ─── 6. Memory / threshold event recording ────────────────────────────────────

fn record_events(world: &mut SimWorld) {
    // Only check occasionally to avoid log spam
    if world.clock.tick % 30 != 0 { return; }

    let day = world.clock.day;
    let time = world.clock.time_label();

    // Collect events first, then log (to avoid borrow conflict with world.actors)
    let mut entries: Vec<(usize, String)> = Vec::new();

    for actor in &world.actors {
        let joy  = actor.node_graph.values[NodeId::Joy      as usize];
        let grief = actor.node_graph.values[NodeId::Grief   as usize];
        let belonging = actor.node_graph.values[NodeId::Belonging as usize];

        if joy > 0.85 {
            entries.push((actor.id, format!(
                "Day {} {} — {} radiates a quiet happiness.", day, time, actor.name)));
        } else if grief > 0.80 {
            entries.push((actor.id, format!(
                "Day {} {} — {} carries a deep grief.", day, time, actor.name)));
        } else if belonging > 0.90 {
            entries.push((actor.id, format!(
                "Day {} {} — {} feels rooted to this valley.", day, time, actor.name)));
        }
    }

    for (id, entry) in entries {
        world.actors[id].push_memory(day, entry.clone());
        world.log(entry);
    }
}

// ─── 0. Process global events ─────────────────────────────────────────────────

fn process_events(world: &mut SimWorld) {
    // Drain the queue into a local vec to avoid borrow conflicts
    let events: Vec<_> = world.pending_events.drain(..).collect();
    for event in events {
        let day = world.clock.day;
        match &event {
            GlobalEvent::PitClosure => {
                world.log(format!(
                    "Day {} — *** {} *** The valley holds its breath.",
                    day, event.label()));
                let n = world.actors.len();
                for i in 0..n {
                    match world.actors[i].role {
                        Role::Miner => {
                            world.actors[i].node_graph.values[NodeId::Grief  as usize] =
                                (world.actors[i].node_graph.values[NodeId::Grief  as usize] + 0.42).min(1.0);
                            world.actors[i].node_graph.values[NodeId::Stress as usize] =
                                (world.actors[i].node_graph.values[NodeId::Stress as usize] + 0.32).min(1.0);
                            world.actors[i].push_memory(day,
                                "The pit closed. Everything changes now.".to_string());
                        }
                        _ => {
                            world.actors[i].node_graph.values[NodeId::Belonging as usize] =
                                (world.actors[i].node_graph.values[NodeId::Belonging as usize] + 0.10).min(1.0);
                            world.actors[i].node_graph.values[NodeId::Grief as usize] =
                                (world.actors[i].node_graph.values[NodeId::Grief     as usize] + 0.06).min(1.0);
                        }
                    }
                }
            }
            GlobalEvent::Eisteddfod => {
                world.log(format!(
                    "Day {} — *** {} *** Music fills the valley.",
                    day, event.label()));
                for actor in &mut world.actors {
                    actor.node_graph.values[NodeId::Joy       as usize] =
                        (actor.node_graph.values[NodeId::Joy       as usize] + 0.28).min(1.0);
                    actor.node_graph.values[NodeId::Belonging as usize] =
                        (actor.node_graph.values[NodeId::Belonging as usize] + 0.15).min(1.0);
                    if actor.role == Role::Musician {
                        actor.node_graph.values[NodeId::Joy as usize] =
                            (actor.node_graph.values[NodeId::Joy as usize] + 0.18).min(1.0);
                    }
                }
            }
            GlobalEvent::HardWinter => {
                world.log(format!(
                    "Day {} — *** {} *** A hard frost settles on Cwm Newydd.",
                    day, event.label()));
                for actor in &mut world.actors {
                    actor.node_graph.drift[NodeId::Hunger  as usize] *= 1.9;
                    actor.node_graph.drift[NodeId::Fatigue as usize] *= 1.6;
                }
            }
            GlobalEvent::Bereavement { actor_id } => {
                let aid = *actor_id;
                let name = world.actors.iter()
                    .find(|a| a.id == aid)
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Someone".to_string());
                let pos = world.actors.iter()
                    .find(|a| a.id == aid)
                    .map(|a| (a.tile_x, a.tile_y));
                world.log(format!(
                    "Day {} — *** {} *** {} has passed. The valley mourns.",
                    day, event.label(), name));
                if let Some((bx, by)) = pos {
                    for actor in &mut world.actors {
                        let dist = (actor.tile_x - bx).abs() + (actor.tile_y - by).abs();
                        if dist < 25 {
                            let extra = if actor.role == Role::Elder { 0.25 } else { 0.0 };
                            actor.node_graph.values[NodeId::Grief as usize] =
                                (actor.node_graph.values[NodeId::Grief as usize] + 0.18 + extra).min(1.0);
                        }
                    }
                }
            }
        }
    }
}

// ─── 7. Relationship drift ────────────────────────────────────────────────────

fn update_relationships(world: &mut SimWorld) {
    if world.clock.tick % 7 != 0 { return; } // throttle to every 7 ticks

    let n = world.actors.len();
    // Collect nudges: (actor_idx, other_idx, delta)
    let mut nudges: Vec<(usize, usize, f32)> = Vec::new();

    for i in 0..n {
        let is_social = matches!(world.actors[i].current_action,
            Action::Socialising | Action::Singing | Action::Helping);
        let high_stress = world.actors[i].node_graph.values[NodeId::Stress as usize] > 0.72;

        for j in 0..n {
            if i == j { continue; }
            let dist = (world.actors[i].tile_x - world.actors[j].tile_x).abs()
                     + (world.actors[i].tile_y - world.actors[j].tile_y).abs();
            if dist > 3 { continue; }

            if is_social {
                // Positive social interaction: strengthen bond both ways
                nudges.push((i, j,  0.012));
                nudges.push((j, i,  0.012));
            }
            if high_stress {
                // High-stress actor erodes bonds slightly for those nearby
                nudges.push((j, i, -0.006));
            }
        }
    }

    // Natural slow decay toward neutral for all existing relationships
    for actor in &mut world.actors {
        for (_, weight) in &mut actor.relationships {
            if weight.abs() > 0.02 {
                *weight *= 0.998; // very slow drift toward 0
            }
        }
    }

    // Apply nudges
    for (from, to, delta) in nudges {
        if let Some(rel) = world.actors[from].relationships.iter_mut().find(|(id, _)| *id == to) {
            rel.1 = (rel.1 + delta).clamp(-1.0, 1.0);
        } else if delta > 0.0 {
            world.actors[from].relationships.push((to, delta));
        }
    }
}

// ─── 8. Weather transitions ───────────────────────────────────────────────────

fn update_weather(world: &mut SimWorld, rng: &mut StdRng) {
    if world.weather_timer > 0 {
        world.weather_timer -= 1;
        return;
    }
    // Weighted: Sunny most common, Fog rarest
    let roll: u32 = rng.gen_range(0..10);
    world.weather = match roll {
        0 | 1 | 2 | 3 => Weather::Sunny,
        4 | 5 | 6     => Weather::Overcast,
        7 | 8         => Weather::Rain,
        _             => Weather::Fog,
    };
    world.weather_timer = rng.gen_range(50u32..140);

    // Log notable weather changes
    match world.weather {
        Weather::Rain => world.log(format!(
            "Day {} — Rain sweeps down the valley.", world.clock.day)),
        Weather::Fog  => world.log(format!(
            "Day {} — A thick fog settles over Cwm Newydd.", world.clock.day)),
        _ => {}
    }
}

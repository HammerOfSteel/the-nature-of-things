use rand::rngs::StdRng;
use rand::Rng;
use crate::constants::*;
use super::actor::{Action, NodeId};
use super::world::{LocationKind, SimWorld};

// ─── Main tick entry point ────────────────────────────────────────────────────

pub fn tick(world: &mut SimWorld, rng: &mut StdRng) {
    world.clock.advance();

    decay_nodes(world);
    propagate_nodes(world);
    apply_contagion(world);
    move_actors(world);
    resolve_arrivals(world, rng);
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

// ─── 4. Movement ─────────────────────────────────────────────────────────────

fn move_actors(world: &mut SimWorld) {
    for actor in &mut world.actors {
        if actor.at_target() { continue; }

        let dx = (actor.target_x - actor.tile_x).signum();
        let dy = (actor.target_y - actor.tile_y).signum();

        // Try horizontal first, then vertical, then diagonal
        let candidates = [(dx, 0), (0, dy), (dx, dy)];
        for (cx, cy) in candidates {
            if cx == 0 && cy == 0 { continue; }
            let nx = actor.tile_x + cx;
            let ny = actor.tile_y + cy;
            if nx < 0 || ny < 0 || nx >= MAP_WIDTH as i32 || ny >= MAP_HEIGHT as i32 { continue; }
            if world.tiles[ny as usize][nx as usize].is_walkable() {
                actor.tile_x = nx;
                actor.tile_y = ny;
                actor.current_action = Action::Walking;
                break;
            }
        }
    }
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

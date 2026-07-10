/// Procedural South Wales valley world generator.
///
/// Topology: a single steep V-shaped glacial valley running east-west.
/// A river occupies the valley floor.  Victorian terraced houses climb the
/// slopes.  Upper slopes are heather moorland; ridges are bare slate.
/// A ruined colliery sits on the mid-slope above one terrace row.

use super::{VoxelWorld, Vox};
use super::building::{build_terrace_house, stamp_building};

// ─── Noise ────────────────────────────────────────────────────────────────────

fn nhash(x: i32, y: i32) -> f32 {
    let mut v = x.wrapping_mul(1_619).wrapping_add(y.wrapping_mul(31_337));
    v ^= v >> 13;
    v  = v.wrapping_mul(1_274_126_177);
    v ^= v >> 16;
    ((v & 0x7fff_ffff) as f32) / (0x7fff_ffff_u32 as f32)
}
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }
fn snoise(x: f32, y: f32) -> f32 {
    let (ix, iy) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (ux, uy) = (fx*fx*(3.0-2.0*fx), fy*fy*(3.0-2.0*fy));
    lerp(lerp(nhash(ix,iy),   nhash(ix+1,iy),   ux),
         lerp(nhash(ix,iy+1), nhash(ix+1,iy+1), ux), uy)
}
fn fbm(x: f32, y: f32) -> f32 {
    snoise(x,y)*0.500 + snoise(x*2.0,y*2.0)*0.250
  + snoise(x*4.0,y*4.0)*0.125 + snoise(x*8.0,y*8.0)*0.063
}

// ─── Settlement zones & street network ───────────────────────────────────────

/// What gets built along a street segment or in a block between side streets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SettlementZone {
    TerracedRow,       // Victorian terraces packed tight — the classic valley look
    MixedResidential,  // Same build type with wider 8-vox gaps between houses
    VillageCore,       // Future: pub + chapel + open green; kept clear for now
    OpenHillside,      // No buildings — trees, bracken, heather only
}

struct SideStreet {
    x_centre:         usize,
    width:            usize,
    north_end:        usize, // Z of north terminus (smaller Z = uphill / north)
    south_end:        usize, // Z of south terminus (larger Z)
    cul_de_sac_north: bool,
    cul_de_sac_south: bool,
}

struct StreetNetwork {
    road_z:      Vec<usize>,               // main road Z-centre per X column
    road_w:      usize,                    // main road width in voxels (16)
    sides:       Vec<SideStreet>,
    /// Outer building-band zone per block between side streets: [north_zone, south_zone]
    outer_zones: Vec<[SettlementZone; 2]>,
}

fn zone_from_prob(p: f32) -> SettlementZone {
    if      p < 0.55 { SettlementZone::TerracedRow }
    else if p < 0.72 { SettlementZone::MixedResidential }
    else if p < 0.92 { SettlementZone::OpenHillside }
    else             { SettlementZone::VillageCore }
}

/// Build an organic street network for the valley.
///
/// The main road curves ±24 vox around the valley centre via smooth noise.
/// Side streets branch N and S at seeded irregular X intervals (60–120 vox).
/// Each block between consecutive side streets gets an outer-band SettlementZone.
fn build_street_network(
    wx: usize, wz: usize,
    valley_centre_z: f32,
    seed: u32,
) -> StreetNetwork {
    let s = seed as f32 * 0.013;

    // Main road: snoise wander ±24 vox around valley centre
    let road_z: Vec<usize> = (0..wx)
        .map(|x| {
            let w = snoise(x as f32 / 280.0 + s * 4.1, s * 0.9) * 24.0;
            (valley_centre_z + w).round().clamp(24.0, wz as f32 - 24.0) as usize
        })
        .collect();

    // Side streets at seeded irregular X spacing
    let mut sides: Vec<SideStreet> = Vec::new();
    let mut x = 40usize;
    let mut idx = 0u32;
    while x + 40 < wx {
        let spacing = 60 + (nhash(seed as i32 * 3 + idx as i32 * 17, 5) * 64.0) as usize;
        let zc      = road_z[x.min(wx - 1)];
        let half_r  = 8usize; // half of 16-vox main road

        // How far each arm extends from the road edge
        let arm_n = 52 + (nhash(x as i32 * 3 + seed as i32, 77) * 72.0) as usize;
        let arm_s = 52 + (nhash(x as i32 * 5 + seed as i32, 55) * 72.0) as usize;
        let w     = if nhash(x as i32 + seed as i32, 29) > 0.5 { 12 } else { 8 };

        sides.push(SideStreet {
            x_centre:         x,
            width:            w,
            north_end:        zc.saturating_sub(half_r + arm_n),
            south_end:        (zc + half_r + arm_s).min(wz.saturating_sub(8)),
            cul_de_sac_north: nhash(x as i32 * 7 + seed as i32, 11) > 0.65,
            cul_de_sac_south: nhash(x as i32 * 9 + seed as i32, 13) > 0.65,
        });

        x += spacing;
        idx += 1;
    }

    // Assign outer building-band zone per block between consecutive side streets
    let block_xs: Vec<usize> = {
        let mut v = vec![0usize];
        v.extend(sides.iter().map(|ss| ss.x_centre));
        v.push(wx);
        v
    };
    let outer_zones: Vec<[SettlementZone; 2]> = (0..block_xs.len().saturating_sub(1))
        .map(|i| {
            let bx = block_xs[i];
            [
                zone_from_prob(nhash(bx as i32 * 11 + seed as i32, 33)),
                zone_from_prob(nhash(bx as i32 * 13 + seed as i32, 44)),
            ]
        })
        .collect();

    StreetNetwork { road_z, road_w: 16, sides, outer_zones }
}

// ─── Main generator ───────────────────────────────────────────────────────────

pub fn generate_wales_valley(wx: usize, wy: usize, wz: usize, seed: u32) -> VoxelWorld {
    let mut world = VoxelWorld::new(wx, wy, wz);
    let s = seed as f32 * 0.013;

    // Valley axis runs along X (east-west).  Z is the cross-valley axis.
    let valley_centre_z = wz as f32 * 0.5;
    let valley_half     = wz as f32 * 0.38;  // half-width of ridge-to-ridge

    // ── Height map ────────────────────────────────────────────────────────────
    let mut hmap = vec![vec![0usize; wx]; wz];
    for gz in 0..wz {
        for gx in 0..wx {
            // Normalised cross-valley position: -1 = north ridge, +1 = south ridge
            let nz  = (gz as f32 - valley_centre_z) / valley_half;
            let nz2 = nz * nz;

            // Ridge height: parabolic profile + large-scale FBM
            // Denominators ×4 vs old 128-world to maintain same visual frequency at 512-wide world
            let ridge  = 0.55 + fbm(gx as f32 / 560.0 + s,  gz as f32 / 560.0 + s * 0.7) * 0.25;
            let valley_floor = 0.18 + fbm(gx as f32 / 240.0 + s * 2.0, gz as f32 / 240.0 + s * 1.3) * 0.06;

            // V-shape: lerp from floor (centre) to ridge (edges) using nz²
            let t   = nz2.min(1.0);
            let h01 = lerp(valley_floor, ridge, t);

            // Fine detail noise
            let detail = fbm(gx as f32 / 88.0 + s * 3.1, gz as f32 / 88.0 + s * 2.4) * 0.06;

            hmap[gz][gx] = ((h01 + detail) * (wy - 4) as f32) as usize + 2;
        }
    }

    // ── Fill terrain ──────────────────────────────────────────────────────────
    let river_y = 20usize;  // river water-table voxel Y  (was 5, ×4)
    let shore_y = 24usize;  // 1 voxel above river = gravel shore  (was 6, ×4)

    for gz in 0..wz {
        for gx in 0..wx {
            let h = hmap[gz][gx].min(wy - 1);

            for y in 0..h {
                let v = if y + 1 == h {
                    surface_material(gz, gx, h, &hmap, wz, wy, s)
                } else if y + 1 >= h.saturating_sub(2) {
                    if h <= shore_y + 1 { Vox::Gravel } else { Vox::Dirt }
                } else {
                    Vox::Stone
                };
                world.set(gx, y, gz, v);
            }

            // River water in valley trough
            if h <= river_y {
                for y in h..=river_y {
                    world.set(gx, y, gz, Vox::Water);
                }
            }
        }
    }

    // ── River banks — gravel + mud ─────────────────────────────────────────
    for gz in 0..wz {
        for gx in 0..wx {
            let h = hmap[gz][gx];
            if h == shore_y || h == shore_y + 1 {
                world.set(gx, h.saturating_sub(1), gz, Vox::Gravel);
                if h == shore_y + 1 {
                    world.set(gx, h.saturating_sub(2), gz, Vox::Mud);
                }
            }
        }
    }

    // ── Pre-compute occupancy (building footprints + clearance) ─────────────
    // Trees must not be stamped inside or immediately beside a building.
    let mut occupied: Vec<Vec<bool>> = vec![vec![false; wx]; wz];

    // Build the organic street network (curved main road + branching side streets)
    let network = build_street_network(wx, wz, valley_centre_z, seed);
    mark_network_occupancy(&network, &hmap, wx, wz, seed, &mut occupied);

    // Also mark a wide strip around the colliery
    let col_x_plan = (nhash(seed as i32 * 7, 5) * (wx - 80) as f32) as usize + 40;
    let col_z_plan = (valley_centre_z + valley_half * 0.45) as usize;
    for dz in 0..80i32 { for dx in -16..80i32 {
        let mx = col_x_plan as i32 + dx;
        let mz = col_z_plan as i32 + dz;
        if mx >= 0 && mz >= 0 && mx < wx as i32 && mz < wz as i32 {
            occupied[mz as usize][mx as usize] = true;
        }
    }}

    // ── Hedgerow trees (skip occupied cells + canopy buffer) ─────────────────
    for gz in 1..wz-1 {
        for gx in 1..wx-1 {
            let h = hmap[gz][gx];
            if h < shore_y + 8 || h + 24 >= wy { continue; }
            let nz = ((gz as f32 - valley_centre_z) / valley_half).abs();
            if nz > 0.80 { continue; }
            // Check occupancy for this cell + 4-voxel canopy buffer (was 1)
            let blocked = (-4i32..=8).any(|dz| (-4i32..=8).any(|dx| {
                let cx = gx as i32 + dx;
                let cz = gz as i32 + dz;
                cx >= 0 && cz >= 0 && cx < wx as i32 && cz < wz as i32
                    && occupied[cz as usize][cx as usize]
            }));
            if blocked { continue; }
            if nhash(gx as i32 * 11 + seed as i32, gz as i32 * 17 + seed as i32) < 0.91 { continue; }
            stamp_tree(&mut world, gx, h, gz, seed);
        }
    }

    // ── Road network (curved main road + branching side streets) ─────────────
    stamp_network_roads(&network, &mut world, &hmap, wx, wz);

    // ── Settlement buildings (inner band always TerracedRow, outer zone-driven) ─
    stamp_network_buildings(&network, &mut world, &hmap, wx, wy, wz, seed);

    // ── Derelict colliery (north slope, mid-height) ───────────────────────────
    let col_x = (nhash(seed as i32 * 7, 5) * (wx - 80) as f32) as usize + 40;
    let col_z = (valley_centre_z + valley_half * 0.45) as usize;
    let col_h = hmap[col_z.min(wz-1)][col_x.min(wx-1)];
    if col_h + 48 < wy {
        stamp_colliery(&mut world, col_x, col_h, col_z);
    }

    world
}

// ─── Surface material selection ───────────────────────────────────────────────

fn surface_material(
    gz: usize, gx: usize, h: usize,
    hmap: &[Vec<usize>], wz: usize, wy: usize, s: f32,
) -> Vox {
    let shore_y = 24usize;
    let wy_f = wy as f32;
    let h_frac = h as f32 / wy_f;

    if h <= shore_y + 1 { return Vox::Gravel; }

    let detail_n = snoise(gx as f32 / 56.0 + s * 6.1, gz as f32 / 56.0 + s * 5.3);
    let moisture  = fbm(gx as f32 / 160.0 + s * 8.0,  gz as f32 / 160.0 + s * 7.0);

    // Slope (approx from height neighbours)
    let dh_x = (hmap[gz][gx.min(gx+1).min(hmap[0].len()-1)] as f32
               - hmap[gz][gx.saturating_sub(1)] as f32).abs();
    let dh_z = (hmap[gz.min(gz+1).min(hmap.len()-1)][gx] as f32
               - hmap[gz.saturating_sub(1)][gx] as f32).abs();
    let slope = (dh_x.max(dh_z) / 2.0).clamp(0.0, 1.0);

    if h_frac > 0.78 {
        // Ridge
        if detail_n > 0.6 { Vox::Slate } else { Vox::Stone }
    } else if h_frac > 0.60 {
        // Upper slope — heather and exposed rock
        if slope > 0.6 { Vox::Stone }
        else if detail_n > 0.5 { Vox::Heather }
        else { Vox::Bracken }
    } else if h_frac > 0.38 {
        // Mid slope — bracken and grass
        if slope > 0.55 { Vox::Stone }
        else if moisture > 0.58 { Vox::Bracken }
        else { Vox::Grass }
    } else {
        // Lower slope and valley floor — meadow
        if moisture > 0.70 && slope < 0.2 { Vox::Mud }
        else { Vox::Grass }
    }
}

// ─── Tree stamp ───────────────────────────────────────────────────────────────

fn stamp_tree(world: &mut VoxelWorld, gx: usize, base_h: usize, gz: usize, seed: u32) {
    // 4× scale: trunk 8–20 vox tall, canopy radius up to 4
    let trunk_h = 8 + (nhash(gx as i32*7+seed as i32, gz as i32*11) * 12.0) as usize;
    for ty in base_h..base_h + trunk_h {
        world.set(gx, ty, gz, Vox::Log);
    }
    let top = base_h + trunk_h;
    // 6 canopy layers: r=4,4,3,3,2,1 (pyramidal crown)
    let radii: [i32; 6] = [4, 4, 3, 3, 2, 1];
    for (cy, &r) in radii.iter().enumerate() {
        for cx in -r..=r {
            for cz in -r..=r {
                if cx*cx + cz*cz > r*r + 2 { continue; }  // rough circle
                let nx = gx as i32 + cx;
                let nz = gz as i32 + cz;
                if nx >= 0 && nz >= 0 && nx < world.wx as i32 && nz < world.wz as i32 {
                    if world.get(nx, (top + cy) as i32, nz).is_air() {
                        world.set(nx as usize, top + cy, nz as usize, Vox::Leaf);
                    }
                }
            }
        }
    }
}

// ─── Occupancy ────────────────────────────────────────────────────────────────

/// Mark every road corridor and both building bands in the occupancy grid.
fn mark_network_occupancy(
    network: &StreetNetwork,
    hmap: &[Vec<usize>],
    wx: usize, wz: usize,
    seed: u32,
    occupied: &mut Vec<Vec<bool>>,
) {
    let half_r = network.road_w / 2;

    // Main road corridor
    for (x, &zc) in network.road_z.iter().enumerate() {
        for dz in 0..network.road_w {
            let gz = zc.saturating_sub(half_r) + dz;
            if gz < wz { occupied[gz][x] = true; }
        }
    }
    // Side street corridors + cul-de-sac pads
    for ss in &network.sides {
        let half_s = ss.width / 2;
        let x0 = ss.x_centre.saturating_sub(half_s);
        let x1 = (ss.x_centre + half_s + 1).min(wx);
        for z in ss.north_end..=ss.south_end {
            for x in x0..x1 { if z < wz { occupied[z][x] = true; } }
        }
    }

    // Inner building band (offset 53 from road centre): always TerracedRow
    mark_band_occ(network, hmap, wx, wz, 53, true,  4, SettlementZone::TerracedRow, 0, wx, seed, occupied);
    mark_band_occ(network, hmap, wx, wz, 53, false, 4, SettlementZone::TerracedRow, 0, wx, seed.wrapping_add(1337), occupied);

    // Outer building band (offset 97): zone varies per block between side streets
    let block_xs = block_x_boundaries(network, wx);
    for (bi, &[nzone, szone]) in network.outer_zones.iter().enumerate() {
        let (x0, x1) = (block_xs[bi], block_xs[bi + 1]);
        let gap_n = if nzone == SettlementZone::MixedResidential { 8 } else { 4 };
        let gap_s = if szone == SettlementZone::MixedResidential { 8 } else { 4 };
        mark_band_occ(network, hmap, wx, wz, 97, true,  gap_n, nzone, x0, x1, seed.wrapping_add(7919), occupied);
        mark_band_occ(network, hmap, wx, wz, 97, false, gap_s, szone, x0, x1, seed.wrapping_add(9256), occupied);
    }
}

/// Precompute the X-boundary list between consecutive side streets.
fn block_x_boundaries(network: &StreetNetwork, wx: usize) -> Vec<usize> {
    let mut v = vec![0usize];
    v.extend(network.sides.iter().map(|ss| ss.x_centre));
    v.push(wx);
    v
}

/// Mark building footprints for one band (inner or outer) within [x_start, x_end).
fn mark_band_occ(
    network: &StreetNetwork,
    hmap: &[Vec<usize>],
    wx: usize, wz: usize,
    offset: usize,      // Z offset from road_z[x] to house origin
    north_side: bool,   // true → house Z = road_z - offset
    house_gap: usize,   // gap between houses (4 for TerracedRow, 8 for Mixed)
    zone: SettlementZone,
    x_start: usize, x_end: usize,
    seed: u32,
    occupied: &mut Vec<Vec<bool>>,
) {
    if matches!(zone, SettlementZone::OpenHillside | SettlementZone::VillageCore) { return; }
    let depth  = 32usize;
    let margin = 4usize;
    let mut gx = x_start.max(12);
    let mut hi = 0u32;
    while gx + 40 < x_end.min(wx) {
        let zc = network.road_z[gx.min(wx - 1)];
        let tz = if north_side { zc.saturating_sub(offset) }
                 else           { (zc + offset).min(wz.saturating_sub(40)) };
        if hmap[tz.min(wz - 1)][gx] < 30 { gx += 1; continue; }
        let w = 20 + (nhash(gx as i32 * 31 + seed as i32, hi as i32 * 13) * 12.0) as usize;
        for fz in 0..depth + margin * 2 {
            for fx in 0..w + margin * 2 {
                let mx = gx.saturating_sub(margin) + fx;
                let mz = tz.saturating_sub(margin) + fz;
                if mx < wx && mz < wz { occupied[mz][mx] = true; }
            }
        }
        gx += w + house_gap;
        hi += 1;
    }
}

// ─── Road stamping ────────────────────────────────────────────────────────────

fn stamp_network_roads(
    network: &StreetNetwork,
    world: &mut VoxelWorld,
    hmap: &[Vec<usize>],
    wx: usize, wz: usize,
) {
    let half_r = network.road_w / 2;

    // Main road (cobble surface + stone sub-base)
    for (x, &zc) in network.road_z.iter().enumerate() {
        for dz in 0..network.road_w {
            let gz = zc.saturating_sub(half_r) + dz;
            if gz >= wz { continue; }
            let h = hmap[gz][x];
            if h == 0 { continue; }
            world.set(x, h, gz, Vox::Cobble);
            if h > 0 { world.set(x, h - 1, gz, Vox::Stone); }
        }
    }

    // Side streets + optional cul-de-sac turning pads
    for ss in &network.sides {
        let half_s = ss.width / 2;
        let x0 = ss.x_centre.saturating_sub(half_s);
        let x1 = (ss.x_centre + half_s + 1).min(wx);
        for z in ss.north_end..=ss.south_end {
            for x in x0..x1 {
                if x >= wx || z >= wz { continue; }
                let h = hmap[z][x];
                if h == 0 { continue; }
                world.set(x, h, z, Vox::Cobble);
                if h > 0 { world.set(x, h - 1, z, Vox::Stone); }
            }
        }

        // Cul-de-sac: 16-vox-wide turning circle at the terminus
        let pad_half = 8usize;
        let pad_x0 = ss.x_centre.saturating_sub(pad_half);
        let pad_x1 = (ss.x_centre + pad_half).min(wx);
        if ss.cul_de_sac_north {
            let zpad = ss.north_end;
            let zpad_end = (zpad + 16).min(wz);
            for z in zpad..zpad_end { for x in pad_x0..pad_x1 {
                if x < wx && z < wz {
                    let h = hmap[z][x];
                    if h > 0 { world.set(x, h, z, Vox::Cobble); if h > 0 { world.set(x, h-1, z, Vox::Stone); } }
                }
            }}
        }
        if ss.cul_de_sac_south && ss.south_end >= 16 {
            let zpad = ss.south_end.saturating_sub(15);
            for z in zpad..=ss.south_end { for x in pad_x0..pad_x1 {
                if x < wx && z < wz {
                    let h = hmap[z][x];
                    if h > 0 { world.set(x, h, z, Vox::Cobble); if h > 0 { world.set(x, h-1, z, Vox::Stone); } }
                }
            }}
        }
    }
}

// ─── Building placement ───────────────────────────────────────────────────────

/// Stamp all buildings for both bands across the network.
fn stamp_network_buildings(
    network: &StreetNetwork,
    world: &mut VoxelWorld,
    hmap: &[Vec<usize>],
    wx: usize, wy: usize, wz: usize,
    seed: u32,
) {
    // Inner band (offset 53): always TerracedRow across full width
    stamp_road_aligned_row(network, world, hmap, wx, wy, wz,
        53, true,  4, SettlementZone::TerracedRow, 0, wx, seed);
    stamp_road_aligned_row(network, world, hmap, wx, wy, wz,
        53, false, 4, SettlementZone::TerracedRow, 0, wx, seed.wrapping_add(1337));

    // Outer band (offset 97): zone driven per block between side streets
    let block_xs = block_x_boundaries(network, wx);
    for (bi, &[nzone, szone]) in network.outer_zones.iter().enumerate() {
        let (x0, x1) = (block_xs[bi], block_xs[bi + 1]);
        let gap_n = if nzone == SettlementZone::MixedResidential { 8 } else { 4 };
        let gap_s = if szone == SettlementZone::MixedResidential { 8 } else { 4 };
        stamp_road_aligned_row(network, world, hmap, wx, wy, wz,
            97, true,  gap_n, nzone, x0, x1, seed.wrapping_add(7919));
        stamp_road_aligned_row(network, world, hmap, wx, wy, wz,
            97, false, gap_s, szone, x0, x1, seed.wrapping_add(9256));
    }
}

/// Stamp one row of houses along the main road within x_start..x_end.
/// `offset` is the Z distance from road_z[x] to the house origin.
/// `north_side = true`  → house at road_z - offset, face_south = true.
/// `north_side = false` → house at road_z + offset, face_south = false.
fn stamp_road_aligned_row(
    network: &StreetNetwork,
    world: &mut VoxelWorld,
    hmap: &[Vec<usize>],
    wx: usize, wy: usize, wz: usize,
    offset: usize,
    north_side: bool,
    house_gap: usize,
    zone: SettlementZone,
    x_start: usize, x_end: usize,
    seed: u32,
) {
    if matches!(zone, SettlementZone::OpenHillside | SettlementZone::VillageCore) { return; }
    let mut gx = x_start.max(12);
    let mut hi = 0u32;
    while gx + 40 < x_end.min(wx) {
        let zc = network.road_z[gx.min(wx - 1)];
        let tz = if north_side { zc.saturating_sub(offset) }
                 else           { (zc + offset).min(wz.saturating_sub(40)) };
        let h = hmap[tz.min(wz - 1)][gx];
        if h < 30 || h + 60 >= wy { gx += 1; continue; }
        let var = nhash(gx as i32 * 31 + seed as i32, hi as i32 * 13);
        let w   = 20 + (var * 12.0) as usize;
        let building = build_terrace_house(gx, h, tz, w, north_side, seed ^ hi);
        stamp_building(world, &building);
        gx += w + house_gap;
        hi += 1;
    }
}

// ─── Derelict colliery ────────────────────────────────────────────────────────

fn stamp_colliery(world: &mut VoxelWorld, ox: usize, base_y: usize, oz: usize) {
    // Engine house: 24×20 footprint, 32 tall, roofless stone walls (was 6×5×8, ×4)
    let ew = 24usize; let ed = 20usize; let eh = 32usize;
    for fy in 0..eh {
        for fz in 0..ed {
            for fx in 0..ew {
                let wall = fx == 0 || fx == ew-1 || fz == 0 || fz == ed-1;
                if wall { world.set(ox + fx, base_y + fy, oz + fz, Vox::Stone); }
            }
        }
    }
    // Arched window openings (8 wide, 12 tall in south wall)
    for fy in 6..18 { for fx in 6..14 { world.set(ox + fx, base_y + fy, oz, Vox::Air); } }

    // Headframe: two stone towers, 48 vox tall, 8 apart (was 12 vox tall, 3 apart)
    let tx = ox + ew + 8;
    for fy in 0..48 { world.set(tx,     base_y + fy, oz + 8, Vox::Stone); }
    for fy in 0..48 { world.set(tx + 12, base_y + fy, oz + 8, Vox::Stone); }
    // Cross beam at top
    for fx in 0..13 {
        world.set(tx + fx, base_y + 48, oz + 8, Vox::Stone);
        world.set(tx + fx, base_y + 47, oz + 8, Vox::Stone);
    }

    // Coal tip mound
    let tip_x = ox as i32 - 32;
    let tip_z = oz as i32 + 8;
    for tz in -16..=16i32 {
        for tx2 in -20..=20i32 {
            let r = (tz*tz + tx2*tx2) as f32;
            if r > 352.0 { continue; }
            let tip_h = ((1.0 - r / 352.0) * 20.0) as usize;
            for ty in 0..=tip_h {
                let wx2 = tip_x + tx2;
                let wz2 = tip_z + tz;
                if wx2 >= 0 && wz2 >= 0 {
                    world.set(
                        wx2 as usize, base_y + ty, wz2 as usize,
                        if ty == tip_h { Vox::Grass } else { Vox::Stone },
                    );
                }
            }
        }
    }
}

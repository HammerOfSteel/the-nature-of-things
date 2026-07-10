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

    // Mark street corridors first
    mark_road_occupancy(&hmap, wx, wz, valley_centre_z, valley_half, &mut occupied);

    // Two terrace rows per side: lower slope (~nz 0.27) and mid slope (~nz 0.50)
    // Rows staggered by seed to avoid identical house widths lining up across rows
    let terrace_bands: [(f32, u32); 2] = [(0.27, 0), (0.50, 7919)];
    for &(band, seed_off) in &terrace_bands {
        let rs = seed.wrapping_add(seed_off);
        plan_terrace_occupancy(&hmap, wx, wz, valley_centre_z, valley_half, true,  band, rs,              &mut occupied);
        plan_terrace_occupancy(&hmap, wx, wz, valley_centre_z, valley_half, false, band, rs.wrapping_add(1337), &mut occupied);
    }
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

    // ── Road network (valley floor + side streets) ────────────────────────────
    stamp_roads(&mut world, &hmap, wx, wz, valley_centre_z, valley_half);

    // ── Terraced houses — two rows per side ───────────────────────────────────
    for &(band, seed_off) in &terrace_bands {
        let rs = seed.wrapping_add(seed_off);
        stamp_terrace_row(&mut world, &hmap, wx, wy, wz, valley_centre_z, valley_half, true,  band, rs);
        stamp_terrace_row(&mut world, &hmap, wx, wy, wz, valley_centre_z, valley_half, false, band, rs.wrapping_add(1337));
    }

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

// ─── Road network ─────────────────────────────────────────────────────────────

/// Mark road corridors in the occupancy grid so trees don't grow on roads.
fn mark_road_occupancy(
    _hmap: &[Vec<usize>],
    wx: usize, wz: usize,
    valley_centre_z: f32, _valley_half: f32,
    occupied: &mut Vec<Vec<bool>>,
) {
    // Main valley road: 16 voxels wide (was 4, ×4)
    let road_z_centre = valley_centre_z as usize;
    for gx in 0..wx {
        for dz in 0..16usize {
            let gz = road_z_centre.saturating_sub(6) + dz;
            if gz < wz { occupied[gz][gx] = true; }
        }
    }
    // Side streets: every 80 voxels along X (was 20, ×4), 16 voxels wide
    for sx in (40..wx).step_by(80) {
        for dx in 0..16usize {
            let gx = sx.saturating_sub(6) + dx;
            if gx >= wx { continue; }
            for gz in 0..wz {
                occupied[gz][gx] = true;
            }
        }
    }
}

/// Stamp Cobble voxels on road surfaces (called after trees, before houses).
fn stamp_roads(
    world: &mut VoxelWorld,
    hmap: &[Vec<usize>],
    wx: usize, wz: usize,
    valley_centre_z: f32, _valley_half: f32,
) {
    let road_z_centre = valley_centre_z as usize;

    // Main valley road (16 vox wide)
    for gx in 0..wx {
        for dz in 0..16usize {
            let gz = road_z_centre.saturating_sub(6) + dz;
            if gz >= wz { continue; }
            let h = hmap[gz][gx];
            if h == 0 { continue; }
            world.set(gx, h, gz, Vox::Cobble);
            world.set(gx, h.saturating_sub(1), gz, Vox::Stone);
        }
    }

    // Side streets (16 vox wide, every 80 vox along X)
    for sx in (40..wx).step_by(80) {
        for dx in 0..16usize {
            let gx = sx.saturating_sub(6) + dx;
            if gx >= wx { continue; }
            for gz in 0..wz {
                let h = hmap[gz][gx];
                if h == 0 { continue; }
                world.set(gx, h, gz, Vox::Cobble);
                world.set(gx, h.saturating_sub(1), gz, Vox::Stone);
            }
        }
    }
}

// ─── Occupancy pre-planner ────────────────────────────────────────────────────
// Mirrors stamp_terrace_row's placement logic but only marks the occupied grid.
// Called before trees so trees never overlap buildings.

fn plan_terrace_occupancy(
    hmap: &[Vec<usize>],
    wx: usize, wz: usize,
    valley_centre_z: f32, valley_half: f32,
    north_side: bool,
    nz_band: f32,   // normalised Z distance from valley centre (0.0=centre, 1.0=ridge)
    seed: u32,
    occupied: &mut Vec<Vec<bool>>,
) {
    let sign: f32 = if north_side { -1.0 } else { 1.0 };
    let target_z  = ((valley_centre_z + sign * nz_band * valley_half) as usize).clamp(2, wz - 20);

    let mut gx = 12usize;
    let mut house_idx = 0u32;
    while gx + 40 < wx {
        let h = hmap[target_z][gx];
        if h < 30 {
            gx += 1;
            continue;
        }
        let var   = nhash(gx as i32 * 31 + seed as i32, house_idx as i32 * 13);
        let width = 20 + (var * 12.0) as usize;  // 20–32 wide (was 5–7, ×4)
        let depth = 32usize; // house depth + garden clearance (was 8, ×4)

        // Mark footprint + 4-voxel margin on all sides (was 1, ×4)
        let margin = 4usize;
        for fz in 0..depth + margin * 2 {
            for fx in 0..width + margin * 2 {
                let mx = gx.saturating_sub(margin) + fx;
                let mz = target_z.saturating_sub(margin) + fz;
                if mx < wx && mz < wz {
                    occupied[mz][mx] = true;
                }
            }
        }

        gx += width + 4;  // 4-voxel gap between houses (was 1)
        house_idx += 1;
    }
}

// ─── Terrace row ──────────────────────────────────────────────────────────────

fn stamp_terrace_row(
    world: &mut VoxelWorld,
    hmap: &[Vec<usize>],
    wx: usize, wy: usize, wz: usize,
    valley_centre_z: f32, valley_half: f32,
    north_side: bool,
    nz_band: f32,   // normalised Z distance from valley centre
    seed: u32,
) {
    let sign: f32 = if north_side { -1.0 } else { 1.0 };
    let terrace_z = ((valley_centre_z + sign * nz_band * valley_half) as usize).clamp(2, wz - 20);

    // March along X, stamping houses with 4-voxel gaps
    let mut gx = 12usize;
    let mut house_idx = 0u32;
    while gx + 40 < wx {
        let h = hmap[terrace_z][gx];
        if h < 30 || h + 60 >= wy {  // need headroom for walls (32) + pitched roof (12) + chimney (8) = 52 + margin
            gx += 1;
            continue;
        }
        let var = nhash(gx as i32 * 31 + seed as i32, house_idx as i32 * 13);
        let width = 20 + (var * 12.0) as usize; // 20–32 wide (was 5–7, ×4)
        let building = build_terrace_house(gx, h, terrace_z, width, north_side, seed ^ house_idx);
        stamp_building(world, &building);
        gx += width + 4;  // 4-voxel gap between houses
        house_idx += 1;
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

/// Per-chunk mesh builder.
///
/// Iterates all voxels in a 64×64×64 chunk and emits one quad per exposed face
/// (solid voxel with an Air neighbour on that side).  All quads for one chunk
/// are merged into a single macroquad `Mesh` so the whole chunk renders in one
/// GPU draw call.
///
/// Directional light (baked into vertex colours):
///   +Y top    ××1.00  (sky light)
///   ±X / ±Z   ××0.75  (side light)
///   -Y bottom ××0.50  (ambient only)

use macroquad::models::{Mesh, Vertex};
use macroquad::math::{Vec2, Vec3, Vec4};

use super::{VoxelWorld, Vox};
use super::vox::vox_color;

/// Scale: world units per voxel.  0.10 = 10 cm/voxel — enough for furniture detail.
pub const VS: f32 = 0.10;

// ─── Face definitions ─────────────────────────────────────────────────────────

/// 6 face directions: index maps to (dx, dy, dz, light_factor, quad_corners).
/// Corners are given in (local_x, local_y, local_z) offsets within [0,1]³.
struct FaceDef {
    dx: i32, dy: i32, dz: i32,
    light: f32,
    corners: [(f32, f32, f32); 4],  // CCW winding when viewed from outside
}

const FACES: [FaceDef; 6] = [
    // +Y top
    FaceDef { dx: 0, dy: 1, dz: 0, light: 1.00,
        corners: [(0.0,1.0,0.0),(1.0,1.0,0.0),(1.0,1.0,1.0),(0.0,1.0,1.0)] },
    // -Y bottom
    FaceDef { dx: 0, dy:-1, dz: 0, light: 0.50,
        corners: [(0.0,0.0,1.0),(1.0,0.0,1.0),(1.0,0.0,0.0),(0.0,0.0,0.0)] },
    // +X right
    FaceDef { dx: 1, dy: 0, dz: 0, light: 0.75,
        corners: [(1.0,0.0,0.0),(1.0,1.0,0.0),(1.0,1.0,1.0),(1.0,0.0,1.0)] },
    // -X left
    FaceDef { dx:-1, dy: 0, dz: 0, light: 0.65,
        corners: [(0.0,0.0,1.0),(0.0,1.0,1.0),(0.0,1.0,0.0),(0.0,0.0,0.0)] },
    // +Z back
    FaceDef { dx: 0, dy: 0, dz: 1, light: 0.70,
        corners: [(1.0,0.0,1.0),(1.0,1.0,1.0),(0.0,1.0,1.0),(0.0,0.0,1.0)] },
    // -Z front
    FaceDef { dx: 0, dy: 0, dz:-1, light: 0.80,
        corners: [(0.0,0.0,0.0),(0.0,1.0,0.0),(1.0,1.0,0.0),(1.0,0.0,0.0)] },
];

// ─── Mesh builder ─────────────────────────────────────────────────────────────

pub fn build_chunk_mesh(world: &VoxelWorld, cx: usize, cy: usize, cz: usize) -> Mesh {
    let mut vertices: Vec<Vertex> = Vec::with_capacity(4096);
    let mut indices:  Vec<u16>    = Vec::with_capacity(6144);

    world.chunk_iter(cx, cy, cz, |wx, wy, wz, v| {
        // Guard: stop adding faces once we approach the u16 index limit.
        // With draw_call_vertex_capacity=65536 and draw_call_index_capacity=98304,
        // a mesh with 60k verts generates 90k indices — safely under both limits.
        // Max index VALUE = 59999 < 65535 (u16 max). ✓
        if vertices.len() >= 60_000 { return; }

        if v.is_air() { return; }

        let base_col = vox_color(v);

        let ox = wx as f32 * VS;
        let oy = wy as f32 * VS;
        let oz = wz as f32 * VS;

        for face in &FACES {
            // Check whether the neighbouring voxel on this side is transparent
            let nx = wx as i32 + face.dx;
            let ny = wy as i32 + face.dy;
            let nz = wz as i32 + face.dz;
            let neighbour = world.get(nx, ny, nz);
            let neighbour_transparent = neighbour.is_air()
                || neighbour == Vox::Glass
                || neighbour == Vox::Water
                || neighbour == Vox::Ice;
            if !neighbour_transparent { continue; }

            // Emit quad
            let vi = vertices.len() as u16;
            for &(lx, ly, lz) in &face.corners {
                let r = (base_col.r * face.light).clamp(0.0, 1.0);
                let g = (base_col.g * face.light).clamp(0.0, 1.0);
                let b = (base_col.b * face.light).clamp(0.0, 1.0);
                vertices.push(Vertex {
                    position: Vec3::new(ox + lx * VS, oy + ly * VS, oz + lz * VS),
                    uv:       Vec2::ZERO,
                    color:    [(r * 255.0) as u8, (g * 255.0) as u8,
                               (b * 255.0) as u8, (base_col.a * 255.0) as u8],
                    normal:   Vec4::ZERO,
                });
            }
            // Two triangles: [0,1,2] and [0,2,3]
            indices.extend_from_slice(&[vi, vi+1, vi+2, vi, vi+2, vi+3]);
        }
    });

    Mesh { vertices, indices, texture: None }
}

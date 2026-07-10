/// High-performance chunk-based voxel renderer.
///
/// Holds one `Mesh` per chunk.  Meshes are built once on world load and only
/// rebuilt when a chunk is marked dirty.  Each frame: one `draw_mesh` call per
/// chunk = ~128–768 draw calls instead of 30 000+ individual cube calls.

use macroquad::models::{Mesh, draw_mesh};
use macroquad::math::Vec3;

use crate::voxel::{VoxelWorld, build_chunk_mesh};

pub struct ChunkRenderer {
    meshes: Vec<Mesh>,
    dirty:  Vec<bool>,
    /// Camera position, updated each frame for sorting.
    pub cam: Vec3,
}

impl ChunkRenderer {
    /// Build all chunk meshes from the world on construction.
    pub fn new(world: &VoxelWorld) -> Self {
        let total = world.total_chunks();
        let mut meshes = Vec::with_capacity(total);
        let dirty  = vec![false; total];

        for cz in 0..world.chunks_z() {
            for cy in 0..world.chunks_y() {
                for cx in 0..world.chunks_x() {
                    meshes.push(build_chunk_mesh(world, cx, cy, cz));
                }
            }
        }

        ChunkRenderer { meshes, dirty, cam: Vec3::ZERO }
    }

    /// Mark one chunk dirty (call after modifying voxels inside it).
    pub fn mark_dirty(&mut self, world: &VoxelWorld, cx: usize, cy: usize, cz: usize) {
        let idx = world.chunk_idx(cx, cy, cz);
        if idx < self.dirty.len() { self.dirty[idx] = true; }
    }

    /// Rebuild any dirty chunk meshes, then draw all chunks.
    pub fn draw(&mut self, world: &VoxelWorld) {
        // Rebuild dirty chunks
        for cz in 0..world.chunks_z() {
            for cy in 0..world.chunks_y() {
                for cx in 0..world.chunks_x() {
                    let idx = world.chunk_idx(cx, cy, cz);
                    if self.dirty[idx] {
                        self.meshes[idx] = build_chunk_mesh(world, cx, cy, cz);
                        self.dirty[idx]  = false;
                    }
                }
            }
        }

        // Sort chunks back-to-front by distance to camera
        // (improves alpha blending; chunks are few so sort is cheap)
        use crate::voxel::world::CHUNK;
        let cam = self.cam;
        let cx_n = world.chunks_x();
        let cy_n = world.chunks_y();
        let cz_n = world.chunks_z();
        let mut order: Vec<(f32, usize)> = (0..self.meshes.len()).map(|idx| {
            let ix = idx % cx_n;
            let iz = (idx / cx_n) % cz_n;
            let iy = idx / (cx_n * cz_n);
            let wx = (ix * CHUNK + CHUNK / 2) as f32 * crate::voxel::mesher::VS;
            let wy = (iy * CHUNK + CHUNK / 2) as f32 * crate::voxel::mesher::VS;
            let wz = (iz * CHUNK + CHUNK / 2) as f32 * crate::voxel::mesher::VS;
            let dx = wx - cam.x; let dy = wy - cam.y; let dz = wz - cam.z;
            let dist2 = dx*dx + dy*dy + dz*dz;
            (dist2, idx)
        }).collect();
        order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        for (_, idx) in &order {
            if !self.meshes[*idx].indices.is_empty() {
                draw_mesh(&self.meshes[*idx]);
                // Flush macroquad's geometry batcher after each chunk.
                // Without this, the 65535-vertex-per-frame accumulation buffer
                // overflows at ~100+ chunks, producing clamped/missing geometry.
                unsafe {
                    macroquad::window::get_internal_gl().flush();
                }
            }
        }
    }

    /// Total triangle count across all chunks (for HUD).
    pub fn triangle_count(&self) -> usize {
        self.meshes.iter().map(|m| m.indices.len() / 3).sum()
    }
}

/// Flat-array voxel world.
///
/// Dimensions are fixed at construction.  Data is stored `[z * WX * WY + y * WX + x]`
/// (X-major, Y second, Z outermost) for cache-friendly X iteration.
///
/// Chunk grid: chunks are 64×64×64 voxels.  The world dimensions should be
/// multiples of 64.

use super::Vox;

pub const CHUNK: usize = 64;

pub struct VoxelWorld {
    pub wx: usize,
    pub wy: usize,
    pub wz: usize,
    data: Vec<Vox>,
}

impl VoxelWorld {
    pub fn new(wx: usize, wy: usize, wz: usize) -> Self {
        VoxelWorld {
            wx, wy, wz,
            data: vec![Vox::Air; wx * wy * wz],
        }
    }

    #[inline]
    fn idx(&self, x: usize, y: usize, z: usize) -> usize {
        z * self.wx * self.wy + y * self.wx + x
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32, z: i32) -> Vox {
        if x < 0 || y < 0 || z < 0
            || x >= self.wx as i32
            || y >= self.wy as i32
            || z >= self.wz as i32
        {
            Vox::Air
        } else {
            self.data[self.idx(x as usize, y as usize, z as usize)]
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, v: Vox) {
        if x < self.wx && y < self.wy && z < self.wz {
            let i = self.idx(x, y, z);
            self.data[i] = v;
        }
    }

    /// Number of chunks along each axis.
    pub fn chunks_x(&self) -> usize { self.wx / CHUNK }
    pub fn chunks_y(&self) -> usize { self.wy / CHUNK }
    pub fn chunks_z(&self) -> usize { self.wz / CHUNK }

    pub fn total_chunks(&self) -> usize {
        self.chunks_x() * self.chunks_y() * self.chunks_z()
    }

    /// Chunk index from chunk-grid coordinates.
    pub fn chunk_idx(&self, cx: usize, cy: usize, cz: usize) -> usize {
        cz * self.chunks_x() * self.chunks_y() + cy * self.chunks_x() + cx
    }

    /// Iterate all voxels in a chunk (cx, cy, cz are chunk-grid coords).
    /// Calls `f(world_x, world_y, world_z, vox)` for each voxel.
    pub fn chunk_iter(
        &self,
        cx: usize, cy: usize, cz: usize,
        mut f: impl FnMut(usize, usize, usize, Vox),
    ) {
        let x0 = cx * CHUNK;
        let y0 = cy * CHUNK;
        let z0 = cz * CHUNK;
        for lz in 0..CHUNK {
            for ly in 0..CHUNK {
                for lx in 0..CHUNK {
                    let v = self.data[self.idx(x0 + lx, y0 + ly, z0 + lz)];
                    f(x0 + lx, y0 + ly, z0 + lz, v);
                }
            }
        }
    }
}

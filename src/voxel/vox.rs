/// Voxel material types + per-face holographic colours.

use macroquad::color::Color;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[repr(u8)]
pub enum Vox {
    #[default]
    Air = 0,
    // ── Terrain ──────────────────────────────────────────────────────────────
    Grass,      // valley-floor meadow / garden lawn
    Dirt,       // sub-surface layer
    Stone,      // deep rock / cliff face
    Slate,      // ridge-top / roof material (dark grey-blue)
    Gravel,     // river bed / path hardcore
    Sand,       // riverbank / estuarine flat
    Mud,        // valley-floor damp ground
    Heather,    // upper-slope moorland (short, purple-tinged)
    Bracken,    // mid-slope fern (taller, mid-green)
    Cobble,     // road/path surface
    // ── Water ────────────────────────────────────────────────────────────────
    Water,
    Ice,
    // ── Vegetation ───────────────────────────────────────────────────────────
    Log,        // tree trunk
    Leaf,       // tree canopy
    // ── Building fabric ──────────────────────────────────────────────────────
    Render,     // white lime-render wall (terraced houses)
    Brick,      // Victorian redbrick (school, chapel)
    Plank,      // wooden floor / ceiling
    Slate2,     // roof slate tile (slightly different shade to terrain Slate)
    Glass,      // window pane (very transparent)
    Door,       // wooden door (walkable gap in wall)
    // ── Furniture & props ─────────────────────────────────────────────────────
    BedFrame,   // iron bed frame
    Mattress,   // mattress / bedding
    Chair,
    TableTop,
    TableLeg,
    Shelf,
    Book,       // book-spine block on shelf
    Fireplace,  // stone surround
    Coal,       // glowing coals (orange-red)
    BarCounter,
    PubStool,
    Pew,        // chapel pew (dark wood)
}

impl Vox {
    #[inline] pub fn is_air(self) -> bool { self == Vox::Air }
    /// Transparent voxels: always draw fill quad so they glow.
    #[inline] pub fn always_fill(self) -> bool {
        matches!(self, Vox::Water | Vox::Ice | Vox::Glass | Vox::Coal)
    }
    /// Is this a solid opaque voxel (blocks view of interior)?
    #[inline] pub fn opaque(self) -> bool {
        !self.is_air() && !self.always_fill() && !matches!(self, Vox::Leaf | Vox::Heather | Vox::Bracken)
    }
}

// ─── Holographic colour palette ───────────────────────────────────────────────
//
// Returns `(fill_rgba, face_rgba)`.  `fill_rgba` is very low alpha (ghost glow).
// `face_rgba` is the solid face colour; alpha drives translucency.
// Top faces get ×1.0, side faces ×0.75, bottom faces ×0.55.

pub fn vox_color(v: Vox) -> Color {
    // Base face colour (top-face, full brightness, alpha = desired translucency)
    let c = |r: f32, g: f32, b: f32, a: f32| Color::new(r, g, b, a);
    match v {
        Vox::Air      => c(0.0, 0.0, 0.0, 0.0),
        // Terrain
        Vox::Grass    => c(0.12, 0.62, 0.28, 0.55),
        Vox::Dirt     => c(0.48, 0.34, 0.16, 0.55),
        Vox::Stone    => c(0.42, 0.52, 0.68, 0.65),
        Vox::Slate    => c(0.28, 0.32, 0.42, 0.72),
        Vox::Gravel   => c(0.50, 0.52, 0.58, 0.55),
        Vox::Sand     => c(0.82, 0.74, 0.42, 0.55),
        Vox::Mud      => c(0.35, 0.28, 0.18, 0.55),
        Vox::Heather  => c(0.62, 0.30, 0.68, 0.50),
        Vox::Bracken  => c(0.28, 0.58, 0.22, 0.48),
        Vox::Cobble   => c(0.48, 0.48, 0.52, 0.68),
        // Water
        Vox::Water    => c(0.18, 0.52, 0.88, 0.55),
        Vox::Ice      => c(0.70, 0.88, 1.00, 0.50),
        // Vegetation
        Vox::Log      => c(0.48, 0.32, 0.14, 0.72),
        Vox::Leaf     => c(0.18, 0.72, 0.32, 0.55),
        // Building
        Vox::Render   => c(0.92, 0.90, 0.86, 0.82),
        Vox::Brick    => c(0.72, 0.32, 0.22, 0.80),
        Vox::Plank    => c(0.62, 0.42, 0.22, 0.78),
        Vox::Slate2   => c(0.22, 0.28, 0.38, 0.85),
        Vox::Glass    => c(0.60, 0.80, 0.95, 0.28),
        Vox::Door     => c(0.38, 0.26, 0.14, 0.82),
        // Furniture
        Vox::BedFrame  => c(0.40, 0.40, 0.42, 0.88),
        Vox::Mattress  => c(0.88, 0.82, 0.72, 0.88),
        Vox::Chair     => c(0.45, 0.28, 0.14, 0.88),
        Vox::TableTop  => c(0.55, 0.38, 0.18, 0.88),
        Vox::TableLeg  => c(0.42, 0.28, 0.14, 0.88),
        Vox::Shelf     => c(0.52, 0.36, 0.18, 0.88),
        Vox::Book      => c(0.72, 0.22, 0.22, 0.90),
        Vox::Fireplace => c(0.38, 0.36, 0.34, 0.90),
        Vox::Coal      => c(1.00, 0.45, 0.10, 0.82),
        Vox::BarCounter=> c(0.52, 0.34, 0.16, 0.88),
        Vox::PubStool  => c(0.42, 0.26, 0.12, 0.88),
        Vox::Pew       => c(0.28, 0.20, 0.12, 0.88),
    }
}

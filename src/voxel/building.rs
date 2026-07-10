/// Component-based building grammar for the South Wales valley world.
///
/// Hierarchy:
///   Primitive components (Wall/Window/Door/Floor/Ceiling/Stairs/Roof)
///   → Room  (a named space bounded by components)
///   → Floor (one storey; multiple rooms packed together)
///   → Building (multiple floors + roof + facade)
///
/// `stamp_building()` walks the Building tree and writes voxels into the world.

use super::{VoxelWorld, Vox};

// ─── Primitive component types ────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub enum FacadeMaterial {
    Render,  // white/cream rendered brick (Vox::Render)
    Brick,   // red/brown engineering brick (Vox::Brick)
    Stone,   // local sandstone / limestone (Vox::Stone)
    Plank,   // timber framing (Vox::Plank)
}

#[derive(Clone, Debug, PartialEq)]
pub enum RoomRole {
    FrontParlour,
    Kitchen,
    BackScullery,
    Bedroom,
    Landing,
    PubBar,
    PubSnug,
    Chapel,
    Classroom,
    Cellar,
}

#[derive(Clone, Debug)]
pub enum BuildingKind {
    TerracedHouse,
    DetachedHouse,
    Pub,
    Chapel,
    School,
    Colliery,
}

#[derive(Clone, Debug)]
pub enum RoofKind {
    Slate,   // double-pitch, Vox::Slate2
    Flat,    // single layer, Vox::Slate2
    Gabled,  // pitched with gable ends
}

// ─── Component ───────────────────────────────────────────────────────────────

/// A single architectural element at an exact world position.
#[derive(Clone, Debug)]
pub struct Component {
    pub kind: ComponentKind,
    /// Origin voxel (bottom-left-front corner of the element)
    pub x: usize,
    pub y: usize,
    pub z: usize,
    /// Size
    pub w: usize,  // X extent
    pub h: usize,  // Y extent
    pub d: usize,  // Z extent
}

#[derive(Clone, Debug)]
pub enum ComponentKind {
    Wall(FacadeMaterial),
    Window,       // Glass opening in a wall
    Door,         // Air opening with door voxel at bottom
    Floor,        // Plank/Cobble horizontal slab
    Ceiling,      // Plank underside slab (same geometry as Floor above)
    Stairs { rise: usize }, // stepped ramp, one voxel per step in X
    RoofSlab,     // Slate2 cap layer
}

// ─── Room ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Room {
    pub role:       RoomRole,
    /// Voxel AABB of the interior space (walls are OUTSIDE this)
    pub x: usize, pub y: usize, pub z: usize,
    pub w: usize, pub h: usize, pub d: usize,
    pub components: Vec<Component>,
    pub furniture:  Vec<Furniture>,
}

// ─── Furniture ───────────────────────────────────────────────────────────────

/// A single piece of furniture at a world-space voxel position.
#[derive(Clone, Debug)]
pub struct Furniture {
    pub x: usize, pub y: usize, pub z: usize,
    pub voxels: Vec<(i32, i32, i32, Vox)>,  // offsets + material
}

impl Furniture {
    fn at(x: usize, y: usize, z: usize, v: Vec<(i32,i32,i32,Vox)>) -> Self {
        Self { x, y, z, voxels: v }
    }

    pub fn fireplace(x: usize, y: usize, z: usize) -> Self {
        // 6-wide stone surround, 5-tall, mantle shelf on top, coal grate at base
        let mut v = vec![];
        // Stone surround (side posts + lintel)
        for dy in 0..5i32 { v.push((0,dy,0,Vox::Stone)); v.push((5,dy,0,Vox::Stone)); }
        for dx in 0..6i32 { v.push((dx,5,0,Vox::Stone)); }  // lintel
        // Mantle shelf
        for dx in -1..7i32 { v.push((dx,6,0,Vox::TableTop)); }
        // Fireplace interior (warm brick glow)
        for dx in 1..5i32 { for dy in 0..4i32 { v.push((dx,dy,1,Vox::Fireplace)); } }
        // Coal grate at base
        v.push((2,0,0,Vox::Coal)); v.push((3,0,0,Vox::Coal));
        Self::at(x, y, z, v)
    }
    pub fn armchair(x: usize, y: usize, z: usize) -> Self {
        Self::at(x, y, z, vec![
            (0,0,0, Vox::Chair), (1,0,0, Vox::Chair),  // seat base
            (0,1,0, Vox::Chair), (1,1,0, Vox::Chair),  // seat cushion
            (0,2,0, Vox::Chair),                        // backrest
            (0,0,1, Vox::Chair), (1,0,1, Vox::Chair),  // arm depth
        ])
    }
    pub fn table_and_chairs(x: usize, y: usize, z: usize) -> Self {
        Self::at(x, y, z, vec![
            // Table top (4 wide × 6 long)
            (0,2,0,Vox::TableTop),(1,2,0,Vox::TableTop),(2,2,0,Vox::TableTop),(3,2,0,Vox::TableTop),
            (0,2,1,Vox::TableTop),(1,2,1,Vox::TableTop),(2,2,1,Vox::TableTop),(3,2,1,Vox::TableTop),
            (0,2,2,Vox::TableTop),(1,2,2,Vox::TableTop),(2,2,2,Vox::TableTop),(3,2,2,Vox::TableTop),
            (0,2,3,Vox::TableTop),(1,2,3,Vox::TableTop),(2,2,3,Vox::TableTop),(3,2,3,Vox::TableTop),
            // Table legs
            (0,0,0,Vox::TableLeg),(3,0,0,Vox::TableLeg),(0,0,3,Vox::TableLeg),(3,0,3,Vox::TableLeg),
            (0,1,0,Vox::TableLeg),(3,1,0,Vox::TableLeg),(0,1,3,Vox::TableLeg),(3,1,3,Vox::TableLeg),
            // Chairs
            (-2,1,1,Vox::Chair),(-2,2,1,Vox::Chair),(-2,1,2,Vox::Chair),
            (5,1,1,Vox::Chair),(5,2,1,Vox::Chair),(5,1,2,Vox::Chair),
        ])
    }
    pub fn bed(x: usize, y: usize, z: usize) -> Self {
        Self::at(x, y, z, vec![
            // Bed frame (6 wide × 10 long × 2 tall)
            (0,0,0,Vox::BedFrame),(1,0,0,Vox::BedFrame),(2,0,0,Vox::BedFrame),
            (3,0,0,Vox::BedFrame),(4,0,0,Vox::BedFrame),(5,0,0,Vox::BedFrame),
            (0,0,1,Vox::BedFrame),(5,0,1,Vox::BedFrame),
            (0,0,2,Vox::BedFrame),(5,0,2,Vox::BedFrame),
            (0,0,3,Vox::BedFrame),(5,0,3,Vox::BedFrame),
            (0,0,4,Vox::BedFrame),(5,0,4,Vox::BedFrame),
            (0,0,5,Vox::BedFrame),(5,0,5,Vox::BedFrame),
            (0,0,6,Vox::BedFrame),(5,0,6,Vox::BedFrame),
            (0,0,7,Vox::BedFrame),(5,0,7,Vox::BedFrame),
            (0,0,8,Vox::BedFrame),(5,0,8,Vox::BedFrame),
            (0,0,9,Vox::BedFrame),(1,0,9,Vox::BedFrame),(2,0,9,Vox::BedFrame),
            (3,0,9,Vox::BedFrame),(4,0,9,Vox::BedFrame),(5,0,9,Vox::BedFrame),
            // Mattress
            (1,1,0,Vox::Mattress),(2,1,0,Vox::Mattress),(3,1,0,Vox::Mattress),(4,1,0,Vox::Mattress),
            (1,1,1,Vox::Mattress),(2,1,1,Vox::Mattress),(3,1,1,Vox::Mattress),(4,1,1,Vox::Mattress),
            (1,1,2,Vox::Mattress),(2,1,2,Vox::Mattress),(3,1,2,Vox::Mattress),(4,1,2,Vox::Mattress),
            (1,1,3,Vox::Mattress),(2,1,3,Vox::Mattress),(3,1,3,Vox::Mattress),(4,1,3,Vox::Mattress),
            (1,1,4,Vox::Mattress),(2,1,4,Vox::Mattress),(3,1,4,Vox::Mattress),(4,1,4,Vox::Mattress),
            (1,1,5,Vox::Mattress),(2,1,5,Vox::Mattress),(3,1,5,Vox::Mattress),(4,1,5,Vox::Mattress),
            (1,1,6,Vox::Mattress),(2,1,6,Vox::Mattress),(3,1,6,Vox::Mattress),(4,1,6,Vox::Mattress),
            (1,1,7,Vox::Mattress),(2,1,7,Vox::Mattress),(3,1,7,Vox::Mattress),(4,1,7,Vox::Mattress),
        ])
    }
    pub fn bookshelf(x: usize, y: usize, z: usize) -> Self {
        Self::at(x, y, z, vec![
            // 4 wide × 8 tall bookshelf with book spines
            (0,0,0,Vox::Shelf),(1,0,0,Vox::Shelf),(2,0,0,Vox::Shelf),(3,0,0,Vox::Shelf),
            (0,2,0,Vox::Shelf),(1,2,0,Vox::Shelf),(2,2,0,Vox::Shelf),(3,2,0,Vox::Shelf),
            (0,4,0,Vox::Shelf),(1,4,0,Vox::Shelf),(2,4,0,Vox::Shelf),(3,4,0,Vox::Shelf),
            (0,6,0,Vox::Shelf),(1,6,0,Vox::Shelf),(2,6,0,Vox::Shelf),(3,6,0,Vox::Shelf),
            // Books between shelves
            (0,1,0,Vox::Book),(1,1,0,Vox::Book),(2,1,0,Vox::Book),(3,1,0,Vox::Book),
            (0,3,0,Vox::Book),(1,3,0,Vox::Book),(2,3,0,Vox::Book),(3,3,0,Vox::Book),
            (0,5,0,Vox::Book),(1,5,0,Vox::Book),(2,5,0,Vox::Book),(3,5,0,Vox::Book),
        ])
    }
    pub fn pub_bar(x: usize, y: usize, z: usize, len: usize) -> Self {
        let mut v = vec![];
        for i in 0..len as i32 {
            v.push((i, 0, 0, Vox::BarCounter));
            v.push((i, -1, 0, Vox::BarCounter));
        }
        for i in 0..len as i32 {
            v.push((i, 0, 1, Vox::PubStool));
        }
        Self::at(x, y, z, v)
    }
    pub fn pew_row(x: usize, y: usize, z: usize, len: usize) -> Self {
        let v = (0..len as i32).map(|i| (i, 0, 0, Vox::Pew)).collect();
        Self::at(x, y, z, v)
    }
}

// ─── Floor ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct BuildingFloor {
    pub level: u8,
    pub rooms: Vec<Room>,
}

// ─── Building ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct Building {
    pub kind:     BuildingKind,
    pub floors:   Vec<BuildingFloor>,
    pub roof:     RoofKind,
    pub facade:   FacadeMaterial,
    /// Bottom-left-front corner in world voxel coords
    pub ox: usize, pub oy: usize, pub oz: usize,
    /// Overall footprint
    pub width: usize, pub depth: usize, pub total_height: usize,
    /// Direction the front door faces: true = south (low Z face), false = north (high Z face)
    pub face_south: bool,
}

// ─── Builder: terraced house ──────────────────────────────────────────────────

/// Construct a `Building` value representing a 2-storey Welsh terraced house.
/// At VS=0.10 (10cm/voxel): width 24-32 ≈ 2.4–3.2 m, depth 24 ≈ 2.4 m, height 32 ≈ 3.2 m eave.
/// Interior per storey: two rooms separated by a party wall — front parlour + back kitchen/bedroom.
/// Does NOT touch the `VoxelWorld` — call `stamp_building` separately.
pub fn build_terrace_house(
    ox: usize, oy: usize, oz: usize,
    width: usize,
    face_south: bool,
    seed: u32,
) -> Building {
    // ── Dimensions (4× the old scale) ────────────────────────────────────
    let height      = 32usize;  // wall height to eave (2 storeys of 16 each)
    let depth       = 24usize;  // front-to-back depth
    let mid         = height / 2;  // = 16, floor divider between storeys
    let wall        = 1usize;      // wall thickness

    let face_z_off  = if face_south { 0usize } else { depth - 1 };
    let back_z_off  = if face_south { depth - 1 } else { 0usize };
    let back_z_mid  = if face_south { depth / 2 + 1 } else { depth / 2 - 1 };

    let _ = seed; // reserved for per-house variation

    // ── Ground floor — front parlour ─────────────────────────────────────
    let gf_y  = oy + wall;
    let fp_x  = ox + width / 2 - 3;     // fireplace centred on back wall
    let fp_z  = oz + back_z_off;
    let fr_z  = if face_south { oz + 3 } else { oz + depth - 6 };

    let mut front_room = Room {
        role: RoomRole::FrontParlour,
        x: ox + wall, y: gf_y, z: oz + wall,
        w: width - wall * 2, h: mid - wall, d: depth / 2 - wall,
        components: vec![],
        furniture: vec![
            Furniture::fireplace(fp_x, gf_y, fp_z),
            Furniture::armchair(fp_x.saturating_sub(4), gf_y, fp_z),
            Furniture::armchair(fp_x + 8, gf_y, fp_z),
            Furniture::table_and_chairs(ox + 4, gf_y, fr_z),
        ],
    };
    // Front door (3 wide × 8 tall, centred)
    let door_x = ox + width / 2 - 1;
    front_room.components.push(Component { kind: ComponentKind::Door,
        x: door_x, y: oy + wall, z: oz + face_z_off, w: 3, h: 8, d: 1 });
    // Two sash windows either side of door
    if width > 8 {
        front_room.components.push(Component { kind: ComponentKind::Window,
            x: ox + 2, y: oy + 3, z: oz + face_z_off, w: 4, h: 6, d: 1 });
        front_room.components.push(Component { kind: ComponentKind::Window,
            x: ox + width - 6, y: oy + 3, z: oz + face_z_off, w: 4, h: 6, d: 1 });
    }

    // ── Ground floor — back kitchen/scullery ──────────────────────────────
    let back_room = Room {
        role: RoomRole::Kitchen,
        x: ox + wall, y: gf_y, z: oz + depth / 2,
        w: width - wall * 2, h: mid - wall, d: depth / 2 - wall,
        components: vec![],
        furniture: vec![
            Furniture::table_and_chairs(ox + 3, gf_y, oz + back_z_mid),
        ],
    };

    // ── First-floor front bedroom ─────────────────────────────────────────
    let ff_y  = oy + mid + wall;
    let bed_z = oz + back_z_off;
    let bs_x  = ox + width.saturating_sub(6);

    let mut ff_bedroom = Room {
        role: RoomRole::Bedroom,
        x: ox + wall, y: ff_y, z: oz + wall,
        w: width - wall * 2, h: mid - wall, d: depth / 2 - wall,
        components: vec![],
        furniture: vec![
            Furniture::bed(ox + 2, ff_y, bed_z),
            Furniture::bookshelf(bs_x, ff_y, bed_z),
        ],
    };
    // First-floor windows
    ff_bedroom.components.push(Component { kind: ComponentKind::Window,
        x: ox + 2, y: oy + mid + 3, z: oz + face_z_off, w: 4, h: 6, d: 1 });
    if width > 12 {
        ff_bedroom.components.push(Component { kind: ComponentKind::Window,
            x: ox + width - 6, y: oy + mid + 3, z: oz + face_z_off, w: 4, h: 6, d: 1 });
    }

    // ── First-floor back bedroom ──────────────────────────────────────────
    let ff_back_bedroom = Room {
        role: RoomRole::Bedroom,
        x: ox + wall, y: ff_y, z: oz + depth / 2,
        w: width - wall * 2, h: mid - wall, d: depth / 2 - wall,
        components: vec![],
        furniture: vec![
            Furniture::bed(ox + 2, ff_y, oz + back_z_off),
        ],
    };

    Building {
        kind:         BuildingKind::TerracedHouse,
        floors:       vec![
            BuildingFloor { level: 0, rooms: vec![front_room, back_room] },
            BuildingFloor { level: 1, rooms: vec![ff_bedroom, ff_back_bedroom] },
        ],
        roof:         RoofKind::Slate,
        facade:       FacadeMaterial::Render,
        ox, oy, oz,
        width, depth,
        total_height: height,  // wall height to eave
        face_south,
    }
}

// ─── Stamp: write a Building into the VoxelWorld ──────────────────────────────

pub fn stamp_building(world: &mut VoxelWorld, b: &Building) {
    let Building { ox, oy, oz, width, depth, total_height: height, .. } = *b;
    // `height` = wall height to eave line; roof is built on top separately
    let mid = height / 2;

    let wall_vox = match &b.facade {
        FacadeMaterial::Render => Vox::Render,
        FacadeMaterial::Brick  => Vox::Brick,
        FacadeMaterial::Stone  => Vox::Stone,
        FacadeMaterial::Plank  => Vox::Plank,
    };

    // 1. Flatten ground beneath footprint
    for fx in 0..width {
        for fz in 0..depth {
            for fy in oy.saturating_sub(2)..oy {
                world.set(ox + fx, fy, oz + fz, Vox::Stone);
            }
        }
    }

    // 2. Outer shell — walls up to eave line only
    for fy in 0..height {
        for fz in 0..depth {
            for fx in 0..width {
                let is_wall = fx == 0 || fx == width - 1 || fz == 0 || fz == depth - 1;
                let v = if is_wall { wall_vox } else if fy == 0 { Vox::Plank } else { Vox::Air };
                world.set(ox + fx, oy + fy, oz + fz, v);
            }
        }
    }

    // 3. Pitched roof (ridge runs along X, slopes down toward Z faces)
    //    z_dist = 0 at front/back walls, increases to depth/2 at the ridge
    for fx in 0..width {
        for fz in 0..depth {
            let z_dist = fz.min(depth - 1 - fz);  // 0 at edges, max at centre
            let is_gable = fx == 0 || fx == width - 1;
            for ph in 0..=z_dist {
                // Gable end walls get wall material; pitched faces get slate
                world.set(ox + fx, oy + height + ph, oz + fz,
                    if is_gable { wall_vox } else { Vox::Slate2 });
            }
        }
    }

    // 4. Chimney stacks (brick towers rising from the ridge)
    let ridge_h = depth / 2;  // height of ridge above eave
    let cz = oz + depth / 2;  // chimney Z sits at the ridge
    if oy + height + ridge_h + 6 < world.wy {
        for &cf in &[width / 3, (2 * width) / 3] {
            for ch in 0..8usize {  // 8 vox chimney pot above ridge (was 4)
                world.set(ox + cf, oy + height + ridge_h + ch, cz, Vox::Brick);
            }
        }
    }

    // 5. Mid-storey floor slab (at oy+mid, solid plank)
    for fx in 0..width {
        for fz in 0..depth {
            world.set(ox + fx, oy + mid, oz + fz, Vox::Plank);
        }
    }

    // 6. Simple staircase — rises over one-quarter of depth on the right side
    let stair_x = ox + width - 3;
    for step in 0..(mid - 1) {
        let sz = oz + depth - 2 - (step * depth / mid).min(depth - 3);
        if sz < oz + depth && stair_x < ox + width - 1 {
            world.set(stair_x, oy + step, sz, Vox::Plank);
        }
    }

    // 7. Stamp each room's components (windows + doors override walls)
    for floor in &b.floors {
        for room in &floor.rooms {
            for comp in &room.components {
                stamp_component(world, comp);
            }
        }
    }

    // 8. Stamp furniture
    for floor in &b.floors {
        for room in &floor.rooms {
            for furn in &room.furniture {
                stamp_furniture(world, furn);
            }
        }
    }
}

fn stamp_component(world: &mut VoxelWorld, c: &Component) {
    match &c.kind {
        ComponentKind::Window => {
            // Fill the full w×h opening with glass
            for dy in 0..c.h {
                for dx in 0..c.w {
                    world.set(c.x + dx, c.y + dy, c.z, Vox::Glass);
                }
            }
        }
        ComponentKind::Door => {
            // Carve a w×h air opening in the wall
            for dy in 0..c.h {
                for dx in 0..c.w {
                    world.set(c.x + dx, c.y + dy, c.z, Vox::Air);
                }
            }
        }
        ComponentKind::Wall(mat) => {
            let v = match mat {
                FacadeMaterial::Render => Vox::Render,
                FacadeMaterial::Brick  => Vox::Brick,
                FacadeMaterial::Stone  => Vox::Stone,
                FacadeMaterial::Plank  => Vox::Plank,
            };
            for dy in 0..c.h {
                for dx in 0..c.w {
                    for dz in 0..c.d {
                        world.set(c.x + dx, c.y + dy, c.z + dz, v);
                    }
                }
            }
        }
        ComponentKind::Floor => {
            for dx in 0..c.w {
                for dz in 0..c.d {
                    world.set(c.x + dx, c.y, c.z + dz, Vox::Plank);
                }
            }
        }
        ComponentKind::Ceiling => {
            for dx in 0..c.w {
                for dz in 0..c.d {
                    world.set(c.x + dx, c.y, c.z + dz, Vox::Plank);
                }
            }
        }
        ComponentKind::Stairs { rise } => {
            for step in 0..*rise {
                world.set(c.x + step, c.y + step, c.z, Vox::Plank);
            }
        }
        ComponentKind::RoofSlab => {
            for dx in 0..c.w {
                for dz in 0..c.d {
                    world.set(c.x + dx, c.y, c.z + dz, Vox::Slate2);
                }
            }
        }
    }
}

fn stamp_furniture(world: &mut VoxelWorld, f: &Furniture) {
    for &(dx, dy, dz, ref v) in &f.voxels {
        let wx = f.x as i32 + dx;
        let wy = f.y as i32 + dy;
        let wz = f.z as i32 + dz;
        if wx >= 0 && wy >= 0 && wz >= 0
            && wx < world.wx as i32
            && wy < world.wy as i32
            && wz < world.wz as i32
        {
            world.set(wx as usize, wy as usize, wz as usize, v.clone());
        }
    }
}

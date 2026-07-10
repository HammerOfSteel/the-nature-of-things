# Asset Pack Knowledge
> Verified understanding of each pack — confirmed by viewing generated village scenes + walk GIFs.
> All packs extracted to `/tmp/asset_preview/` (re-run extraction if `/tmp` is cleared).
> Village previews + walk GIFs live in `tools/previews/`.

---

## Pack 1 — Sunnyside World V2 ⭐ RECOMMENDED

**Preview**: `sunnyside_v2_village.png` · `sunnyside_v2_walk.gif`
**Source**: `/tmp/asset_preview/sunnyside/` (zip extract)

### Perspective
TRUE TOP-DOWN — you see rooftops from directly above (flat roof surfaces, no front-face).

### Ground Tiles
- `Tileset.png`: 1024×1024, 16px grid = 64×64 tile grid. Rich autotile set.
- Grass, dirt paths, water, stone roads, sand — all present.

### Buildings
- Component-based tiles in the main tileset.
- **`Sunnyside_World_ExampleScene.png`** (4434×2470): Pre-assembled complete village with
  - Multi-story houses, windmill, dock, river, goblin characters placed in scene.
- Buildings require a tilemap editor (e.g. Tiled) to assemble from components.
- Building roofs visible from above (flat perspective).

### Characters
- `WALKING/` folder: **layered** sprites — `base_walk_strip8.png` + 6 hair variants.
- Frame: 96×64 px, 8-frame horizontal strip (total 768×64).
- System: render base → then hair layer on top → then tools/items layer.
- Also includes **goblin** sprites in same layered format.
- Multiple hair styles: BowlHair, Cap, Bun, Braid, etc.

### Animals / Props
- Goblins visible in ExampleScene. Full prop set in tileset.

### Notes
- Best pack for a true top-down game world.
- ExampleScene.png is invaluable — use it as a reference for tile placement.
- Walk GIF: smooth 8-frame cycle, characters face down by default (row 0 = south).

---

## Pack 2 — Little Dreamyland

**Preview**: `little_dreamyland_village.png` · `little_dreamyland_walk.gif`
**Source**: `/tmp/asset_preview/little_dreamyland/` (zip extract)

### Perspective
3/4 FRONT VIEW — buildings face the viewer (like LTTP / classic SNES RPG).

### Ground Tiles
- `Autotile_Grass_and_Dirt_Path_Tileset.png`: 336×176, 16px grid = 21×11 tiles.
- Interior flat grass: **col 3, row 0** (rgb ≈ 100,180,70).
- Dirt path interior: **col 16, row 1** (rgb ≈ 187,155,99).
- Autotile edges for transitions between grass and dirt.

### Buildings
- `House_Tileset.png`: 608×304, 16px — a MIX of:
  - **Assembled sprites** at specific positions (usable directly as sprites):
    - Red house front-view: `crop(0, 0, 48, 80)` ← 3×5 tiles, red tiled roof + wood walls + chimney post
    - Green house front-view: `crop(0, 160, 48, 224)` ← green leafy roof + wood walls + chimney post
  - Remaining area: individual roof component tiles for tilemap editor assembly.
- `Exterior_Tileset.png`: 400×176 — benches `(4×16, 5×16, 7×16, 7×16)`, wells `(7×16, 0, 10×16, 3×16)`, lamp posts, fences, market stalls, boats.

### Nature
- `Nature_Tileset.png`: 336×192, 16px.
  - Single round tree: `crop(0, 0, 32, 48)` ← 32×48
  - Double round tree cluster: `crop(96, 0, 160, 64)` ← 64×64
  - Also: dead trees, stumps, conifer trees, logs, mushrooms, rocks, water plants.

### Characters
- Bunny sprites in `Sprites/Characters/Bunny/` — multiple action folders:
  - `RUN/Bunny_Run.png`: 384×192 = 8 frames × 4 directions, 48×48 per frame
  - Also: IDLE, HOE, SWORD, etc.
- Direction order (rows): Down, Left, Right, Up.

### Animals
- `Chicken_Idle.png`, `Cow_Idle.png` — separate files.

### Notes
- The "bunny" is the player character — cute anthropomorphic rabbit.
- Houses show as 3/4 front view — nice for classic RPG feel.
- Rich exterior props set (best in the free packs for exterior village items).
- Walk GIF: 8-frame bunny run cycle. Characters are small (48×48).

---

## Pack 3 — Cute Fantasy Free

**Preview**: `cute_fantasy_village.png` · `cute_fantasy_walk.gif`
**Source**: `/tmp/asset_preview/cute_fantasy/` (zip extract)

### Perspective
3/4 FRONT VIEW — buildings face viewer, like LTTP.

### Ground Tiles
- Individual PNG files (NOT a tileset sheet):
  - `Grass_Middle.png`: 16×16 — flat grass fill
  - `Path_Middle.png`: 16×16 — dirt path fill
  - `Water_Middle.png`: 16×16 — water fill
  - `Water_Tile.png`: 48×96 — animated water (3 frames × 48×32)

### Buildings
- **Complete assembled sprites** (not component tiles):
  - `House_1_Wood_Base_Blue.png`: 96×128 — full front-view house, wood frame + blue roof
  - Similar variants exist (different colors/styles).
  - `Oak_Tree.png`: 64×80 — large single tree
  - `Oak_Tree_Small.png`: smaller variant

### Characters
- `Player.png`: 192×320 — 4 columns × variable rows, **48×48 per frame**.
  - Walk animation: **row 2** (y=96-143).
  - Row 0: idle? Row 1: idle facing? Row 2: walk down, Row 3: walk up? etc.
- Walk GIF: 4-frame walk cycle, cute pixel character.

### Animals
- Separate files: `Pig.png`, `Chicken.png`, `Sheep.png`, `Cow.png`
- Each ~32×32 or 48×48 sprites.

### Notes
- Simplest pack to work with — individual tile files = just use them directly.
- All house sprites are COMPLETE (no component assembly needed).
- Best for rapid prototyping — minimal code complexity.
- Cute warm art style, LTTP-inspired.

---

## Pack 4 — Farm RPG 16×16 Tiny

**Preview**: `farm_rpg_village.png` · `farm_rpg_walk.gif`
**Source**: `/tmp/asset_preview/farm_rpg/` (zip extract)

### Perspective
3/4 FRONT VIEW — buildings face viewer.

### Ground Tiles
- `Tileset Spring.png`: 192×320, 16px grid = 12×20 tiles.
- Interior grass: **col 8, row 0**.
- Path/road: `Road.png` separate sheet, col 2, row 1.

### Buildings
- `House.png`: 224×112 — contains TWO assembled house sprites:
  - **Left (x=0-80)**: Large barn/farmhouse with brown wood roof.
  - **Right (x=128-224)**: Smaller cottage with chimney, window, door.
  - Middle (x=80-128): Individual component tiles (windows, doors, chimney).
- `Maple Tree.png`: 160×48 — 5 tree variants × 32×48.

### Characters
- `Walk.png`: 192×96 — **16×24 per frame**, 12 frames per row.
  - Very small characters (16×24 = 1×1.5 tiles). Hard to see detail.
- `Idle.png`: similar format.
- Characters face LEFT in walk GIF — may need horizontal flip in code.

### Animals
- `Chicken Red.png`, `Male Cow Brown.png`, `Female Cow Brown.png`, `Baby Chicken Yellow.png`

### Notes
- The most "farm simulation" appropriate pack (Stardew Valley feel).
- Characters are VERY small (16×24) — will look tiny unless scaled up 3-4×.
- Two pre-assembled house sprites in one sheet = easy to use.
- Good farm animal variety.

---

## Pack 5 — Pixel 16 v2 Village

**Preview**: `pixel16_village_sheet.png` (overview only, no tileset)
**Source**: `/tmp/asset_preview/pixel16/` (zip extract — free version only)

### Perspective
3/4 FRONT VIEW — detailed Tudor/medieval style buildings.

### What's In The Free Pack
- **Preview sheet only** — no individual tile files exported.
- The preview sheet (282×276) shows:
  - **Cobblestone ground** with tree (top-down cobblestone + 3/4 tree = mixed perspective)
  - **Props**: fence, lamp post, bench, shop shelf, jug, flower pots
  - **Market stall**: blue/white striped awning with table
  - **House 1**: Red-orange tiled roof, Tudor wooden frame, arched door, windows
  - **House 2**: Blue tiled roof, red door, multiple windows, stone foundation

### What's Missing In Free Version
- No tileset PNG with individual tiles.
- No character sprites.
- No ground/grass/path tiles exported.

### Notes
- Most visually DETAILED and polished of all free packs.
- Houses are very realistic-looking for pixel art.
- NOT USABLE without purchasing the full pack — preview only.
- Best for visual reference of target art quality.

---

## Pack 6 — Shining Fields

**Preview**: `shining_fields_village.png` · `shining_fields_walk.gif`
**Source**: `/tmp/asset_preview/shining_fields/` (rar extract via unar)

### Perspective
MIXED — fields are top-down, walls are component tiles.

### What It Contains
- `Grass.png`: 16×16 solid green tile (uniform, no variation)
- `Soil.png`: 16×16 solid brown/tan tile
- `Fields.png`: 96×128 — wheat/crop autotile (animated crop rows)
- `Walls_01.png`: 96×64 — stone brick wall component tiles (NOT assembled buildings)
- NO assembled house sprites. Stone walls can be stacked to form a building outline
  but there's no roof imagery.

### Characters
- Run animation sheet: **48×32 per frame** (4 frames × 4 directions = 160×128 total?)
- Character: young girl/woman, brown hair, teal outfit.
- Walk GIF: smooth, cute character — actually the BEST looking character sprite
  of all the free packs in terms of detail.
- Only a RUN animation (no idle confirmed).

### Notes
- NOT suitable as a village pack — no houses, no trees, no exterior props.
- Excellent for **field/farmland areas** (wheat crops look great).
- Stone walls usable for dungeon/castle interiors.
- The character sprite (Shining Fields girl) is actually the most detailed of all packs.
- Would work well COMBINED with another pack for village vs farmland zones.

---

## Summary Comparison

| Pack | Perspective | Houses | Characters | Frame Size | Free/Complete |
|------|-------------|--------|------------|------------|---------------|
| Sunnyside V2 | True top-down | Component tiles | Layered strips | 96×64 | ✅ Complete |
| Little Dreamyland | 3/4 front | Assembled sprites + components | Bunny, 8-dir | 48×48 | ✅ Complete |
| Cute Fantasy | 3/4 front | Assembled sprites | Human player | 48×48 | ✅ Complete |
| Farm RPG | 3/4 front | Assembled sheet | Tiny human | 16×24 | ✅ Complete |
| Pixel 16 v2 | 3/4 front | Preview only | None in free | — | ⚠️ Preview only |
| Shining Fields | Top-down fields | None | Girl runner | 48×32 | ✅ But limited |

## Recommendation for "The Nature of Things"

**Primary choice: Sunnyside World V2** — only pack with true top-down camera, rich tileset,
complete village example, and layered character system. Best overall for a simulation game.

**Secondary / complement: Little Dreamyland** — if a 3/4 perspective is preferred (LTTP style),
Little Dreamyland has the richest exterior prop set and cute bunny characters that fit
a whimsical nature/valley theme.

**Farm areas: Shining Fields** — the wheat field tiles are perfect for farmland zones,
and the character sprite is beautiful. Could use this for field areas while using
Sunnyside V2 for the village.

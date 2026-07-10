#!/usr/bin/env python3
"""
Environment-Only Village Scenes — one per asset pack.
No characters. Pure scenery.
Output: tools/previews/env_*.png
Run:    python3 tools/village_env.py
"""
from PIL import Image, ImageDraw
import os

PACKS = "/tmp/asset_preview"
OUT   = os.path.join(os.path.dirname(__file__), "previews")
os.makedirs(OUT, exist_ok=True)

# ─── Helpers ──────────────────────────────────────────────────────────────────

def t(sheet, col, row, tw=16, th=16):
    return sheet.crop((col*tw, row*th, (col+1)*tw, (row+1)*th)).convert("RGBA")

def paste(canvas, img, cx, cy):
    canvas.paste(img, (cx - img.width//2, cy - img.height//2), img)

def fill(canvas, tile, x0=0, y0=0, x1=None, y1=None):
    x1 = canvas.width  if x1 is None else x1
    y1 = canvas.height if y1 is None else y1
    for y in range(y0, y1, tile.height):
        for x in range(x0, x1, tile.width):
            canvas.paste(tile, (x, y), tile)

def up(img, s=3):
    return img.resize((img.width*s, img.height*s), Image.NEAREST)

def save(img, name, scale=3):
    out = up(img, scale)
    if out.mode == "RGBA":
        bg = Image.new("RGB", out.size, (20, 20, 20))
        bg.paste(out, mask=out.split()[3])
        out = bg
    path = f"{OUT}/{name}"
    out.save(path)
    print(f"  {path}")

def bar(img, text):
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, img.width, 17], fill=(0, 0, 0, 200))
    d.text((5, 3), text, fill=(255, 255, 180))


# ══════════════════════════════════════════════════════════════════════════════
#  SUNNYSIDE WORLD V2  —  ExampleScene crop (already the best possible source)
# ══════════════════════════════════════════════════════════════════════════════
def env_sunnyside():
    print("\n── Sunnyside World V2 ──────────────────────────")
    base = (f"{PACKS}/sunnyside_v2/"
            "Sunnyside_World_ASSET_PACK_V2.1/Sunnyside_World_Assets")
    scene = Image.open(f"{base}/Sunnyside_World_ExampleScene.png")
    print(f"  ExampleScene: {scene.size}")

    # Wide crop showing the full village: buildings, river, windmill, forest
    crop = scene.crop((500, 200, 4050, 2300))   # 3550×2100
    out_w = 1600
    out = crop.resize((out_w, int(crop.height * out_w / crop.width)), Image.NEAREST)
    bar(out, ("Sunnyside World V2  |  TRUE TOP-DOWN  |  ExampleScene.png  |  "
              "buildings from above, river, windmill, goblins, animals, forest, market"))
    if out.mode == "RGBA":
        bg = Image.new("RGB", out.size, (0,0,0))
        bg.paste(out, mask=out.split()[3]); out = bg
    out.save(f"{OUT}/env_sunnyside.png")
    print(f"  {OUT}/env_sunnyside.png")


# ══════════════════════════════════════════════════════════════════════════════
#  LITTLE DREAMYLAND  —  rich custom village environment
# ══════════════════════════════════════════════════════════════════════════════
def env_dreamyland():
    print("\n── Little Dreamyland ───────────────────────────")
    base = f"{PACKS}/little_dreamyland/Little Dreamyland - Free Pack"

    ground = Image.open(
        f"{base}/Tileset/Autotile_Grass_and_Dirt_Path_Tileset.png").convert("RGBA")
    grass = t(ground, 3, 0)          # col 3, row 0 = interior flat grass (semi-transparent overlay)
    dirt  = t(ground, 16, 1)         # col 16, row 1 = interior dirt path
    water = Image.new("RGBA", (16, 16), (45, 112, 195, 255))

    house_ts = Image.open(f"{base}/Tileset/House_Tileset.png").convert("RGBA")
    red_house   = house_ts.crop((0,   0, 48,  80))   # 48×80
    green_house = house_ts.crop((0, 160, 48, 224))   # 48×64

    nat = Image.open(f"{base}/Tileset/Nature_Tileset.png").convert("RGBA")
    tree_s = nat.crop((0,  0,  32, 48))   # single round tree (32×48)
    tree_d = nat.crop((96, 0, 160, 64))   # double cluster (64×64)

    ext = Image.open(f"{base}/Tileset/Exterior_Tileset.png").convert("RGBA")
    # LD exterior is 400×176 = 25×11 tiles at 16px
    # Lamp post: cols 8-9, rows 0-4 (32×80 region with tall street lamp)
    lamp     = ext.crop((8*16, 0,    10*16, 5*16))   # 32×80
    # Market stall (teal awning): cols 6-7, rows 0-3
    stall    = ext.crop((6*16, 0,     8*16, 4*16))   # 32×64
    # White picket fence horizontal panel: cols 2-5, rows 0-2
    fence_h  = ext.crop((2*16, 0,     6*16, 2*16))   # 64×32 horizontal fence
    # Bench: cols 2-4, rows 3-4 (stone/wood bench area)
    bench    = ext.crop((2*16, 3*16,  5*16, 5*16))   # 48×32
    # Small round table: cols 0-1, rows 2-3
    table    = ext.crop((0,    2*16,  2*16, 4*16))   # 32×32
    # Chairs: cols 0-1, rows 0-1
    chairs   = ext.crop((0,    0,     2*16, 2*16))   # 32×32
    # Wooden box/crate area: cols 0-2, rows 4-5
    crates   = ext.crop((0,    4*16,  3*16, 6*16))   # 48×32
    # Barrel area: cols 11-13, rows 2-3
    barrels  = ext.crop((11*16, 2*16, 14*16, 4*16))  # 48×32

    print(f"  Lamp:{lamp.size} Stall:{stall.size} Bench:{bench.size} Fence:{fence_h.size}")

    # ── Canvas: 640×400 ───────────────────────────────────────────────────────
    W, H = 640, 400
    c = Image.new("RGBA", (W, H), (100, 178, 70, 255))   # solid green bg
    fill(c, grass)

    # Roads: horizontal at y=168..200, vertical at x=288..320
    fill(c, dirt, 0, 168, W, 200)
    fill(c, dirt, 288, 0, 320, H - 80)
    # Waterfront at bottom
    fill(c, water, 0, H - 80, W, H)

    # ── Buildings ──  (placed in 4 quadrants of the cross-road)
    paste(c, red_house,   140, 108)   # top-left
    paste(c, green_house, 520, 108)   # top-right
    paste(c, red_house,   140, 292)   # bottom-left
    paste(c, green_house, 520, 292)   # bottom-right
    # Extra house in center-top area
    paste(c, green_house, 304, 68)    # straddling the vertical path, set back

    # ── Trees ──
    paste(c, tree_d,  58,  56)    # corner cluster TL
    paste(c, tree_s, 200,  52)    # between TL house and center
    paste(c, tree_s, 380,  52)    # between center and TR
    paste(c, tree_s, 570,  56)    # corner TR
    paste(c, tree_d,  58, 300)    # corner cluster BL
    paste(c, tree_d, 556, 300)    # corner cluster BR
    paste(c, tree_s, 210, 248)    # bottom of vertical road, left
    paste(c, tree_s, 390, 248)    # bottom of vertical road, right

    # ── Props: lamps, benches, stalls along the road ──
    paste(c, lamp,   285, 120)    # lamp at top of vertical road, left edge
    paste(c, lamp,   318, 120)    # lamp at top of vertical road, right edge
    paste(c, lamp,   285, 240)    # lamp at bottom section
    paste(c, lamp,   318, 240)    # lamp at bottom section
    paste(c, bench,  160, 157)    # bench along north edge of H road, west
    paste(c, bench,  460, 157)    # bench along north edge of H road, east
    paste(c, stall,  420, 260)    # market stall bottom-right quad
    paste(c, stall,  190, 260)    # market stall bottom-left quad
    paste(c, table,  232, 260)    # table near stall
    paste(c, chairs, 232, 290)    # chairs near table
    paste(c, fence_h, 96, 148)   # fence along road edge west
    paste(c, fence_h, 480, 148)  # fence along road edge east
    paste(c, barrels, 500, 270)   # barrels near stall
    paste(c, crates,  170, 290)   # crates near stall

    bar(c, ("Little Dreamyland  |  3/4 front-view  |  "
            "red+green houses, cross-roads, lamps, benches, market stalls, trees, waterfront"))
    save(c, "env_dreamyland.png")


# ══════════════════════════════════════════════════════════════════════════════
#  CUTE FANTASY FREE  —  rich custom village environment
# ══════════════════════════════════════════════════════════════════════════════
def env_cutefantasy():
    print("\n── Cute Fantasy Free ───────────────────────────")
    base = f"{PACKS}/cute_fantasy/Cute_Fantasy_Free"

    grass_t  = Image.open(f"{base}/Tiles/Grass_Middle.png").convert("RGBA").crop((0,0,16,16))
    path_t   = Image.open(f"{base}/Tiles/Path_Middle.png").convert("RGBA").crop((0,0,16,16))
    water_t  = Image.open(f"{base}/Tiles/Water_Middle.png").convert("RGBA").crop((0,0,16,16))

    house   = Image.open(f"{base}/Outdoor decoration/House_1_Wood_Base_Blue.png").convert("RGBA")
    tree    = Image.open(f"{base}/Outdoor decoration/Oak_Tree.png").convert("RGBA")
    tree_s  = Image.open(f"{base}/Outdoor decoration/Oak_Tree_Small.png").convert("RGBA")
    bridge  = Image.open(f"{base}/Outdoor decoration/Bridge_Wood.png").convert("RGBA")
    fences  = Image.open(f"{base}/Outdoor decoration/Fences.png").convert("RGBA")
    decor   = Image.open(f"{base}/Outdoor decoration/Outdoor_Decor_Free.png").convert("RGBA")
    print(f"  House:{house.size} Tree:{tree.size} Bridge:{bridge.size} Fences:{fences.size}")
    print(f"  Decor sheet:{decor.size}")

    # Extract individual fence tiles from Fences.png (64×64 = 4×4 at 16px)
    fence_h  = fences.crop((0, 0, 16, 16))    # horizontal rail
    fence_v  = fences.crop((0, 16, 16, 32))   # vertical post

    # Flowers from decor sheet (112×192 = 7×12 at 16px)
    # Rows 8-11 are flower pots. Row 8 = y=128..143
    flower_r = decor.crop((0*16, 8*16, 1*16, 9*16))   # red flower
    flower_p = decor.crop((1*16, 8*16, 2*16, 9*16))   # pink flower
    flower_y = decor.crop((2*16, 8*16, 3*16, 9*16))   # yellow/orange flower
    flower_w = decor.crop((3*16, 8*16, 4*16, 9*16))   # white flower

    # Rocks: row 2-3, col 0 is a stump, cols 1-3 are rocks
    rock_s   = decor.crop((1*16, 2*16, 2*16, 3*16))   # small rock
    rock_l   = decor.crop((1*16, 3*16, 4*16, 5*16))   # large rock cluster (48×32)

    # Lamp post: tall lamp at col 5, rows 4-6 in decor sheet
    lamp_cf  = decor.crop((5*16, 4*16, 6*16, 7*16))   # 16×48 street lamp

    print(f"  Bridge:{bridge.size}  lamp:{lamp_cf.size}")

    # ── Canvas: 640×400 ───────────────────────────────────────────────────────
    W, H = 640, 400
    c = Image.new("RGBA", (W, H), (90, 175, 65, 255))
    fill(c, grass_t)

    # Roads: main H at y=172..192, secondary H at y=276..292
    fill(c, path_t, 0, 172, W, 192)
    fill(c, path_t, 0, 276, W, 292)
    # Vertical connector at x=288..304
    fill(c, path_t, 288, 0, 304, H - 32)
    # Water pond: bottom-left corner (128×112)
    fill(c, water_t, 0, H - 112, 144, H)

    # ── Bridge over the pond entrance ──
    paste(c, bridge, 72, H - 112)   # bridge crosses the pond edge

    # ── Houses ──  (house is 96×128)
    paste(c, house,  96,  86)    # top-left
    paste(c, house, 528,  86)    # top-right
    paste(c, house,  96, 234)    # bottom-left
    paste(c, house, 528, 234)    # bottom-right
    paste(c, house, 368,  86)    # upper-center

    # ── Trees ──
    paste(c, tree,   28,  44)    # far left
    paste(c, tree,  200,  44)    # between houses
    paste(c, tree_s, 300,  60)   # center top  
    paste(c, tree,  448,  44)    # between center and right
    paste(c, tree,  612,  44)    # far right
    paste(c, tree_s, 210, 228)   # between paths left
    paste(c, tree,  368, 234)    # center lower
    paste(c, tree_s, 490, 228)   # between paths right
    paste(c, tree,  200, 346)    # below second path
    paste(c, tree,  500, 346)    # below second path right
    paste(c, tree,  145, H-50)   # behind/beside pond

    # ── Fences (line along the main path top edge) ──
    for x in range(32, 240, 16):
        paste(c, fence_h, x, 162)   # fence along left section of path
    for x in range(320, 544, 16):
        paste(c, fence_h, x, 162)   # fence along right section

    # ── Flowers scattered around ──
    flowers = [flower_r, flower_p, flower_y, flower_w]
    positions = [
        (36, 168), (52, 168), (156, 168), (172, 168),   # left of path
        (456, 168), (472, 168), (572, 168), (588, 168), # right of path
        (240, 150), (256, 150), (272, 150),             # near gate center
        (36, 292), (52, 292), (160, 292), (176, 292),  # along second path
        (456, 292), (472, 292),
    ]
    for i, (px, py) in enumerate(positions):
        paste(c, flowers[i % len(flowers)], px, py)

    # ── Lamp posts at intersections ──
    paste(c, lamp_cf, 285, 130)
    paste(c, lamp_cf, 306, 130)
    paste(c, lamp_cf, 285, 245)
    paste(c, lamp_cf, 306, 245)

    # ── Rocks ──
    paste(c, rock_s, 155, 350)
    paste(c, rock_l, 260, 360)
    paste(c, rock_s, 400, 350)

    bar(c, ("Cute Fantasy Free  |  3/4 view  |  "
            "4×house, oak trees, wood bridge over pond, fences, flowers, lamp posts"))
    save(c, "env_cutefantasy.png")


# ══════════════════════════════════════════════════════════════════════════════
#  FARM RPG 16×16 TINY  —  farm environment with crops, fences, orchard
# ══════════════════════════════════════════════════════════════════════════════
def env_farmrpg():
    print("\n── Farm RPG 16×16 Tiny ─────────────────────────")
    base = f"{PACKS}/farm_rpg/Farm RPG FREE 16x16 - Tiny Asset Pack"

    ts    = Image.open(f"{base}/Tileset/Tileset Spring.png").convert("RGBA")
    grass = t(ts, 9, 2)    # confirmed bright green
    water = Image.new("RGBA", (16, 16), (50, 120, 195, 255))

    try:
        road_s = Image.open(f"{base}/Objects/Road copiar.png").convert("RGBA")
        path_t = t(road_s, 2, 1)
    except FileNotFoundError:
        path_t = Image.new("RGBA", (16, 16), (175, 140, 88, 255))

    house_s = Image.open(f"{base}/Objects/House.png").convert("RGBA")
    barn    = house_s.crop((0,   0,  80, 112))   # 80×112
    cottage = house_s.crop((128, 0, 224, 112))   # 96×112

    maple_s = Image.open(f"{base}/Objects/Maple Tree.png").convert("RGBA")
    maple_a = maple_s.crop(( 0, 0,  32, 48))
    maple_b = maple_s.crop((32, 0,  64, 48))
    maple_c = maple_s.crop((64, 0,  96, 48))   # larger variant
    maple_d = maple_s.crop((96, 0, 128, 48))   # biggest variant
    print(f"  Barn:{barn.size} Cottage:{cottage.size} Maple sheet:{maple_s.size}")

    # Crops: Spring Crops.png (224×128 = 14×8 at 16px)
    # Row 0 = strawberry growth stages. Col 5 = full-grown leafy plant (before fruit)
    # Col 6 = full grown with berries (harvest-ready, red)
    crops_s = Image.open(f"{base}/Objects/Spring Crops.png").convert("RGBA")
    crop_green   = crops_s.crop((5*16, 0, 6*16, 1*16))   # full grown leaves
    crop_ripe    = crops_s.crop((6*16, 0, 7*16, 1*16))   # ripe with berries
    crop_mid     = crops_s.crop((4*16, 0, 5*16, 1*16))   # mid-growth
    # Row 2 = different crop type
    crop2_ripe   = crops_s.crop((5*16, 2*16, 6*16, 3*16))
    print(f"  Crops sheet:{crops_s.size}")

    # Fence: "Fence's copiar.png" (48×80 = 3×5 tiles at 16px)
    fence_s  = Image.open(f"{base}/Objects/Fence's copiar.png").convert("RGBA")
    # Row 0: fence top/header pieces; Row 1: horizontal rails; Row 2: vertical posts
    fence_h   = fence_s.crop((0, 0, 16, 16))   # top-left fence corner/horizontal
    fence_mid = fence_s.crop((0, 16, 48, 32))  # middle horizontal rail (full width = 3 tiles)
    fence_v   = fence_s.crop((0, 32, 16, 48))  # vertical post piece
    print(f"  Fence sheet:{fence_s.size}")

    # Tileset: get a dirt/tilled soil tile for crop plots
    # Tileset Spring.png is 192×320 = 12×20 at 16px
    soil  = t(ts, 0, 6)    # try a soil/plowed tile (approximate)
    water_ts = t(ts, 0, 0)  # try water tile from tileset

    # ── Canvas: 640×400 ───────────────────────────────────────────────────────
    W, H = 640, 400
    c = Image.new("RGBA", (W, H), (95, 162, 60, 255))
    fill(c, grass)

    # Main farm road: horizontal at y=180..212 (2 tiles)
    fill(c, path_t, 0, 180, W, 212)
    # Side road: vertical at x=300..316 (1 tile)
    fill(c, path_t, 300, 0, 316, 180)
    fill(c, path_t, 300, 212, 316, H - 48)
    # Pond: bottom-right corner (128×80)
    fill(c, water, W - 128, H - 80, W, H)

    # ── Buildings ──
    paste(c, barn,     80,  96)    # barn: top-left
    paste(c, cottage,  548,  96)   # cottage: top-right
    paste(c, barn,     80, 296)    # barn: bottom-left
    paste(c, cottage,  548, 296)   # cottage: bottom-right

    # ── Orchard: rows of maple trees along the north ──
    tree_y = 36
    for i, (mx, mt) in enumerate([(200,maple_a),(232,maple_b),(264,maple_c),(432,maple_c),(464,maple_d),(496,maple_a)]):
        paste(c, mt, mx, tree_y)

    # ── Maple trees along south edge ──
    for mx, mt in [(200,maple_b),(264,maple_c),(432,maple_a),(496,maple_d)]:
        paste(c, mt, mx, 340)

    # ── Crop fields: tiled rows of growing crops ──
    # Left crop field: x=148..260, y=36..172 (between barn and orchard)
    FIELD_X0, FIELD_Y0 = 148, 60
    FIELD_X1, FIELD_Y1 = 268, 172
    for fy in range(FIELD_Y0, FIELD_Y1, 16):
        for fx in range(FIELD_X0, FIELD_X1, 16):
            # Alternate between green and ripe crops for variety
            crop_tile = crop_ripe if (fx + fy) % 32 == 0 else crop_green
            c.paste(crop_tile, (fx, fy), crop_tile)

    # Right crop field: x=352..480, y=60..172
    FIELD2_X0, FIELD2_Y0 = 352, 60
    FIELD2_X1, FIELD2_Y1 = 484, 172
    for fy in range(FIELD2_Y0, FIELD2_Y1, 16):
        for fx in range(FIELD2_X0, FIELD2_X1, 16):
            crop_tile = crop2_ripe if (fx + fy) % 32 == 0 else crop_mid
            c.paste(crop_tile, (fx, fy), crop_tile)

    # ── Fences around crop fields ──
    # Top fence of left field
    for fx in range(FIELD_X0, FIELD_X1, 16):
        c.paste(fence_h, (fx, FIELD_Y0 - 16), fence_h)
    # Top fence of right field
    for fx in range(FIELD2_X0, FIELD2_X1, 16):
        c.paste(fence_h, (fx, FIELD2_Y0 - 16), fence_h)

    # Some vertical fence posts at field corners
    for fy in range(FIELD_Y0, FIELD_Y1, 16):
        c.paste(fence_v, (FIELD_X0 - 16, fy), fence_v)
        c.paste(fence_v, (FIELD_X1, fy), fence_v)
    for fy in range(FIELD2_Y0, FIELD2_Y1, 16):
        c.paste(fence_v, (FIELD2_X0 - 16, fy), fence_v)
        c.paste(fence_v, (FIELD2_X1, fy), fence_v)

    # Fences along road edges south side
    for fx in range(0, W, 48):
        c.paste(fence_mid, (fx, H - 48 - 16), fence_mid)

    # ── Some trees near the pond and buildings ──
    paste(c, maple_a, 580, 340)
    paste(c, maple_d, 595, 310)

    bar(c, ("Farm RPG 16×16 Tiny  |  3/4 view  |  "
            "2×barn + 2×cottage, crop field, orchard, fences, water pond"))
    save(c, "env_farmrpg.png")


# ══════════════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print(f"Output: {OUT}")
    for fn in [env_sunnyside, env_dreamyland, env_cutefantasy, env_farmrpg]:
        try:
            fn()
        except Exception as e:
            import traceback
            print(f"  ERROR in {fn.__name__}: {e}")
            traceback.print_exc()
    print("\nDone →", OUT)

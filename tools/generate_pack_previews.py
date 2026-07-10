#!/usr/bin/env python3
"""
Asset Pack Village Preview Generator
Run: python3 tools/generate_pack_previews.py
Outputs: tools/previews/
"""
from PIL import Image, ImageDraw
import os

PACKS_BASE = "/tmp/asset_preview"
OUT_DIR = os.path.join(os.path.dirname(__file__), "previews")
os.makedirs(OUT_DIR, exist_ok=True)
SCALE = 3
CANVAS_W, CANVAS_H = 640, 400

def tile(sheet, col, row, tw=16, th=16):
    x, y = col * tw, row * th
    return sheet.crop((x, y, x+tw, y+th)).convert("RGBA")

def frame(strip, idx, fw, fh):
    return strip.crop((idx*fw, 0, (idx+1)*fw, fh)).convert("RGBA")

def tile_fill(canvas, t, x0=0, y0=0, x1=None, y1=None):
    x1 = x1 or canvas.width; y1 = y1 or canvas.height
    for ty in range(y0, y1, t.height):
        for tx in range(x0, x1, t.width):
            canvas.paste(t, (tx, ty), t)

def paste_c(canvas, sprite, cx, cy):
    canvas.paste(sprite, (cx-sprite.width//2, cy-sprite.height//2), sprite)

def upscale(img, s=SCALE):
    return img.resize((img.width*s, img.height*s), Image.NEAREST)

def add_label(img, text):
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, img.width, 18], fill=(0,0,0,200))
    d.text((4, 3), text, fill=(255,255,200,255))

def solid(color, size=(16,16)):
    return Image.new("RGBA", size, color)

def make_gif(frames, path, ms=130):
    def to_rgb(f):
        bg = Image.new("RGB", f.size, (80,160,70))
        bg.paste(f, mask=f)
        return bg
    scaled = [upscale(f) for f in frames]
    rgb = [to_rgb(f) for f in scaled]
    rgb[0].save(path, save_all=True, append_images=rgb[1:], loop=0, duration=ms)
    print(f"  GIF  {path}")

# ── Pack 1: Sunnyside World V2 ──────────────────────────────────────────────
def preview_sunnyside_v2():
    print("\n=== Sunnyside World V2 ===")
    base = f"{PACKS_BASE}/sunnyside_v2/Sunnyside_World_ASSET_PACK_V2.1/Sunnyside_World_Assets"
    scene = Image.open(f"{base}/Sunnyside_World_ExampleScene.png")
    # 4434×2470 — crop the main village area
    crop = scene.crop((1400, 800, 3000, 1800))
    s = 900/crop.width
    out = crop.resize((int(crop.width*s), int(crop.height*s)), Image.NEAREST)
    add_label(out, "Sunnyside World V2  |  ExampleScene.png village crop (red+blue roofed houses, river, windmill, goblins)")
    out.save(f"{OUT_DIR}/sunnyside_v2_village.png")
    print(f"  PNG  {OUT_DIR}/sunnyside_v2_village.png")

    # detail crop
    detail = scene.crop((1650, 1050, 2500, 1650))
    ds = 900/detail.width
    dout = detail.resize((int(detail.width*ds), int(detail.height*ds)), Image.NEAREST)
    add_label(dout, "Sunnyside V2  |  close-up: building+river detail (note blue tiled roof house)")
    dout.save(f"{OUT_DIR}/sunnyside_v2_detail.png")
    print(f"  PNG  {OUT_DIR}/sunnyside_v2_detail.png")

    # walk gif
    walk_base = Image.open(f"{base}/Characters/Human/WALKING/base_walk_strip8.png").convert("RGBA")
    walk_hair = Image.open(f"{base}/Characters/Human/WALKING/bowlhair_walk_strip8.png").convert("RGBA")
    fw, fh = walk_base.width//8, walk_base.height
    chars = []
    for i in range(8):
        b = frame(walk_base, i, fw, fh)
        h = frame(walk_hair, i, fw, fh)
        c = b.copy(); c.paste(h, (0,0), h); chars.append(c)
    bg = Image.new("RGBA", (fw*12, fh+16), (95,193,76,255))
    gfs = []
    for fi in range(8):
        fc = bg.copy()
        for ci in range(3):
            x = (fi*20 + ci*120) % (bg.width+fw) - fw
            paste_c(fc, chars[fi], x, fh//2+8)
        gfs.append(fc)
    make_gif(gfs, f"{OUT_DIR}/sunnyside_v2_walk.gif")
    print(f"  Tileset: 16px, 64×64 tile grid | char frame: {fw}x{fh} | layered body+hair")

# ── Pack 2: Little Dreamyland ────────────────────────────────────────────────
def preview_little_dreamyland():
    print("\n=== Little Dreamyland ===")
    base = f"{PACKS_BASE}/little_dreamyland/Little Dreamyland - Free Pack"
    tw = 16
    grass_ts = Image.open(f"{base}/Tileset/Autotile_Grass_and_Dirt_Path_Tileset.png").convert("RGBA")
    # flat interior grass: col 3, row 0 (confirmed solid green)
    grass_t = tile(grass_ts, 3, 0)
    # dirt path interior: col 16, row 1 (confirmed tan/brown rgb≈187,155,99)
    dirt_t  = tile(grass_ts, 16, 1)
    water_t = solid((55,130,200,255))

    house_ts = Image.open(f"{base}/Tileset/House_Tileset.png").convert("RGBA")
    # Red house: assembled front-view sprite at top-left (0,0,48,80) — 3×5 tiles
    red_house   = house_ts.crop((0,   0, 48,  80))
    # Green house: assembled front-view sprite starting at y=160, trimmed clean
    green_house = house_ts.crop((0, 160, 48, 224))

    ext_ts = Image.open(f"{base}/Tileset/Exterior_Tileset.png").convert("RGBA")
    bench = ext_ts.crop((4*16, 5*16, 7*16, 7*16))
    well  = ext_ts.crop((7*16, 0,    10*16, 3*16))

    nat_ts = Image.open(f"{base}/Tileset/Nature_Tileset.png").convert("RGBA")
    tree_a = nat_ts.crop((0,   0,  32, 48))   # single round tree
    tree_b = nat_ts.crop((96,  0, 160, 64))   # double round tree cluster

    bunny = Image.open(f"{base}/Sprites/Characters/Bunny/RUN/Bunny_Run.png").convert("RGBA")
    bfw, bfh = bunny.width//8, bunny.height//4
    chars = [frame(bunny.crop((0, 0, bunny.width, bfh)), i, bfw, bfh) for i in range(8)]
    print(f"  Bunny frame: {bfw}x{bfh}")

    W, H = CANVAS_W, CANVAS_H
    canvas = Image.new("RGBA", (W,H), (100,180,70,255))
    tile_fill(canvas, grass_t)
    path_y = H//2 - tw
    tile_fill(canvas, dirt_t, 0, path_y, W, path_y + tw*3)
    tile_fill(canvas, water_t, 0, H-tw*4)
    paste_c(canvas, red_house,   120, path_y - red_house.height//2 - 4)
    paste_c(canvas, green_house, 430, path_y - green_house.height//2 - 4)
    paste_c(canvas, tree_a,  60,  path_y-85)
    paste_c(canvas, tree_b,  260, path_y-75)
    paste_c(canvas, tree_a,  540, path_y-85)
    paste_c(canvas, bench,   320, path_y+tw*4)
    paste_c(canvas, well,    510, path_y+tw*3)
    add_label(canvas, "Little Dreamyland  |  16px autotile grass+dirt, component roof tiles, bunny characters, exterior props")
    upscale(canvas).save(f"{OUT_DIR}/little_dreamyland_village.png")
    print(f"  PNG  {OUT_DIR}/little_dreamyland_village.png")

    bg = Image.new("RGBA", (bfw*12, bfh+16), (100,180,70,255))
    gfs = []
    for fi in range(8):
        fc = bg.copy()
        for ci in range(3):
            x = (fi*18 + ci*100) % (bg.width+bfw) - bfw
            paste_c(fc, chars[fi], x, bfh//2+8)
        gfs.append(fc)
    make_gif(gfs, f"{OUT_DIR}/little_dreamyland_walk.gif")

# ── Pack 3: Cute Fantasy ─────────────────────────────────────────────────────
def preview_cute_fantasy():
    print("\n=== Cute Fantasy ===")
    base = f"{PACKS_BASE}/cute_fantasy/Cute_Fantasy_Free"
    tw = 16
    grass_t = Image.open(f"{base}/Tiles/Grass_Middle.png").convert("RGBA")
    path_t  = Image.open(f"{base}/Tiles/Path_Middle.png").convert("RGBA")
    water_t = Image.open(f"{base}/Tiles/Water_Middle.png").convert("RGBA").crop((0,0,tw,tw))
    house   = Image.open(f"{base}/Outdoor decoration/House_1_Wood_Base_Blue.png").convert("RGBA")
    tree    = Image.open(f"{base}/Outdoor decoration/Oak_Tree.png").convert("RGBA")
    tree_s  = Image.open(f"{base}/Outdoor decoration/Oak_Tree_Small.png").convert("RGBA")
    pig     = Image.open(f"{base}/Animals/Pig/Pig.png").convert("RGBA")
    chicken = Image.open(f"{base}/Animals/Chicken/Chicken.png").convert("RGBA")

    player = Image.open(f"{base}/Player/Player.png").convert("RGBA")
    fw_p = player.width//4
    fh_p = fw_p
    walk_row = min(2, player.height//fh_p - 1)
    chars = [frame(player.crop((0, walk_row*fh_p, player.width, (walk_row+1)*fh_p)), i, fw_p, fh_p) for i in range(4)]
    print(f"  House: {house.size}  Player frame: {fw_p}x{fh_p}")

    W, H = CANVAS_W, CANVAS_H
    canvas = Image.new("RGBA", (W,H), (100,180,70,255))
    tile_fill(canvas, grass_t)
    path_y = H//2 - tw
    tile_fill(canvas, path_t, 0, path_y, W, path_y+tw*3)
    tile_fill(canvas, water_t, 0, H-tw*4)
    paste_c(canvas, house,  110, path_y-house.height//2-12)
    paste_c(canvas, house,  440, path_y-house.height//2-12)
    paste_c(canvas, tree,    60, path_y-65)
    paste_c(canvas, tree_s, 270, path_y-42)
    paste_c(canvas, tree,   590, path_y-65)
    paste_c(canvas, pig,    190, path_y-25)
    paste_c(canvas, chicken,370, path_y-22)
    add_label(canvas, "Cute Fantasy Free  |  16px INDIVIDUAL tile PNGs + assembled house/tree sprites + animals")
    upscale(canvas).save(f"{OUT_DIR}/cute_fantasy_village.png")
    print(f"  PNG  {OUT_DIR}/cute_fantasy_village.png")

    bg = Image.new("RGBA", (fw_p*12, fh_p+16), (100,180,70,255))
    gfs = []
    for fi in range(4):
        fc = bg.copy()
        for ci in range(4):
            x = (fi*22+ci*80)%(bg.width+fw_p)-fw_p
            paste_c(fc, chars[fi], x, fh_p//2+8)
        gfs.append(fc)
    make_gif(gfs, f"{OUT_DIR}/cute_fantasy_walk.gif")

# ── Pack 4: Farm RPG 16×16 ───────────────────────────────────────────────────
def preview_farm_rpg():
    print("\n=== Farm RPG 16x16 ===")
    base = f"{PACKS_BASE}/farm_rpg/Farm RPG FREE 16x16 - Tiny Asset Pack"
    tw = 16
    ts     = Image.open(f"{base}/Tileset/Tileset Spring.png").convert("RGBA")
    grass_t = tile(ts, 8, 0)
    road   = Image.open(f"{base}/Objects/Road copiar.png").convert("RGBA")
    path_t = tile(road, 2, 1)
    water_t = solid((55,130,200,255))
    house  = Image.open(f"{base}/Objects/House.png").convert("RGBA")
    trees  = Image.open(f"{base}/Objects/Maple Tree.png").convert("RGBA")
    maple  = trees.crop((0,0,32,48))
    chicken = Image.open(f"{base}/Farm Animals/Chicken Red.png").convert("RGBA")
    cow    = Image.open(f"{base}/Farm Animals/Male Cow Brown.png").convert("RGBA")

    walk_sheet = Image.open(f"{base}/Character/Walk.png").convert("RGBA")
    fw_c, fh_c = 16, 24
    chars = [frame(walk_sheet.crop((0,0,walk_sheet.width,fh_c)), i, fw_c, fh_c) for i in range(walk_sheet.width//fw_c)]
    print(f"  House: {house.size}  Walk frame: {fw_c}x{fh_c}")

    W, H = CANVAS_W, CANVAS_H
    canvas = Image.new("RGBA", (W,H), (100,180,70,255))
    tile_fill(canvas, grass_t)
    path_y = H//2-tw
    tile_fill(canvas, path_t, 0, path_y, W, path_y+tw*3)
    tile_fill(canvas, water_t, 0, H-tw*4)
    paste_c(canvas, house,  100, path_y-house.height//2-8)
    paste_c(canvas, house,  390, path_y-house.height//2-8)
    paste_c(canvas, maple,   50, path_y-65)
    paste_c(canvas, maple,  250, path_y-65)
    paste_c(canvas, maple,  550, path_y-65)
    paste_c(canvas, chicken,200, path_y-20)
    paste_c(canvas, cow,    460, path_y-30)
    add_label(canvas, "Farm RPG 16x16 Tiny  |  tileset ground, assembled house/tree sprites, farm animals")
    upscale(canvas).save(f"{OUT_DIR}/farm_rpg_village.png")
    print(f"  PNG  {OUT_DIR}/farm_rpg_village.png")
    make_gif(chars[:8], f"{OUT_DIR}/farm_rpg_walk.gif")

# ── Pack 5: Pixel 16 v2 Village ──────────────────────────────────────────────
def preview_pixel16():
    print("\n=== Pixel 16 v2 Village ===")
    base = f"{PACKS_BASE}/pixel16_village/Pixel 16 v2 village free"
    sheet = Image.open(f"{base}/Pixel 16 v2 village free.png").convert("RGBA")
    img = upscale(sheet, 4)
    add_label(img, "Pixel 16 v2  |  Preview sheet only — no tileset ground in free pack (houses, market stall, props)")
    img.save(f"{OUT_DIR}/pixel16_village_sheet.png")
    print(f"  PNG  {OUT_DIR}/pixel16_village_sheet.png")

# ── Pack 6: Shining Fields ───────────────────────────────────────────────────
def preview_shining_fields():
    print("\n=== Shining Fields ===")
    base = f"{PACKS_BASE}/shining_fields/Shining Fields_free_v1.1/Sprites"
    tw = 16
    grass_t = Image.open(f"{base}/Tileset/Grass.png").convert("RGBA")
    try:
        wpath = Image.open(f"{base}/Tileset/Wooden path.png").convert("RGBA")
        path_t = wpath.crop((0,0,tw,tw))
    except:
        path_t = Image.open(f"{base}/Tileset/Soil.png").convert("RGBA")
    water_t = solid((55,130,200,255))

    wall = Image.open(f"{base}/Tileset/Walls/Walls_01.png").convert("RGBA")
    wt = wall.crop((0,0,tw,tw))
    house = Image.new("RGBA", (tw*5, tw*4), (0,0,0,0))
    for r in range(4):
        for c in range(5):
            house.paste(wt, (c*tw, r*tw), wt)

    run = Image.open(f"{base}/Characters/Player/run animation Sheet.png").convert("RGBA")
    rw, rh = run.size
    fw_c, fh_c = rw//4, rh//4
    chars = [frame(run.crop((0,0,rw,fh_c)), i, fw_c, fh_c) for i in range(4)]
    print(f"  Run frame: {fw_c}x{fh_c}")

    W, H = CANVAS_W, CANVAS_H
    canvas = Image.new("RGBA", (W,H), (100,180,70,255))
    tile_fill(canvas, grass_t)
    path_y = H//2-tw
    tile_fill(canvas, path_t, 0, path_y, W, path_y+tw*3)
    tile_fill(canvas, water_t, 0, H-tw*4)
    paste_c(canvas, house, 140, path_y-house.height//2-4)
    paste_c(canvas, house, 460, path_y-house.height//2-4)
    add_label(canvas, "Shining Fields  |  Outdoor field tiles + stone wall buildings (field/dungeon focus, not village)")
    upscale(canvas).save(f"{OUT_DIR}/shining_fields_village.png")
    print(f"  PNG  {OUT_DIR}/shining_fields_village.png")
    make_gif(chars, f"{OUT_DIR}/shining_fields_walk.gif")

# ── Tileset overviews ─────────────────────────────────────────────────────────
def save_overviews():
    items = [
        ("Sunnyside_V2_tileset_16px", f"{PACKS_BASE}/sunnyside_v2/Sunnyside_World_ASSET_PACK_V2.1/Sunnyside_World_Assets/Tileset/spr_tileset_sunnysideworld_16px.png", 2),
        ("LittleDreamyland_house_tileset", f"{PACKS_BASE}/little_dreamyland/Little Dreamyland - Free Pack/Tileset/House_Tileset.png", 3),
        ("LittleDreamyland_ground_autotile", f"{PACKS_BASE}/little_dreamyland/Little Dreamyland - Free Pack/Tileset/Autotile_Grass_and_Dirt_Path_Tileset.png", 4),
        ("LittleDreamyland_exterior_items", f"{PACKS_BASE}/little_dreamyland/Little Dreamyland - Free Pack/Tileset/Exterior_Tileset.png", 3),
        ("FarmRPG_tileset_spring", f"{PACKS_BASE}/farm_rpg/Farm RPG FREE 16x16 - Tiny Asset Pack/Tileset/Tileset Spring.png", 4),
        ("Pixel16_village_sheet", f"{PACKS_BASE}/pixel16_village/Pixel 16 v2 village free/Pixel 16 v2 village free.png", 4),
        ("ShiningFields_grass_soil_tileset", f"{PACKS_BASE}/shining_fields/Shining Fields_free_v1.1/Sprites/Tileset/Fields.png", 6),
    ]
    for name, path, s in items:
        try:
            img = Image.open(path).convert("RGBA")
            out = img.resize((img.width*s, img.height*s), Image.NEAREST)
            add_label(out, name.replace("_"," "))
            out.save(f"{OUT_DIR}/{name}_overview.png")
            print(f"  Overview: {name}")
        except Exception as e:
            print(f"  SKIP {name}: {e}")

if __name__ == "__main__":
    print(f"Output: {OUT_DIR}")
    save_overviews()
    preview_sunnyside_v2()
    preview_little_dreamyland()
    preview_cute_fantasy()
    preview_farm_rpg()
    preview_pixel16()
    preview_shining_fields()
    print("\nDone! open", OUT_DIR)

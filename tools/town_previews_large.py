#!/usr/bin/env python3
"""
Large Complex Village Preview Generator (FIXED)
Fixes:
  - Little Dreamyland grass: col 3, row 0  (was col 2, row 8 = water tile)
  - Farm RPG walk: RPG Maker VX format, 3 frames per dir from char 0
  - Sunnyside walk: layered base+hair, larger background context

Output: tools/previews/large_*.png / large_*.gif
Run:    python3 tools/town_previews_large.py
"""
from PIL import Image, ImageDraw
import os, sys

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

def save(img, name, scale=3, bg=(0,0,0)):
    out = up(img, scale)
    if out.mode == "RGBA":
        bg_img = Image.new("RGB", out.size, bg)
        bg_img.paste(out, mask=out.split()[3])
        out = bg_img
    path = f"{OUT}/{name}"
    out.save(path)
    print(f"  PNG  {path}")
    return out

def caption(img, text):
    """Add a dark label bar at the top."""
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, img.width, 17], fill=(0, 0, 0, 200))
    d.text((5, 3), text, fill=(255, 255, 180))
    return img

def gif(frames_rgba, name, bg=(100, 155, 65), scale=3, ms=120):
    out = []
    for f in frames_rgba:
        bg_img = Image.new("RGB", f.size, bg)
        if f.mode == "RGBA":
            bg_img.paste(f, mask=f.split()[3])
        else:
            bg_img.paste(f)
        out.append(bg_img.resize((bg_img.width*scale, bg_img.height*scale), Image.NEAREST))
    path = f"{OUT}/{name}"
    out[0].save(path, save_all=True, append_images=out[1:], loop=0, duration=ms)
    print(f"  GIF  {path}")

# ══════════════════════════════════════════════════════════════════════════════
# SUNNYSIDE WORLD V2  —  true top-down, ExampleScene crop
# ══════════════════════════════════════════════════════════════════════════════
def do_sunnyside():
    print("\n═══ Sunnyside World V2 ═════════════════════════")
    base = (f"{PACKS}/sunnyside_v2/"
            "Sunnyside_World_ASSET_PACK_V2.1/Sunnyside_World_Assets")
    scene = Image.open(f"{base}/Sunnyside_World_ExampleScene.png")
    print(f"  ExampleScene: {scene.size}")

    # Very large crop — show the whole village area
    crop = scene.crop((600, 280, 4050, 2220))   # 3450 × 1940
    out_w = 1500
    large = crop.resize((out_w, int(crop.height * out_w / crop.width)), Image.NEAREST)
    caption(large,
        "Sunnyside V2  |  TRUE TOP-DOWN  |  ExampleScene.png  |  "
        "buildings seen from above, river, windmill, goblins, pig, chicken")
    save(large, "large_sunnyside.png", scale=1)

    # Walk GIF: layer base body + hair strip, walk across a grass background
    walk_b = Image.open(
        f"{base}/Characters/Human/WALKING/base_walk_strip8.png").convert("RGBA")
    fw, fh = walk_b.width // 8, walk_b.height   # 96 × 64
    print(f"  Walk frame: {fw}×{fh}")

    walk_h = None
    for hv in ["spikeyhair", "bowlhair", "longhair", "bald"]:
        try:
            walk_h = Image.open(
                f"{base}/Characters/Human/WALKING/{hv}_walk_strip8.png").convert("RGBA")
            print(f"  Hair: {hv}")
            break
        except FileNotFoundError:
            pass

    char_frames = []
    for i in range(8):
        body = walk_b.crop((i*fw, 0, (i+1)*fw, fh))
        if walk_h:
            hair = walk_h.crop((i*fw, 0, (i+1)*fw, fh))
            body = body.copy()
            body.paste(hair, (0, 0), hair)
        char_frames.append(body)

    # Background: large grass strip from within the ExampleScene
    BG_W, BG_H = fw * 6, fh * 2
    try:
        # Pull a grass area from the scene (avoid buildings)
        grass_bg = scene.crop((2600, 1450, 2600 + BG_W, 1450 + BG_H)).convert("RGBA")
        print(f"  Grass bg from scene: {grass_bg.size}")
    except Exception:
        grass_bg = Image.new("RGBA", (BG_W, BG_H), (95, 160, 60, 255))

    walk_anim = []
    for fi in range(8):
        fc = grass_bg.copy()
        # Two characters at different phases, walking left to right
        for ci in range(2):
            speed = fw // 3
            x = (fi * speed + ci * fw * 3) % (BG_W + fw) - fw
            paste(fc, char_frames[fi], x + fw//2, BG_H // 2)
        walk_anim.append(fc)
    gif(walk_anim, "large_sunnyside_walk.gif", bg=(95, 155, 60), ms=100)


# ══════════════════════════════════════════════════════════════════════════════
# LITTLE DREAMYLAND  —  FIXED: correct grass tile (col 3, row 0)
# ══════════════════════════════════════════════════════════════════════════════
def do_dreamyland():
    print("\n═══ Little Dreamyland ══════════════════════════")
    base = f"{PACKS}/little_dreamyland/Little Dreamyland - Free Pack"

    ground = Image.open(
        f"{base}/Tileset/Autotile_Grass_and_Dirt_Path_Tileset.png").convert("RGBA")
    print(f"  Ground tileset: {ground.size}")
    # FIXED: col 3, row 0 = confirmed solid flat green grass
    grass = t(ground, 3, 0)
    # Interior dirt path: col 16, row 1 = confirmed tan (rgb≈187,155,99)
    dirt  = t(ground, 16, 1)
    water = Image.new("RGBA", (16, 16), (45, 115, 195, 255))

    house_ts = Image.open(f"{base}/Tileset/House_Tileset.png").convert("RGBA")
    print(f"  House tileset: {house_ts.size}")
    red_house   = house_ts.crop((0,   0, 48,  80))   # 48×80 — 3-tile wide, 5-tile tall
    green_house = house_ts.crop((0, 160, 48, 224))   # 48×64 — 3-tile wide, 4-tile tall

    nat = Image.open(f"{base}/Tileset/Nature_Tileset.png").convert("RGBA")
    print(f"  Nature tileset: {nat.size}")
    tree_s = nat.crop((0,  0,  32, 48))   # single round tree
    tree_d = nat.crop((96, 0, 160, 64))   # double round tree cluster (64×64)

    ext = Image.open(f"{base}/Tileset/Exterior_Tileset.png").convert("RGBA")
    bench = ext.crop((4*16, 5*16, 7*16, 7*16))  # bench 48×32
    well  = ext.crop((7*16, 0,   10*16, 3*16))  # well  48×48

    bunny = Image.open(
        f"{base}/Sprites/Characters/Bunny/RUN/Bunny_Run.png").convert("RGBA")
    bw, bh = bunny.width // 8, bunny.height // 4   # 48×48 per frame
    print(f"  Bunny frame: {bw}×{bh}  (sheet: {bunny.size})")

    def bframes(row):
        return [bunny.crop((f*bw, row*bh, (f+1)*bw, (row+1)*bh)) for f in range(8)]

    # Row 0=South, 1=West, 2=East, 3=North  (confirmed from visual inspection)
    south_f = bframes(0)
    west_f  = bframes(1)
    east_f  = bframes(2)
    north_f = bframes(3)

    # ── Canvas: 480×288 (30×18 tiles at 16px) ─────────────────────────────────
    W, H = 480, 288
    # Solid green background — the autotile grass overlay is semi-transparent
    c = Image.new("RGBA", (W, H), (100, 180, 70, 255))
    fill(c, grass)

    # Roads: horizontal y=120..152, vertical x=208..240
    fill(c, dirt, 0, 120, W, 152)
    fill(c, dirt, 208, 0, 240, H - 48)
    # Water strip at the bottom
    fill(c, water, 0, H - 48, W, H)

    # ── Buildings ──
    # Top-left quadrant
    paste(c, red_house,   104,  60)
    # Top-right quadrant
    paste(c, green_house, 368,  60)
    # Bottom-left (below path, above water)
    paste(c, red_house,   104, 218)
    # Bottom-right
    paste(c, green_house, 368, 218)

    # ── Nature ──
    paste(c, tree_d,  52,  38)    # double cluster, top-left
    paste(c, tree_s, 175,  40)    # single, between TL house and road
    paste(c, tree_s, 288,  40)    # single, above intersection
    paste(c, tree_s, 440,  38)    # single, top-right corner
    paste(c, tree_d, 308, 190)    # double cluster, right side below path
    paste(c, tree_s, 148, 218)    # single, bottom-left area
    paste(c, tree_d,  52, 222)    # double cluster, bottom-left corner

    # ── Props ──
    paste(c, bench, 175, 108)     # bench along top edge of horizontal road
    paste(c, well,  312, 230)     # well in bottom-right area

    # ── Characters: one per direction at/near the intersection ──
    paste(c, east_f[2],  120, 136)   # walking east (left of intersection)
    paste(c, south_f[0], 224, 136)   # walking south (at intersection center)
    paste(c, west_f[2],  348, 136)   # walking west (right of intersection)
    paste(c, north_f[0], 224,  75)   # walking north (on vertical road above)

    caption(c,
        "Little Dreamyland  |  FIXED grass=col3,row0  |  "
        "cross-roads  |  red+green houses  |  bunnies 4 dirs  |  bench+well")
    save(c, "large_dreamyland.png")

    # ── 4-direction animated bunny GIF ──
    PAD = 8
    gw = bw * 2 + PAD * 3   # 48*2 + 8*3 = 120
    gh = bh * 2 + PAD * 3   # 120
    grid_frames = []
    for fi in range(8):
        p = Image.new("RGBA", (gw, gh))
        fill(p, grass)
        # top-left=South, top-right=North, bottom-left=West, bottom-right=East
        paste(p, south_f[fi], PAD + bw//2,            PAD + bh//2)
        paste(p, north_f[fi], PAD*2 + bw + bw//2,     PAD + bh//2)
        paste(p, west_f[fi],  PAD + bw//2,             PAD*2 + bh + bh//2)
        paste(p, east_f[fi],  PAD*2 + bw + bw//2,      PAD*2 + bh + bh//2)
        d = ImageDraw.Draw(p)
        d.text((2,                   2),               "S↓", fill=(255,255,100))
        d.text((PAD + bw + 2,        2),               "N↑", fill=(255,255,100))
        d.text((2,                   PAD + bh + 2),    "W←", fill=(255,255,100))
        d.text((PAD + bw + 2,        PAD + bh + 2),   "E→", fill=(255,255,100))
        grid_frames.append(p)
    gif(grid_frames, "large_dreamyland_walk.gif", bg=(100, 160, 70), ms=100)


# ══════════════════════════════════════════════════════════════════════════════
# CUTE FANTASY FREE
# ══════════════════════════════════════════════════════════════════════════════
def do_cutefantasy():
    print("\n═══ Cute Fantasy Free ══════════════════════════")
    base = f"{PACKS}/cute_fantasy/Cute_Fantasy_Free"

    grass_t  = Image.open(f"{base}/Tiles/Grass_Middle.png").convert("RGBA").crop((0,0,16,16))
    path_t   = Image.open(f"{base}/Tiles/Path_Middle.png").convert("RGBA").crop((0,0,16,16))
    water_t  = Image.open(f"{base}/Tiles/Water_Middle.png").convert("RGBA").crop((0,0,16,16))

    house    = Image.open(f"{base}/Outdoor decoration/House_1_Wood_Base_Blue.png").convert("RGBA")
    tree     = Image.open(f"{base}/Outdoor decoration/Oak_Tree.png").convert("RGBA")
    tree_s   = Image.open(f"{base}/Outdoor decoration/Oak_Tree_Small.png").convert("RGBA")
    pig_sheet = Image.open(f"{base}/Animals/Pig/Pig.png").convert("RGBA")
    chk_sheet = Image.open(f"{base}/Animals/Chicken/Chicken.png").convert("RGBA")
    print(f"  House: {house.size}  Tree: {tree.size}  TreeS: {tree_s.size}")
    print(f"  Pig sheet: {pig_sheet.size}  Chicken sheet: {chk_sheet.size}")
    # Crop single standing frame (these are sprite sheets — each frame is square,
    # frame size = sheet_height / num_rows; for CF animals typically 16×16 or 32×32)
    pig_fw = pig_sheet.height // 4 if pig_sheet.height >= 32 else pig_sheet.height
    pig = pig_sheet.crop((0, 0, pig_fw, pig_fw))
    chk_fw = chk_sheet.height // 4 if chk_sheet.height >= 32 else chk_sheet.height
    chicken = chk_sheet.crop((0, 0, chk_fw, chk_fw))
    print(f"  Pig frame: {pig.size}  Chicken frame: {chicken.size}")

    player = Image.open(f"{base}/Player/Player.png").convert("RGBA")
    # width//4 gives correct frame size (confirmed from earlier preview work)
    fw_p = player.width // 4
    fh_p = fw_p
    total_rows = player.height // fh_p
    print(f"  Player: {player.size}  frame={fw_p}×{fh_p}  rows={total_rows}")

    def prow(row, n=4):
        return [player.crop((i*fw_p, row*fh_p, (i+1)*fw_p, (row+1)*fh_p)) for i in range(n)]

    # Row 1 = south walk (confirmed ✅), Row 2 = north walk (confirmed ✅)
    south_f = prow(1)
    north_f = prow(2)
    east_f  = prow(3) if total_rows > 3 else prow(1)
    west_f  = prow(4) if total_rows > 4 else prow(1)

    # ── Canvas: 480×320 ───────────────────────────────────────────────────────
    W, H = 480, 320
    c = Image.new("RGBA", (W, H), (100, 178, 70, 255))  # solid green bg in case tile has alpha
    fill(c, grass_t)

    # Main horizontal path at y=144..160
    fill(c, path_t, 0, 144, W, 160)
    # Second horizontal path at y=256..272
    fill(c, path_t, 0, 256, W, 272)
    # Vertical path at x=224..240
    fill(c, path_t, 224, 0, 240, H)
    # Water pond: bottom-left corner
    fill(c, water_t, 0, H - 64, 112, H)

    # ── Buildings (house ~96×128) ──
    paste(c, house,  80,  78)    # top-left
    paste(c, house, 396,  78)    # top-right
    paste(c, house,  80, 218)    # bottom-left
    paste(c, house, 396, 218)    # bottom-right

    # ── Trees ──
    paste(c, tree,   28,  42)
    paste(c, tree,  170,  42)
    paste(c, tree_s, 292,  54)
    paste(c, tree,  452,  42)
    paste(c, tree_s, 168, 210)
    paste(c, tree,  295, 210)
    paste(c, tree_s, 170, 290)
    paste(c, tree,  456, 210)

    # ── Animals ──
    paste(c, pig,     140, 152)
    paste(c, pig,     182, 155)
    paste(c, chicken, 330, 152)
    paste(c, chicken, 362, 152)
    paste(c, chicken, 150, 265)

    # ── Player at intersection ──
    paste(c, south_f[1], 232, 152)

    caption(c,
        "Cute Fantasy Free  |  16px tile PNGs  |  house 96×128  |  "
        "oak trees  |  pig & chicken  |  player south row1 ✓  north row2 ✓")
    save(c, "large_cutefantasy.png")

    # ── Walk GIF: 2×2 grid showing all 4 directions ──
    dirs    = [south_f, north_f, east_f,  west_f]
    labels  = [("S↓",0,0), ("N↑",1,0), ("E→",0,1), ("W←",1,1)]
    PAD = 6
    gw = fw_p * 2 + PAD * 3
    gh = fh_p * 2 + PAD * 3
    grid_frames = []
    for fi in range(len(south_f)):
        p = Image.new("RGBA", (gw, gh))
        fill(p, grass_t)
        d = ImageDraw.Draw(p)
        for (lbl, gx, gy), row in zip(labels, dirs):
            cx = PAD + gx * (fw_p + PAD) + fw_p//2
            cy = PAD + gy * (fh_p + PAD) + fh_p//2
            paste(p, row[fi % len(row)], cx, cy)
            d.text((PAD + gx*(fw_p+PAD) + 1, PAD + gy*(fh_p+PAD) + 1),
                   lbl, fill=(255, 255, 100))
        grid_frames.append(p)
    gif(grid_frames, "large_cutefantasy_walk.gif", ms=130)


# ══════════════════════════════════════════════════════════════════════════════
# FARM RPG 16×16 — FIXED walk (RPG Maker VX: 3 frames per dir, char 0)
# ══════════════════════════════════════════════════════════════════════════════
def do_farmrpg():
    print("\n═══ Farm RPG 16×16 Tiny ════════════════════════")
    base = f"{PACKS}/farm_rpg/Farm RPG FREE 16x16 - Tiny Asset Pack"

    ts    = Image.open(f"{base}/Tileset/Tileset Spring.png").convert("RGBA")
    grass = t(ts, 9, 2)    # confirmed bright green from town_farmrpg.png ✅
    print(f"  Tileset: {ts.size}  grass tile: col9,row2")

    # Road/path tile
    try:
        road_s = Image.open(f"{base}/Objects/Road copiar.png").convert("RGBA")
        path_t = t(road_s, 2, 1)
        print(f"  Road sheet: {road_s.size}")
    except FileNotFoundError:
        print("  Road file not found — using tileset row 10")
        path_t = t(ts, 9, 10)

    water = Image.new("RGBA", (16, 16), (45, 115, 190, 255))

    # Houses: crop properly from the sheet
    house_s = Image.open(f"{base}/Objects/House.png").convert("RGBA")
    print(f"  House sheet: {house_s.size}")
    barn    = house_s.crop((0,   0,  80, 112))   # 80×112
    cottage = house_s.crop((128, 0, 224, 112))   # 96×112

    # Maple trees: 5 variants at 32×48 each in a 160×48 strip
    maple_s = Image.open(f"{base}/Objects/Maple Tree.png").convert("RGBA")
    print(f"  Maple sheet: {maple_s.size}")
    maple_a = maple_s.crop(( 0, 0,  32, 48))   # variant A
    maple_b = maple_s.crop((32, 0,  64, 48))   # variant B
    maple_c = maple_s.crop((96, 0, 128, 48))   # variant C (big tree)

    chk_s = Image.open(f"{base}/Farm Animals/Chicken Red.png").convert("RGBA")
    cow_s = Image.open(f"{base}/Farm Animals/Male Cow Brown.png").convert("RGBA")
    print(f"  Chicken sheet: {chk_s.size}  Cow sheet: {cow_s.size}")
    # RPG Maker VX style: 4 chars × (3 cols × 4 rows); chicken is 16×16 per frame
    chicken = chk_s.crop((0, 0, 16, 16))   # single south-facing chicken frame
    # Cow: likely 32×32 or 16×24 per frame — use first frame
    cow_fw = min(32, cow_s.width // 4)
    cow_fh = min(32, cow_s.height // 4)
    cow = cow_s.crop((0, 0, cow_fw, cow_fh))
    print(f"  Chicken frame: {chicken.size}  Cow frame: {cow.size}")

    # ── Walk: FIXED RPG Maker VX layout ──
    # walk.png: 192×96 = 4 characters × (3 cols × 4 rows) at 16×24 per frame
    # Char 0 (x=0..47):
    #   row 0 (y=0..23)  = South  (3 frames at x=0,16,32)
    #   row 1 (y=24..47) = West   (3 frames)
    #   row 2 (y=48..71) = East   (3 frames)
    #   row 3 (y=72..95) = North  (3 frames)
    walk = Image.open(f"{base}/Character/Walk.png").convert("RGBA")
    fw_c, fh_c = 16, 24
    print(f"  Walk sheet: {walk.size}  frame={fw_c}×{fh_c}")

    def wdir(row, char_idx=0):
        ox = char_idx * 3 * fw_c   # character x-offset (48 per character)
        y0 = row * fh_c
        return [walk.crop((ox + col*fw_c, y0, ox + (col+1)*fw_c, y0+fh_c))
                for col in range(3)]

    south_f = wdir(0)   # row 0 = south
    west_f  = wdir(1)   # row 1 = west
    east_f  = wdir(2)   # row 2 = east
    north_f = wdir(3)   # row 3 = north
    # Walk loop: middle → left foot → middle → right foot
    south_loop = [south_f[1], south_f[0], south_f[1], south_f[2]]

    # ── Canvas: 480×320 (30×20 tiles at 16px) — taller for room ───────────────
    W, H = 480, 320
    c = Image.new("RGBA", (W, H), (100, 160, 60, 255))  # solid green bg
    fill(c, grass)

    # Paths (horizontal only — stepping-stone vertical path)
    fill(c, path_t, 0,   140, W, 156)      # main horizontal road
    fill(c, path_t, 212,  0, 228, 140)     # vertical road top section
    fill(c, path_t, 212, 156, 228, H - 32) # vertical road bottom section
    # Water: small pond bottom-right
    fill(c, water, W - 80, H - 48, W, H)

    # ── Buildings (barn 80×112, cottage 96×112) ──
    paste(c, barn,      80,  72)   # barn: top-left
    paste(c, cottage,  392,  72)   # cottage: top-right
    paste(c, barn,      80, 248)   # barn: bottom-left (bottom=248+56=304 < 320 ✓)
    paste(c, cottage,  392, 248)   # cottage: bottom-right

    # ── Trees ──
    paste(c, maple_c,  30,  36)
    paste(c, maple_a, 168,  36)
    paste(c, maple_b, 290,  36)
    paste(c, maple_c, 456,  36)
    paste(c, maple_a, 148, 220)
    paste(c, maple_b, 296, 220)
    paste(c, maple_c, 456, 220)

    # ── Animals: single frames ──
    paste(c, chicken, 142, 148)
    paste(c, chicken, 162, 148)
    paste(c, chicken, 182, 148)
    paste(c, cow,     330, 148)
    paste(c, cow,     366, 148)
    paste(c, chicken, 142, 270)
    paste(c, chicken, 158, 270)
    paste(c, cow,     420, 260)

    # ── Farmer at intersection ──
    paste(c, south_f[1], 220, 148)

    caption(c,
        "Farm RPG 16×16  |  grass col9,row2  |  barn+cottage  |  3 maple variants  |  "
        "chickens+cow (single frames)  |  FIXED walk: VX char0 row0=S,1=W,2=E,3=N")
    save(c, "large_farmrpg.png")

    # ── Walk GIF: 2×2 grid, 4 directions, FIXED crops ──
    dirs   = [south_f, north_f, east_f, west_f]
    labels = [("S↓",0,0), ("N↑",1,0), ("E→",0,1), ("W←",1,1)]
    PAD    = 4
    gw     = fw_c * 2 + PAD * 3
    gh     = fh_c * 2 + PAD * 3
    walk_frames = []
    for fi in range(3):
        p = Image.new("RGBA", (gw, gh))
        fill(p, grass)
        d = ImageDraw.Draw(p)
        for (lbl, gx, gy), row in zip(labels, dirs):
            cx = PAD + gx * (fw_c + PAD) + fw_c//2
            cy = PAD + gy * (fh_c + PAD) + fh_c//2
            paste(p, row[fi], cx, cy)
            d.text((PAD + gx*(fw_c+PAD), PAD + gy*(fh_c+PAD)), lbl, fill=(255,255,100))
        walk_frames.append(p)
    # Loop: 0 → 1 → 2 → 1
    walk_frames = [walk_frames[0], walk_frames[1], walk_frames[2], walk_frames[1]]
    gif(walk_frames, "large_farmrpg_walk.gif", bg=(100, 155, 65), scale=5, ms=150)


# ══════════════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print(f"Output: {OUT}")
    try:
        do_sunnyside()
    except Exception as e:
        print(f"  ERROR sunnyside: {e}")
    try:
        do_dreamyland()
    except Exception as e:
        print(f"  ERROR dreamyland: {e}")
    try:
        do_cutefantasy()
    except Exception as e:
        print(f"  ERROR cutefantasy: {e}")
    try:
        do_farmrpg()
    except Exception as e:
        print(f"  ERROR farmrpg: {e}")
    print("\nDone →", OUT)

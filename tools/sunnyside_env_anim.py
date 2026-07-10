#!/usr/bin/env python3
"""
Sunnyside World V2 — Environment-Only Animated Village
No characters. Animates: campfire fire, chimney smoke, animals, flying bird.

Frame budget: LCM(fire=4, smoke=30, animals=4, bird=4) = 60 frames
  60 frames @ 8fps = 7.5 s seamless loop

Output: tools/previews/sun_env.gif  (1200×800, 2× display scale)
Run:    python3 tools/sunnyside_env_anim.py
"""
from PIL import Image
import os, glob

BASE = ("/tmp/asset_preview/sunnyside_v2/"
        "Sunnyside_World_ASSET_PACK_V2.1/Sunnyside_World_Assets")
OUT  = os.path.join(os.path.dirname(__file__), "previews")
os.makedirs(OUT, exist_ok=True)

# ─── helpers ──────────────────────────────────────────────────────────────────

def load_strip(path, n):
    """Load a horizontal sprite strip into n RGBA frames."""
    img = Image.open(path).convert("RGBA")
    fw = img.width // n
    return [img.crop((i*fw, 0, (i+1)*fw, img.height)) for i in range(n)]

def scale_nn(frame, factor):
    """Nearest-neighbour scale by float or int factor."""
    w = max(1, round(frame.width  * factor))
    h = max(1, round(frame.height * factor))
    return frame.resize((w, h), Image.NEAREST)

def paste_alpha(canvas, img, x, y):
    """Paste RGBA img onto RGBA canvas at top-left (x,y)."""
    # Clamp to canvas bounds
    cx1, cy1 = max(0, x), max(0, y)
    ix1 = cx1 - x; iy1 = cy1 - y
    cx2 = min(canvas.width,  x + img.width)
    cy2 = min(canvas.height, y + img.height)
    if cx2 <= cx1 or cy2 <= cy1:
        return
    region = img.crop((ix1, iy1, ix1+(cx2-cx1), iy1+(cy2-cy1)))
    canvas.paste(region, (cx1, cy1), region)

# ─── load all environment animations ──────────────────────────────────────────

# VFX
fire_frames    = load_strip(f"{BASE}/Elements/VFX/Fire/spr_deco_fire_01_strip4.png", 4)
smoke_frames   = load_strip(f"{BASE}/Elements/VFX/Chimney Smoke/chimneysmoke_01_strip30.png", 30)
glint_frames   = load_strip(f"{BASE}/Elements/VFX/Glint/spr_deco_glint_01_strip6.png", 6)

# Animals
chicken_frames = load_strip(f"{BASE}/Elements/Animals/spr_deco_chicken_01_strip4.png", 4)
duck_frames    = load_strip(f"{BASE}/Elements/Animals/spr_deco_duck_01_strip4.png", 4)
pig_frames     = load_strip(f"{BASE}/Elements/Animals/spr_deco_pig_01_strip4.png", 4)
sheep_frames   = load_strip(f"{BASE}/Elements/Animals/spr_deco_sheep_01_strip4.png", 4)
bird_frames    = load_strip(f"{BASE}/Elements/Animals/spr_deco_bird_01_strip4.png", 4)
cow_frames     = load_strip(f"{BASE}/Elements/Animals/spr_deco_cow_strip4.png", 4)

# Windmill
windmill_frames = load_strip(f"{BASE}/Elements/Other/spr_deco_windmill_strip9.png", 9)

# ─── scale all sprites to 2× (matching scene display scale) ──────────────────
# Scene tiles are 16px at 1× → displayed 2× in output.
# All element sprites are at the same 16px base scale.

S = 2.0   # base display scale — elements at 1:1 feel right in the village
FIRE_S   = 3.0   # fire needs bigger boost so it reads clearly over the campfire
SMOKE_S  = 2.0
ANIM_S   = 2.0   # animals at 2× source = 64×64 on screen per animal

fire_s   = [scale_nn(f, FIRE_S)  for f in fire_frames]     # 15×30  each
smoke_s  = [scale_nn(f, SMOKE_S) for f in smoke_frames]    # 30×74  each
glint_s  = [scale_nn(f, S)       for f in glint_frames]    # 14×14  each
chick_s  = [scale_nn(f, ANIM_S)  for f in chicken_frames]  # 64×64  each
duck_s   = [scale_nn(f, ANIM_S)  for f in duck_frames]     # 32×32  each
pig_s    = [scale_nn(f, ANIM_S)  for f in pig_frames]      # 64×64  each
sheep_s  = [scale_nn(f, ANIM_S)  for f in sheep_frames]    # 64×64  each
bird_s   = [scale_nn(f, ANIM_S)  for f in bird_frames]     # 32×32  each
cow_s    = [scale_nn(f, ANIM_S)  for f in cow_frames]      # 64×64  each
mill_s   = [scale_nn(f, S)       for f in windmill_frames] # 224×224 each

print(f"  fire frame size:     {fire_s[0].size}")
print(f"  smoke frame size:    {smoke_s[0].size}")
print(f"  chicken frame size:  {chick_s[0].size}")
print(f"  windmill frame size: {mill_s[0].size}")

# ─── scene background ─────────────────────────────────────────────────────────
# All sprite positions below are in SOURCE 1× coordinates.
# The final output scales everything DISPLAY_SCALE × at the end.

scene = Image.open(f"{BASE}/Sunnyside_World_ExampleScene.png")

# This gorgeous crop shows: campfire, winding dirt path, red-roof house, goblin,
# tall conifers, bright flowers. Confirmed via grid-overlay analysis.
SX, SY, SW, SH = 1400, 800, 600, 400
DISPLAY = 2   # final output scale applied to entire composited frame

bg_src = scene.crop((SX, SY, SX+SW, SY+SH)).convert("RGB")
print(f"  scene bg: {bg_src.size} → output {SW*DISPLAY}×{SH*DISPLAY}")

# ─── animation element placements (in 1× source coords) ──────────────────────
#
# Determined via pixel-color detection and grid-overlay visual inspection:
#   campfire orange-cluster center:  (154, 106)  — flame bottom ~y=110
#   chimney (grey cylinder on roof): (~300, 168) — smoke exits from top
#   right grass (open area):         x=490-560,  y=140-195
#   left grass:                      x=30-100,   y=200-340
#
# Sprite origin = top-left corner of the (already-scaled) frame.
# For fire/smoke we want bottom-of-frame to align with the emission point.
# For animals we want bottom-of-frame = feet on ground (y_feet).

def placement(scaled_frame, cx, cy_bottom):
    """Return (px, py) so frame is centered-x at cx, bottom at cy_bottom."""
    return (cx - scaled_frame.width // 2,
            cy_bottom - scaled_frame.height)

N_FRAMES  = 60          # LCM(4, 30) — perfect loop for fire & smoke
OUTPUT_FPS = 8
MS = 1000 // OUTPUT_FPS

# ─── build GIF frames ─────────────────────────────────────────────────────────

def make_frame(fi, bg_src, display):
    """Composite one GIF frame at SOURCE scale, then upscale."""
    canvas = bg_src.copy().convert("RGBA")

    # ── campfire flame ──
    ff = fire_s[fi % 4]
    px, py = placement(ff, 154, 112)
    paste_alpha(canvas, ff, px, py)

    # ── chimney smoke ──
    sf = smoke_s[fi % 30]
    # smoke rises — bottom of frame at chimney tip y=165
    px, py = placement(sf, 300, 165)
    paste_alpha(canvas, sf, px, py)

    # ── glint on campfire stones (alternates) ──
    if fi % 12 < 6:
        gf = glint_s[fi % 6]
        paste_alpha(canvas, gf, 148, 108)

    # ── chicken — right grass, near path ──
    cf = chick_s[fi % 4]
    px, py = placement(cf, 530, 180)
    paste_alpha(canvas, cf, px, py)

    # ── duck — right grass, slightly lower ──
    df = duck_s[fi % 4]
    px, py = placement(df, 490, 165)
    paste_alpha(canvas, df, px, py)

    # ── sheep — left grass, mid-height ──
    shf = sheep_s[fi % 4]
    px, py = placement(shf, 60, 230)
    paste_alpha(canvas, shf, px, py)

    # ── pig — left grass, lower ──
    pf = pig_s[fi % 4]
    px, py = placement(pf, 85, 310)
    paste_alpha(canvas, pf, px, py)

    # ── cow — bottom-right open area ──
    cvf = cow_s[fi % 4]
    px, py = placement(cvf, 555, 200)
    paste_alpha(canvas, cvf, px, py)

    # ── flying bird — crosses from left to right over ~60 frames ──
    bird_x = -32 + fi * 12        # travels 720px in 60 frames → wraps off-screen
    bird_y = 25
    bf = bird_s[fi % 4]
    paste_alpha(canvas, bf, bird_x, bird_y)

    # ── windmill — placed top-right open area (no house collision) ──
    mf = mill_s[fi % 9]
    # windmill bottom at y=110, centered at x=490
    px, py = placement(mf, 490, 110)
    paste_alpha(canvas, mf, px, py)

    # ── flatten RGBA → RGB then upscale ──
    rgb = Image.new("RGB", canvas.size, (0, 0, 0))
    rgb.paste(canvas, mask=canvas.split()[3])
    return rgb.resize((rgb.width * display, rgb.height * display), Image.NEAREST)


print(f"\nRendering {N_FRAMES} frames...")
frames = [make_frame(fi, bg_src, DISPLAY) for fi in range(N_FRAMES)]

path = f"{OUT}/sun_env.gif"
frames[0].save(
    path,
    save_all=True,
    append_images=frames[1:],
    loop=0,
    duration=MS,
    optimize=False,
)
print(f"Done → {path}  ({frames[0].size}, {N_FRAMES} frames @ {OUTPUT_FPS}fps)")

#!/usr/bin/env python3
"""
Sunnyside World V2 — Sprite Overview + Animated Village
Outputs:
  tools/previews/sun_sprites.png   — all character types (walk strip grid)
  tools/previews/sun_village.gif   — animated village scene
Run: python3 tools/sunnyside_anim.py
"""
from PIL import Image, ImageDraw
import os

BASE = ("/tmp/asset_preview/sunnyside_v2/"
        "Sunnyside_World_ASSET_PACK_V2.1/Sunnyside_World_Assets")
OUT  = os.path.join(os.path.dirname(__file__), "previews")
os.makedirs(OUT, exist_ok=True)

# ─── helpers ──────────────────────────────────────────────────────────────────

def load_strip(path, n):
    img = Image.open(path).convert("RGBA")
    fw = img.width // n
    return [img.crop((i*fw, 0, (i+1)*fw, img.height)) for i in range(n)]

def flip(frame):
    return frame.transpose(Image.FLIP_LEFT_RIGHT)

def paste(canvas, img, cx, cy):
    canvas.paste(img, (cx - img.width//2, cy - img.height//2), img)

def human_walk(hair):
    base = load_strip(f"{BASE}/Characters/Human/WALKING/base_walk_strip8.png", 8)
    h    = load_strip(f"{BASE}/Characters/Human/WALKING/{hair}_walk_strip8.png", 8)
    out  = []
    for b, hf in zip(base, h):
        c = b.copy(); c.paste(hf, (0,0), hf); out.append(c)
    return out   # 8 RGBA frames, 96×64 each

def human_idle(hair):
    base = load_strip(f"{BASE}/Characters/Human/IDLE/base_idle_strip9.png", 9)
    h    = load_strip(f"{BASE}/Characters/Human/IDLE/{hair}_idle_strip9.png", 9)
    out  = []
    for b, hf in zip(base, h):
        c = b.copy(); c.paste(hf, (0,0), hf); out.append(c)
    return out   # 9 RGBA frames, 96×64 each


# ══════════════════════════════════════════════════════════════════════════════
#  SPRITE OVERVIEW
# ══════════════════════════════════════════════════════════════════════════════
def make_sprites_overview():
    print("\n── Sprite Overview ─────────────────────────────")
    FW, FH = 96, 64
    SCALE  = 2          # display scale
    PAD    = 6          # padding between rows
    LABEL  = 18         # label bar height

    characters = [
        ("Human — spikeyhair",   human_walk("spikeyhair")),
        ("Human — longhair",     human_walk("longhair")),
        ("Human — curlyhair",    human_walk("curlyhair")),
        ("Human — mophair",      human_walk("mophair")),
        ("Human — shorthair",    human_walk("shorthair")),
        ("Human — bowlhair",     human_walk("bowlhair")),
        ("Goblin — walk",        load_strip(f"{BASE}/Characters/Goblin/PNG/spr_walk_strip8.png", 8)),
        ("Skeleton — walk",      load_strip(f"{BASE}/Characters/Skeleton/PNG/skeleton_walk_strip8.png", 8)),
    ]

    n_frames = max(len(f) for _, f in characters)
    row_h    = FH * SCALE + LABEL + PAD
    img_w    = FW * SCALE * n_frames
    img_h    = row_h * len(characters) + PAD

    canvas = Image.new("RGB", (img_w, img_h), (30, 30, 40))
    d = ImageDraw.Draw(canvas)

    for row, (name, frames) in enumerate(characters):
        y0 = row * row_h + PAD
        # label bar
        d.rectangle([0, y0, img_w, y0 + LABEL], fill=(10, 10, 18))
        d.text((6, y0 + 3), name, fill=(220, 220, 130))
        # frames
        for fi, frame in enumerate(frames):
            scaled = frame.resize((FW*SCALE, FH*SCALE), Image.NEAREST)
            bg = Image.new("RGB", scaled.size, (45, 60, 45))
            bg.paste(scaled, mask=scaled.split()[3])
            canvas.paste(bg, (fi * FW * SCALE, y0 + LABEL))

    path = f"{OUT}/sun_sprites.png"
    canvas.save(path)
    print(f"  PNG  {path}  ({img_w}×{img_h})")


# ══════════════════════════════════════════════════════════════════════════════
#  ANIMATED VILLAGE GIF
# ══════════════════════════════════════════════════════════════════════════════
def scale_frame(frame, factor):
    """Scale a sprite frame preserving RGBA by factor (e.g. 1.5, 2.0)."""
    w = int(frame.width * factor)
    h = int(frame.height * factor)
    return frame.resize((w, h), Image.NEAREST)


def make_village_gif():
    print("\n── Animated Village GIF ────────────────────────")
    scene = Image.open(f"{BASE}/Sunnyside_World_ExampleScene.png")

    # Village area: campfire, red-roof house, goblin at axe, winding dirt path
    SX, SY, SW, SH = 1400, 800, 600, 400
    bg = scene.crop((SX, SY, SX+SW, SY+SH)).convert("RGB")
    print(f"  Scene bg: {bg.size} from ({SX},{SY})")

    # Sprite scale: ExampleScene's own goblin prop is ~80px wide at 2× display.
    # Walk frames at 1× = 96×64. Scale by 1.5 → 144×96. Visible char body ≈ 50px.
    CHAR_SCALE = 1.5   # applied before pasting onto the 600×400 source canvas
    FW_S = int(96 * CHAR_SCALE)   # 144
    FH_S = int(64 * CHAR_SCALE)   # 96

    # Load and pre-scale all walk animations
    hairs   = ["spikeyhair", "longhair", "curlyhair", "mophair", "bowlhair"]
    walkers = [[scale_frame(f, CHAR_SCALE) for f in human_walk(h)] for h in hairs]

    def gs(name):
        return [scale_frame(f, CHAR_SCALE)
                for f in load_strip(f"{BASE}/Characters/Goblin/PNG/{name}", 8)]
    def gi(name):
        return [scale_frame(f, CHAR_SCALE)
                for f in load_strip(f"{BASE}/Characters/Goblin/PNG/{name}", 9)]

    goblin_walk = gs("spr_walk_strip8.png")
    goblin_idle = gi("spr_idle_strip9.png")

    # ── character routes ──
    # Winding path in the scene passes through roughly y=200-320 at 1× source.
    # Start chars at on-screen x so they're visible from frame 1.
    # Each: (x_start, y_feet, speed_px_per_frame, anim_frames, anim_phase, flip_if_left)
    chars = [
        # x_start              y_feet  spd  anim         phase  flip
        (50,                   215,    +3,  walkers[0],  0,     True),   # spiky  →
        (SW - FW_S - 50,       255,    -4,  walkers[1],  2,     True),   # long   ←
        (120,                  305,    +2,  walkers[2],  5,     True),   # curly  →
        (SW - FW_S - 100,      185,    -3,  walkers[3],  1,     True),   # mop    ←
        (200,                  335,    +5,  goblin_walk, 4,     True),   # goblin →
        (SW - FW_S - 30,       270,    -2,  walkers[4],  7,     True),   # bowl   ←
        (350,                  225,    +4,  goblin_walk, 3,     True),   # goblin →
        (210 - FW_S//2,        235,    +0,  goblin_idle, 0,     False),  # idle at campfire
    ]

    N = 48           # 6 s at 8 fps
    DISPLAY_SCALE = 2
    OUTPUT_FPS = 8
    MS = 1000 // OUTPUT_FPS

    gif_frames = []
    for fi in range(N):
        canvas = bg.copy().convert("RGBA")

        for (x0, y_feet, speed, anim, phase, can_flip) in chars:
            # wrap x to keep char looping across scene
            total = x0 + speed * fi
            x = int(total % (SW + FW_S)) - FW_S // 4
            if x > SW or x + FW_S < 0:
                continue

            frame = anim[(fi + phase) % len(anim)]
            if can_flip and speed < 0:
                frame = flip(frame)

            # paste so the frame bottom aligns to y_feet
            px = x
            py = y_feet - FH_S
            canvas.paste(frame, (px, py), frame)

        rgb = Image.new("RGB", canvas.size, (0, 0, 0))
        rgb.paste(canvas, mask=canvas.split()[3])
        scaled = rgb.resize((rgb.width * DISPLAY_SCALE, rgb.height * DISPLAY_SCALE),
                            Image.NEAREST)
        gif_frames.append(scaled)

    path = f"{OUT}/sun_village.gif"
    gif_frames[0].save(
        path,
        save_all=True,
        append_images=gif_frames[1:],
        loop=0,
        duration=MS,
        optimize=False,
    )
    print(f"  GIF  {path}  ({gif_frames[0].size}, {N} frames @ {OUTPUT_FPS}fps)")


# ══════════════════════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print(f"Output: {OUT}")
    make_sprites_overview()
    make_village_gif()
    print("\nDone.")

#!/usr/bin/env python3
"""
Generates annotated PNG previews of the Kenney Tiny-Town and Sunnyside tilesets.
Each tile is labelled with its (col, row) = flat-index and colour-classified.
Outputs:
  tools/out_tiny_town.png
  tools/out_sunnyside.png   (first 16×16 tiles; full sheet is 64×64)

Run:  python3 tools/inspect_tileset.py
Then open the PNGs in Preview to verify tile positions before editing draw_sprites.rs.
"""
import os, sys
from PIL import Image, ImageDraw, ImageFont

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
ROOT       = os.path.dirname(SCRIPT_DIR)
OUT_DIR    = SCRIPT_DIR

# ── helpers ────────────────────────────────────────────────────────────────────

def avg_color(region):
    px = [p for p in region.convert("RGBA").getdata() if p[3] > 64]
    if not px:
        return (0, 0, 0)
    return (
        sum(p[0] for p in px) // len(px),
        sum(p[1] for p in px) // len(px),
        sum(p[2] for p in px) // len(px),
    )

def classify(r, g, b):
    if g > r + 20 and g > b + 10 and g > 80:
        return "GRASS"
    if b > r + 20 and b > g:
        return "WATER"
    if r > 160 and g < 120 and b < 110:
        return "ROOF"
    if abs(r - g) < 25 and abs(g - b) < 25 and r > 80:
        return "STONE"
    if r > 130 and g > 90 and b < 90 and r > g:
        return "BROWN"
    if r > 150 and g > 130 and abs(r - g) < 50:
        return "TAN"
    return "?"

LABEL_COLOR = {
    "GRASS": "#1a7a1a",
    "WATER": "#1a3d7a",
    "ROOF":  "#7a2020",
    "STONE": "#505060",
    "BROWN": "#5a3a10",
    "TAN":   "#806040",
    "?":     "#888888",
}

def make_annotated(
    src_path: str,
    out_path: str,
    tile_w: int,
    tile_h: int,
    max_cols: int | None = None,  # None = all
    max_rows: int | None = None,
    scale: int = 4,
    label_size: int = 7,
):
    img = Image.open(src_path).convert("RGBA")
    iw, ih = img.size
    cols = iw // tile_w
    rows = ih // tile_h
    if max_cols: cols = min(cols, max_cols)
    if max_rows: rows = min(rows, max_rows)

    out_w = cols * tile_w * scale
    out_h = rows * tile_h * scale
    out   = Image.new("RGBA", (out_w, out_h), (30, 30, 35, 255))
    draw  = ImageDraw.Draw(out)

    try:
        # Try to get a small monospace font; fall back to default
        font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", label_size)
    except Exception:
        font = ImageFont.load_default()

    for row in range(rows):
        for col in range(cols):
            sx, sy = col * tile_w, row * tile_h
            region = img.crop((sx, sy, sx + tile_w, sy + tile_h))
            r, g, b = avg_color(region)
            kind    = classify(r, g, b)

            dx, dy = col * tile_w * scale, row * tile_h * scale
            dw, dh = tile_w * scale, tile_h * scale

            # Paste scaled tile
            tile_scaled = region.resize((dw, dh), Image.NEAREST)
            out.paste(tile_scaled, (dx, dy), tile_scaled)

            # Grid border (thin, semi-transparent)
            draw.rectangle(
                [dx, dy, dx + dw - 1, dy + dh - 1],
                outline=(200, 200, 200, 80),
            )

            # Flat tile index
            flat_idx = row * (iw // tile_w) + col
            label    = f"{flat_idx}\n{col},{row}"
            lc       = LABEL_COLOR.get(kind, "#aaa")

            # Tiny background for readability
            draw.text((dx + 2, dy + 2), label, fill=lc, font=font)

    # Legend
    lex = 6
    ley = out_h - 60
    for i, (k, lc) in enumerate(LABEL_COLOR.items()):
        draw.rectangle([lex, ley + i * 9, lex + 6, ley + i * 9 + 6], fill=lc)
        draw.text((lex + 9, ley + i * 9), k, fill="#cccccc", font=font)

    out.save(out_path)
    print(f"  Saved → {out_path}  ({cols}×{rows} tiles, {scale}× scale)")

# ── tiny_town ──────────────────────────────────────────────────────────────────
print("Generating tiny_town preview…")
make_annotated(
    src_path   = os.path.join(ROOT, "assets", "tiles", "tiny_town.png"),
    out_path   = os.path.join(OUT_DIR, "out_tiny_town.png"),
    tile_w     = 16,
    tile_h     = 16,
    scale      = 5,      # 16 → 80 px per tile, easily readable
    label_size = 8,
)

# ── sunnyside (first 20×20 tiles) ─────────────────────────────────────────────
print("Generating sunnyside preview (first 20×20 tiles)…")
make_annotated(
    src_path   = os.path.join(ROOT, "assets", "tiles", "sunnyside.png"),
    out_path   = os.path.join(OUT_DIR, "out_sunnyside.png"),
    tile_w     = 16,
    tile_h     = 16,
    max_cols   = 20,
    max_rows   = 20,
    scale      = 5,
    label_size = 8,
)

print("Done. Open with: open tools/out_tiny_town.png tools/out_sunnyside.png")

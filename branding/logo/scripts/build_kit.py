"""Build the SysVane logo kit sources (SVG) into ../kit: symbol, small cut, reversed, wordmark, lockups, app icon.
Wordmark = Inter SemiBold (OFL) shaped with HarfBuzz (kerning on) and converted to outlines.

    cd desktop && npm install            # provides the Inter font file
    uv run --no-project --with fonttools --with brotli --with uharfbuzz python branding/logo/scripts/build_kit.py
"""
import io, math, pathlib
from fontTools.ttLib import TTFont
from fontTools.pens.recordingPen import RecordingPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.boundsPen import BoundsPen
import uharfbuzz as hb

here = pathlib.Path(__file__).resolve().parent
out = here.parent / "kit"; out.mkdir(exist_ok=True)
woff2 = here.parents[2] / "desktop/node_modules/@fontsource-variable/inter/files/inter-latin-wght-normal.woff2"
INK, LIT, WHITE = "#171717", "#3ecf8e", "#ffffff"
FACE_GAP = {INK: 10, WHITE: 12}  # reversed marks look heavier, so their gap is wider
T = math.sqrt(3)
f = lambda v: f"{v:.2f}".rstrip("0").rstrip(".")
path = lambda poly: "M" + " L".join(f"{f(x)},{f(y)}" for x, y in poly) + "Z"

def clip(poly, p0, n):
    d = lambda p: (p[0] - p0[0]) * n[0] + (p[1] - p0[1]) * n[1]
    res = []
    for i, a in enumerate(poly):
        b = poly[(i + 1) % len(poly)]
        da, db = d(a), d(b)
        if da >= 0: res.append(a)
        if (da >= 0) != (db >= 0):
            t = da / (da - db)
            res.append((a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])))
    return res

def shrink(face, arms, g):
    cx = sum(p[0] for p in face) / len(face); cy = sum(p[1] for p in face) / len(face)
    for a, b in arms:
        n = (-(b[1] - a[1]), b[0] - a[0]); k = math.hypot(*n); n = (n[0] / k, n[1] / k)
        if (cx - a[0]) * n[0] + (cy - a[1]) * n[1] < 0: n = (-n[0], -n[1])
        face = clip(face, (a[0] + n[0] * g / 2, a[1] + n[1] * g / 2), n)
    return face

def crate(cx, cy, R, g, side):
    """The mark: an isometric cube as three faces separated by a Y-shaped gap g. The lid is always emerald."""
    h = R * T / 2
    c = (cx, cy)
    top, ur, lr, bot, ll, ul = (cx, cy - R), (cx + h, cy - R / 2), (cx + h, cy + R / 2), (cx, cy + R), (cx - h, cy + R / 2), (cx - h, cy - R / 2)
    left = shrink([ul, c, bot, ll], [(c, ul), (c, bot)], g)
    right = shrink([ur, lr, bot, c], [(c, ur), (c, bot)], g)
    tp = shrink([top, ur, c, ul], [(c, ul), (c, ur)], g)
    return f'<path fill="{side}" d="{path(left)} {path(right)}"/><path fill="{LIT}" d="{path(tp)}"/>'

def svg(title, vb, body):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{vb}" fill-rule="evenodd"><title>{title}</title>{body}</svg>'

# wordmark: shape once, then emit the outlines wherever a lockup needs them
CAP, TRACK = 104, -0.012  # cap height against the 256 symbol; tracking in em
tt = TTFont(woff2); tt.flavor = None
buf = io.BytesIO(); tt.save(buf)
upem = tt["head"].unitsPerEm
font = hb.Font(hb.Face(hb.Blob(buf.getvalue()))); font.set_variations({"wght": 600})
hbuf = hb.Buffer(); hbuf.add_str("SysVane"); hbuf.guess_segment_properties()
hb.shape(font, hbuf, {"kern": True, "liga": False})
S = CAP / tt["OS/2"].sCapHeight
RUN, x = [], 0  # (glyph outline, x in font units)
for info, pos in zip(hbuf.glyph_infos, hbuf.glyph_positions):
    rec = RecordingPen(); font.draw_glyph_with_pen(info.codepoint, rec)
    RUN.append((rec, x + pos.x_offset)); x += pos.x_advance + TRACK * upem

def wordmark_path(tx=0, base=0):
    ds = []
    for rec, gx in RUN:
        sp = SVGPathPen(None, ntos=f); rec.replay(TransformPen(sp, (S, 0, 0, -S, tx + gx * S, base))); ds.append(sp.getCommands())
    return " ".join(ds)

bp = BoundsPen(None)
for rec, gx in RUN: rec.replay(TransformPen(bp, (S, 0, 0, -S, gx * S, 0)))
x0, y0, x1, y1 = bp.bounds  # ink bounds with the baseline at y=0
W = x1 - x0
text = lambda fill, tx, base: f'<path fill="{fill}" fill-rule="nonzero" d="{wordmark_path(tx, base)}"/>'  # nonzero: Inter's variable outlines overlap

# 1. symbols
R = 112
sym = lambda g, side, r=R: crate(128, 128, r, g, side)
(out / "sysvane-symbol.svg").write_text(svg("SysVane", "0 0 256 256", sym(FACE_GAP[INK], INK)))
(out / "sysvane-symbol-reversed.svg").write_text(svg("SysVane (reversed)", "0 0 256 256", sym(FACE_GAP[WHITE], WHITE)))
(out / "sysvane-symbol-small.svg").write_text(svg("SysVane (small sizes)", "0 0 256 256", sym(20, INK, 120)))
(out / "sysvane-symbol-small-reversed.svg").write_text(svg("SysVane (small sizes, reversed)", "0 0 256 256", sym(22, WHITE, 120)))

# 2. app icon: ink tile, reversed crate
tile = '<rect x="0" y="0" width="256" height="256" rx="56" fill="#171717"/>'
(out / "sysvane-app-icon.svg").write_text(svg("SysVane app icon", "0 0 256 256", tile + crate(128, 128, 80, 11, WHITE)))
(out / "sysvane-app-icon-small.svg").write_text(svg("SysVane app icon (small sizes)", "0 0 256 256", tile + crate(128, 128, 92, 20, WHITE)))

# 3. wordmark and lockups
LOCKUP_GAP = 56  # between the cube and the wordmark
cube_l, cube_r = 128 - R * T / 2, 128 + R * T / 2
def horizontal(side):
    body = sym(FACE_GAP[side], side) + text(side, cube_r + LOCKUP_GAP - x0, 128 + CAP / 2)
    return svg("SysVane", f"{f(cube_l)} 0 {f(cube_r + LOCKUP_GAP + W - cube_l)} 256", body)

SC = 1.15  # stacked: crate on top, scaled, wordmark centred below
def stacked(side):
    ww = max(W, R * T * SC)
    base = 2 * R * SC + 44 + CAP
    body = crate(ww / 2, R * SC, R * SC, FACE_GAP[side], side) + text(side, (ww - W) / 2 - x0, base)
    return svg("SysVane", f"0 0 {f(ww)} {f(base + y1)}", body)

(out / "sysvane-wordmark.svg").write_text(svg("SysVane wordmark", f"0 {f(y0)} {f(W)} {f(y1 - y0)}", text(INK, -x0, 0)))
(out / "sysvane-horizontal.svg").write_text(horizontal(INK))
(out / "sysvane-horizontal-reversed.svg").write_text(horizontal(WHITE))
(out / "sysvane-stacked.svg").write_text(stacked(INK))
(out / "sysvane-stacked-reversed.svg").write_text(stacked(WHITE))
print(sorted(p.name for p in out.glob("*.svg")), "wordmark width", f(W))

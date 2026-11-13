"""Render the SysVane icon sets from the two app-icon tile SVGs in ../kit and write them where they are used:
desktop/src-tauri/icons (Windows and Store icons), core/assets/tray-32.rgba (tray) and ../kit/icons (web only).
Rendering goes through the logo-design skill's render_png.py (Chrome, rsvg, Inkscape or cairosvg).

    uv run --no-project python branding/logo/scripts/build_icons.py <skill scripts dir>
"""
import concurrent.futures, pathlib, shutil, struct, subprocess, sys, tempfile, zlib

render_png = pathlib.Path(sys.argv[1]) / "render_png.py"
here = pathlib.Path(__file__).resolve().parent
kit, root = here.parent / "kit", here.parents[2]
web, tauri, core = kit / "icons", root / "desktop/src-tauri/icons", root / "core/assets"
big, small = kit / "sysvane-app-icon.svg", kit / "sysvane-app-icon-small.svg"
src = lambda s: small if s <= 48 else big  # the small cut has wider gaps

NAMED = {  # folder -> {size: file name}
    tauri: {32: "32x32.png", 128: "128x128.png", 256: "128x128@2x.png", 512: "icon.png", 30: "Square30x30Logo.png",
            44: "Square44x44Logo.png", 50: "StoreLogo.png", 71: "Square71x71Logo.png", 89: "Square89x89Logo.png",
            107: "Square107x107Logo.png", 142: "Square142x142Logo.png", 150: "Square150x150Logo.png",
            284: "Square284x284Logo.png", 310: "Square310x310Logo.png"},
    web: {180: "apple-touch-icon.png", 192: "icon-192.png"},
}
ICO = {tauri / "icon.ico": (16, 24, 32, 48, 64, 128, 256), web / "favicon.ico": (16, 32, 48)}
sizes = sorted({s for m in NAMED.values() for s in m} | {s for v in ICO.values() for s in v})

def ico(entries):  # entries: [(size, png_bytes)]
    head = struct.pack("<HHH", 0, 1, len(entries))
    off = 6 + 16 * len(entries); dirs = b""; data = b""
    for size, png in entries:
        dirs += struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(png), off + len(data))
        data += png
    return head + dirs + data

def rgba(png_bytes):
    """Decode an 8-bit RGBA, non-interlaced PNG to raw RGBA (the core draws its tray icon without an image decoder)."""
    pos, idat = 8, b""
    while pos < len(png_bytes):
        n, kind = struct.unpack(">I4s", png_bytes[pos:pos + 8]); body = png_bytes[pos + 8:pos + 8 + n]; pos += 12 + n
        if kind == b"IHDR": w, h, depth, ctype, _, _, inter = struct.unpack(">IIBBBBB", body)
        elif kind == b"IDAT": idat += body
    assert (depth, ctype, inter) == (8, 6, 0), "expected 8-bit RGBA, non-interlaced"
    raw, stride, prev, out = zlib.decompress(idat), w * 4, bytearray(w * 4), bytearray()
    for y in range(h):
        ft, row = raw[y * (stride + 1)], bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a, b, c = (row[i - 4] if i >= 4 else 0), prev[i], (prev[i - 4] if i >= 4 else 0)
            p = a + b - c; pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
            row[i] = (row[i] + (0, a, b, (a + b) // 2, a if pa <= pb and pa <= pc else b if pb <= pc else c)[ft]) & 255
        out += row; prev = row
    return bytes(out)

with tempfile.TemporaryDirectory() as tmp:
    def render(s):
        d = pathlib.Path(tmp) / str(s); d.mkdir()
        subprocess.run([sys.executable, str(render_png), str(src(s)), "--out-dir", str(d), "--size", str(s)], check=True, capture_output=True)
        return (d / f"{src(s).stem}.png").read_bytes()
    with concurrent.futures.ThreadPoolExecutor(4) as pool:
        png = dict(zip(sizes, pool.map(render, sizes)))

for d in (web, tauri, core): d.mkdir(parents=True, exist_ok=True)
for folder, names in NAMED.items():
    for s, name in names.items(): (folder / name).write_bytes(png[s])
for target, ss in ICO.items(): target.write_bytes(ico([(s, png[s]) for s in ss]))
shutil.copy(small, web / "favicon.svg")
(core / "tray-32.rgba").write_bytes(rgba(png[32]))
print("wrote", len(sizes), "sizes to", tauri, web, core)

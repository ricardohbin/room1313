#!/usr/bin/env python3
"""Cache-busting for the web build.

Run after wasm-pack: renames www/pkg/room_1313{.js,_bg.wasm} to carry a content
hash, points the JS at the renamed wasm, deletes stale files from earlier builds,
and renders www/index.html from www/index.template.html with the new names.
Browsers (and CDNs) then can't serve an old game: only index.html is unversioned.
"""
import hashlib
import pathlib
import sys

root = pathlib.Path(__file__).resolve().parent.parent
www, pkg = root / "www", root / "www" / "pkg"


def digest(*paths):
    h = hashlib.sha256()
    for p in paths:
        h.update(p.read_bytes())
    return h.hexdigest()[:10]


wasm, js = pkg / "room_1313_bg.wasm", pkg / "room_1313.js"
if not wasm.exists() or not js.exists():
    sys.exit("stamp: run wasm-pack first (www/pkg/room_1313*.* not found)")

tag = digest(wasm, js)
new_wasm, new_js = f"room_1313-{tag}_bg.wasm", f"room_1313-{tag}.js"
code = js.read_text(encoding="utf-8")
if "room_1313_bg.wasm" not in code:
    sys.exit("stamp: wasm filename not found in the generated JS")
(pkg / new_js).write_text(code.replace("room_1313_bg.wasm", new_wasm), encoding="utf-8")
(pkg / new_wasm).write_bytes(wasm.read_bytes())

for f in pkg.iterdir():
    if f.name not in (new_js, new_wasm):
        f.unlink()

html = (www / "index.template.html").read_text(encoding="utf-8")
html = html.replace("@JS@", new_js).replace("@STYLE@", digest(www / "style.css"))
(www / "index.html").write_text(html, encoding="utf-8")
print(f"stamped {new_js}")

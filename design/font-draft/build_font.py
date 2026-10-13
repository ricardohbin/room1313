"""Build the Room 13 font from glyphs.py.

    .venv/bin/python build_font.py

Outputs room13.otf (desktop install) here and www/fonts/room13.woff2 (the game).
Kerning is generated from the outlines: each pair is nudged so its tightest gap
matches the gap between two straight stems (see `kerning`).
Overlapping parts are merged with skia-pathops. Glyphs with eyes get a COLRv0
layer so the eye stays gold whatever colour the text is set in; renderers without
colour-font support fall back to the plain merged outline.
"""

from pathlib import Path

import pathops
from fontTools.agl import UV2AGL
from fontTools.colorLib.builder import buildCOLR, buildCPAL
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.basePen import BasePen
from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.t2CharStringPen import T2CharStringPen
from fontTools.pens.transformPen import TransformPen
from fontTools.svgLib.path import parse_path

from glyphs import G, SPACE

HERE = Path(__file__).parent
ROOT = HERE.parent.parent
FAMILY = "Room 13"
PS_NAME = "Room13-Regular"
BASELINE = 850  # SVG y of the baseline; font y = BASELINE - svg y
EYE_GOLD = (0xC9 / 255, 0xA2 / 255, 0x27 / 255, 1.0)
# typographic variants that should reuse our glyphs
ALIASES = {"\u2018": "'", "\u2019": "'", "\u201c": '"', "\u201d": '"', "\u2013": "-", "\u2014": "-"}
STRAIGHT = "HIMNU"  # glyphs whose stem-to-stem gap defines the target spacing
KERN_BAND = 10  # scanline step, font units
KERN_REACH = 3  # bands above/below a gap may borrow from (diagonals)
KERN_STRENGTH = 0.7
KERN_MIN, KERN_CLAMP = 15, (-200, 80)


def glyph_name(ch):
    return ch if ch.isalpha() else UV2AGL[ord(ch)]


def to_path(parts):
    """Union SVG parts (each with its own fill rule) into one clean pathops.Path."""
    builder = pathops.OpBuilder(fix_winding=True, keep_starting_points=False)
    for d, opts in parts:
        p = pathops.Path()
        parse_path(d, TransformPen(p.getPen(), (1, 0, 0, -1, 0, BASELINE)))
        if opts.get("rule") == "evenodd":
            p.fillType = pathops.FillType.EVEN_ODD
        builder.add(p, pathops.PathOp.UNION)
    return builder.resolve()


def charstring(path, width):
    pen = T2CharStringPen(width, None)
    path.draw(pen)
    return pen.getCharString()


def lsb(path):
    pen = BoundsPen(None)
    path.draw(pen)
    return round(pen.bounds[0]) if pen.bounds else 0


class EdgePen(BasePen):
    """Flattens an outline into line segments."""

    def __init__(self):
        super().__init__(None)
        self.segments = []

    def _moveTo(self, pt):
        self.start = self.last = pt

    def _lineTo(self, pt):
        self.segments.append((self.last, pt))
        self.last = pt

    def _curveToOne(self, p1, p2, p3):
        p0 = self.last
        for i in range(1, 9):
            t = i / 8
            u = 1 - t
            self._lineTo((u**3 * p0[0] + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t**3 * p3[0],
                          u**3 * p0[1] + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t**3 * p3[1]))

    def _closePath(self):
        if self.last != self.start:
            self._lineTo(self.start)


def profile(path):
    """{band: (leftmost x, rightmost x)} of the ink on each horizontal scanline."""
    pen = EdgePen()
    path.draw(pen)
    bands = {}
    for band in range(-250, 1000, KERN_BAND):
        y = band + 0.5
        xs = [x0 + (y - y0) * (x1 - x0) / (y1 - y0)
              for (x0, y0), (x1, y1) in pen.segments if (y0 <= y < y1) or (y1 <= y < y0)]
        if xs:
            bands[band] = (min(xs), max(xs))
    return bands


def tight_gap(left, right):
    """Mean of the three smallest gaps between `left` (with advance) and `right`, or None."""
    (adv, lp), (_, rp) = left, right
    gaps = []
    for band, (_, rmax) in lp.items():
        near = [rp[b][0] for b in range(band - KERN_REACH * KERN_BAND, band + (KERN_REACH + 1) * KERN_BAND, KERN_BAND) if b in rp]
        if near:
            gaps.append(adv - rmax + min(near))
    if not gaps:
        return None
    gaps.sort()
    return sum(gaps[:3]) / len(gaps[:3])


def kerning(outlines, names):
    profiles = {n: (outlines[n][0], profile(outlines[n][1])) for n in names}
    straight = [tight_gap(profiles[a], profiles[b]) for a in STRAIGHT for b in STRAIGHT]
    target = sorted(straight)[len(straight) // 2]
    pairs = {}
    for a in names:
        for b in names:
            gap = tight_gap(profiles[a], profiles[b])
            if gap is None:
                continue
            k = round((target - gap) * KERN_STRENGTH / 5) * 5
            k = max(KERN_CLAMP[0], min(KERN_CLAMP[1], k))
            if abs(k) >= KERN_MIN:
                pairs[(a, b)] = k
    return target, pairs


def build():
    outlines = {}  # glyph name -> (advance, pathops.Path)
    layers = {}

    outlines[".notdef"] = (500, to_path([("M50 150 L450 150 L450 850 L50 850 Z M100 200 L100 800 L400 800 L400 200 Z",
                                          {"rule": "evenodd"})]))
    outlines["space"] = (SPACE, pathops.Path())

    for ch, (adv, parts) in G.items():
        name = glyph_name(ch)
        outlines[name] = (adv, to_path(parts))
        eyes = [p for p in parts if p[1].get("cls") == "eye"]
        if eyes:
            body = [p for p in parts if p[1].get("cls") != "eye"]
            outlines[f"{name}.body"] = (adv, to_path(body))
            outlines[f"{name}.eye"] = (adv, to_path(eyes))
            layers[name] = [(f"{name}.body", 0xFFFF), (f"{name}.eye", 0)]  # 0xFFFF = text colour

    order = list(outlines)
    cmap = {ord(" "): "space"}
    for ch in G:
        cmap[ord(ch)] = glyph_name(ch)
        cmap[ord(ch.lower())] = glyph_name(ch)  # lowercase renders as caps
    for alias, ch in ALIASES.items():
        cmap[ord(alias)] = glyph_name(ch)

    target, pairs = kerning(outlines, [glyph_name(ch) for ch in G])

    fb = FontBuilder(1000, isTTF=False)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap(cmap)
    fb.setupCFF(PS_NAME, {"FullName": FAMILY, "FamilyName": FAMILY},
                {name: charstring(p, adv) for name, (adv, p) in outlines.items()}, {})
    fb.setupHorizontalMetrics({name: (adv, lsb(p)) for name, (adv, p) in outlines.items()})
    fb.setupHorizontalHeader(ascent=930, descent=-200)
    fb.setupNameTable({"familyName": FAMILY, "styleName": "Regular"})
    fb.setupOS2(sTypoAscender=930, sTypoDescender=-200, sTypoLineGap=0,
                usWinAscent=930, usWinDescent=200, sCapHeight=700, sxHeight=700)
    fb.setupPost()
    fb.addOpenTypeFeatures("feature kern {\n"
                           + "".join(f"  pos {a} {b} {k};\n" for (a, b), k in sorted(pairs.items()))
                           + "} kern;\n")
    fb.font["COLR"] = buildCOLR(layers, version=0)
    fb.font["CPAL"] = buildCPAL([[EYE_GOLD]])

    otf = HERE / "room13.otf"
    fb.save(otf)
    woff2 = ROOT / "www" / "fonts" / "room13.woff2"
    woff2.parent.mkdir(exist_ok=True)
    fb.font.flavor = "woff2"
    fb.font.save(woff2)
    print(f"wrote {otf} and {woff2}: {len(G)} glyphs, {len(layers)} colour, "
          f"{len(pairs)} kern pairs (target gap {target:.0f})")
    return pairs


if __name__ == "__main__":
    build()

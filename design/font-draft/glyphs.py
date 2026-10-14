"""Room 13 display font — glyph source.

Em = 1000 units, SVG coords (y down). Cap height y=150, baseline y=850.
Each glyph is (advance_width, [parts]); a part is (path_d, opts) where opts may set
"rule": "evenodd" (counters/holes) or "cls": "eye" (accent colour layer in the font).

Build the font with `.venv/bin/python build_font.py`.
"""

import math
import unicodedata

from fontTools.pens.boundsPen import BoundsPen
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.svgLib.path import parse_path


def f(v):
    return f"{v:.0f}" if abs(v - round(v)) < 0.05 else f"{v:.1f}"


def poly(*pts):
    return "M" + " L".join(f"{f(x)} {f(y)}" for x, y in pts) + " Z"


def circle(cx, cy, r):
    return f"M{f(cx - r)} {f(cy)} a{f(r)} {f(r)} 0 1 0 {f(2 * r)} 0 a{f(r)} {f(r)} 0 1 0 {f(-2 * r)} 0 Z"


def ellipse(cx, cy, rx, ry):
    return f"M{f(cx - rx)} {f(cy)} a{f(rx)} {f(ry)} 0 1 0 {f(2 * rx)} 0 a{f(rx)} {f(ry)} 0 1 0 {f(-2 * rx)} 0 Z"


def stem(x0=100, top=True, bottom=True, y_top=None):
    """Chiselled blackletter stem, 92 units wide, starting at x0."""
    d = x0 - 100
    pts = []
    if top and y_top is None:
        pts += [(100 + d, 200), (150 + d, 140), (198 + d, 178)]
    else:
        yt = y_top or 200
        pts += [(100 + d, yt), (192 + d, yt)]
    if bottom:
        pts += [(192 + d, 820), (232 + d, 864), (70 + d, 864), (100 + d, 820)]
    else:
        pts += [(192 + d, 864), (100 + d, 864)]
    return (poly(*pts), {})


def drip(x, y, length=80, w=26):
    h = w / 2
    return (
        f"M{f(x - h)} {f(y)} L{f(x + h)} {f(y)} "
        f"C{f(x + h + 2)} {f(y + length * 0.5)} {f(x + w * 0.9)} {f(y + length * 0.78)} {f(x)} {f(y + length)} "
        f"C{f(x - w * 0.9)} {f(y + length * 0.78)} {f(x - h - 2)} {f(y + length * 0.5)} {f(x - h)} {f(y)} Z",
        {},
    )


def eye(cx, cy, w, h):
    """Almond eye with a vertical slit pupil."""
    return (
        f"M{f(cx - w / 2)} {f(cy)} Q{f(cx)} {f(cy - h)} {f(cx + w / 2)} {f(cy)} "
        f"Q{f(cx)} {f(cy + h)} {f(cx - w / 2)} {f(cy)} Z "
        f"M{f(cx)} {f(cy - h * 0.58)} Q{f(cx + w * 0.07)} {f(cy)} {f(cx)} {f(cy + h * 0.58)} "
        f"Q{f(cx - w * 0.07)} {f(cy)} {f(cx)} {f(cy - h * 0.58)} Z",
        {"rule": "evenodd", "cls": "eye"},
    )


def skull(cx, cy, s=1.0):
    """Skull ~100w x 120h at s=1, centred on (cx, cy). Sockets/nose/teeth are holes."""
    def p(x, y):
        return f"{f(cx + x * s)} {f(cy + y * s)}"

    d = (
        f"M{p(-50, -5)} C{p(-50, -72)} {p(50, -72)} {p(50, -5)} "
        f"C{p(50, 20)} {p(40, 28)} {p(32, 32)} L{p(32, 60)} L{p(-32, 60)} L{p(-32, 32)} "
        f"C{p(-40, 28)} {p(-50, 20)} {p(-50, -5)} Z "
    )
    d += circle(cx - 20 * s, cy, 14 * s) + " " + circle(cx + 20 * s, cy, 14 * s) + " "
    d += poly((cx, cy + 14 * s), (cx - 8 * s, cy + 28 * s), (cx + 8 * s, cy + 28 * s)) + " "
    for tx in (-16, 0, 16):
        d += poly((cx + (tx - 3) * s, cy + 38 * s), (cx + (tx + 3) * s, cy + 38 * s),
                  (cx + (tx + 3) * s, cy + 54 * s), (cx + (tx - 3) * s, cy + 54 * s)) + " "
    return (d.strip(), {"rule": "evenodd"})


def bat(cx, cy, s=1.0):
    """Spread-wing bat, 400w at s=1, scalloped wing edges."""
    def p(x, y):
        return f"{f(cx + x * s)} {f(cy + y * s)}"

    top = [(-200, -20), (-150, -55), (-90, -48), (-45, -28), (-24, -32), (-22, -68), (-8, -40),
           (8, -40), (22, -68), (24, -32), (45, -28), (90, -48), (150, -55), (200, -20)]
    d = "M" + " L".join(p(*q) for q in top)
    d += f" Q{p(165, 0)} {p(140, 30)} Q{p(112, 5)} {p(80, 40)} Q{p(55, 15)} {p(28, 42)}"
    d += f" L{p(14, 72)} L{p(0, 92)} L{p(-14, 72)} L{p(-28, 42)}"
    d += f" Q{p(-55, 15)} {p(-80, 40)} Q{p(-112, 5)} {p(-140, 30)} Q{p(-165, 0)} {p(-200, -20)} Z"
    return (d, {})


def hanging_bat(x, y, s=1.0):
    """Folded bat hanging upside down from (x, y)."""
    def p(dx, dy):
        return f"{f(x + dx * s)} {f(y + dy * s)}"

    d = (f"M{p(-8, 0)} L{p(8, 0)} L{p(14, 10)} C{p(40, 30)} {p(42, 80)} {p(26, 112)} "
         f"L{p(18, 130)} L{p(8, 116)} L{p(-8, 116)} L{p(-18, 130)} L{p(-26, 112)} "
         f"C{p(-42, 80)} {p(-40, 30)} {p(-14, 10)} Z")
    return (d, {})


def centipede(p1, p2, n, r, leg, head=True):
    """Beaded centipede from tail p1 to head p2. Returns separate parts (union on render)."""
    (x1, y1), (x2, y2) = p1, p2
    length = math.hypot(x2 - x1, y2 - y1)
    ux, uy = (x2 - x1) / length, (y2 - y1) / length
    nx, ny = -uy, ux
    parts = []
    for i in range(n):
        t = i / (n - 1)
        cx, cy = x1 + (x2 - x1) * t, y1 + (y2 - y1) * t
        is_head = head and i == n - 1
        parts.append((circle(cx, cy, r * (1.15 if is_head else 1.0)), {}))
        if is_head:
            for side in (1, -1):
                bx, by = cx + ux * r * 0.8, cy + uy * r * 0.8
                tx = cx + ux * leg * 1.9 + nx * side * leg * 0.9
                ty = cy + uy * leg * 1.9 + ny * side * leg * 0.9
                parts.append((poly((bx - nx * 4, by - ny * 4), (bx + nx * 4, by + ny * 4), (tx, ty)), {}))
            continue
        for side in (1, -1):
            def q(a, b):
                return (cx + nx * side * a + ux * b, cy + ny * side * a + uy * b)
            parts.append((poly(q(r * 0.5, -6), q(r * 0.5, 6), q(r + leg * 0.55, 6),
                               q(r + leg, -10), q(r + leg * 0.55, -4)), {}))
    if head:  # tail pincers
        for side in (1, -1):
            bx, by = x1 - ux * r * 0.6, y1 - uy * r * 0.6
            tx = x1 - ux * leg * 1.3 + nx * side * leg * 0.5
            ty = y1 - uy * leg * 1.3 + ny * side * leg * 0.5
            parts.append((poly((bx - nx * 5, by - ny * 5), (bx + nx * 5, by + ny * 5), (tx, ty)), {}))
    return parts


def transform(d, matrix):
    """Apply an affine (a, b, c, d, e, f) matrix to an SVG path string."""
    pen = SVGPathPen(None)
    parse_path(d, TransformPen(pen, matrix))
    return pen.getCommands()


def blood_drop(cx, bottom, h=150, w=100):
    """Teardrop, point up, resting on `bottom`."""
    r = w / 2
    top = bottom - h
    return (f"M{f(cx)} {f(top)} C{f(cx + r * 0.4)} {f(top + h * 0.27)} {f(cx + r)} {f(bottom - r * 1.4)} "
            f"{f(cx + r)} {f(bottom - r)} C{f(cx + r)} {f(bottom - r * 0.45)} {f(cx + r * 0.55)} {f(bottom)} "
            f"{f(cx)} {f(bottom)} C{f(cx - r * 0.55)} {f(bottom)} {f(cx - r)} {f(bottom - r * 0.45)} "
            f"{f(cx - r)} {f(bottom - r)} C{f(cx - r)} {f(bottom - r * 1.4)} {f(cx - r * 0.4)} {f(top + h * 0.27)} "
            f"{f(cx)} {f(top)} Z", {})


def fang(x, top=140, w=90, h=200):
    return (f"M{f(x)} {f(top)} L{f(x + w)} {f(top)} C{f(x + w * 0.95)} {f(top + h * 0.4)} "
            f"{f(x + w * 0.66)} {f(top + h * 0.7)} {f(x + w * 0.4)} {f(top + h)} "
            f"C{f(x + w * 0.35)} {f(top + h * 0.65)} {f(x + w * 0.17)} {f(top + h * 0.35)} {f(x)} {f(top)} Z", {})


def spline(pts, steps=10):
    """Catmull-Rom curve through `pts`, sampled into a polyline."""
    ext = [pts[0]] + list(pts) + [pts[-1]]
    out = []
    for i in range(1, len(ext) - 2):
        p0, p1, p2, p3 = ext[i - 1], ext[i], ext[i + 1], ext[i + 2]
        for j in range(steps):
            t = j / steps
            out.append(tuple(0.5 * (2 * p1[k] + (-p0[k] + p2[k]) * t + (2 * p0[k] - 5 * p1[k] + 4 * p2[k] - p3[k]) * t * t
                                    + (-p0[k] + 3 * p1[k] - 3 * p2[k] + p3[k]) * t ** 3) for k in (0, 1)))
    out.append(pts[-1])
    return out


def nib_stroke(line, thin=34, thick=96, nib=math.radians(35)):
    """Broad-nib pen along a polyline: segment quads plus round joints (union on render)."""
    parts = []
    widths = []
    for i, (x, y) in enumerate(line):
        (ax, ay), (bx, by) = line[max(i - 1, 0)], line[min(i + 1, len(line) - 1)]
        ang = math.atan2(by - ay, bx - ax)
        widths.append(thin + (thick - thin) * abs(math.sin(ang - nib)))
    for i in range(len(line) - 1):
        (x0, y0), (x1, y1) = line[i], line[i + 1]
        seg = math.hypot(x1 - x0, y1 - y0) or 1
        nx, ny = -(y1 - y0) / seg, (x1 - x0) / seg
        w0, w1 = widths[i] / 2, widths[i + 1] / 2
        parts.append((poly((x0 + nx * w0, y0 + ny * w0), (x1 + nx * w1, y1 + ny * w1),
                           (x1 - nx * w1, y1 - ny * w1), (x0 - nx * w0, y0 - ny * w0)), {}))
    for (x, y), w in zip(line[1:-1], widths[1:-1]):
        parts.append((circle(x, y, w / 2), {}))
    return parts


# Shared pieces --------------------------------------------------------------

TENTACLE_FLAG = ("M150 140 C108 108 58 124 52 172 C48 204 78 222 98 206 C80 200 74 180 92 166 "
                 "C110 154 132 162 146 178 Z", {})

UPPER_BOWL = ("M195 176 L380 140 C470 150 502 230 482 300 L432 450 L195 470 L195 420 L380 410 "
              "L410 300 C420 240 400 205 360 200 L195 226 Z", {})

SUCKERS_C = ("M119 560 a16 16 0 1 0 32 0 a16 16 0 1 0 -32 0 Z "
             "M128 640 a13 13 0 1 0 26 0 a13 13 0 1 0 -26 0 Z "
             "M147 712 a10 10 0 1 0 20 0 a10 10 0 1 0 -20 0 Z")

O_RING = ellipse(330, 505, 265, 360) + " " + ellipse(330, 505, 165, 272)


# Glyphs ---------------------------------------------------------------------

G = {}

G["A"] = (660, [
    ("M330 70 L402 190 L548 800 L630 862 L412 862 L455 815 L318 250 Z", {}),
    ("M296 205 L326 255 L178 800 C162 852 108 872 72 846 C46 826 58 786 90 792 "
     "C72 812 94 836 120 826 C142 816 146 790 152 768 L280 228 Z", {}),
    (poly((296, 205), (238, 160), (312, 180)), {}),
    ("M198 548 L398 520 L408 576 L304 592 L290 650 C292 690 300 712 284 728 "
     "C268 712 276 690 278 650 L262 598 L194 608 Z", {}),
])

G["B"] = (640, [
    stem(), TENTACLE_FLAG, UPPER_BOWL,
    ("M195 470 L430 450 C540 470 580 560 560 650 L520 790 C500 840 450 862 400 862 "
     "L195 862 L195 810 L400 805 C440 800 455 780 465 740 L490 640 C500 560 470 515 "
     "420 515 L195 522 Z", {}),
    (poly((552, 610), (628, 642), (548, 684)), {}),
])

G["C"] = (580, [
    ("M480 230 L410 140 L270 145 C140 170 85 300 85 500 C85 700 150 830 290 860 "
     "C390 880 470 840 510 770 C530 735 525 700 500 695 C480 692 470 712 485 725 "
     "C490 740 470 780 430 795 C380 810 320 800 270 770 C210 730 185 630 185 500 "
     "C185 340 220 250 290 225 L380 205 Z " + SUCKERS_C, {"rule": "evenodd"}),
    (poly((398, 150), (505, 86), (472, 205)), {}),
])

G["D"] = (640, [
    stem(),
    (poly((112, 186), (40, 150), (96, 232)), {}),
    ("M195 160 L400 150 C530 170 580 290 580 500 C580 700 520 830 390 862 L195 862 "
     "L195 810 L380 800 C450 780 480 680 480 500 C480 320 450 230 380 215 L195 215 Z", {}),
    eye(340, 510, 180, 110),
])

G["E"] = (540, [
    stem(),
    ("M195 150 L430 135 L488 86 L470 232 L420 205 L195 215 Z", {}),
    ("M195 448 C270 430 352 440 396 476 C432 506 418 556 380 556 C350 556 342 524 362 512 "
     "C336 500 280 506 195 534 Z", {}),
    ("M195 796 L430 786 L500 756 L472 864 L195 864 Z", {}),
    drip(309, 860, 82),
])

# F: top arm frayed into bat-wing scallops
G["F"] = (540, [
    stem(), TENTACLE_FLAG,
    ("M195 150 L430 135 L488 86 L475 232 Q445 196 400 212 Q366 188 330 222 "
     "Q296 190 260 214 Q230 196 195 226 Z", {}),
    (poly((195, 462), (372, 452), (424, 420), (404, 522), (195, 534)), {}),
])

G["G"] = (620, [
    ("M480 230 L410 140 L270 145 C140 170 85 300 85 500 C85 700 150 830 290 860 "
     "C380 876 450 850 505 800 L505 575 L412 575 L412 762 C380 792 330 800 280 770 "
     "C210 730 185 630 185 500 C185 340 220 250 290 225 L380 205 Z " + SUCKERS_C,
     {"rule": "evenodd"}),
    (poly((398, 150), (505, 86), (472, 205)), {}),
    (poly((300, 515), (522, 508), (596, 466), (566, 590), (300, 580)), {}),
    drip(458, 840, 70, 24),
])

# H: crossbar is a centipede crawling out to the right
G["H"] = (720, [
    stem(), stem(470),
    *centipede((150, 495), (630, 495), 8, 46, 40),
])

# I: crowned with a skull
G["I"] = (300, [
    (poly((100, 330), (192, 330), (192, 820), (232, 864), (70, 864), (100, 820)), {}),
    skull(146, 240, 1.65),
])

G["J"] = (480, [
    ("M250 200 L300 140 L348 178 L342 820 C340 920 290 975 210 975 C130 975 90 910 110 860 "
     "C125 820 175 818 185 850 C190 872 168 885 155 872 C150 905 180 925 215 922 "
     "C245 920 255 880 255 820 Z", {}),
    (poly((262, 192), (196, 160), (252, 244)), {}),
])

G["K"] = (640, [
    stem(), TENTACLE_FLAG,
    (poly((195, 440), (470, 162), (566, 112), (532, 206), (262, 500), (195, 522)), {}),
    (poly((300, 470), (524, 800), (608, 864), (404, 864), (446, 822), (240, 520)), {}),
])

G["L"] = (520, [
    stem(), TENTACLE_FLAG,
    (poly((195, 796), (430, 786), (500, 740), (480, 864), (195, 864)), {}),
    drip(320, 860, 90),
])

# M: a bat hangs from the valley
G["M"] = (820, [
    stem(), stem(640),
    (poly((172, 175), (250, 165), (440, 590), (410, 650)), {}),
    (poly((385, 598), (620, 168), (646, 200), (410, 650)), {}),
    hanging_bat(410, 636, 1.5),
])

G["N"] = (680, [
    stem(), stem(490),
    ("M160 150 L250 160 L582 800 L560 864 L476 840 L170 262 Z "
     + circle(318, 446, 13) + " " + circle(368, 540, 11) + " " + circle(414, 628, 9),
     {"rule": "evenodd"}),
])

# O: the great eye
G["O"] = (660, [
    (O_RING, {"rule": "evenodd"}),
    (poly((150, 230), (110, 120), (230, 170)), {}),
    (poly((510, 780), (550, 890), (430, 840)), {}),
    eye(330, 505, 290, 170),
])

G["P"] = (600, [
    stem(), TENTACLE_FLAG,
    ("M195 160 L400 150 C510 165 545 240 535 330 C525 430 470 520 400 530 L195 540 "
     "L195 485 L380 475 C420 470 440 420 440 340 C440 260 420 215 380 212 L195 215 Z", {}),
    (poly((528, 300), (598, 330), (526, 372)), {}),
    drip(146, 860, 90),
])

# Q: the O's tail is a centipede
G["Q"] = (680, [
    (O_RING, {"rule": "evenodd"}),
    (poly((150, 230), (110, 120), (230, 170)), {}),
    *centipede((400, 790), (640, 975), 7, 27, 28),
])

G["R"] = (640, [
    stem(), TENTACLE_FLAG, UPPER_BOWL,
    (poly((290, 455), (400, 450), (540, 800), (620, 864), (420, 864), (458, 822), (330, 500)), {}),
])

G["S"] = (580, [
    ("M500 110 L470 240 L400 215 C330 200 220 210 215 300 C212 360 260 395 330 430 L420 480 "
     "C510 530 540 600 520 690 C495 800 400 865 280 865 C180 865 110 820 90 760 "
     "C80 730 95 700 120 705 C140 710 140 735 125 740 C140 770 200 800 270 800 "
     "C350 800 420 770 425 700 C428 640 380 600 320 570 L230 525 C140 480 100 420 110 330 "
     "C120 220 220 150 330 145 L420 140 Z", {}),
])

# T: the crossbar is a bat with its wings spread
G["T"] = (640, [
    (poly((274, 290), (366, 290), (362, 820), (402, 864), (240, 864), (278, 820)), {}),
    bat(320, 205, 1.42),
])

G["U"] = (680, [
    ("M100 200 L150 140 L198 178 L195 700 C195 770 240 800 300 800 C380 800 470 760 470 680 "
     "L470 200 L520 140 L568 178 L565 820 L605 864 L470 864 L470 800 C430 845 370 868 290 868 "
     "C170 868 100 800 100 700 Z", {}),
    drip(290, 862, 80),
])

G["V"] = (640, [
    (poly((40, 150), (95, 118), (150, 150), (356, 790), (320, 880)), {}),
    (poly((535, 152), (606, 132), (340, 862), (322, 792)), {}),
    drip(322, 870, 80, 22),
])

G["W"] = (940, [
    (poly((40, 150), (95, 118), (150, 150), (356, 790), (320, 880)), {}),
    (poly((440, 330), (500, 330), (340, 862), (322, 792)), {}),
    (poly((440, 330), (500, 330), (656, 790), (620, 880)), {}),
    (poly((835, 152), (906, 132), (640, 862), (622, 792)), {}),
    drip(320, 870, 70, 20),
    drip(620, 870, 90, 22),
])

G["X"] = (660, [
    (poly((50, 150), (130, 140), (590, 820), (628, 864), (510, 864), (60, 195)), {}),
    ("M560 140 L600 165 L180 800 C160 850 110 870 75 846 C50 826 60 788 90 792 "
     "C72 812 94 836 120 826 C140 816 150 790 160 770 Z", {}),
])

# Y: a skull sits where the arms meet
G["Y"] = (640, [
    (poly((50, 160), (100, 130), (150, 150), (290, 368), (238, 400)), {}),
    (poly((540, 150), (600, 140), (400, 378), (362, 352)), {}),
    (poly((274, 538), (366, 538), (362, 820), (402, 864), (240, 864), (278, 820)), {}),
    skull(320, 445, 1.6),
])

# Z: the diagonal is a centipede
G["Z"] = (600, [
    (poly((80, 150), (470, 140), (540, 95), (520, 230), (470, 212), (80, 218)), {}),
    (poly((70, 790), (520, 784), (568, 752), (548, 864), (60, 864)), {}),
    *centipede((118, 790), (470, 212), 9, 42, 26, head=False),
])


# Digits ---------------------------------------------------------------------

G["0"] = (600, [
    (ellipse(300, 505, 215, 360) + " " + ellipse(300, 505, 118, 272), {"rule": "evenodd"}),
    (poly((300, 395), (334, 505), (300, 615), (266, 505)), {}),
    (poly((130, 240), (96, 130), (205, 175)), {}),
])

G["1"] = (400, [
    stem(160), ("M210 140 C168 108 118 124 112 172 C108 204 138 222 158 206 "
                                   "C140 200 134 180 152 166 C170 154 192 162 206 178 Z", {}),
    (poly((160, 250), (72, 330), (84, 356), (160, 312)), {}),
])

G["2"] = (580, [
    ("M95 330 C80 290 95 240 130 205 C170 165 230 145 300 145 C410 145 485 215 485 320 "
     "C485 420 420 490 330 570 L195 790 L420 786 L490 750 L470 864 L80 864 L80 830 L260 580 "
     "C340 480 385 410 385 320 C385 250 350 205 295 205 C240 205 200 240 190 290 "
     "C210 290 225 305 222 325 C218 350 190 362 165 355 C130 350 105 345 95 330 Z", {}),
    drip(300, 860, 80),
])

G["3"] = (570, [
    ("M90 250 C120 180 200 145 290 145 C400 145 470 210 470 300 C470 370 430 420 370 450 "
     "C450 480 500 550 500 640 C500 780 400 865 270 865 C170 865 100 820 80 760 "
     "C75 730 95 705 125 710 C150 715 155 740 140 750 C160 790 210 805 260 805 "
     "C350 805 400 740 400 645 C400 560 350 500 260 495 L240 495 L240 440 L270 440 "
     "C340 430 375 380 375 310 C375 240 340 205 285 205 C220 205 180 230 160 270 Z", {}),
    (poly((440, 210), (520, 120), (500, 260)), {}),
])

G["4"] = (600, [
    (poly((330, 200), (380, 140), (428, 178), (422, 820), (462, 864), (300, 864), (330, 820)), {}),
    (poly((360, 160), (398, 205), (138, 600), (70, 600)), {}),
    (poly((60, 600), (500, 590), (556, 556), (534, 668), (60, 660)), {}),
])

G["5"] = (570, [
    (poly((150, 150), (420, 140), (480, 95), (460, 230), (420, 210), (200, 215)), {}),
    (poly((120, 150), (205, 150), (200, 440), (115, 470)), {}),
    ("M115 470 C180 420 240 410 290 410 C420 410 500 500 500 630 C500 770 400 865 270 865 "
     "C170 865 100 820 80 760 C75 730 95 705 125 710 C150 715 155 740 140 750 "
     "C160 790 210 805 260 805 C350 805 400 740 400 640 C400 540 350 470 280 470 "
     "C230 470 190 490 150 520 Z", {}),
])

SIX = ("M440 200 L400 145 C200 150 90 320 90 560 C90 760 170 865 300 865 C430 865 510 770 510 650 "
       "C510 530 430 450 320 450 C270 450 220 465 190 495 C200 330 260 215 370 205 Z "
       "M190 650 C190 560 240 505 300 505 C370 505 410 560 410 650 C410 750 370 805 300 805 "
       "C230 805 190 740 190 650 Z")

G["6"] = (600, [
    (SIX, {"rule": "evenodd"}),
    (poly((390, 150), (482, 88), (452, 215)), {}),
])

G["7"] = (560, [
    (poly((80, 150), (470, 140), (522, 106), (510, 222), (80, 218)), {}),
    (poly((420, 215), (510, 215), (262, 864), (150, 864), (196, 820)), {}),
    drip(214, 862, 90),
])

G["8"] = (600, [
    (ellipse(300, 318, 165, 175) + " " + ellipse(300, 318, 80, 110), {"rule": "evenodd"}),
    (ellipse(300, 650, 200, 215) + " " + ellipse(300, 650, 102, 140), {"rule": "evenodd"}),
])

# 9 is the 6 turned upside down
G["9"] = (600, [
    (transform(SIX, (-1, 0, 0, -1, 600, 1010)), {"rule": "evenodd"}),
    (transform(poly((390, 150), (482, 88), (452, 215)), (-1, 0, 0, -1, 600, 1010)), {}),
])

# Punctuation ----------------------------------------------------------------

G["."] = (300, [blood_drop(150, 866)])

G[","] = (300, [(poly((150, 736), (206, 792), (166, 934), (118, 946), (138, 852), (94, 792)), {})])

G[":"] = (300, [blood_drop(150, 560), blood_drop(150, 866)])

G[";"] = (300, [blood_drop(150, 560),
                (poly((150, 736), (206, 792), (166, 934), (118, 946), (138, 852), (94, 792)), {})])

G["!"] = (310, [
    (poly((105, 190), (155, 135), (205, 185), (172, 640), (138, 640)), {}),
    blood_drop(155, 866, 130, 90),
])

G["?"] = (540, [
    ("M90 260 C110 180 190 140 280 140 C400 140 470 210 470 300 C470 400 400 450 320 490 "
     "L290 650 L250 650 L235 460 C320 420 370 380 370 300 C370 240 330 200 280 200 "
     "C230 200 190 230 175 270 C195 275 205 295 200 315 C192 340 160 350 135 340 "
     "C105 330 88 300 90 260 Z", {}),
    blood_drop(270, 866, 130, 90),
])

G["\u2026"] = (720, [blood_drop(130, 866), blood_drop(360, 866), blood_drop(590, 866)])

G["'"] = (260, [fang(85)])

G['"'] = (400, [fang(85), fang(225)])

# hyphen: a bone
G["-"] = (460, [
    (circle(92, 498, 30) + " " + circle(92, 552, 30) + " " + circle(368, 498, 30) + " " + circle(368, 552, 30), {}),
    (poly((92, 505), (368, 505), (368, 545), (92, 545)), {}),
])

# Symbols --------------------------------------------------------------------

PAREN = ("M300 110 C150 250 110 400 110 525 C110 650 150 800 300 940 L322 912 "
         "C222 790 202 650 202 525 C202 400 222 262 322 138 Z")
PAREN_THORN = poly((116, 492), (44, 525), (116, 558))

G["("] = (380, [(PAREN, {}), (PAREN_THORN, {})])
G[")"] = (380, [(transform(PAREN, (-1, 0, 0, 1, 380, 0)), {}),
                (transform(PAREN_THORN, (-1, 0, 0, 1, 380, 0)), {})])

G["/"] = (500, [
    (poly((372, 118), (420, 96), (446, 130), (128, 940), (60, 940)), {}),
    (poly((300, 430), (372, 410), (318, 470)), {}),
])

G["#"] = (660, [
    (poly((262, 150), (336, 150), (262, 850), (188, 850)), {}),
    (poly((452, 150), (526, 150), (452, 850), (378, 850)), {}),
    (poly((80, 372), (580, 362), (636, 330), (618, 440), (70, 446)), {}),
    (poly((60, 590), (560, 580), (616, 548), (598, 658), (50, 664)), {}),
    drip(330, 655, 85, 22),
])

G["&"] = (660, [
    *nib_stroke(spline([(600, 862), (470, 740), (330, 600), (232, 482), (192, 378), (204, 262),
                        (272, 178), (352, 190), (384, 272), (342, 362), (232, 452), (132, 562),
                        (112, 682), (172, 800), (292, 852), (422, 822), (520, 702), (566, 596)])),
    (poly((548, 612), (640, 520), (600, 650)), {}),
])

AT_C, AT_R = (420, 520), 330
AT_SPIRAL = [(AT_C[0] + (AT_R - 70 * t) * math.cos(0.42 - t * 2 * math.pi * 0.9),
              AT_C[1] + (AT_R - 70 * t) * math.sin(0.42 - t * 2 * math.pi * 0.9)) for t in [i / 24 for i in range(25)]]

# @: the "a" is an eye, wrapped in a spiral tentacle
G["@"] = (840, [
    *nib_stroke(spline([(592, 360), (598, 560), (626, 640), (676, 668)] + AT_SPIRAL[1:]), thin=30, thick=84),
    eye(410, 520, 300, 190),
])


# Accented capitals ----------------------------------------------------------
# Built from the base letter plus a mark centred over its ink. Marks sit in
# MARK_TOP..MARK_BOTTOM, above the tallest cap (the apex of A, y=70).

MARK_BOTTOM, MARK_TOP = 50, -60


def ink_bounds(parts):
    pen = BoundsPen(None)
    for d, _ in parts:
        parse_path(d, pen)
    return pen.bounds  # xmin, ymin, xmax, ymax in SVG coords


def mark(name, cx):
    b, t = MARK_BOTTOM, MARK_TOP
    h = b - t
    if name == "acute":  # a slanted blade, heavy at the top
        return [(poly((cx - 46, b), (cx + 22, t), (cx + 74, t + 22), (cx - 24, b + 4)), {})]
    if name == "grave":
        return [(poly((cx + 46, b), (cx - 22, t), (cx - 74, t + 22), (cx + 24, b + 4)), {})]
    if name == "circumflex":  # a chevron, like the A's apex
        return [(poly((cx - 96, b), (cx - 14, t), (cx + 14, t), (cx + 96, b), (cx + 58, b),
                      (cx, t + h * 0.48), (cx - 58, b)), {})]
    if name == "tilde":  # a pen-drawn wave
        return nib_stroke(spline([(cx - 100, b - 16), (cx - 52, t + 18), (cx + 2, b - 34),
                                  (cx + 54, b - 6), (cx + 100, t + 14)], steps=8), thin=24, thick=58)
    if name == "dieresis":  # two drops of blood
        return [blood_drop(cx - 62, b, h, 64), blood_drop(cx + 62, b, h, 64)]
    if name == "cedilla":  # a little hook under the letter, like the comma
        comma = poly((150, 736), (206, 792), (166, 934), (118, 946), (138, 852), (94, 792))
        return [(transform(comma, (0.8, 0, 0, 0.8, cx - 132, 850 - 0.8 * 736 - 14)), {})]
    raise ValueError(name)


ACCENTS = {
    "acute": "AEIOU", "grave": "AEIOU", "circumflex": "AEIOU", "tilde": "AON",
    "dieresis": "AEIOU", "cedilla": "C",
}

ACCENTED = {}  # accented letter -> base letter, for kerning
for _name, _bases in ACCENTS.items():
    for _base in _bases:
        _ch = unicodedata.normalize("NFC", _base + {
            "acute": "́", "grave": "̀", "circumflex": "̂",
            "tilde": "̃", "dieresis": "̈", "cedilla": "̧"}[_name])
        _adv, _parts = G[_base]
        _x0, _, _x1, _ = ink_bounds(_parts)
        G[_ch] = (_adv, _parts + mark(_name, (_x0 + _x1) / 2))
        ACCENTED[_ch] = _base


# Advance width of the space glyph (used by build_font.py).
SPACE = 300

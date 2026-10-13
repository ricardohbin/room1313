//! Pixel art for the cinematics. Every shot is a pure function of time,
//! repainted each frame onto a W×H canvas that CSS scales up with
//! `image-rendering: pixelated`.
//!
//! This module holds the painter (`Pen`), the pixel font and the pieces the
//! shots share; `intro` and `endings` hold the shots themselves.

mod endings;
mod intro;

use std::f64::consts::TAU;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement};

pub use endings::*;
pub use intro::{spark, LIGHTNING};

pub const W: f64 = 240.0;
pub const H: f64 = 135.0;

#[derive(Clone, Copy, PartialEq)]
pub enum Shot {
    // Intro
    City,
    Hotel,
    Kindle,
    Room,
    Cctv,
    Office,
    // Endings
    Badge,
    Leaving,
    DriveBy,
    Corridor,
    Stairs,
    Orderlies,
    Cell,
    Collapse,
    Hospital,
    Overload,
    Aftermath,
    Document(Doc),
}

/// The visible canvas plus two offscreen ones for shots that composite
/// (the office shows the CCTV feed on a monitor and pulls back from it).
pub struct Art {
    screen: Pen,
    office: Pen,
    feed: Pen,
}

impl Art {
    /// Returns the art and the canvas to put on the page.
    pub fn new(doc: &Document) -> Result<(Self, HtmlCanvasElement), JsValue> {
        let screen = Pen::new(doc)?;
        let canvas = screen.canvas.clone();
        let art = Self { screen, office: Pen::new(doc)?, feed: Pen::new(doc)? };
        Ok((art, canvas))
    }

    pub fn draw(&self, shot: Shot, t: f64) {
        let p = &self.screen;
        match shot {
            Shot::City => intro::city(p, t),
            Shot::Hotel => intro::hotel(p, t),
            Shot::Kindle => intro::kindle(p, t),
            Shot::Room => intro::room(p, t),
            Shot::Cctv => intro::cctv(p, t),
            Shot::Office => intro::pull_back(self, t),
            Shot::Badge => endings::badge(p, t),
            Shot::Leaving => endings::leaving(p, t),
            Shot::DriveBy => endings::drive_by(p, t),
            Shot::Corridor => endings::corridor(p, t),
            Shot::Stairs => endings::stairs(p, t),
            Shot::Orderlies => endings::orderlies(p, t),
            Shot::Cell => endings::cell(p, t),
            Shot::Collapse => endings::collapse(p, t),
            Shot::Hospital => endings::hospital(p, t),
            Shot::Overload => endings::overload(p, t),
            Shot::Aftermath => endings::aftermath(p, t),
            Shot::Document(doc) => endings::document(p, t, doc),
        }
    }

    /// Covers the frame with black (fades between shots).
    pub fn fade(&self, alpha: f64) {
        if alpha > 0.0 {
            self.screen.rect(0.0, 0.0, W, H, &format!("rgba(0,0,0,{:.3})", alpha.min(1.0)));
        }
    }
}

// ---------- Painter ----------

struct Pen {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
}

impl Pen {
    fn new(doc: &Document) -> Result<Self, JsValue> {
        let canvas = doc.create_element("canvas")?.dyn_into::<HtmlCanvasElement>()?;
        canvas.set_width(W as u32);
        canvas.set_height(H as u32);
        let ctx = canvas
            .get_context("2d")?
            .ok_or("no 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;
        ctx.set_image_smoothing_enabled(false);
        Ok(Self { canvas, ctx })
    }

    fn ink(&self, color: &str) {
        self.ctx.set_fill_style_str(color);
    }

    /// One pixel in the current ink.
    fn dot(&self, x: f64, y: f64) {
        self.ctx.fill_rect(x.floor(), y.floor(), 1.0, 1.0);
    }

    fn rect(&self, x: f64, y: f64, w: f64, h: f64, color: &str) {
        self.ink(color);
        self.ctx.fill_rect(x.floor(), y.floor(), w.round(), h.round());
    }

    fn px(&self, x: f64, y: f64, color: &str) {
        self.rect(x, y, 1.0, 1.0, color);
    }

    /// Bresenham, so diagonals stay one pixel wide.
    fn line(&self, x0: f64, y0: f64, x1: f64, y1: f64, color: &str) {
        self.ink(color);
        let (mut x, mut y) = (x0.round() as i32, y0.round() as i32);
        let (x1, y1) = (x1.round() as i32, y1.round() as i32);
        let (dx, dy) = ((x1 - x).abs(), -(y1 - y).abs());
        let (sx, sy) = (if x < x1 { 1 } else { -1 }, if y < y1 { 1 } else { -1 });
        let mut err = dx + dy;
        loop {
            self.dot(x as f64, y as f64);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// 3×5 pixel font, 4px advance.
    fn text(&self, s: &str, x: f64, y: f64, color: &str) {
        self.text_at_scale(s, x, y, 1.0, color);
    }

    /// The pixel font with each pixel drawn `scale`×`scale`.
    fn text_at_scale(&self, s: &str, x: f64, y: f64, scale: f64, color: &str) {
        self.ink(color);
        for (i, ch) in s.chars().enumerate() {
            let Some(rows) = glyph(ch) else { continue };
            for (dy, row) in rows.iter().enumerate() {
                for (dx, b) in row.bytes().enumerate() {
                    if b == b'#' {
                        let (px, py) = (x + (i * 4 + dx) as f64 * scale, y + dy as f64 * scale);
                        self.ctx.fill_rect(px.floor(), py.floor(), scale, scale);
                    }
                }
            }
        }
    }

    /// A soft disc of `rgb` light (e.g. "150,170,200") fading out at radius `r`.
    fn glow(&self, cx: f64, cy: f64, r: f64, rgb: &str, alpha: f64) {
        let Ok(g) = self.ctx.create_radial_gradient(cx, cy, 0.0, cx, cy, r) else { return };
        let _ = g.add_color_stop(0.0, &format!("rgba({rgb},{alpha:.3})"));
        let _ = g.add_color_stop(1.0, &format!("rgba({rgb},0)"));
        self.ctx.set_fill_style_canvas_gradient(&g);
        self.ctx.fill_rect(cx - r, cy - r, r * 2.0, r * 2.0);
    }

    /// Darkness everywhere except a soft circle of light around (cx, cy).
    fn light(&self, cx: f64, cy: f64, r: f64, dark: f64) {
        let Ok(g) = self.ctx.create_radial_gradient(cx, cy, r * 0.2, cx, cy, r) else { return };
        let _ = g.add_color_stop(0.0, "rgba(0,0,0,0)");
        let _ = g.add_color_stop(1.0, &format!("rgba(0,0,0,{dark})"));
        self.ctx.set_fill_style_canvas_gradient(&g);
        self.ctx.fill_rect(0.0, 0.0, W, H);
    }
}

fn glyph(ch: char) -> Option<[&'static str; 5]> {
    Some(match ch {
        '0' => ["###", "#.#", "#.#", "#.#", "###"],
        '1' => [".#.", "##.", ".#.", ".#.", "###"],
        '2' => ["###", "..#", "###", "#..", "###"],
        '3' => ["###", "..#", ".##", "..#", "###"],
        '4' => ["#.#", "#.#", "###", "..#", "..#"],
        '5' => ["###", "#..", "###", "..#", "###"],
        '6' => ["###", "#..", "###", "#.#", "###"],
        '7' => ["###", "..#", ".#.", ".#.", ".#."],
        '8' => ["###", "#.#", "###", "#.#", "###"],
        '9' => ["###", "#.#", "###", "..#", "###"],
        ':' => ["...", ".#.", "...", ".#.", "..."],
        '/' => ["..#", "..#", ".#.", "#..", "#.."],
        'A' => [".#.", "#.#", "###", "#.#", "#.#"],
        'B' => ["##.", "#.#", "##.", "#.#", "##."],
        'C' => [".##", "#..", "#..", "#..", ".##"],
        'D' => ["##.", "#.#", "#.#", "#.#", "##."],
        'E' => ["###", "#..", "##.", "#..", "###"],
        'F' => ["###", "#..", "##.", "#..", "#.."],
        'G' => [".##", "#..", "#.#", "#.#", ".##"],
        'H' => ["#.#", "#.#", "###", "#.#", "#.#"],
        'I' => ["###", ".#.", ".#.", ".#.", "###"],
        'J' => ["..#", "..#", "..#", "#.#", ".#."],
        'K' => ["#.#", "#.#", "##.", "#.#", "#.#"],
        'L' => ["#..", "#..", "#..", "#..", "###"],
        'M' => ["#.#", "###", "###", "#.#", "#.#"],
        'N' => ["##.", "#.#", "#.#", "#.#", "#.#"],
        'O' => [".#.", "#.#", "#.#", "#.#", ".#."],
        'P' => ["##.", "#.#", "##.", "#..", "#.."],
        'Q' => [".#.", "#.#", "#.#", "##.", ".##"],
        'R' => ["##.", "#.#", "##.", "#.#", "#.#"],
        'S' => [".##", "#..", ".#.", "..#", "##."],
        'T' => ["###", ".#.", ".#.", ".#.", ".#."],
        'U' => ["#.#", "#.#", "#.#", "#.#", "###"],
        'V' => ["#.#", "#.#", "#.#", "#.#", ".#."],
        'W' => ["#.#", "#.#", "###", "###", "#.#"],
        'X' => ["#.#", "#.#", ".#.", "#.#", "#.#"],
        'Y' => ["#.#", "#.#", ".#.", ".#.", ".#."],
        'Z' => ["###", "..#", ".#.", "#..", "###"],
        '.' => ["...", "...", "...", "...", ".#."],
        ',' => ["...", "...", "...", ".#.", "#.."],
        _ => return None,
    })
}

/// Stable pseudo-random value in [0, 1) for `n`, so scenes are deterministic.
fn hash(n: u32) -> f64 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    x as f64 / 4_294_967_296.0
}

fn smoothstep(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Brightness of the sky for lightning striking at `strikes` (a double flash, then decay).
fn lightning(t: f64, strikes: &[f64]) -> f64 {
    strikes
        .iter()
        .map(|&s| match t - s {
            d if !(0.0..0.6).contains(&d) => 0.0,
            d if d < 0.05 => 1.0,
            d if d < 0.11 => 0.15,
            d if d < 0.2 => 0.9,
            d => 0.9 * (1.0 - (d - 0.2) / 0.4),
        })
        .fold(0.0, f64::max)
}

// ---------- Shared pieces ----------

fn sky(p: &Pen, bottom: f64, flash: f64) {
    const DARK: [&str; 6] = ["#06070a", "#090b0e", "#0d0f13", "#111419", "#161a20", "#1c2128"];
    const LIT: [&str; 6] = ["#2a2f38", "#353b46", "#414856", "#4d5564", "#5a6272", "#677082"];
    let bands = if flash > 0.35 { &LIT } else { &DARK };
    let bh = (bottom / 6.0).ceil();
    for (i, c) in bands.iter().enumerate() {
        p.rect(0.0, i as f64 * bh, W, bh, c);
    }
    // Checkerboard dither on each seam.
    for i in 1..6 {
        let y = i as f64 * bh;
        for x in (0..W as i32).step_by(2) {
            p.px(x as f64, y - 1.0, bands[i]);
            p.px(x as f64 + 1.0, y, bands[i - 1]);
        }
    }
}

fn clouds(p: &Pen, t: f64, flash: f64) {
    let color = if flash > 0.35 { "#7d8697" } else { "#0e1015" };
    for i in 0..7 {
        let w = 40.0 + hash(i + 70) * 50.0;
        let x = (hash(i + 50) * W * 1.5 + t * 2.5 * (1.0 + hash(i))).rem_euclid(W + 100.0) - 80.0;
        let y = 5.0 + hash(i + 60) * 28.0;
        p.rect(x, y, w, 2.0, color);
        p.rect(x + 8.0, y - 1.0, w - 20.0, 1.0, color);
        p.rect(x + 4.0, y + 2.0, w - 10.0, 1.0, color);
    }
}

fn bolt(p: &Pen, mut x: f64, seed: u32) {
    let mut y = 0.0;
    for k in 0..10 {
        let nx = x + (hash(seed + k) - 0.5) * 12.0;
        let ny = y + 5.0 + hash(seed + k + 20) * 4.0;
        p.line(x, y, nx, ny, "#e8ecf5");
        if k == 4 {
            p.line(nx, ny, nx + 9.0, ny + 8.0, "#9aa3b5");
        }
        (x, y) = (nx, ny);
    }
}

/// A row of buildings standing on `base`; lit windows if `windows`.
fn skyline(
    p: &Pen,
    seed: u32,
    base: f64,
    (hmin, hmax): (f64, f64), color: &str, windows: bool, t: f64) {
    let mut x = -4.0;
    let mut i = 0;
    while x < W {
        let w = 10.0 + (hash(seed * 1000 + i) * 16.0).floor();
        let h = hmin + (hash(seed * 1000 + i + 500) * (hmax - hmin)).floor();
        let top = base - h;
        p.rect(x, top, w, h, color);
        if hash(seed * 1000 + i + 900) < 0.3 {
            p.rect(x + (w / 2.0).floor(), top - 7.0, 1.0, 7.0, color);
            if windows && (t * 0.8 + i as f64 * 0.37).fract() < 0.12 {
                p.px(x + (w / 2.0).floor(), top - 8.0, "#d0302a");
            }
        }
        if windows {
            for wy in (top as i32 + 3..base as i32 - 2).step_by(4) {
                for wx in (x as i32 + 2..(x + w) as i32 - 2).step_by(3) {
                    let r = hash(seed * 7919 + i * 131 + wy as u32 * 17 + wx as u32);
                    // A few windows flicker like a failing tube.
                    let on = r > 0.01 || (t * 9.0 + r * 900.0).sin() > 0.3;
                    if r < 0.13 && on {
                        let c = if r < 0.045 { "#8a7444" } else { "#4a5462" };
                        p.rect(wx as f64, wy as f64, 1.0, 2.0, c);
                    }
                }
            }
        }
        x += w;
        i += 1;
    }
}

/// Diagonal rain, `n` drops; every fourth drop is closer: longer, brighter, faster.
fn rain(p: &Pen, t: f64, n: u32, seed: u32) {
    for i in 0..n {
        let near = i % 4 == 0;
        let speed = if near { 220.0 } else { 150.0 } + hash(seed + i) * 40.0;
        let y = (hash(seed + i + 500) * H + t * speed).rem_euclid(H + 8.0) - 4.0;
        let x = (hash(seed + i + 900) * (W + 30.0) - t * speed * 0.2).rem_euclid(W + 30.0) - 10.0;
        let (len, color) = if near {
            (4.0, "rgba(170,185,205,0.5)")
        } else {
            (2.0, "rgba(120,135,155,0.3)")
        };
        p.line(x, y, x - len * 0.25, y + len, color);
    }
}

fn splashes(p: &Pen, t: f64, top: f64, n: u32) {
    let f = (t * 14.0) as u32;
    p.ink("rgba(160,175,195,0.45)");
    for i in 0..n {
        let x = hash(f * 97 + i) * W;
        let y = top + hash(f * 31 + i + 7) * (H - top);
        p.dot(x - 1.0, y);
        p.dot(x + 1.0, y);
    }
}

fn streetlamp(p: &Pen, x: f64, top: f64, ground: f64, t: f64) {
    p.rect(x, top, 1.0, ground - top, "#050607");
    p.rect(x, top, 5.0, 1.0, "#050607");
    p.rect(x + 4.0, top + 1.0, 3.0, 1.0, "#d8c890");
    let cx = x + 5.5;
    for y in (top + 2.0) as i32..ground as i32 {
        let hw = (y as f64 - top) * 0.25;
        p.rect(cx - hw, y as f64, hw * 2.0, 1.0, "rgba(210,190,130,0.03)");
    }
    // Wobbling reflection on the wet street.
    for y in ground as i32..H as i32 {
        let k = (y as f64 - ground) / (H - ground);
        let dx = ((y as f64 * 0.9 + t * 6.0).sin() * 1.5).round();
        let c = format!("rgba(210,190,130,{:.3})", 0.16 * (1.0 - k));
        p.rect(cx - 1.0 + dx, y as f64, 3.0, 1.0, &c);
    }
}

/// Lucifer's seal crossed with something older: a ringed V over an inverted
/// cross, an eye, and tentacles curling off the corners.
fn sigil(p: &Pen, cx: f64, cy: f64, r: f64, ink: &str, eye: &str) {
    ring(p, cx, cy, r, ink);
    ring(p, cx, cy, r - 2.0, ink);
    p.ink(ink);
    for k in 0..12 {
        let (s, c) = (k as f64 * TAU / 12.0 + 0.26).sin_cos();
        p.dot((cx + c * (r - 1.0)).round(), (cy + s * (r - 1.0)).round());
    }
    let s = r / 16.0;
    let (ax, ay) = (cx - 9.0 * s, cy - 7.0 * s);
    let (bx, by) = (cx + 9.0 * s, cy - 7.0 * s);
    let (vx, vy) = (cx, cy + 4.0 * s);
    p.line(ax, ay, bx, by, ink);
    p.line(ax, ay, vx, vy, ink);
    p.line(bx, by, vx, vy, ink);
    p.line(ax + 4.0 * s, ay, vx + 2.0 * s, vy - 3.0 * s, ink);
    p.line(bx - 4.0 * s, by, vx - 2.0 * s, vy - 3.0 * s, ink);
    p.line(cx, cy - 12.0 * s, cx, cy + 12.0 * s, ink);
    p.line(cx - 4.0 * s, cy + 8.0 * s, cx + 4.0 * s, cy + 8.0 * s, ink);
    p.rect(cx - 2.0 * s, cy - 4.0 * s, 4.0 * s + 1.0, 1.0, ink);
    p.px(cx, cy - 4.0 * s, eye);
    curl(p, ax, ay, -1.0, -1.0, s, ink);
    curl(p, bx, by, 1.0, -1.0, s, ink);
    curl(p, cx, cy + 12.0 * s, -1.0, 1.0, s * 0.8, ink);
    curl(p, cx, cy + 12.0 * s, 1.0, 1.0, s * 0.8, ink);
}

fn ring(p: &Pen, cx: f64, cy: f64, r: f64, ink: &str) {
    p.ink(ink);
    let n = (r * 8.0) as i32;
    for k in 0..n {
        let (s, c) = (k as f64 * TAU / n as f64).sin_cos();
        p.dot((cx + c * r).round(), (cy + s * r).round());
    }
}

/// A tentacle spiralling out of (x, y) toward `dir` (±1) and `up` (-1 up, +1 down).
fn curl(p: &Pen, x: f64, y: f64, dir: f64, up: f64, s: f64, ink: &str) {
    let (cx, cy) = (x + dir * 2.0 * s, y + up * 4.0 * s);
    let a0 = (y - cy).atan2(x - cx);
    let mut prev = (x, y);
    for k in 1..24 {
        let a = a0 + dir * -up * k as f64 * 0.4;
        let rad = 4.5 * s * (1.0 - k as f64 / 24.0);
        let next = (cx + a.cos() * rad, cy + a.sin() * rad);
        p.line(prev.0, prev.1, next.0, next.1, ink);
        prev = next;
    }
}


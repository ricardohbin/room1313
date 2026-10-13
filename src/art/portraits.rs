//! The card portraits: one still, bust-length picture per character, in the
//! cinematics' style (near-black rooms, one light source, a rim of light
//! so nobody disappears into the dark). Also the HUD's attribute icons.
//!
//! Portraits are S×S. The sitter faces us, centred on `CX`; most are lit
//! from one side, so every shape gets a shadow side and a lit rim.

use super::*;
use crate::cards::Character;
use crate::engine::Stat;

/// Portrait size in pixels (the card scales it up, pixelated).
const S: f64 = 96.0;
const CX: f64 = 48.0;

pub struct Portrait {
    pen: Pen,
}

impl Portrait {
    /// Returns the painter and the canvas to put on the card.
    pub fn new(doc: &Document) -> Result<(Self, HtmlCanvasElement), JsValue> {
        let pen = Pen::sized(doc, S, S)?;
        let canvas = pen.canvas.clone();
        Ok((Self { pen }, canvas))
    }

    pub fn paint(&self, who: Character) {
        let p = &self.pen;
        p.rect(0.0, 0.0, S, S, "#000");
        match who {
            Character::HotelOwner => owner(p),
            Character::Reporter => reporter(p),
            Character::Occultist => occultist(p),
            Character::Conspiracist => conspiracist(p),
            Character::Fanatic => fanatic(p),
            Character::Agent => agent(p),
        }
        finish(p, who as u32);
    }
}

// ---------- Shading helpers ----------

/// Shadow, body and rim colours of one material.
#[derive(Clone, Copy)]
struct Tones {
    dark: &'static str,
    base: &'static str,
    lit: &'static str,
}

const fn tones(dark: &'static str, base: &'static str, lit: &'static str) -> Tones {
    Tones { dark, base, lit }
}

/// Where the key light comes from.
#[derive(Clone, Copy, PartialEq)]
enum Light {
    Left,
    Right,
}

/// Fills rows `y0..y1`, each spanning `span(y)`. The side away from the light
/// gets `shade` of the width in shadow, the lit edge a one-pixel rim.
fn fill(p: &Pen, y0: f64, y1: f64, span: impl Fn(f64) -> Option<(f64, f64)>, t: Tones, light: Light, shade: f64) {
    for y in y0 as i32..y1 as i32 {
        let y = y as f64;
        let Some((a, b)) = span(y + 0.5) else { continue };
        let (a, b) = (a.round(), b.round());
        let w = b - a;
        if w < 1.0 {
            continue;
        }
        let dark = (w * shade).round();
        p.rect(a, y, w, 1.0, t.base);
        match light {
            Light::Right => {
                p.rect(a, y, dark, 1.0, t.dark);
                p.rect(b - 1.0, y, 1.0, 1.0, t.lit);
            }
            Light::Left => {
                p.rect(b - dark, y, dark, 1.0, t.dark);
                p.rect(a, y, 1.0, 1.0, t.lit);
            }
        }
    }
}

fn ellipse(cx: f64, cy: f64, rx: f64, ry: f64) -> impl Fn(f64) -> Option<(f64, f64)> {
    move |y| {
        let d = (y - cy) / ry;
        (d.abs() < 1.0).then(|| {
            let hw = rx * (1.0 - d * d).sqrt();
            (cx - hw, cx + hw)
        })
    }
}

fn oval(p: &Pen, (cx, cy): (f64, f64), (rx, ry): (f64, f64), t: Tones, light: Light, shade: f64) {
    fill(p, cy - ry, cy + ry + 1.0, ellipse(cx, cy, rx, ry), t, light, shade);
}

/// A flat oval in one colour.
fn blot(p: &Pen, cx: f64, cy: f64, rx: f64, ry: f64, color: &'static str) {
    let t = tones(color, color, color);
    oval(p, (cx, cy), (rx, ry), t, Light::Right, 0.0);
}

/// Shoulders and chest from `top` to the bottom of the frame: `neck` half-width
/// at the top, rounding out to `half` over `drop` pixels.
fn shoulders(p: &Pen, top: f64, neck: f64, half: f64, drop: f64, t: Tones, light: Light) {
    fill(p, top, S, |y| Some(chest(y, top, neck, half, drop)), t, light, 0.3);
}

fn chest(y: f64, top: f64, neck: f64, half: f64, drop: f64) -> (f64, f64) {
    let k = ((y - top) / drop).clamp(0.0, 1.0).sqrt();
    let hw = neck + (half - neck) * k;
    (CX - hw, CX + hw)
}

/// A rectangle outline, one pixel wide.
fn frame(p: &Pen, x: f64, y: f64, w: f64, h: f64, color: &str) {
    p.rect(x, y, w, 1.0, color);
    p.rect(x, y + h - 1.0, w, 1.0, color);
    p.rect(x, y, 1.0, h, color);
    p.rect(x + w - 1.0, y, 1.0, h, color);
}

/// Film grain and the vignette, like the cinematics.
fn finish(p: &Pen, seed: u32) {
    p.ink("rgba(255,240,220,0.05)");
    for i in 0..260 {
        p.dot(hash(seed * 977 + i) * S, hash(seed * 613 + i + 7) * S);
    }
    p.ink("rgba(0,0,0,0.18)");
    for i in 0..260 {
        p.dot(hash(seed * 331 + i + 3) * S, hash(seed * 197 + i + 11) * S);
    }
    p.light(CX, 42.0, 78.0, 0.62);
}

// ---------- The cast ----------

/// Mr. Ostrowski, behind his own reception desk: old, bald, polite, scared.
/// He holds up the key tag of room 1313.
fn owner(p: &Pen) {
    // Green-striped hotel wallpaper (the same as room 1313) and the key cabinet.
    p.rect(0.0, 0.0, S, S, "#18201b");
    p.ink("#141a16");
    for x in (0..S as i32).step_by(6) {
        p.ctx.fill_rect(x as f64, 0.0, 2.0, S);
    }
    p.rect(4.0, 6.0, 88.0, 34.0, "#24180f");
    p.rect(4.0, 6.0, 88.0, 1.0, "#3a2a1a");
    for row in 0..3u32 {
        for col in 0..9u32 {
            let (x, y) = (7.0 + col as f64 * 9.5, 9.0 + row as f64 * 10.0);
            p.rect(x, y, 8.0, 8.0, "#110b07");
            p.px(x + 4.0, y + 1.0, "#6a5838");
            if hash(row * 9 + col + 40) > 0.3 {
                p.rect(x + 4.0, y + 2.0, 1.0, 3.0, "#8a7444");
                p.rect(x + 3.0, y + 5.0, 3.0, 2.0, "#5a1a16");
            }
        }
    }
    p.glow(84.0, 18.0, 50.0, "230,190,120", 0.2);

    let skin = tones("#5a4236", "#8a6a56", "#b08a70");
    let jacket = tones("#1a130f", "#30241c", "#5a4434");
    fill(p, 46.0, 58.0, |_| Some((CX - 6.0, CX + 6.0)), skin, Light::Right, 0.4);
    shoulders(p, 54.0, 10.0, 38.0, 14.0, jacket, Light::Right);
    // Burgundy waistcoat between the lapels, a white shirt in its V, a bow tie.
    for y in 58..96 {
        let y = y as f64;
        let hw = (5.0 + (y - 58.0) * 0.55).min(17.0);
        p.rect(CX - hw, y, hw * 2.0, 1.0, "#4a1618");
        p.px(CX - hw - 1.0, y, "#120c0a");
        p.px(CX + hw, y, "#120c0a");
        let shirt = ((78.0 - y) * 0.4).max(0.0);
        p.rect(CX - shirt, y, shirt * 2.0, 1.0, "#a8a090");
        p.rect(CX - shirt, y, shirt * 0.6, 1.0, "#7a7468");
    }
    for y in [82.0, 88.0, 94.0] {
        p.px(CX, y, "#b09a60");
    }
    p.rect(CX - 4.0, 57.0, 3.0, 3.0, "#6a1a18");
    p.rect(CX + 1.0, 57.0, 3.0, 3.0, "#7a2420");
    p.rect(CX - 1.0, 58.0, 2.0, 1.0, "#3a0e0e");

    // Head: ears, grey tufts over them, a bald dome.
    oval(p, (CX - 11.0, 38.0), (2.0, 3.0), skin, Light::Right, 0.5);
    oval(p, (CX + 11.0, 38.0), (2.0, 3.0), skin, Light::Right, 0.0);
    oval(p, (CX, 36.0), (11.0, 14.0), skin, Light::Right, 0.3);
    p.rect(CX + 2.0, 24.0, 4.0, 1.0, "#c8a088");
    p.rect(CX + 4.0, 25.0, 2.0, 1.0, "#c8a088");
    p.rect(CX - 12.0, 30.0, 3.0, 7.0, "#6a6862");
    p.rect(CX + 9.0, 30.0, 3.0, 7.0, "#a8a6a0");
    p.rect(CX - 7.0, 31.0, 4.0, 1.0, "#7a7872");
    p.rect(CX + 3.0, 31.0, 4.0, 1.0, "#a8a6a0");
    // Round spectacles, tired eyes behind them.
    for ex in [CX - 5.0, CX + 5.0] {
        p.rect(ex - 3.0, 33.0, 6.0, 4.0, "rgba(200,220,230,0.12)");
        frame(p, ex - 3.0, 33.0, 6.0, 4.0, "#b8a060");
        p.rect(ex - 1.0, 35.0, 2.0, 1.0, "#1a1210");
        p.px(ex + 1.0, 34.0, "#f0e8d0");
        p.rect(ex - 2.0, 38.0, 3.0, 1.0, "#6a4e40");
    }
    p.rect(CX - 1.0, 34.0, 2.0, 1.0, "#b8a060");
    p.rect(CX - 1.0, 38.0, 1.0, 3.0, "#5a4236");
    p.px(CX, 41.0, "#5a4236");
    p.rect(CX - 3.0, 42.0, 6.0, 1.0, "#8a8882");
    p.rect(CX - 4.0, 43.0, 8.0, 1.0, "#a8a6a0");
    p.rect(CX - 2.0, 45.0, 4.0, 1.0, "#4a2a24");
    p.rect(CX - 6.0, 46.0, 2.0, 1.0, "#6a4e40");
    p.rect(CX + 4.0, 46.0, 2.0, 1.0, "#6a4e40");

    // His arm comes up; he holds the key out by its tag.
    for y in 64..96 {
        let x = 69.0 + (y - 64) as f64 * 0.4;
        p.rect(x - 5.0, y as f64, 11.0, 1.0, jacket.base);
        p.px(x - 5.0, y as f64, jacket.dark);
        p.px(x + 5.0, y as f64, jacket.lit);
    }
    p.rect(64.0, 63.0, 12.0, 3.0, "#a8a090");
    oval(p, (70.0, 61.0), (5.0, 4.0), skin, Light::Right, 0.3);
    p.rect(60.0, 65.0, 20.0, 9.0, "#7a2420");
    frame(p, 60.0, 65.0, 20.0, 9.0, "#4a1210");
    p.rect(61.0, 65.0, 18.0, 1.0, "#9a3a2c");
    p.text("1313", 62.0, 67.0, "#e0d4b8");
    ring(p, 70.0, 77.0, 2.0, "#c8a040");
    p.rect(69.0, 79.0, 2.0, 10.0, "#c8a040");
    p.px(70.0, 79.0, "#f0d070");
    p.rect(71.0, 85.0, 2.0, 1.0, "#c8a040");
    p.rect(71.0, 87.0, 3.0, 1.0, "#c8a040");
}

/// Vera Kask of the Evening Star: fedora with a press card, a black bob,
/// red lips, her camera's flash going off.
fn reporter(p: &Pen) {
    // Precinct wall, the blinds' shadow slanting across it.
    p.rect(0.0, 0.0, S, S, "#14161c");
    p.ink("#20242c");
    for y in 0..S as i32 {
        for x in 0..S as i32 {
            if ((y as f64 + x as f64 * 0.3) as i32).rem_euclid(7) < 3 {
                p.dot(x as f64, y as f64);
            }
        }
    }

    let skin = tones("#9a7a6a", "#c8a490", "#ecd0bc");
    let hair = tones("#08070a", "#141216", "#2e2a34");
    let coat = tones("#3a3226", "#5e5040", "#9a8a6a");
    let hat = tones("#1c1a1e", "#34303a", "#5e5866");

    // The bob, behind the face.
    fill(p, 22.0, 48.0, ellipse(CX, 34.0, 15.0, 17.0), hair, Light::Left, 0.4);
    fill(p, 44.0, 60.0, |_| Some((CX - 4.0, CX + 4.0)), skin, Light::Left, 0.5);
    shoulders(p, 58.0, 8.0, 40.0, 12.0, coat, Light::Left);
    // Black turtleneck in the V, the trench's lapels and raised collar.
    for y in 56..80 {
        let hw = ((80 - y) as f64 * 0.35).max(0.0);
        p.rect(CX - hw, y as f64, hw * 2.0, 1.0, "#121116");
    }
    p.line(CX - 6.0, 55.0, CX - 15.0, 72.0, "#2a241c");
    p.line(CX + 6.0, 55.0, CX + 15.0, 72.0, "#2a241c");
    p.line(CX - 7.0, 54.0, CX - 11.0, 50.0, coat.lit);
    p.line(CX + 7.0, 54.0, CX + 11.0, 50.0, coat.base);
    p.rect(CX - 20.0, 82.0, 1.0, 14.0, "#2a241c");
    p.rect(CX - 18.0, 84.0, 2.0, 2.0, "#2a241c");

    oval(p, (CX, 37.0), (10.0, 13.0), skin, Light::Left, 0.3);
    // Blunt bangs, the hair hanging straight to the jaw on both sides.
    p.rect(CX - 11.0, 24.0, 22.0, 7.0, hair.base);
    p.rect(CX - 11.0, 24.0, 22.0, 1.0, hair.lit);
    p.rect(CX - 13.0, 30.0, 4.0, 16.0, hair.base);
    p.rect(CX + 9.0, 30.0, 4.0, 16.0, hair.dark);
    p.px(CX - 13.0, 32.0, hair.lit);
    // Eyes lined in black, arched brows, red lips, a beauty mark.
    for ex in [CX - 4.0, CX + 4.0] {
        p.rect(ex - 2.0, 33.0, 4.0, 1.0, "#2a2026");
        p.rect(ex - 1.0, 36.0, 3.0, 1.0, "#141014");
        p.px(ex, 36.0, "#4a6a7a");
    }
    p.px(CX - 7.0, 35.0, "#141014");
    p.px(CX + 7.0, 35.0, "#141014");
    p.px(CX - 1.0, 39.0, "#9a7a6a");
    p.px(CX - 1.0, 40.0, "#9a7a6a");
    p.rect(CX - 3.0, 43.0, 5.0, 1.0, "#c0392b");
    p.rect(CX - 2.0, 44.0, 3.0, 1.0, "#8a2420");
    p.px(CX + 5.0, 41.0, "#3a2020");
    p.rect(CX + 3.0, 40.0, 3.0, 1.0, "rgba(200,80,70,0.25)");

    // Fedora with a press card in the band, tipped down to the left.
    fill(p, 12.0, 23.0, |y| Some((CX - 12.0 + (23.0 - y) * 0.2, CX + 12.0 - (23.0 - y) * 0.2)), hat, Light::Left, 0.3);
    p.rect(CX - 11.0, 15.0, 6.0, 1.0, hat.dark); // the pinch
    p.rect(CX - 12.0, 20.0, 24.0, 2.0, "#0e0c0e");
    p.rect(CX + 4.0, 16.0, 6.0, 5.0, "#d8d0bc");
    p.rect(CX + 5.0, 17.0, 4.0, 1.0, "#2a2420");
    p.rect(CX + 5.0, 19.0, 3.0, 1.0, "#6a6050");
    p.rect(CX - 19.0, 23.0, 38.0, 2.0, hat.base);
    p.rect(CX - 19.0, 23.0, 38.0, 1.0, hat.lit);
    p.rect(CX - 21.0, 24.0, 4.0, 1.0, hat.base);

    // The camera: a press box camera with a flash dish, and her hand on it.
    p.rect(4.0, 64.0, 24.0, 18.0, "#16161a");
    p.rect(6.0, 62.0, 20.0, 2.0, "#26262c");
    p.rect(4.0, 64.0, 1.0, 18.0, "#4a4a56");
    blot(p, 16.0, 73.0, 6.0, 6.0, "#2e2e38");
    blot(p, 16.0, 73.0, 4.0, 4.0, "#08080c");
    p.px(14.0, 71.0, "#e0e0f0");
    p.rect(11.0, 56.0, 2.0, 8.0, "#2a2a30");
    oval(p, (12.0, 50.0), (9.0, 9.0), tones("#5a5a62", "#9a9aa4", "#d8d8e0"), Light::Right, 0.35);
    blot(p, 12.0, 50.0, 3.0, 3.0, "#fffef0");
    oval(p, (30.0, 74.0), (4.0, 5.0), skin, Light::Left, 0.4);
    p.glow(12.0, 50.0, 46.0, "255,250,230", 0.5);
}

/// Ambrose Quill: wide-brimmed hat, round black glasses, grey beard,
/// a candle lighting him from below, the sigil chalked on the wall behind.
fn occultist(p: &Pen) {
    p.rect(0.0, 0.0, S, S, "#0d0b0e");
    sigil(p, CX, 44.0, 34.0, "#2a2228", "#4a1a1a");

    let skin = tones("#3e3430", "#6a5648", "#a88a64");
    let coat = tones("#101016", "#1c1c24", "#3a3a48");
    let hat = tones("#0e0c10", "#1c1a1e", "#3a3438");
    let beard = "#5a5650";

    fill(p, 46.0, 58.0, |_| Some((CX - 5.0, CX + 5.0)), skin, Light::Right, 0.4);
    shoulders(p, 56.0, 10.0, 42.0, 14.0, coat, Light::Right);
    // High collar standing up on both sides, a red scarf wound high.
    p.rect(CX - 15.0, 48.0, 4.0, 12.0, coat.base);
    p.rect(CX + 11.0, 48.0, 4.0, 12.0, coat.base);
    p.rect(CX + 14.0, 48.0, 1.0, 12.0, coat.lit);
    p.rect(CX - 10.0, 52.0, 20.0, 6.0, "#4a1416");
    p.rect(CX - 10.0, 52.0, 20.0, 1.0, "#6a1c1c");
    p.rect(CX - 9.0, 58.0, 6.0, 22.0, "#3a1012");
    p.rect(CX - 4.0, 58.0, 1.0, 22.0, "#5a1818");
    // An eye amulet on a brass chain.
    p.line(CX - 2.0, 58.0, CX + 4.0, 66.0, "#6a5838");
    p.line(CX + 10.0, 58.0, CX + 4.0, 66.0, "#6a5838");
    ring(p, CX + 4.0, 69.0, 3.0, "#b09a50");
    p.px(CX + 4.0, 69.0, "#c0392b");

    oval(p, (CX, 38.0), (10.0, 13.0), skin, Light::Right, 0.35);
    // A long grey beard, streaked.
    for y in 42..62 {
        let hw = (8.0 - (y - 42) as f64 * 0.32).max(1.0);
        p.rect(CX - hw, y as f64, hw * 2.0, 1.0, beard);
        for x in ((CX - hw) as i32..(CX + hw) as i32).step_by(3) {
            p.px(x as f64 + (y % 2) as f64, y as f64, "#7a766e");
        }
    }
    p.rect(CX - 5.0, 42.0, 10.0, 2.0, "#6a665e");
    p.rect(CX - 2.0, 44.0, 4.0, 1.0, "#1a1210");
    // The hat's brim throws the eyes into shadow; the glasses catch the flame.
    p.rect(CX - 10.0, 26.0, 20.0, 6.0, "rgba(0,0,0,0.45)");
    blot(p, CX - 4.0, 36.0, 3.0, 2.5, "#040404");
    blot(p, CX + 4.0, 36.0, 3.0, 2.5, "#040404");
    p.rect(CX - 1.0, 35.0, 2.0, 1.0, "#8a7444");
    p.rect(CX - 1.0, 39.0, 2.0, 2.0, "#4a3e36");
    fill(p, 8.0, 25.0, |y| Some((CX - 11.0 + (25.0 - y) * 0.1, CX + 11.0 - (25.0 - y) * 0.1)), hat, Light::Right, 0.3);
    p.rect(CX - 3.0, 8.0, 6.0, 2.0, hat.dark); // dent in the crown
    p.rect(CX - 11.0, 21.0, 22.0, 3.0, "#3a2a1a");
    p.rect(CX - 25.0, 24.0, 50.0, 3.0, hat.base);
    p.rect(CX - 25.0, 26.0, 50.0, 1.0, hat.lit);
    p.rect(CX - 27.0, 25.0, 3.0, 1.0, hat.base);
    p.rect(CX + 24.0, 25.0, 3.0, 1.0, hat.base);

    // The candle, cupped in both hands.
    let (kx, ky) = (64.0, 76.0);
    p.rect(kx - 2.0, ky, 5.0, 12.0, "#c8c0a0");
    p.rect(kx + 2.0, ky, 1.0, 12.0, "#8a8470");
    p.rect(kx - 2.0, ky, 1.0, 4.0, "#f0ead0");
    p.px(kx, ky - 1.0, "#1a1210");
    oval(p, (kx - 6.0, ky + 12.0), (6.0, 4.0), skin, Light::Right, 0.2);
    oval(p, (kx + 6.0, ky + 12.0), (6.0, 4.0), skin, Light::Left, 0.2);
    p.glow(kx, ky - 6.0, 56.0, "255,200,100", 0.38);
    p.rect(kx - 1.0, ky - 7.0, 3.0, 5.0, "#ffb040");
    p.rect(kx, ky - 9.0, 1.0, 3.0, "#ffe080");
    p.px(kx, ky - 4.0, "#fff8d0");
    p.px(CX - 3.0, 37.0, "#ffe080");
    p.px(CX + 5.0, 37.0, "#ffe080");
}

/// Dale Pruitt: big, sweating, cap lined with tinfoil, tinted glasses,
/// a stack of cassettes, his corkboard of red string behind him.
fn conspiracist(p: &Pen) {
    p.rect(0.0, 0.0, S, S, "#3a2a1a");
    p.ink("#2e2014");
    for i in 0..500 {
        p.dot(hash(i + 30) * S, hash(i + 90) * S);
    }
    let notes = [
        (3.0, 5.0, 14.0, 10.0, "#8a8474"),
        (76.0, 4.0, 16.0, 12.0, "#9a9282"),
        (4.0, 28.0, 13.0, 12.0, "#5a5a56"),
        (79.0, 30.0, 13.0, 10.0, "#8a8474"),
        (2.0, 50.0, 16.0, 9.0, "#9a9282"),
        (26.0, 3.0, 10.0, 8.0, "#7a766a"),
        (62.0, 8.0, 9.0, 11.0, "#5a5a56"),
    ];
    for &(x, y, w, h, c) in &notes {
        p.rect(x + 1.0, y + 1.0, w, h, "rgba(0,0,0,0.35)");
        p.rect(x, y, w, h, c);
    }
    // A black van in one photo, a clipping that says 358.
    p.rect(6.0, 33.0, 8.0, 3.0, "#0e0e0e");
    p.rect(6.0, 31.0, 5.0, 2.0, "#0e0e0e");
    p.text("358", 79.0, 7.0, "#2a2420");
    for pair in [(0, 2), (2, 4), (0, 5), (5, 6), (6, 1), (1, 3), (2, 6), (3, 5)] {
        let (a, b) = (notes[pair.0], notes[pair.1]);
        p.line(a.0 + a.2 / 2.0, a.1 + 1.0, b.0 + b.2 / 2.0, b.1 + 1.0, "#a0281e");
    }
    for &(x, y, w, _, _) in &notes {
        p.px(x + w / 2.0, y + 1.0, "#e04a3a");
    }
    p.glow(CX, 0.0, 80.0, "170,210,180", 0.14);

    let skin = tones("#7a5040", "#b07a62", "#d8a088");
    let flannel = tones("#3a1a16", "#5a2a22", "#7a3a2e");
    shoulders(p, 58.0, 12.0, 52.0, 10.0, flannel, Light::Right);
    // Plaid.
    for y in 58..96 {
        let (a, b) = chest(y as f64 + 0.5, 58.0, 12.0, 52.0, 10.0);
        if y % 5 == 0 {
            p.rect(a, y as f64, b - a, 1.0, "rgba(20,8,6,0.5)");
        }
        p.ink("rgba(20,8,6,0.5)");
        for x in (a as i32..b as i32).filter(|x| x % 5 == 0) {
            p.dot(x as f64, y as f64);
        }
    }
    // The T-shirt: an eye in a pyramid, and a stain.
    fill(p, 58.0, S, |y| Some((CX - 10.0 - (y - 58.0) * 0.35, CX + 10.0 + (y - 58.0) * 0.35)), tones("#6a665a", "#8a8476", "#a8a294"), Light::Right, 0.25);
    p.line(CX - 6.0, 82.0, CX, 72.0, "#2a2622");
    p.line(CX + 6.0, 82.0, CX, 72.0, "#2a2622");
    p.line(CX - 6.0, 82.0, CX + 6.0, 82.0, "#2a2622");
    p.px(CX, 78.0, "#2a2622");
    blot(p, CX - 9.0, 88.0, 3.0, 2.0, "#6a6252");

    // A thick neck, a double chin, a big round face.
    oval(p, (CX, 54.0), (12.0, 6.0), skin, Light::Right, 0.3);
    oval(p, (CX, 40.0), (14.0, 15.0), skin, Light::Right, 0.3);
    p.rect(CX - 9.0, 52.0, 18.0, 1.0, "#8a5a48");
    p.ink("#6a4434");
    for i in 0..40 {
        let a = hash(i + 300) * std::f64::consts::PI;
        let r = 10.0 + hash(i + 340) * 4.0;
        p.dot(CX + a.cos() * r, 46.0 + a.sin() * 6.0);
    }
    p.rect(CX - 11.0, 44.0, 4.0, 2.0, "rgba(220,60,50,0.3)");
    p.rect(CX + 7.0, 44.0, 4.0, 2.0, "rgba(220,60,50,0.3)");
    // Tinted square glasses.
    for ex in [CX - 10.0, CX + 3.0] {
        p.rect(ex, 36.0, 7.0, 5.0, "#2c3a30");
        frame(p, ex, 36.0, 7.0, 5.0, "#141414");
        p.px(ex + 3.0, 38.0, "#c8d8c0");
        p.px(ex + 5.0, 37.0, "#8a9a88");
    }
    p.rect(CX - 3.0, 37.0, 6.0, 1.0, "#141414");
    p.rect(CX - 1.0, 42.0, 3.0, 3.0, "#9a6a54");
    p.rect(CX - 3.0, 48.0, 6.0, 2.0, "#2a0e0a");
    p.rect(CX - 2.0, 48.0, 4.0, 1.0, "#c8bca8");
    p.px(CX - 8.0, 33.0, "#e0f0f0");
    p.px(CX + 10.0, 35.0, "#e0f0f0");
    p.px(CX + 10.0, 36.0, "#a8c0c8");
    // Cap with the brim to the side; tinfoil peeking out underneath.
    fill(p, 20.0, 31.0, ellipse(CX, 31.0, 15.0, 11.0), tones("#1e2a44", "#2e3e60", "#4a5e88"), Light::Right, 0.3);
    p.rect(CX - 4.0, 22.0, 8.0, 7.0, "#c8c0a8");
    p.text("X", CX - 1.0, 23.0, "#a0281e");
    p.rect(CX + 8.0, 30.0, 18.0, 3.0, "#1a2440");
    p.rect(CX + 8.0, 30.0, 18.0, 1.0, "#3a4a70");
    p.rect(CX - 15.0, 31.0, 9.0, 1.0, "#c8ccd0");
    p.px(CX - 12.0, 31.0, "#ffffff");
    p.px(CX - 14.0, 32.0, "#9a9ea4");

    // A stack of cassettes in his fist.
    for k in 0..3 {
        let y = 62.0 + k as f64 * 6.0;
        p.rect(6.0, y, 20.0, 5.0, "#141418");
        p.rect(8.0, y + 1.0, 16.0, 2.0, if k == 1 { "#a04030" } else { "#c8c0a0" });
        p.px(12.0, y + 3.0, "#3a3a40");
        p.px(18.0, y + 3.0, "#3a3a40");
        p.rect(6.0, y, 20.0, 1.0, "#2a2a30");
    }
    oval(p, (16.0, 82.0), (7.0, 5.0), skin, Light::Right, 0.3);
    p.rect(11.0, 80.0, 10.0, 1.0, "#8a5a48");
}

/// Brother Josiah: gaunt, wild white hair, staring eyes, shouting,
/// a wooden cross held high; red light from the vigil candles below.
fn fanatic(p: &Pen) {
    p.rect(0.0, 0.0, S, S, "#140e10");
    p.ink("#1a1214");
    for i in 0..300 {
        p.dot(hash(i + 70) * S, hash(i + 170) * S);
    }
    p.text_at_scale("358", 4.0, 8.0, 3.0, "#5a1612");
    for (x, len) in [(7.0, 6.0), (16.0, 3.0), (28.0, 9.0), (33.0, 4.0)] {
        p.rect(x, 23.0, 1.0, len, "#4a1210");
    }

    let skin = tones("#5a4a40", "#9a8470", "#c8b098");
    let robe = tones("#0c0a0e", "#1a1620", "#3a3044");

    fill(p, 46.0, 58.0, |_| Some((CX - 3.0, CX + 3.0)), skin, Light::Right, 0.4);
    p.line(CX - 1.0, 48.0, CX - 2.0, 56.0, skin.dark);
    shoulders(p, 57.0, 7.0, 38.0, 12.0, robe, Light::Right);
    p.rect(CX - 6.0, 54.0, 12.0, 4.0, "#121016");
    p.rect(CX - 1.0, 55.0, 3.0, 3.0, "#d8d4c8");
    for y in [66.0, 74.0, 82.0, 90.0] {
        p.px(CX, y, "#3a3044");
    }

    // Wild white hair, standing out all round.
    for k in 0..26u32 {
        let a = std::f64::consts::PI * (1.0 + k as f64 / 25.0) + (hash(k + 5) - 0.5) * 0.2;
        let (s, c) = a.sin_cos();
        let len = 4.0 + hash(k + 50) * 6.0;
        let color = if c > 0.0 { "#d8d4cc" } else { "#8a8680" };
        p.line(CX + c * 8.0, 34.0 + s * 12.0, CX + c * (10.0 + len), 34.0 + s * (13.0 + len), color);
    }
    p.rect(CX - 12.0, 30.0, 3.0, 12.0, "#8a8680");
    p.rect(CX + 9.0, 30.0, 3.0, 12.0, "#d8d4cc");
    oval(p, (CX, 36.0), (9.0, 14.0), skin, Light::Right, 0.3);
    // Hollow cheeks, sunken sockets, eyes too wide open, red-rimmed.
    p.rect(CX - 7.0, 40.0, 2.0, 6.0, "#6a5848");
    p.rect(CX + 5.0, 40.0, 2.0, 6.0, "#7a6a58");
    for ex in [CX - 4.0, CX + 3.0] {
        p.rect(ex - 2.0, 32.0, 5.0, 5.0, "#3a2a24");
        p.rect(ex - 1.0, 33.0, 3.0, 2.0, "#e8e2d4");
        p.px(ex, 33.0, "#0a0a0a");
        p.rect(ex - 1.0, 35.0, 3.0, 1.0, "#8a2a22");
    }
    p.line(CX - 7.0, 30.0, CX - 2.0, 31.0, "#d0ccc4");
    p.line(CX + 1.0, 31.0, CX + 6.0, 30.0, "#d0ccc4");
    p.rect(CX - 1.0, 37.0, 1.0, 4.0, skin.dark);
    blot(p, CX, 45.0, 3.0, 2.0, "#1a0606");
    p.rect(CX - 2.0, 43.0, 4.0, 1.0, "#c8c0a8");

    // The cross, held up high.
    p.rect(73.0, 10.0, 4.0, 34.0, "#5a3a1e");
    p.rect(76.0, 10.0, 1.0, 34.0, "#9a6a3a");
    p.rect(65.0, 18.0, 20.0, 4.0, "#5a3a1e");
    p.rect(65.0, 18.0, 20.0, 1.0, "#9a6a3a");
    for y in 44..96 {
        let x = 74.0 - (y - 44) as f64 * 0.12;
        p.rect(x - 4.0, y as f64, 9.0, 1.0, robe.base);
        p.px(x - 4.0, y as f64, robe.dark);
        p.px(x + 4.0, y as f64, robe.lit);
    }
    oval(p, (75.0, 41.0), (4.0, 5.0), skin, Light::Right, 0.3);
    p.rect(72.0, 39.0, 6.0, 1.0, skin.dark);
    p.rect(72.0, 42.0, 6.0, 1.0, skin.dark);

    p.glow(CX, 104.0, 74.0, "210,50,30", 0.32);
    p.glow(75.0, 20.0, 18.0, "220,120,60", 0.12);
}

/// The Agent: black suit, black tie, black glasses, no expression, no name.
/// The ID he shows you says nothing.
fn agent(p: &Pen) {
    for (i, c) in ["#1e232a", "#1b2026", "#181c22", "#151920", "#13161c", "#101318"].iter().enumerate() {
        p.rect(0.0, i as f64 * 16.0, S, 16.0, c);
    }
    p.rect(82.0, 0.0, 2.0, S, "#0c0e12");
    p.rect(84.0, 0.0, 1.0, S, "#2a3038");
    p.glow(92.0, 26.0, 60.0, "160,190,230", 0.16);

    let skin = tones("#5a5054", "#8a7c7a", "#b8aaa4");
    let suit = tones("#060608", "#111115", "#3a4250");
    let hair = tones("#08080a", "#16161a", "#3a3e48");

    fill(p, 44.0, 58.0, |_| Some((CX - 5.0, CX + 5.0)), skin, Light::Right, 0.4);
    shoulders(p, 57.0, 10.0, 42.0, 12.0, suit, Light::Right);
    // White shirt, a thin black tie, sharp lapels.
    for y in 55..84 {
        let hw = ((84 - y) as f64 * 0.28).max(0.0);
        p.rect(CX - hw, y as f64, hw * 2.0, 1.0, "#c8ccd0");
        p.rect(CX - hw, y as f64, hw * 0.5, 1.0, "#8a9098");
    }
    p.rect(CX - 2.0, 56.0, 4.0, 3.0, "#0a0a0c");
    for y in 59..86 {
        let hw = if y < 80 { 1.0 } else { 2.0 };
        p.rect(CX - hw, y as f64, hw * 2.0, 1.0, "#050506");
    }
    p.line(CX - 7.0, 56.0, CX - 14.0, 84.0, "#22242c");
    p.line(CX + 7.0, 56.0, CX + 14.0, 84.0, suit.lit);

    // Square jaw, ears, slicked hair with a hard part.
    oval(p, (CX - 11.0, 35.0), (2.0, 3.0), skin, Light::Right, 0.5);
    oval(p, (CX + 11.0, 35.0), (2.0, 3.0), skin, Light::Right, 0.0);
    oval(p, (CX, 34.0), (11.0, 14.0), skin, Light::Right, 0.3);
    fill(p, 36.0, 45.0, |_| Some((CX - 10.0, CX + 10.0)), skin, Light::Right, 0.3);
    oval(p, (CX, 45.0), (8.0, 3.0), skin, Light::Right, 0.3);
    fill(p, 20.0, 29.0, ellipse(CX, 31.0, 12.0, 11.0), hair, Light::Right, 0.2);
    p.rect(CX - 12.0, 26.0, 2.0, 7.0, hair.base);
    p.rect(CX + 10.0, 26.0, 2.0, 7.0, hair.base);
    p.line(CX - 6.0, 23.0, CX + 6.0, 22.0, "#4a5060");
    p.line(CX - 4.0, 21.0, CX - 4.0, 26.0, hair.dark);
    // Black glasses. In each lens, something small and red.
    p.rect(CX - 10.0, 32.0, 20.0, 2.0, "#020203");
    p.rect(CX - 9.0, 33.0, 8.0, 4.0, "#020203");
    p.rect(CX + 1.0, 33.0, 8.0, 4.0, "#020203");
    p.px(CX + 6.0, 33.0, "#c8d0e0");
    p.px(CX + 7.0, 34.0, "#6a7688");
    p.px(CX - 4.0, 34.0, "#3a4250");
    p.px(CX - 6.0, 35.0, "#7a1a14");
    p.px(CX + 4.0, 35.0, "#7a1a14");
    p.rect(CX - 1.0, 38.0, 1.0, 4.0, skin.dark);
    p.rect(CX - 3.0, 44.0, 6.0, 1.0, "#4a3a3a");
    // Earpiece wire curling down into the collar.
    for y in 37..56 {
        p.px(CX + 12.0 + (y % 2) as f64, y as f64, "#a8b0b8");
    }

    // The ID: a badge with no agency, a card with everything blacked out.
    p.rect(4.0, 58.0, 36.0, 22.0, "#0e0c0c");
    p.rect(4.0, 58.0, 36.0, 1.0, "#2a2626");
    oval(p, (13.0, 69.0), (6.0, 7.0), tones("#6a5428", "#a08840", "#d8c070"), Light::Right, 0.3);
    p.rect(12.0, 66.0, 3.0, 6.0, "#6a5420");
    p.rect(10.0, 68.0, 7.0, 2.0, "#6a5420");
    p.rect(22.0, 60.0, 16.0, 18.0, "#c8c4b8");
    p.rect(24.0, 62.0, 5.0, 6.0, "#4a4a4a");
    p.rect(30.0, 63.0, 6.0, 1.0, "#6a6660");
    p.rect(30.0, 65.0, 4.0, 1.0, "#6a6660");
    p.rect(24.0, 70.0, 12.0, 2.0, "#050505");
    p.rect(24.0, 73.0, 9.0, 2.0, "#050505");
    oval(p, (22.0, 81.0), (8.0, 3.0), skin, Light::Right, 0.3);
    p.rect(26.0, 76.0, 3.0, 5.0, skin.base);
}

// ---------- HUD icons ----------

/// A 9×9 icon for an attribute: badge, eye, heart.
pub fn stat_icon(doc: &Document, stat: Stat) -> Result<HtmlCanvasElement, JsValue> {
    let rows: [&str; 9] = match stat {
        Stat::Reputation => [
            "....#....",
            "....#....",
            "...###...",
            "#########",
            ".#######.",
            "..#####..",
            "..##.##..",
            ".##...##.",
            "##.....##",
        ],
        Stat::Sensitivity => [
            ".........",
            "..#####..",
            ".#.....#.",
            "#..###..#",
            "#..#.#..#",
            "#..###..#",
            ".#.....#.",
            "..#####..",
            ".........",
        ],
        Stat::Arousal => [
            ".##...##.",
            "####.####",
            "#########",
            "#########",
            ".#######.",
            "..#####..",
            "...###...",
            "....#....",
            ".........",
        ],
    };
    let pen = Pen::sized(doc, 9.0, 9.0)?;
    pen.ink("#cfc4b6");
    for (y, row) in rows.iter().enumerate() {
        for (x, b) in row.bytes().enumerate() {
            if b == b'#' {
                pen.dot(x as f64, y as f64);
            }
        }
    }
    Ok(pen.canvas)
}

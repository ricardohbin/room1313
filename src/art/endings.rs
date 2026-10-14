//! The endings' shots: one tragedy per attribute extreme (see GAME.md), each
//! closing on a document: newspaper, police report or hospital record.
//!
//! Moments that sound has to hit (a badge landing, gunfire, a fall) are
//! public constants or functions, so `endings.rs` syncs to the same numbers.

use std::f64::consts::TAU;

use super::*;

// ---------- Timings shared with the sound ----------

/// Dismissal: the badge, then the gun, hit the captain's desk.
pub const BADGE_DROP: f64 = 2.6;
pub const GUN_DROP: f64 = 5.0;

/// Assassination: the motorcycle's pass (start, length), the burst of fire, the fall.
pub const ENGINE: (f64, f64) = (1.2, 5.0);
pub const FIRE: (f64, f64) = (3.2, 4.3);
pub const FIRE_RATE: f64 = 11.0;
pub const SHOT_DOWN: f64 = 3.6;

/// Fatal haunting: doors slamming, a locked handle rattling, the fall down the stairs.
pub const DOOR_SLAMS: [f64; 2] = [3.4, 5.2];
pub const RATTLES: [f64; 3] = [3.15, 3.5, 3.85];
pub const STAIRS_HITS: [f64; 6] = [1.8, 2.1, 2.4, 2.7, 3.0, 3.4];
/// Seconds between footsteps while running.
pub const STRIDE: f64 = 0.18;

/// Institutionalization: the van's doors shut.
pub const VAN_SHUT: f64 = 9.4;

/// Coma: the collapse in the bullpen.
pub const COLLAPSE_FALL: f64 = 4.2;
/// Coma: heart-monitor period and first beep (the sound uses the same numbers).
pub const MONITOR_BEAT: f64 = 1.2;
pub const MONITOR_FIRST: f64 = 0.5;

/// Fatal overload: the moment the body hits the floor (the music dies with it).
pub const OVERLOAD_FALL: f64 = 7.5;
/// Fatal overload aftermath: the phone rings (first ring, period, length, count).
pub const RINGS: (f64, f64, f64, u32) = (1.0, 4.0, 1.6, 3);

/// Documents: when the stamp comes down.
pub const STAMP_AT: f64 = 2.2;

/// Whether the detective is running in the corridor shot (for footsteps).
pub fn corridor_running(t: f64) -> bool {
    corridor_x(t).1 == Stride::Run
}

/// The closing document of each ending.
#[derive(Clone, Copy, PartialEq)]
pub enum Doc {
    Dismissal,
    Assassination,
    Haunting,
    Committed,
    Coma,
    Overload,
    Truth,
}

// ---------- The detective ----------

#[derive(Clone, Copy, PartialEq)]
enum Pose {
    Stand,
    Walk(u32),
    Run(u32),
    /// Doubled over, one hand on a wall to the right, the other on the chest.
    Lean,
    Kneel,
    Lie,
    /// Sitting against a wall, legs out.
    Slump,
    /// Held up by the arms, knees buckled, feet dragging.
    Dragged,
}

const SKIN: &str = "#7a6252";
/// The detective's trench coat: dark, but never lost in the dark.
const COAT: &str = "#3a3440";

/// The detective: long coat, no hat, nothing that gives anything away.
/// Faces right; `x` is the left edge, `ground` the floor line.
fn agent(p: &Pen, x: f64, ground: f64, pose: Pose, coat: &str, rim: &str) {
    match pose {
        Pose::Stand | Pose::Walk(_) | Pose::Run(_) | Pose::Dragged => {
            let lean = if matches!(pose, Pose::Run(_)) { 2.0 } else { 0.0 };
            let drop = if pose == Pose::Dragged { 3.0 } else { 0.0 };
            let top = ground - 26.0 + drop;
            p.rect(x + 2.0 + lean, top, 4.0, 4.0, coat);
            p.px(x + 5.0 + lean, top + 1.0, SKIN);
            p.rect(x + 1.0 + lean / 2.0, top + 4.0, 7.0, 13.0, coat);
            p.rect(x + lean / 2.0, top + 15.0, 9.0, 2.0, coat);
            p.rect(x + 7.0 + lean / 2.0, top + 5.0, 1.0, 10.0, rim);
            let legs = ground - top - 17.0;
            match pose {
                Pose::Stand => {
                    p.rect(x + 2.0, ground - legs, 2.0, legs, coat);
                    p.rect(x + 5.0, ground - legs, 2.0, legs, coat);
                }
                Pose::Walk(f) if f % 2 == 0 => p.rect(x + 3.0, ground - legs, 3.0, legs, coat),
                Pose::Walk(_) => {
                    p.rect(x + 1.0, ground - legs, 2.0, legs, coat);
                    p.rect(x + 6.0, ground - legs, 2.0, legs, coat);
                }
                Pose::Run(f) if f % 2 == 1 => p.rect(x + 4.0, ground - legs, 3.0, legs, coat),
                Pose::Run(f) => {
                    // Stride: front leg reaching, back leg kicked up.
                    let (front, back) = if f % 4 == 0 { (1.0, 0.0) } else { (0.0, 1.0) };
                    p.rect(x + 6.0 + front, ground - legs, 2.0, legs - 3.0, coat);
                    p.rect(x + 8.0 + front, ground - 3.0, 2.0, 3.0, coat);
                    p.rect(x + 1.0 + back, ground - legs, 2.0, 4.0, coat);
                    p.rect(x - 1.0 + back, ground - legs + 3.0, 2.0, 3.0, coat);
                    p.rect(x + 8.0, top + 6.0, 2.0, 2.0, coat); // arm pumping
                }
                _ => {
                    // Dragged: knees bent, feet trailing behind.
                    p.rect(x + 2.0, ground - legs, 2.0, legs - 2.0, coat);
                    p.rect(x - 2.0, ground - 2.0, 5.0, 2.0, coat);
                    p.rect(x + 5.0, ground - legs, 2.0, legs - 3.0, coat);
                    p.rect(x + 1.0, ground - 3.0, 5.0, 2.0, coat);
                }
            }
        }
        Pose::Lean => {
            let top = ground - 23.0;
            p.rect(x + 6.0, top, 4.0, 4.0, coat);
            p.px(x + 9.0, top + 2.0, SKIN);
            for k in 0..6 {
                p.rect(x + 5.0 - k as f64 * 0.5, top + 4.0 + k as f64 * 2.0, 7.0, 2.0, coat);
            }
            p.rect(x + 8.0, top + 5.0, 5.0, 2.0, coat); // arm out to the wall
            p.rect(x + 13.0, top + 4.0, 2.0, 2.0, SKIN);
            p.px(x + 8.0, top + 9.0, SKIN); // hand clutching the chest
            p.rect(x + 2.0, ground - 7.0, 2.0, 7.0, coat);
            p.rect(x + 6.0, ground - 7.0, 2.0, 7.0, coat);
            p.rect(x + 11.0, top + 6.0, 1.0, 8.0, rim);
        }
        Pose::Kneel => {
            let top = ground - 17.0;
            p.rect(x + 3.0, top, 4.0, 4.0, coat);
            p.px(x + 6.0, top + 1.0, SKIN);
            p.rect(x + 2.0, top + 4.0, 7.0, 9.0, coat);
            p.rect(x + 1.0, ground - 4.0, 10.0, 4.0, coat);
            p.rect(x + 8.0, top + 5.0, 1.0, 7.0, rim);
        }
        Pose::Lie => {
            p.rect(x, ground - 3.0, 7.0, 3.0, coat);
            p.rect(x + 6.0, ground - 5.0, 15.0, 5.0, coat);
            p.rect(x + 21.0, ground - 4.0, 4.0, 4.0, coat);
            p.px(x + 23.0, ground - 1.0, SKIN);
            p.rect(x + 6.0, ground - 5.0, 15.0, 1.0, rim);
        }
        Pose::Slump => {
            let top = ground - 15.0;
            p.rect(x + 1.0, top, 4.0, 4.0, coat);
            p.px(x + 4.0, top + 2.0, SKIN);
            p.rect(x, top + 4.0, 7.0, 8.0, coat);
            p.rect(x + 4.0, ground - 3.0, 11.0, 3.0, coat);
            p.rect(x + 6.0, top + 5.0, 1.0, 6.0, rim);
        }
    }
}

fn lerp(a: f64, b: f64, k: f64) -> f64 {
    a + (b - a) * k.clamp(0.0, 1.0)
}

// ---------- Reputation, low: dismissal ----------

/// The captain's office. Your hand puts the badge down, then the gun. The captain never looks up.
pub(super) fn badge(p: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, H, "#141114");
    p.ink("#110e11");
    for x in (0..W as i32).step_by(16) {
        p.ctx.fill_rect(x as f64, 0.0, 1.0, 84.0);
    }
    // Blinds with rain sliding down, a framed commendation, the flag.
    p.rect(16.0, 10.0, 50.0, 50.0, "#07080b");
    for y in (11..59).step_by(3) {
        p.rect(17.0, y as f64, 48.0, 2.0, "#171b22");
    }
    for i in 0..12 {
        let x = 18.0 + hash(i + 10) * 46.0;
        let y = 11.0 + (hash(i + 30) * 48.0 + t * 18.0 * (0.5 + hash(i + 50))).rem_euclid(46.0);
        p.rect(x, y, 1.0, 3.0, "rgba(0,0,0,0.55)");
    }
    p.rect(196.0, 16.0, 22.0, 16.0, "#2a2218");
    p.rect(198.0, 18.0, 18.0, 12.0, "#4a463c");
    p.rect(201.0, 21.0, 12.0, 1.0, "#2a2620");
    p.rect(201.0, 24.0, 9.0, 1.0, "#2a2620");
    p.rect(178.0, 8.0, 1.0, 60.0, "#3a3328");
    p.rect(179.0, 9.0, 12.0, 8.0, "#2a1e40");
    p.rect(179.0, 17.0, 12.0, 6.0, "#5a1a18");

    // The captain: broad, still, reading.
    p.rect(108.0, 34.0, 14.0, 14.0, "#08070a");
    p.rect(96.0, 47.0, 38.0, 40.0, "#08070a");
    p.rect(121.0, 38.0, 1.0, 8.0, "#2a2620");

    // Desk, nameplate, lamp and its pool of light.
    p.rect(0.0, 84.0, W, H - 84.0, "#1e1611");
    p.rect(0.0, 84.0, W, 1.0, "#2e2219");
    p.rect(100.0, 86.0, 30.0, 6.0, "#3a2e1e");
    p.text("CAPTAIN", 101.0, 87.0, "#b49a62");
    p.rect(150.0, 78.0, 8.0, 6.0, "#d8d2c0"); // papers
    p.rect(154.0, 80.0, 1.0, 3.0, "#08070a"); // the pen, scratching away
    p.px(155.0 + (t * 9.0).sin().round(), 82.0, "#08070a");
    for (i, y) in (60..66).enumerate() {
        p.rect(196.0 - i as f64, y as f64, 14.0 + 2.0 * i as f64, 1.0, "#1d3a28");
    }
    p.rect(191.0, 66.0, 24.0, 1.0, "#e6cf8f");
    p.rect(202.0, 67.0, 2.0, 17.0, "#2a2a26");
    p.glow(150.0, 92.0, 70.0, "230,190,120", 0.16);

    // The badge, slid across by your hand, then the gun.
    let badge_at = (112.0, 100.0);
    let gun_at = (136.0, 104.0);
    let (bx, by) = slide(t, 0.6, BADGE_DROP, badge_at);
    if t > 0.6 {
        shield(p, bx, by);
    }
    let (gx, gy) = slide(t, 3.4, GUN_DROP, gun_at);
    if t > 3.4 {
        gun(p, gx, gy);
    }
    let hand = |at: f64, drop: f64, (hx, hy): (f64, f64)| {
        // Reaches from the bottom left, lets go, pulls back.
        let k = if t < drop { (t - at) / (drop - at) } else { 1.0 - (t - drop) / 0.9 };
        if k > 0.0 {
            let k = smoothstep(k);
            let (x, y) = (lerp(-30.0, hx + 4.0, k), lerp(150.0, hy + 6.0, k));
            for i in 0..14 {
                let f = i as f64 / 14.0;
                p.rect(lerp(-30.0, x, f) - 4.0, lerp(150.0, y, f), 9.0, 5.0, "#0e0c10");
            }
            p.rect(x - 1.0, y - 2.0, 6.0, 4.0, SKIN);
        }
    };
    hand(0.6, BADGE_DROP, badge_at);
    hand(3.4, GUN_DROP, gun_at);
    p.light(140.0, 80.0, 150.0, 0.55);
}

/// Where an object carried by the hand is: in from the corner, resting once dropped.
fn slide(t: f64, at: f64, drop: f64, rest: (f64, f64)) -> (f64, f64) {
    let k = smoothstep((t - at) / (drop - at));
    (lerp(-26.0, rest.0, k), lerp(146.0, rest.1, k))
}

fn shield(p: &Pen, x: f64, y: f64) {
    const ROWS: [&str; 8] = [".#####.", "#######", "##...##", "#######", "#######", ".#####.", "..###..", "...#..."];
    for (dy, row) in ROWS.iter().enumerate() {
        for (dx, b) in row.bytes().enumerate() {
            let c = match b {
                b'#' => "#b8962e",
                b'.' if (1..3).contains(&dy) => "#7a5e18",
                _ => continue,
            };
            p.px(x + dx as f64, y + dy as f64, c);
        }
    }
    p.px(x + 1.0, y + 1.0, "#f0d070");
}

fn gun(p: &Pen, x: f64, y: f64) {
    p.rect(x, y, 11.0, 2.0, "#2a2a2c");
    p.rect(x + 7.0, y + 2.0, 3.0, 4.0, "#3a2a1e");
    p.rect(x + 5.0, y + 2.0, 2.0, 1.0, "#2a2a2c");
    p.rect(x + 1.0, y, 6.0, 1.0, "#4a4a50");
}

/// Outside the precinct at night: you walk off into the rain with a box of your things.
pub(super) fn leaving(p: &Pen, t: f64) {
    sky(p, 112.0, 0.0);
    skyline(p, 5, 112.0, (30.0, 70.0), "#0b0c0f", true, t);
    // The precinct: stone facade, POLICE over the door, the blue lamp.
    p.rect(0.0, 30.0, 124.0, 82.0, "#1a1a1e");
    p.rect(0.0, 28.0, 126.0, 3.0, "#26262b");
    for x in (8..120).step_by(24) {
        for y in [42.0, 66.0] {
            let lit = hash(x as u32 + y as u32) < 0.5;
            p.rect(x as f64, y, 10.0, 14.0, if lit { "#4a4232" } else { "#0a0a0d" });
        }
    }
    p.rect(46.0, 82.0, 26.0, 22.0, "#0e0e11");
    p.rect(48.0, 84.0, 22.0, 20.0, "#3a3220");
    p.rect(58.0, 84.0, 1.0, 20.0, "#1a160e");
    p.text("POLICE", 47.0, 75.0, "#c8c8d0");
    p.glow(59.0, 66.0, 18.0, "80,120,255", 0.35);
    p.rect(57.0, 64.0, 4.0, 4.0, "#5a8aff");
    for (k, w) in [(0.0, 34.0), (1.0, 40.0), (2.0, 46.0)] {
        p.rect(59.0 - w / 2.0, 104.0 + k * 3.0, w, 3.0, "#24242a");
    }
    p.rect(0.0, 112.0, W, H - 112.0, "#0a0a0d");
    p.rect(0.0, 112.0, W, 1.0, "#16181d");
    streetlamp(p, 196.0, 84.0, 112.0, t);

    // You, carrying the box, slowly out of frame.
    let x = 64.0 + t * 9.0;
    let ground = if x < 80.0 { 104.0 + ((x - 64.0) / 16.0 * 8.0).floor() } else { 112.0 };
    agent(p, x, ground, Pose::Walk((t * 3.0) as u32), COAT, "#2a2e38");
    p.rect(x + 5.0, ground - 18.0, 7.0, 6.0, "#6a5436");
    p.rect(x + 5.0, ground - 18.0, 7.0, 1.0, "#8a7046");
    rain(p, t, 130, 21);
    splashes(p, t, 113.0, 8);
    p.light(150.0, 80.0, 190.0, 0.4);
}

// ---------- Reputation, high: assassination ----------

/// A night street. You walk; a motorcycle comes up the road; the passenger opens fire.
pub(super) fn drive_by(p: &Pen, t: f64) {
    sky(p, 50.0, 0.0);
    // Storefronts: shutters, a bar sign, a shop window that won't survive.
    p.rect(0.0, 20.0, W, 72.0, "#18141a");
    p.ink("#120f14");
    for (row, y) in (22..90).step_by(5).enumerate() {
        p.ctx.fill_rect(0.0, y as f64, W, 1.0);
        for x in ((row % 2) * 5..W as usize).step_by(10) {
            p.ctx.fill_rect(x as f64, y as f64 + 1.0, 1.0, 4.0);
        }
    }
    p.rect(14.0, 58.0, 40.0, 34.0, "#202024");
    for y in (59..92).step_by(2) {
        p.rect(15.0, y as f64, 38.0, 1.0, "#2a2a30");
    }
    let shattered = t > FIRE.0 + 0.4;
    p.rect(100.0, 56.0, 44.0, 30.0, if shattered { "#0c0e12" } else { "#1e2a36" });
    p.rect(98.0, 54.0, 48.0, 2.0, "#2a2226");
    if shattered {
        for (dx, dy) in [(-10.0, -8.0), (12.0, -9.0), (8.0, 9.0), (-12.0, 7.0), (2.0, -12.0)] {
            p.line(122.0, 70.0, 122.0 + dx, 70.0 + dy, "#5a6a7a");
        }
        p.ink("#7a8a9a");
        for i in 0..14 {
            p.dot(100.0 + hash(i + 500) * 44.0, 90.0 + hash(i + 520) * 4.0);
        }
    } else {
        p.rect(104.0, 60.0, 6.0, 20.0, "rgba(255,255,255,0.06)");
    }
    let flicker = (t * 6.0).sin() > -0.7;
    p.glow(190.0, 52.0, 18.0, "220,60,40", if flicker { 0.3 } else { 0.08 });
    p.text_at_scale("BAR", 180.0, 46.0, 2.0, if flicker { "#e0483a" } else { "#3a1512" });
    p.rect(160.0, 62.0, 18.0, 30.0, "#0c0a0c");

    // Sidewalk, curb, road.
    p.rect(0.0, 92.0, W, 16.0, "#1a191c");
    p.rect(0.0, 106.0, W, 2.0, "#2a292c");
    p.rect(0.0, 108.0, W, H - 108.0, "#0c0c0f");
    for x in (0..W as i32).step_by(30) {
        p.rect(x as f64 + 6.0, 121.0, 14.0, 1.0, "#3a3a2a");
    }
    streetlamp(p, 70.0, 60.0, 106.0, t);

    // Bullet chips in the wall where the burst went.
    if t > FIRE.0 {
        let n = (((t.min(FIRE.1) - FIRE.0) * FIRE_RATE) as u32).min(14);
        p.ink("#3a343a");
        for i in 0..n {
            p.dot(60.0 + hash(i + 700) * 60.0, 70.0 + hash(i + 720) * 22.0);
        }
    }

    // You.
    let (ax, pose) = if t < SHOT_DOWN {
        (40.0 + t * 10.0, Pose::Walk((t * 3.0) as u32))
    } else if t < SHOT_DOWN + 0.3 {
        (40.0 + SHOT_DOWN * 10.0, Pose::Kneel)
    } else {
        (40.0 + SHOT_DOWN * 10.0 - 6.0, Pose::Lie)
    };
    if t > SHOT_DOWN + 0.5 {
        let r = (t - SHOT_DOWN - 0.5).min(3.0) * 4.0;
        p.rect(ax + 6.0 - r, 101.0, 12.0 + 2.0 * r, 2.0, "#3a0808");
        p.rect(ax + 8.0 - r * 0.7, 103.0, 8.0 + 1.4 * r, 1.0, "#2a0606");
    }
    agent(p, ax, 102.0, pose, COAT, "#3a3a44");

    // The motorcycle, right to left, passenger firing at the sidewalk.
    let mx = 270.0 - (t - 2.2) * 80.0;
    if mx > -40.0 && t > 2.2 {
        motorcycle(p, mx, 124.0, t);
        let firing = (FIRE.0..FIRE.1).contains(&t);
        if firing && ((t * FIRE_RATE * 2.0) as u32).is_multiple_of(2) {
            let (gx, gy) = (mx + 9.0, 124.0 - 23.0);
            p.rect(gx - 2.0, gy - 1.0, 3.0, 3.0, "#fff3c0");
            p.px(gx - 3.0, gy, "#ffb040");
            p.px(gx - 1.0, gy - 2.0, "#ffb040");
            p.glow(gx, gy, 20.0, "255,200,120", 0.35);
            let tx = 60.0 + hash((t * 40.0) as u32) * 60.0;
            p.line(gx, gy, tx, 70.0 + hash((t * 40.0) as u32 + 9) * 22.0, "rgba(255,220,150,0.5)");
        }
    }
    p.light(120.0, 80.0, 180.0, 0.45);
}

fn motorcycle(p: &Pen, x: f64, ground: f64, t: f64) {
    let dark = "#26262e";
    p.glow(x + 1.0, ground - 11.0, 26.0, "240,224,160", 0.35);
    for wx in [x + 3.0, x + 21.0] {
        ring(p, wx, ground - 4.0, 4.0, "#6a6a74");
        p.rect(wx - 3.0, ground - 7.0, 6.0, 6.0, dark);
        let a = -t * 20.0;
        p.line(wx, ground - 4.0, wx + a.cos() * 3.0, ground - 4.0 + a.sin() * 3.0, "#4a4a50");
    }
    p.rect(x + 5.0, ground - 11.0, 15.0, 4.0, "#34343e");
    p.rect(x + 5.0, ground - 11.0, 15.0, 1.0, "#6a6a78");
    p.rect(x + 7.0, ground - 13.0, 6.0, 2.0, "#5a1414"); // tank
    p.line(x + 5.0, ground - 11.0, x + 3.0, ground - 16.0, "#3a3a40"); // fork and bars
    p.rect(x + 1.0, ground - 12.0, 2.0, 2.0, "#fff4c0");
    // Headlight on the wet road ahead.
    for k in 1..20 {
        let hw = k as f64 * 0.3;
        p.rect(x - k as f64 * 2.0, ground - 1.0 - hw, 2.0, hw * 2.0 + 1.0, "rgba(240,224,160,0.05)");
    }
    // Rider, and the passenger with the gun raised at the sidewalk.
    p.rect(x + 7.0, ground - 20.0, 5.0, 4.0, dark);
    p.rect(x + 7.0, ground - 19.0, 2.0, 1.0, "#3a4a6a");
    p.rect(x + 7.0, ground - 16.0, 6.0, 6.0, dark);
    p.line(x + 7.0, ground - 14.0, x + 3.0, ground - 16.0, dark);
    p.rect(x + 14.0, ground - 22.0, 5.0, 4.0, dark);
    p.rect(x + 14.0, ground - 18.0, 6.0, 8.0, dark);
    // Rim light from the street lamps along the riders' backs.
    p.rect(x + 12.0, ground - 16.0, 1.0, 6.0, "#5a5a68");
    p.rect(x + 19.0, ground - 18.0, 1.0, 8.0, "#5a5a68");
    p.rect(x + 7.0, ground - 20.0, 5.0, 1.0, "#5a5a68");
    p.rect(x + 14.0, ground - 22.0, 5.0, 1.0, "#5a5a68");
    p.line(x + 14.0, ground - 17.0, x + 10.0, ground - 22.0, dark);
    p.rect(x + 8.0, ground - 23.0, 3.0, 1.0, "#2a2a2c");
    // Exhaust.
    for k in 0..4 {
        let age = (t * 3.0 + k as f64 * 0.25).fract();
        let c = format!("rgba(120,120,130,{:.3})", 0.2 * (1.0 - age));
        p.rect(x + 25.0 + age * 18.0, ground - 6.0 - age * 4.0, 3.0, 2.0, &c);
    }
}

// ---------- Sensitivity, low: fatal haunting ----------

#[derive(Clone, Copy, PartialEq)]
enum Stride {
    Run,
    Stop,
}

/// Where you are in the corridor, and whether you're running.
fn corridor_x(t: f64) -> (f64, Stride) {
    if t < 3.0 {
        (-10.0 + t * 30.0, Stride::Run)
    } else if t < 4.4 {
        (80.0, Stride::Stop)
    } else {
        (80.0 + (t - 4.4) * 36.0, Stride::Run)
    }
}

/// Your own hallway at night. Something you can't see slams the doors and
/// puts out the lights behind you.
pub(super) fn corridor(p: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, 10.0, COAT);
    p.rect(0.0, 10.0, W, 62.0, "#1c1a24");
    p.ink("#18161f");
    for x in (0..W as i32).step_by(6) {
        p.ctx.fill_rect(x as f64, 10.0, 2.0, 62.0);
    }
    p.rect(0.0, 72.0, W, 24.0, "#17120f");
    p.rect(0.0, 72.0, W, 1.0, "#2a2018");
    p.rect(0.0, 96.0, W, H - 96.0, "#1a120c");
    for y in (99..H as i32).step_by(6) {
        p.rect(0.0, y as f64, W, 1.0, "#120c08");
    }
    p.rect(0.0, 104.0, W, 12.0, "#3a1416");

    // Doors: open on black until something slams them.
    let doors = [(18.0, None), (86.0, Some(DOOR_SLAMS[0])), (150.0, Some(DOOR_SLAMS[1])), (210.0, None)];
    for (i, &(dx, slam)) in doors.iter().enumerate() {
        p.rect(dx - 2.0, 50.0, 20.0, 46.0, "#2e2016");
        let open = slam.is_some_and(|s| t < s);
        if open {
            p.rect(dx, 52.0, 16.0, 44.0, "#030203");
            p.rect(dx - 5.0, 52.0, 4.0, 44.0, "#24180f");
        } else {
            p.rect(dx, 52.0, 16.0, 44.0, "#24180f");
            p.rect(dx + 2.0, 55.0, 12.0, 16.0, "#1e140c");
            p.rect(dx + 2.0, 75.0, 12.0, 18.0, "#1e140c");
            // The locked one rattles when you try it.
            let shake = i == 1 && (3.0..4.4).contains(&t) && (t * 30.0).sin() > 0.0;
            p.px(dx + 13.0 + if shake { 1.0 } else { 0.0 }, 74.0, "#a08a5a");
        }
    }
    // Picture frames; one falls.
    for (i, x) in [48.0, 118.0, 180.0].into_iter().enumerate() {
        let fallen = i == 2 && t > 6.0;
        let y = if fallen { lerp(30.0, 92.0, (t - 6.0) * 4.0) } else { 30.0 };
        p.rect(x, y, 14.0, 10.0, "#3a2c1a");
        p.rect(x + 1.0, y + 1.0, 12.0, 8.0, "#20242a");
    }

    // Sconces go out as you pass, and the dark follows you down the hall.
    let sconces = [(64.0, 2.4), (136.0, 5.6), (196.0, 7.2)];
    let mut dark_to: f64 = 0.0;
    for &(x, off) in &sconces {
        let dying = t > off - 0.3 && t < off && (t * 40.0).sin() > 0.0;
        p.rect(x - 2.0, 40.0, 5.0, 4.0, "#3a3020");
        if t < off && !dying {
            p.rect(x - 1.0, 38.0, 3.0, 2.0, "#f0d890");
            p.glow(x, 42.0, 32.0, "240,200,130", 0.3);
        } else {
            dark_to = dark_to.max(x + 30.0);
        }
    }
    // Handprints appearing on the wall behind you.
    for i in 0..((t * 1.2) as u32).min(7) {
        let hx = 10.0 + i as f64 * 26.0 + hash(i + 40) * 8.0;
        let hy = 22.0 + hash(i + 60) * 30.0;
        p.rect(hx, hy + 2.0, 4.0, 3.0, "#3a0a0c");
        for f in 0..4 {
            p.rect(hx + f as f64, hy - (f % 2) as f64, 1.0, 2.0, "#3a0a0c");
        }
    }

    let (x, stride) = corridor_x(t);
    let pose = match stride {
        Stride::Run => Pose::Run((t / STRIDE) as u32),
        Stride::Stop => Pose::Stand,
    };
    agent(p, x, 112.0, pose, COAT, "#4a4030");

    if dark_to > 0.0 {
        p.rect(0.0, 0.0, dark_to - 30.0, H, "rgba(0,0,0,0.85)");
        for k in 0..30 {
            let a = 0.85 * (1.0 - k as f64 / 30.0);
            p.rect(dark_to - 30.0 + k as f64, 0.0, 1.0, H, &format!("rgba(0,0,0,{a:.3})"));
        }
    }
    p.light(120.0, 70.0, 170.0, 0.5);
}

/// Where you are tumbling down the stairs: (time, x, pose) keyframes.
const TUMBLE: [(f64, f64, Pose); 6] = [
    (1.8, 50.0, Pose::Kneel),
    (2.1, 72.0, Pose::Lie),
    (2.4, 96.0, Pose::Slump),
    (2.7, 118.0, Pose::Lie),
    (3.0, 142.0, Pose::Kneel),
    (3.4, 188.0, Pose::Lie),
];

/// The floor height of the staircase at `x`.
fn stair_floor(x: f64) -> f64 {
    let k = ((x - 60.0) / 13.0).floor().clamp(-1.0, 9.0);
    if k < 0.0 {
        50.0
    } else {
        57.0 + k * 7.0
    }
}

/// The top of the stairs in the dark. You trip, and fall all the way down.
pub(super) fn stairs(p: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, H, "#0c0b10");
    // Moonlight through a window at the bottom.
    p.rect(200.0, 40.0, 24.0, 34.0, "#1a2232");
    p.rect(211.0, 40.0, 1.0, 34.0, "#0c0b10");
    p.rect(200.0, 56.0, 24.0, 1.0, "#0c0b10");
    p.rect(150.0, 118.0, 60.0, 17.0, "rgba(90,110,150,0.10)");
    // Landing and steps.
    p.rect(0.0, 50.0, 60.0, H - 50.0, "#16120f");
    p.rect(0.0, 50.0, 60.0, 1.0, "#2a221a");
    for k in 0..10 {
        let (x, y) = (60.0 + k as f64 * 13.0, 57.0 + k as f64 * 7.0);
        p.rect(x, y, 13.0, H - y, "#16120f");
        p.rect(x, y, 13.0, 1.0, "#2a221a");
    }
    p.rect(190.0, 120.0, 50.0, 15.0, "#16120f");
    // Banister.
    p.line(0.0, 30.0, 60.0, 30.0, "#2a1e14");
    p.line(60.0, 30.0, 190.0, 100.0, "#2a1e14");
    for k in 0..10 {
        let x = 64.0 + k as f64 * 13.0;
        let top = 30.0 + (x - 60.0) * 70.0 / 130.0;
        p.rect(x, top, 1.0, stair_floor(x) - top, "#1e150e");
    }

    // Running in, tripping, tumbling.
    if t < TUMBLE[0].0 {
        let x = -10.0 + t * 33.0;
        agent(p, x, 50.0, Pose::Run((t / STRIDE) as u32), COAT, "#2a2e3a");
    } else {
        let i = TUMBLE.iter().rposition(|k| k.0 <= t).unwrap_or(0);
        let (t0, x0, pose) = TUMBLE[i];
        let x = match TUMBLE.get(i + 1) {
            Some(&(t1, x1, _)) => lerp(x0, x1, (t - t0) / (t1 - t0)),
            None => x0,
        };
        let bounce = if i + 1 < TUMBLE.len() { ((t - t0) * 10.0).sin().abs() * 3.0 } else { 0.0 };
        agent(p, x, stair_floor(x + 6.0) - bounce, pose, COAT, "#2a2e3a");
    }
    // Dust settling after.
    if t > 3.4 {
        let age = t - 3.4;
        p.ink(&format!("rgba(150,140,130,{:.3})", (0.3 - age * 0.06).max(0.0)));
        for i in 0..20 {
            p.dot(180.0 + hash(i + 900) * 40.0, 118.0 - hash(i + 920) * 10.0 - age * 2.0);
        }
    }
    p.light(170.0, 100.0, 140.0, 0.6);
}

// ---------- Sensitivity, high: institutionalization ----------

/// Two orderlies march you to a van. The visions watch from the windows.
pub(super) fn orderlies(p: &Pen, t: f64) {
    sky(p, 112.0, 0.0);
    skyline(p, 7, 112.0, (36.0, 80.0), "#0b0c0f", true, t);
    // Red eyes in the dark windows: only you see them.
    for i in 0..6u32 {
        let on = ((t * 0.7 + hash(i + 80) * 5.0) % 3.0) < 1.2;
        if on {
            let (x, y) = (hash(i + 60) * 220.0 + 10.0, 50.0 + hash(i + 70) * 40.0);
            p.px(x, y, "#c0392b");
            p.px(x + 2.0, y, "#c0392b");
        }
    }
    sigil(p, 60.0, 26.0, 9.0, "rgba(192,57,43,0.25)", "rgba(192,57,43,0.4)");

    p.rect(0.0, 112.0, W, H - 112.0, "#0a0a0d");
    p.rect(0.0, 112.0, W, 1.0, "#16181d");

    // The van, back doors open on a pale inside.
    let (vx, vy) = (168.0, 88.0);
    p.rect(vx, vy, 62.0, 30.0, "#c8c8c0");
    p.rect(vx + 40.0, vy - 4.0, 22.0, 8.0, "#c8c8c0");
    p.rect(vx + 44.0, vy - 2.0, 12.0, 6.0, "#1c2430");
    p.rect(vx + 26.0, vy + 6.0, 10.0, 3.0, "#a02a20");
    p.rect(vx + 29.0, vy + 3.0, 4.0, 9.0, "#a02a20");
    for wx in [vx + 10.0, vx + 50.0] {
        p.rect(wx - 4.0, 116.0, 9.0, 6.0, "#060606");
    }
    if t < VAN_SHUT {
        p.rect(vx, vy + 2.0, 14.0, 26.0, "#e8e4d0");
        p.rect(vx - 10.0, vy + 2.0, 9.0, 26.0, "#b0b0a8");
        p.glow(vx + 6.0, vy + 16.0, 30.0, "230,230,210", 0.15);
    } else {
        p.rect(vx + 6.0, vy + 2.0, 1.0, 26.0, "#8a8a84");
    }
    p.rect(vx - 1.0, vy + 22.0, 2.0, 3.0, "#ff3b30");
    p.glow(vx + 62.0, vy + 24.0, 30.0, "240,224,160", 0.12);

    // The three of you, walking to it.
    let x = 10.0 + t * 15.0;
    if x < vx - 2.0 {
        let jitter = ((t * 14.0) as u32 % 3) as f64 - 1.0;
        let white = "#b8b8b0";
        agent(p, x - 10.0, 118.0, Pose::Walk((t * 3.0) as u32), white, "#e8e8e0");
        agent(p, x + jitter, 118.0, Pose::Dragged, COAT, "#3a3a44");
        agent(p, x + 10.0, 118.0, Pose::Walk((t * 3.0) as u32 + 1), white, "#e8e8e0");
        p.rect(x - 3.0, 101.0, 4.0, 2.0, white);
        p.rect(x + 8.0, 101.0, 4.0, 2.0, white);
    }
    rain(p, t, 90, 33);
    p.light(150.0, 90.0, 180.0, 0.45);
}

/// A padded cell. You rock in the corner; the sigil is scratched into the padding.
pub(super) fn cell(p: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, H, "#4a4e46");
    p.ink("#3e423a");
    for y in (0..110).step_by(12) {
        for x in (0..W as i32).step_by(12) {
            let o = if (y / 12) % 2 == 0 { 0 } else { 6 };
            p.ctx.fill_rect((x + o) as f64, y as f64, 1.0, 1.0);
            p.ctx.fill_rect((x + o) as f64 - 1.0, y as f64 + 6.0, 3.0, 1.0);
        }
    }
    p.rect(0.0, 110.0, W, H - 110.0, "#3a3d36");
    sigil(p, 150.0, 50.0, 18.0, "#26281f", "#26281f");
    // Tally marks: 13, again and again.
    p.ink("#2e3029");
    for g in 0..4 {
        for k in 0..13 {
            let x = 30.0 + g as f64 * 22.0 + k as f64 * 1.5;
            p.ctx.fill_rect(x.floor(), 24.0 + (k % 2) as f64, 1.0, 6.0);
        }
    }
    // Door with a slot; sometimes someone looks in.
    p.rect(196.0, 30.0, 34.0, 80.0, "#5a5e54");
    p.rect(206.0, 44.0, 14.0, 8.0, "#0c0c0c");
    if (t % 6.0) > 3.5 {
        p.rect(209.0, 45.0, 8.0, 7.0, "#2a221e");
        p.px(211.0, 47.0, "#d8d0c0");
        p.px(215.0, 47.0, "#d8d0c0");
    }
    // You, in the straitjacket, rocking.
    let rock = ((t * 2.0).sin() * 1.2).round();
    agent(p, 40.0 + rock, 114.0, Pose::Slump, "#b8b4a8", "#d8d4c8");
    // Failing fluorescent tube.
    p.rect(90.0, 2.0, 60.0, 2.0, "#e8f0e0");
    let dim = if hash((t * 12.0) as u32) < 0.08 { 0.6 } else { 0.25 };
    p.rect(0.0, 0.0, W, H, &format!("rgba(0,0,0,{dim})"));
    p.light(110.0, 60.0, 170.0, 0.5);
}

// ---------- Arousal, low: coma ----------

/// The bullpen. You sway, the room goes dark at the edges, and you hit the floor.
pub(super) fn collapse(p: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, 84.0, "#1c1e1c");
    p.ink("#181a18");
    for x in (0..W as i32).step_by(20) {
        p.ctx.fill_rect(x as f64, 0.0, 1.0, 84.0);
    }
    for x in [20.0, 140.0] {
        p.rect(x, 12.0, 40.0, 30.0, "#0a0c10");
        for y in (13..41).step_by(3) {
            p.rect(x + 1.0, y as f64, 38.0, 2.0, "#20262c");
        }
    }
    p.rect(80.0, 16.0, 40.0, 26.0, "#3a2c1c");
    for (x, y) in [(84.0, 19.0), (98.0, 21.0), (108.0, 28.0), (88.0, 31.0)] {
        p.rect(x, y, 8.0, 6.0, "#8a8476");
    }
    for x in [196.0, 212.0] {
        p.rect(x, 50.0, 14.0, 34.0, "#2c2e30");
        p.rect(x + 1.0, 58.0, 12.0, 1.0, "#1a1c1e");
        p.rect(x + 1.0, 70.0, 12.0, 1.0, "#1a1c1e");
    }
    p.rect(0.0, 84.0, W, H - 84.0, "#2a2620");
    for y in (88..H as i32).step_by(8) {
        p.rect(0.0, y as f64, W, 1.0, "#24201b");
    }

    // Desks and the officers at them; they jump up when you fall.
    let rush = smoothstep((t - COLLAPSE_FALL - 0.6) / 1.5);
    for (i, (dx, ox)) in [(10.0, 30.0), (60.0, 76.0), (168.0, 176.0)].into_iter().enumerate() {
        p.rect(dx, 78.0, 34.0, 4.0, "#3a2c1e");
        p.rect(dx + 2.0, 82.0, 2.0, 16.0, "#2a1e14");
        p.rect(dx + 30.0, 82.0, 2.0, 16.0, "#2a1e14");
        p.rect(dx + 10.0, 66.0, 14.0, 12.0, "#1e1e20");
        p.rect(dx + 12.0, 68.0, 10.0, 7.0, "#2e4a3a");
        let standing = t > COLLAPSE_FALL + 0.5 + i as f64 * 0.2;
        let target = [96.0, 110.0, 134.0][i];
        let x = if standing { lerp(ox, target, rush) } else { ox };
        let pose = if !standing {
            Pose::Slump
        } else if rush < 1.0 {
            Pose::Run((t / STRIDE) as u32)
        } else {
            Pose::Kneel
        };
        agent(p, x, 106.0, pose, "#141a2a", "#3a4a6a");
    }

    // You, swaying, then down. Coffee everywhere.
    let ax = 112.0;
    let sway = if t < COLLAPSE_FALL { (t * 2.5).sin() * (t / COLLAPSE_FALL) * 2.0 } else { 0.0 };
    let pose = if t < COLLAPSE_FALL {
        Pose::Stand
    } else if t < COLLAPSE_FALL + 0.3 {
        Pose::Kneel
    } else {
        Pose::Lie
    };
    agent(p, ax + sway.round(), 112.0, pose, COAT, "#4a4a40");
    if t < COLLAPSE_FALL {
        p.rect(ax + 8.0 + sway.round(), 96.0, 3.0, 3.0, "#d8d0c0");
    } else {
        let r = ((t - COLLAPSE_FALL) * 6.0).min(6.0);
        p.rect(ax + 26.0 - r, 111.0, 4.0 + 2.0 * r, 1.0, "#3a2010");
        p.rect(ax + 26.0, 109.0, 3.0, 3.0, "#d8d0c0");
    }
    // Vision closing in: wide while you stand, a pinhole after the fall.
    let r = if t < COLLAPSE_FALL {
        lerp(170.0, 80.0, t / COLLAPSE_FALL) + (t * 4.0).sin() * 8.0
    } else {
        lerp(80.0, 45.0, (t - COLLAPSE_FALL) / 3.0)
    };
    p.light(ax + 10.0, 100.0, r, 0.92);
}

/// A hospital room at night: the monitor keeps time, nobody sits in the chair.
pub(super) fn hospital(p: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, 92.0, "#1a2024");
    p.rect(0.0, 92.0, W, H - 92.0, "#22282a");
    // Window with rain.
    p.rect(26.0, 16.0, 40.0, 40.0, "#0a0f17");
    for i in 0..14 {
        let x = 26.0 + hash(i + 300) * 40.0;
        let y = 16.0 + (hash(i + 330) * 40.0 + t * 20.0).rem_euclid(38.0);
        p.rect(x, y, 1.0, 2.0, "rgba(150,170,200,0.35)");
    }
    p.rect(45.0, 16.0, 1.0, 40.0, "#3a4044");
    p.rect(26.0, 35.0, 40.0, 1.0, "#3a4044");
    // Bed, you in it.
    p.rect(70.0, 88.0, 96.0, 12.0, "#cfd4d0");
    p.rect(70.0, 100.0, 96.0, 3.0, "#8a9090");
    p.rect(72.0, 103.0, 2.0, 16.0, "#5a6060");
    p.rect(162.0, 103.0, 2.0, 16.0, "#5a6060");
    p.rect(70.0, 76.0, 3.0, 24.0, "#5a6060");
    p.rect(74.0, 84.0, 14.0, 5.0, "#e8ece8"); // pillow
    p.rect(80.0, 80.0, 6.0, 5.0, SKIN);
    p.rect(80.0, 80.0, 6.0, 2.0, "#2a2420");
    p.rect(84.0, 82.0, 2.0, 1.0, "#4a3a30"); // closed eye
    p.rect(87.0, 81.0, 62.0, 8.0, "#9aaab0"); // blanket over the body
    p.rect(87.0, 81.0, 62.0, 1.0, "#c8d4d8");
    p.rect(86.0, 84.0, 4.0, 1.0, "#c8dce8"); // breathing tube
    // IV drip.
    p.rect(176.0, 30.0, 1.0, 70.0, "#6a7074");
    p.rect(172.0, 32.0, 8.0, 12.0, "rgba(200,220,230,0.5)");
    if (t * 1.3).fract() < 0.5 {
        p.px(176.0, 46.0 + ((t * 1.3).fract() * 10.0).floor(), "#c8dce8");
    }
    p.line(176.0, 44.0, 150.0, 86.0, "rgba(200,220,230,0.3)");
    // Empty chair.
    p.rect(30.0, 96.0, 16.0, 3.0, "#3a3028");
    p.rect(30.0, 80.0, 3.0, 16.0, "#3a3028");
    p.rect(31.0, 99.0, 2.0, 12.0, "#2a221c");
    p.rect(43.0, 99.0, 2.0, 12.0, "#2a221c");
    // Heart monitor with a live trace.
    p.rect(188.0, 50.0, 42.0, 30.0, "#1e2224");
    p.rect(190.0, 52.0, 38.0, 22.0, "#030805");
    p.glow(209.0, 63.0, 40.0, "74,223,106", 0.12);
    let mut prev = None;
    for c in 0..38 {
        let at = t - (37 - c) as f64 * 0.04;
        let y = 66.0 - ecg(at) * 10.0;
        if let Some((px, py)) = prev {
            p.line(px, py, 190.0 + c as f64, y, "#4adf6a");
        }
        prev = Some((190.0 + c as f64, y));
    }
    p.text("72", 220.0, 75.0, "#4adf6a");
    p.light(140.0, 70.0, 170.0, 0.55);
}

/// One heartbeat on the monitor: a spike right after each beep.
fn ecg(t: f64) -> f64 {
    let u = (t - MONITOR_FIRST).rem_euclid(MONITOR_BEAT);
    match u {
        u if u < 0.04 => -0.2,
        u if u < 0.08 => 1.0,
        u if u < 0.12 => -0.4,
        u if (0.3..0.45).contains(&u) => 0.2,
        _ => 0.0,
    }
}

// ---------- Arousal, high: fatal overload ----------

/// Home, alone. Heart racing, hand on the wall, short of breath. Then down.
pub(super) fn overload(p: &Pen, t: f64) {
    let down = t >= OVERLOAD_FALL;
    // Heart rate climbs until it stops; the screen shakes and flushes red with it.
    let beats = 1.4 * t + 0.09 * t * t;
    let beat = if down { 0.0 } else { (1.0 - beats.fract() * 4.0).max(0.0) };
    let shake = if down { 0.0 } else { (t / OVERLOAD_FALL) * 1.5 };
    let (sx, sy) = (
        ((hash((t * 30.0) as u32) - 0.5) * 2.0 * shake).round(),
        ((hash((t * 30.0) as u32 + 7) - 0.5) * 2.0 * shake).round(),
    );
    p.ctx.save();
    p.ctx.translate(sx, sy).ok();
    apartment(p, t, false);
    let pose = if !down {
        Pose::Lean
    } else if t < OVERLOAD_FALL + 0.3 {
        Pose::Kneel
    } else {
        Pose::Lie
    };
    let breath = if down { 0.0 } else { ((t * 5.0).sin() * 1.0).round() };
    let x = if pose == Pose::Lie { 130.0 } else { 145.0 };
    agent(p, x, 112.0 + breath.min(0.0), pose, COAT, "#5a4a30");
    p.ctx.restore();
    if beat > 0.0 {
        p.light(120.0, 70.0, 130.0, 0.5 * beat);
        p.rect(0.0, 0.0, W, H, &format!("rgba(150,10,10,{:.3})", 0.18 * beat));
    }
    p.light(150.0, 80.0, 150.0, 0.45);
}

/// The same room, later. Still. The phone on the side table rings and rings.
pub(super) fn aftermath(p: &Pen, t: f64) {
    apartment(p, t, ringing(t));
    agent(p, 130.0, 112.0, Pose::Lie, COAT, "#2a2a30");
    // Grey dawn creeping in.
    let dawn = (t / 12.0).min(1.0);
    p.rect(0.0, 0.0, W, H, &format!("rgba(120,130,150,{:.3})", 0.06 * dawn));
    p.light(120.0, 70.0, 160.0, 0.65 - 0.15 * dawn);
}

fn ringing(t: f64) -> bool {
    let (first, every, len, times) = RINGS;
    let k = (t - first) / every;
    k >= 0.0 && k < times as f64 && (t - first).rem_euclid(every) < len
}

/// A small apartment at night: a lamp, case files on the floor, the wall
/// you lean on, a phone on the side table (shaking when `ring`).
fn apartment(p: &Pen, t: f64, ring: bool) {
    p.rect(0.0, 0.0, W, 96.0, "#211c1a");
    p.ink("#1d1816");
    for x in (0..W as i32).step_by(10) {
        p.ctx.fill_rect(x as f64, 0.0, 4.0, 96.0);
    }
    p.rect(0.0, 96.0, W, H - 96.0, "#1a1410");
    for y in (99..H as i32).step_by(6) {
        p.rect(0.0, y as f64, W, 1.0, "#140f0c");
    }
    // Window: city lights.
    p.rect(20.0, 16.0, 44.0, 40.0, "#0a0d14");
    for i in 0..18 {
        let c = if hash(i + 60) < 0.3 { "#8a7444" } else { "#3a4452" };
        p.px(22.0 + hash(i + 20) * 40.0, 30.0 + hash(i + 40) * 24.0, c);
    }
    p.rect(41.0, 16.0, 1.0, 40.0, "#2a2420");
    // The wall corner you lean on.
    p.rect(160.0, 0.0, 10.0, 112.0, "#2a2422");
    p.rect(160.0, 0.0, 1.0, 112.0, "#3a3230");
    // Lamp and side table with the phone.
    p.rect(90.0, 78.0, 18.0, 3.0, "#3a2c1e");
    p.rect(92.0, 81.0, 2.0, 15.0, "#2a1e14");
    p.rect(104.0, 81.0, 2.0, 15.0, "#2a1e14");
    let shake = if ring && (t * 30.0).sin() > 0.0 { 1.0 } else { 0.0 };
    p.rect(95.0 + shake, 73.0, 8.0, 5.0, "#141414");
    p.rect(94.0 + shake, 71.0, 10.0, 2.0, "#141414");
    if ring {
        p.glow(99.0, 72.0, 14.0, "255,240,200", 0.2);
    }
    p.rect(200.0, 46.0, 1.0, 50.0, "#2a2620");
    for (i, y) in (40..46).enumerate() {
        p.rect(197.0 - i as f64, y as f64, 8.0 + 2.0 * i as f64, 1.0, "#5a4a30");
    }
    p.glow(200.0, 50.0, 50.0, "230,190,120", 0.15);
    // Case files everywhere.
    for i in 0..10 {
        let x = 20.0 + hash(i + 100) * 200.0;
        let y = 104.0 + hash(i + 120) * 26.0;
        p.rect(x, y, 8.0, 3.0, "#6a5838");
        p.rect(x + 1.0, y - 1.0, 6.0, 1.0, "#c8c0a8");
    }
}

// ---------- The truth ----------

/// The thirteenth floor. Door 1313 stands open on a blinding light; the
/// shadow waits beside it. You walk the length of the corridor and go in.
pub(super) fn threshold(p: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, 96.0, "#18201b");
    p.ink("#141a16");
    for x in (0..W as i32).step_by(8) {
        p.ctx.fill_rect(x as f64, 0.0, 3.0, 96.0);
    }
    p.rect(0.0, 0.0, W, 8.0, "#0e0c0c");
    p.rect(0.0, 96.0, W, H - 96.0, "#1a0e10");
    p.rect(0.0, 104.0, W, 14.0, "#3a1416");
    p.rect(0.0, 96.0, W, 1.0, "#2a2018");
    // Doors 1311 and 1312, shut. 1313, wide open.
    for (x, n) in [(24.0, "1311"), (92.0, "1312")] {
        p.rect(x - 2.0, 46.0, 22.0, 50.0, "#2e2016");
        p.rect(x, 48.0, 18.0, 48.0, "#24180f");
        p.text(n, x + 1.0, 40.0, "#8a7444");
        p.px(x + 15.0, 74.0, "#a08a5a");
    }
    let (dx, open) = (176.0, 18.0);
    p.rect(dx - 2.0, 46.0, 22.0, 50.0, "#2e2016");
    p.rect(dx, 48.0, 18.0, 48.0, "#fff6e0");
    p.rect(dx - open * 0.3, 48.0, 4.0, 48.0, "#24180f");
    p.text("1313", dx + 1.0, 40.0, "#c0392b");
    // The light pours out across the carpet and up the walls, breathing.
    let breath = 0.75 + 0.25 * (t * 1.3).sin();
    p.glow(dx + 9.0, 72.0, 90.0, "255,244,214", 0.45 * breath);
    for y in 96..H as i32 {
        let k = (y - 96) as f64;
        let c = format!("rgba(255,244,214,{:.3})", 0.22 * breath * (1.0 - k / 40.0));
        p.rect(dx - k * 1.4, y as f64, 18.0 + k * 1.4, 1.0, &c);
    }
    // Sconces, failing one after another as you pass.
    for (i, x) in [60.0, 132.0].into_iter().enumerate() {
        let dying = t > 3.0 + i as f64 * 3.5;
        p.rect(x - 2.0, 30.0, 5.0, 4.0, "#3a3020");
        if !dying || hash((t * 20.0) as u32 + i as u32) < 0.1 {
            p.glow(x, 32.0, 26.0, "240,200,130", 0.25);
            p.rect(x - 1.0, 28.0, 3.0, 2.0, "#f0d890");
        }
    }
    // The shadow, waiting at the door.
    let sx = 208.0;
    for y in 52..112 {
        let yf = y as f64;
        let hw = if yf < 62.0 { 3.0 } else { (3.0 + (yf - 62.0) * 0.12).min(6.0) };
        let drift = ((yf * 0.3 + t * 2.0).sin()).round();
        p.rect(sx - hw + drift, yf, hw * 2.0, 1.0, "#000000");
        p.px(sx - hw - 1.0 - hash(y as u32 + (t * 6.0) as u32) * 3.0 + drift, yf, "rgba(32,28,44,0.6)");
    }
    p.px(sx - 1.0, 56.0, "#ece8f4");
    p.px(sx + 1.0, 56.0, "#ece8f4");
    // You, walking into the light.
    let x = 16.0 + t * 14.0;
    if x < dx + 4.0 {
        let fade = ((dx + 4.0 - x) / 20.0).min(1.0);
        agent(p, x, 112.0, Pose::Walk((t * 3.0) as u32), COAT, "#e8d8b0");
        if fade < 1.0 {
            p.glow(x + 4.0, 100.0, 20.0, "255,244,214", 0.8 * (1.0 - fade));
        }
    }
    p.light(150.0, 70.0, 190.0, 0.5);
}

/// Inside 1313 there is no room: stars, a serpent of gold biting its own
/// tail, and at its heart you, facing someone made of light who has your shape.
pub(super) fn revelation(p: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, H, "#04040a");
    for i in 0..120u32 {
        let tw = (t * (1.0 + hash(i + 70) * 2.0) + hash(i + 90) * TAU).sin();
        if tw > -0.3 {
            let c = if hash(i + 110) < 0.15 { "#c8a040" } else { "#8a8aa8" };
            p.px(hash(i + 30) * W, hash(i + 50) * H, c);
        }
    }
    let (cx, cy) = (120.0, 64.0);
    let grow = smoothstep(t / 8.0);
    p.glow(cx, cy, 70.0 + 40.0 * grow, "255,236,190", 0.25 + 0.35 * grow);
    sigil(p, cx, cy, 52.0, "rgba(192,57,43,0.18)", "rgba(192,57,43,0.4)");
    // The serpent: a ring of scales sliding round, the head swallowing the tail at the top.
    let turn = t * 0.25;
    for k in 0..180 {
        let a = k as f64 / 180.0 * TAU;
        if (a - TAU * 0.75).abs() < 0.12 {
            continue; // the gap the head closes
        }
        for (r, shade) in [(44.0, 0.0), (45.0, 1.0), (46.0, 0.0), (47.0, 1.0)] {
            let scale = ((a * 30.0 + turn * 30.0).sin() > 0.0) as u8 as f64;
            let c = match (shade + scale) as u8 {
                0 => "#6a4e18",
                1 => "#c8a040",
                _ => "#f0d878",
            };
            p.px((cx + a.cos() * r).round(), (cy + a.sin() * r * 0.8).round(), c);
        }
    }
    let (hx, hy) = (cx + 8.0, cy - 47.0 * 0.8);
    p.rect(hx - 6.0, hy - 3.0, 10.0, 6.0, "#c8a040");
    p.rect(hx - 6.0, hy - 3.0, 10.0, 1.0, "#f0d878");
    p.px(hx - 3.0, hy - 1.0, "#c0392b");
    p.rect(hx - 10.0, hy, 4.0, 2.0, "#6a4e18");
    // You, and the one who wears your shape in light.
    agent(p, cx - 20.0, cy + 26.0, Pose::Stand, COAT, "#f0d878");
    let pulse = 0.8 + 0.2 * (t * 2.0).sin();
    p.glow(cx + 12.0, cy + 12.0, 24.0, "255,250,230", 0.6 * pulse);
    for y in 0..26 {
        let yf = y as f64;
        let (x0, w) = if yf < 4.0 { (4.0, 4.0) } else if yf < 17.0 { (3.0, 7.0) } else { (3.0, 6.0) };
        p.rect(cx + 6.0 + x0, cy + yf, w, 1.0, "#fff8e8");
    }
    // At the very end, everything goes white.
    let white = smoothstep((t - 9.0) / 3.0);
    if white > 0.0 {
        p.rect(0.0, 0.0, W, H, &format!("rgba(255,248,232,{white:.3})"));
    }
}

// ---------- Documents ----------

/// The ending's last word: a paper on a dark desk, a headline, maybe a stamp.
pub(super) fn document(p: &Pen, t: f64, doc: Doc) {
    p.rect(0.0, 0.0, W, H, "#120e0c");
    p.ink("#16110e");
    for y in (0..H as i32).step_by(7) {
        p.ctx.fill_rect(0.0, y as f64, W, 1.0);
    }
    let rise = (1.0 - smoothstep(t / 0.9)) * 120.0;
    let (x0, y0) = (64.0, 8.0 + rise);
    p.rect(x0 + 3.0, y0 + 3.0, 112.0, 124.0, "rgba(0,0,0,0.5)");
    p.rect(x0, y0, 112.0, 124.0, "#d6ceb8");
    p.rect(x0, y0, 112.0, 1.0, "#e8e2d0");

    let ink = "#1e1a16";
    let (masthead, headline, stamp): (&str, [&str; 2], Option<&str>) = match doc {
        Doc::Dismissal => ("THE EVENING STAR", ["DETECTIVE", "DISMISSED"], None),
        Doc::Assassination => ("THE EVENING STAR", ["DETECTIVE", "SHOT DEAD"], None),
        Doc::Haunting => ("POLICE INCIDENT REPORT", ["ACCIDENTAL", "DEATH"], Some("CLOSED")),
        Doc::Committed => ("ST. AGNES HOSPITAL", ["INVOLUNTARY", "COMMITMENT"], Some("NO VISITS")),
        Doc::Coma => ("CITY GENERAL HOSPITAL", ["PATIENT", "IN COMA"], None),
        Doc::Overload => ("OFFICE OF THE CORONER", ["CARDIAC", "ARREST"], Some("DECEASED")),
        Doc::Truth => ("THE EVENING STAR", ["DETECTIVE", "VANISHES"], Some("CLOSED")),
    };
    let center = |s: &str, scale: f64| x0 + 56.0 - (s.len() as f64 * 4.0 - 1.0) * scale / 2.0;
    p.text(masthead, center(masthead, 1.0), y0 + 6.0, ink);
    p.rect(x0 + 6.0, y0 + 13.0, 100.0, 1.0, ink);
    for (i, line) in headline.iter().enumerate() {
        p.text_at_scale(line, center(line, 2.0), y0 + 18.0 + i as f64 * 13.0, 2.0, ink);
    }
    p.rect(x0 + 6.0, y0 + 44.0, 100.0, 1.0, ink);

    // A picture box, then columns of text you can't quite read.
    let (fx, fy) = (x0 + 8.0, y0 + 50.0);
    p.rect(fx, fy, 40.0, 30.0, "#8a8476");
    match doc {
        Doc::Dismissal => shield(p, fx + 16.0, fy + 10.0),
        Doc::Assassination => {
            p.rect(fx + 1.0, fy + 18.0, 38.0, 11.0, "#5a5650");
            p.line(fx + 8.0, fy + 22.0, fx + 30.0, fy + 22.0, "#e8e4d8");
            p.line(fx + 8.0, fy + 26.0, fx + 30.0, fy + 26.0, "#e8e4d8");
            ring(p, fx + 32.0, fy + 24.0, 2.0, "#e8e4d8");
        }
        Doc::Haunting => {
            p.ink("#5a5650");
            for k in 0..6 {
                p.ctx.fill_rect(fx + 4.0 + k as f64 * 5.0, fy + 4.0 + k as f64 * 4.0, 6.0, 30.0 - k as f64 * 4.0 - 4.0);
            }
        }
        Doc::Committed => sigil(p, fx + 20.0, fy + 15.0, 12.0, "#5a1612", "#5a1612"),
        Doc::Truth => {
            // The door of 1313, ajar, light spilling out.
            p.text("1313", fx + 13.0, fy + 2.0, "#1e1a16");
            p.rect(fx + 13.0, fy + 8.0, 14.0, 22.0, "#3a362e");
            p.rect(fx + 15.0, fy + 10.0, 10.0, 20.0, "#f0ead8");
            p.rect(fx + 15.0, fy + 10.0, 4.0, 20.0, "#5a5446");
        }
        Doc::Coma | Doc::Overload => {
            let flat = doc == Doc::Overload;
            let mut prev = (fx + 2.0, fy + 15.0);
            for c in 1..36 {
                let v = if flat && c > 18 { 0.0 } else { ecg(c as f64 * 0.04 + MONITOR_FIRST - 0.2) };
                let next = (fx + 2.0 + c as f64, fy + 15.0 - v * 8.0);
                p.line(prev.0, prev.1, next.0, next.1, "#1e3a26");
                prev = next;
            }
        }
    }
    p.ink("#8a8476");
    for row in 0..12 {
        let y = y0 + 50.0 + row as f64 * 3.0;
        let w = 50.0 - hash(row + doc as u32 * 31) * 10.0;
        p.ctx.fill_rect(x0 + 54.0, y.floor(), w.floor(), 1.0);
    }
    for row in 0..12 {
        let y = y0 + 86.0 + row as f64 * 3.0;
        if y > y0 + 120.0 {
            break;
        }
        let w = 98.0 - hash(row + 40 + doc as u32 * 31) * 20.0;
        p.ctx.fill_rect(x0 + 8.0, y.floor(), w.floor(), 1.0);
    }
    if doc == Doc::Committed {
        // The patient's scribbles in the margin.
        for k in 0..5 {
            let a = k as f64 * TAU / 5.0 + 0.3;
            p.px(x0 + 100.0 + a.cos() * 4.0, y0 + 100.0 + a.sin() * 4.0, "#5a1612");
        }
    }

    if let Some(stamp) = stamp {
        if t >= STAMP_AT {
            let w = stamp.len() as f64 * 8.0 + 6.0;
            let (sx, sy) = (x0 + 104.0 - w, y0 + 96.0);
            p.rect(sx, sy, w, 16.0, "rgba(170,30,25,0.85)");
            p.rect(sx + 2.0, sy + 2.0, w - 4.0, 12.0, "#d6ceb8");
            p.text_at_scale(stamp, sx + 4.0, sy + 3.0, 2.0, "rgba(170,30,25,0.9)");
        }
    }
    p.light(120.0, 68.0, 150.0, 0.55);
}

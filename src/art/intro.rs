//! The intro's shots: city, hotel, e-reader, room 1313, CCTV, office.

use std::f64::consts::TAU;

use super::*;

/// Seconds into the city shot when lightning strikes (the thunder follows).
pub const LIGHTNING: [f64; 2] = [2.6, 8.4];

/// Where the CCTV footage is by the time the office shot picks it up on the monitor.
const FEED_OFFSET: f64 = 13.0;
/// The monitor's screen inside the office shot: exactly 1/5 of the frame,
/// so the pull-back starts with the feed filling the whole screen.
const MONITOR: (f64, f64, f64, f64) = (96.0, 46.0, 48.0, 27.0);

/// Starts on the monitor filling the frame, then eases back to the whole office.
pub(super) fn pull_back(art: &Art, t: f64) {
    cctv(&art.feed, FEED_OFFSET + t);
    office(&art.office, &art.feed, t);

    let k = smoothstep((t - 0.6) / 4.5);
    let (mx, my, mw, mh) = MONITOR;
    let sw = mw + (W - mw) * k;
    let (sx, sy) = (mx * (1.0 - k), my * (1.0 - k));
    let scale = W / sw;
    let ctx = &art.screen.ctx;
    let _ = ctx.draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
        &art.office.canvas,
        sx,
        sy,
        sw,
        sw * H / W,
        0.0,
        0.0,
        W,
        H,
    );
    // Redraw the feed at full resolution over the monitor so it stays crisp while zoomed.
    let _ = ctx.draw_image_with_html_canvas_element_and_dw_and_dh(
        &art.feed.canvas,
        (mx - sx) * scale,
        (my - sy) * scale,
        mw * scale,
        mh * scale,
    );
}

/// 13 October: a grey city drowning in rain.
pub(super) fn city(p: &Pen, t: f64) {
    let flash = lightning(t, &LIGHTNING);
    sky(p, 100.0, flash);
    clouds(p, t, flash);
    if flash > 0.6 {
        let first = t < LIGHTNING[1];
        bolt(p, if first { 172.0 } else { 58.0 }, if first { 11 } else { 47 });
    }
    let far = if flash > 0.35 { "#20242b" } else { "#101216" };
    skyline(p, 1, 112.0, (40.0, 78.0), far, false, t);
    p.rect(0.0, 60.0, W, 52.0, "rgba(40,46,56,0.18)");
    skyline(p, 2, 112.0, (26.0, 62.0), "#07080a", true, t);

    p.rect(0.0, 112.0, W, H - 112.0, "#0a0b0e");
    p.rect(0.0, 112.0, W, 1.0, "#16181d");
    for x in [34.0, 126.0, 204.0] {
        streetlamp(p, x, 84.0, 112.0, t);
    }
    rain(p, t, 170, 0);
    splashes(p, t, 113.0, 12);
}

/// The Hotel Lumen at night, a patrol car outside, and one window on the 13th floor.
pub(super) fn hotel(p: &Pen, t: f64) {
    sky(p, 120.0, 0.0);
    clouds(p, t, 0.0);
    skyline(p, 3, 120.0, (40.0, 86.0), "#0b0c0f", true, t);

    let (x0, x1) = (78.0, 162.0);
    let w = x1 - x0;
    // Water tower and cornice.
    p.rect(132.0, 1.0, 12.0, 8.0, "#120e0f");
    p.rect(133.0, 0.0, 10.0, 1.0, "#120e0f");
    p.rect(133.0, 9.0, 1.0, 2.0, "#120e0f");
    p.rect(142.0, 9.0, 1.0, 2.0, "#120e0f");
    p.rect(x0 - 3.0, 10.0, w + 6.0, 3.0, "#2a2123");
    p.rect(x0, 13.0, w, 107.0, "#1a1416");
    p.ink("#120e10");
    for i in 0..260 {
        p.ctx.fill_rect((x0 + hash(i + 700) * (w - 2.0)).floor(), (14.0 + hash(i + 800) * 90.0).floor(), 2.0, 1.0);
    }

    for floor in 1..=13u32 {
        let y = 104.0 - floor as f64 * 7.0;
        p.rect(x0, y + 6.0, w, 1.0, "#130e10");
        for j in 0..8u32 {
            let (wx, wy) = (82.0 + j as f64 * 10.0, y + 1.0);
            p.rect(wx - 1.0, wy + 4.0, 7.0, 1.0, "#2a2123");
            if floor == 13 && j == 3 {
                // Room 1313: dark, but something in there glows.
                let a = 0.35 + 0.25 * (t * 1.7).sin();
                p.rect(wx - 2.0, wy - 2.0, 9.0, 8.0, &format!("rgba(170,25,15,{:.3})", a * 0.25));
                p.rect(wx, wy, 5.0, 4.0, "#140808");
                p.rect(wx, wy, 5.0, 4.0, &format!("rgba(170,25,15,{a:.3})"));
                continue;
            }
            let r = hash(floor * 31 + j);
            let lit = r < 0.22 && (r > 0.03 || (t * 8.0).sin() > 0.0);
            p.rect(wx, wy, 5.0, 4.0, if lit { "#5e4c2c" } else { "#0b0a0c" });
            if lit && r < 0.12 {
                p.rect(wx, wy, 2.0, 4.0, "#3a2a1c"); // a drawn curtain
            }
        }
    }

    // Lobby, canopy and doors.
    p.rect(x0, 104.0, w, 16.0, "#151012");
    p.rect(84.0, 107.0, 20.0, 8.0, "#2a2216");
    p.rect(136.0, 107.0, 20.0, 8.0, "#2a2216");
    p.rect(110.0, 106.0, 20.0, 14.0, "#3e301c");
    p.rect(119.0, 106.0, 1.0, 14.0, "#1a120a");
    p.rect(100.0, 97.0, 40.0, 7.0, "#3a1213");
    p.text("LUMEN", 111.0, 98.0, "#b49a62");
    p.line(100.0, 113.0, 140.0, 111.0, "#b39a2e"); // police tape
    p.ink("#151012");
    for x in (102..140).step_by(5) {
        p.dot(x as f64, 113.0 - (x - 100) as f64 / 20.0);
    }

    // Vertical neon, its E dying.
    p.rect(67.0, 27.0, 10.0, 41.0, "#0c0809");
    p.rect(77.0, 32.0, 1.0, 1.0, "#2a2123");
    p.rect(77.0, 60.0, 1.0, 1.0, "#2a2123");
    p.rect(64.0, 24.0, 16.0, 47.0, "rgba(220,60,40,0.08)");
    for (k, letter) in ["H", "O", "T", "E", "L"].iter().enumerate() {
        let dead = k == 3 && ((t * 7.0).sin() < -0.2 || hash((t * 10.0) as u32) < 0.25);
        let color = if dead { "#3a1512" } else { "#e0483a" };
        p.text(letter, 71.0, 30.0 + k as f64 * 7.0, color);
    }

    // Street and the neon's reflection.
    p.rect(0.0, 120.0, W, H - 120.0, "#0a0a0d");
    p.rect(0.0, 120.0, W, 1.0, "#1a1a1e");
    for y in 121..H as i32 {
        let k = (y - 121) as f64 / 14.0;
        let dx = ((y as f64 + t * 5.0).sin() * 1.2).round();
        p.rect(69.0 + dx, y as f64, 6.0, 1.0, &format!("rgba(220,60,40,{:.3})", 0.2 * (1.0 - k)));
    }
    p.rect(108.0, 121.0, 24.0, 5.0, "rgba(160,120,60,0.08)");

    // Patrol car with its light bar washing over everything.
    let red = (t * 4.0) as i32 % 2 == 0;
    p.rect(28.0, 123.0, 30.0, 5.0, "#0c0d10");
    p.rect(34.0, 119.0, 16.0, 4.0, "#0c0d10");
    p.rect(36.0, 120.0, 5.0, 2.0, "#1c2430");
    p.rect(43.0, 120.0, 5.0, 2.0, "#1c2430");
    p.rect(31.0, 127.0, 4.0, 2.0, "#030303");
    p.rect(50.0, 127.0, 4.0, 2.0, "#030303");
    p.rect(39.0, 118.0, 3.0, 1.0, if red { "#ff3b30" } else { "#401010" });
    p.rect(42.0, 118.0, 3.0, 1.0, if red { "#10183a" } else { "#3a6bff" });
    rain(p, t, 120, 7);
    splashes(p, t, 121.0, 8);
    p.rect(0.0, 0.0, W, H, if red { "rgba(200,30,30,0.05)" } else { "rgba(40,70,220,0.05)" });
}

/// The broken e-reader on the hotel carpet, its screen stuck on the sigil.
pub(super) fn kindle(p: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, H, "#211012");
    // Old hotel carpet: a diamond lattice with a dot in each cell.
    p.ink("#2e1719");
    for y in 0..H as i32 {
        for x in 0..W as i32 {
            if (x + y) % 14 == 0 || (x - y).rem_euclid(14) == 0 {
                p.dot(x as f64, y as f64);
            }
        }
    }
    p.ink("#3b1c1e");
    for y in (0..H as i32).step_by(14) {
        for x in (7..W as i32).step_by(14) {
            p.dot(x as f64, y as f64);
        }
    }
    // Scorch marks and grit.
    for i in 0..16 {
        let (s, c) = (hash(i + 40) * TAU).sin_cos();
        let d = 26.0 + hash(i + 60) * 30.0;
        let (w, h) = (6.0 + hash(i + 80) * 10.0, 3.0 + hash(i + 90) * 5.0);
        p.rect(120.0 + c * d - w / 2.0, 64.0 + s * d * 0.7 - h / 2.0, w, h, "rgba(6,3,3,0.22)");
    }
    for i in 0..40 {
        let glint = i % 5 == 0 && (t * 2.0 + i as f64).sin() > 0.6;
        let c = if glint { "#b8b6ae" } else { "#4a4844" };
        p.px(hash(i + 200) * W, hash(i + 260) * H, c);
    }

    // A red halo that has no business being there.
    let a = 0.1 + 0.07 * (t * 2.1).sin();
    for k in 1..=4 {
        let k = k as f64;
        let c = format!("rgba(150,20,10,{:.3})", a / k);
        p.rect(92.0 - k, 24.0 - k, 56.0 + 2.0 * k, 80.0 + 2.0 * k, &c);
    }
    p.rect(92.0, 24.0, 56.0, 80.0, "#131315");
    p.rect(92.0, 24.0, 56.0, 1.0, "#2a2a2e");
    p.rect(92.0, 24.0, 1.0, 80.0, "#2a2a2e");
    p.rect(92.0, 103.0, 56.0, 1.0, "#070708");
    p.rect(147.0, 24.0, 1.0, 80.0, "#070708");
    p.rect(97.0, 29.0, 46.0, 62.0, "#0b0b0d");
    p.rect(112.0, 96.0, 16.0, 1.0, "#222226");

    // E-ink: every few seconds it does a full refresh (inverts for a beat).
    let refresh = t.rem_euclid(3.7) < 0.09;
    let (paper, ink) = if refresh { ("#121210", "#8d8b82") } else { ("#7b7970", "#15130f") };
    p.rect(98.0, 30.0, 44.0, 58.0, paper);
    // The book it was showing, burned under the sigil.
    for (i, y) in (33..87).step_by(3).enumerate() {
        let w = 30.0 + hash(i as u32 + 120) * 8.0;
        p.rect(101.0, y as f64, w, 1.0, "rgba(20,18,14,0.12)");
    }
    let eye = if t.rem_euclid(5.0) > 4.4 { "#c0392b" } else { ink };
    sigil(p, 120.0, 58.0, 17.0, ink, eye);
    p.rect(110.0, 30.0, 1.0, 58.0, "#3a3934");
    p.rect(131.0, 81.0, 6.0, 4.0, "#0a0a0a");
    // Cracks from the impact point, kept inside the glass.
    p.ctx.save();
    p.ctx.begin_path();
    p.ctx.rect(98.0, 30.0, 44.0, 58.0);
    p.ctx.clip();
    for k in 0..7u32 {
        let (mut x, mut y) = (131.0, 38.0);
        let mut a = hash(k + 300) * TAU;
        for seg in 0..4u32 {
            a += (hash(k * 10 + seg + 400) - 0.5) * 1.2;
            let len = 4.0 + hash(k * 10 + seg + 500) * 6.0;
            let (nx, ny) = (x + a.cos() * len, y + a.sin() * len);
            p.line(x, y, nx, ny, "#d8d6cf");
            (x, y) = (nx, ny);
        }
    }
    p.ctx.restore();

    let (lx, ly) = (120.0 + (t * 0.7).sin() * 12.0, 62.0 + (t * 0.5).cos() * 6.0);
    p.light(lx, ly, 90.0, 0.92);
}

/// Room 1313: torn apart, no power, sulfur hanging in the air.
pub(super) fn room(p: &Pen, t: f64) {
    // Wallpaper, stripes and damask dots.
    p.rect(0.0, 0.0, W, 88.0, "#18201b");
    p.ink("#141a16");
    for x in (0..W as i32).step_by(8) {
        p.ctx.fill_rect(x as f64, 0.0, 3.0, 88.0);
    }
    p.ink("#222c25");
    for y in (4..84).step_by(8) {
        for x in (5..W as i32).step_by(8) {
            p.dot(x as f64, y as f64);
        }
    }
    for (x, y, w, h) in [(22.0, 14.0, 9.0, 6.0), (58.0, 40.0, 6.0, 9.0), (104.0, 8.0, 8.0, 5.0)] {
        p.rect(x, y, w, h, "#2a261f");
        p.rect(x + 1.0, y + h, w - 2.0, 1.0, "#0f120f");
    }
    p.rect(0.0, 86.0, W, 2.0, "#0f0c0b");
    p.rect(0.0, 88.0, W, H - 88.0, "#211113");
    for y in (91..H as i32).step_by(5) {
        p.rect(0.0, y as f64, W, 1.0, "#190c0e");
    }

    // Window: rain, a cracked pane, lightning far away; one curtain torn down.
    let (wx, wy, ww, wh) = (150.0, 16.0, 36.0, 40.0);
    let flash = lightning(t, &[3.2, 9.7]);
    p.rect(wx - 2.0, wy - 2.0, ww + 4.0, wh + 4.0, "#2a2724");
    p.rect(wx, wy, ww, wh, if flash > 0.35 { "#3e4a5c" } else { "#0a0f17" });
    for i in 0..18 {
        let x = wx + hash(i) * ww;
        let y = wy + (hash(i + 30) * wh + t * (10.0 + hash(i + 60) * 25.0)).rem_euclid(wh - 2.0);
        p.rect(x, y, 1.0, 2.0, "rgba(150,170,200,0.35)");
    }
    p.rect(wx + ww / 2.0, wy, 1.0, wh, "#2a2724");
    p.rect(wx, wy + wh / 2.0, ww, 1.0, "#2a2724");
    p.line(wx + 4.0, wy + 5.0, wx + 12.0, wy + 14.0, "#5a6170");
    p.line(wx + 12.0, wy + 14.0, wx + 9.0, wy + 19.0, "#5a6170");
    p.line(wx + 12.0, wy + 14.0, wx + 16.0, wy + 16.0, "#5a6170");
    p.rect(wx - 8.0, wy - 5.0, ww + 16.0, 1.0, "#3a3530");
    p.rect(wx - 8.0, wy - 4.0, 7.0, 50.0, "#3b1517");
    p.rect(wx - 6.0, wy - 4.0, 1.0, 50.0, "#2a0e10");
    p.rect(wx + ww + 2.0, wy - 4.0, 5.0, 11.0, "#3b1517");
    p.rect(wx + ww + 3.0, wy + 7.0, 2.0, 6.0, "#3b1517");

    // Claw marks, and the sigil daubed on the wall.
    for k in 0..4 {
        let x = 70.0 + k as f64 * 4.0;
        p.line(x, 20.0, x + 7.0, 48.0, "#0b0d0b");
        p.line(x + 1.0, 20.0, x + 8.0, 48.0, "#26302a");
    }
    sigil(p, 124.0, 38.0, 11.0, "#5a1612", "#5a1612");
    p.rect(122.0, 50.0, 1.0, 5.0, "#4a1210"); // a drip

    // Wardrobe, one door hanging open on a black gap.
    p.rect(0.0, 24.0, 28.0, 2.0, "#2a1d14");
    p.rect(0.0, 26.0, 26.0, 62.0, "#1e1510");
    p.rect(1.0, 28.0, 12.0, 58.0, "#24190f");
    p.px(11.0, 56.0, "#5a4a30");
    p.rect(13.0, 28.0, 11.0, 58.0, "#050403");
    p.rect(15.0, 46.0, 5.0, 20.0, "#0d0a08"); // a coat on a hanger
    p.rect(24.0, 30.0, 5.0, 58.0, "#2a1d14");

    // A crooked picture over the bed.
    let frame = [(40.0, 16.0), (56.0, 13.0), (58.0, 25.0), (42.0, 28.0)];
    for k in 1..12 {
        let f = k as f64 / 12.0;
        let a = (frame[0].0 + (frame[3].0 - frame[0].0) * f, frame[0].1 + (frame[3].1 - frame[0].1) * f);
        let b = (frame[1].0 + (frame[2].0 - frame[1].0) * f, frame[1].1 + (frame[2].1 - frame[1].1) * f);
        p.line(a.0, a.1, b.0, b.1, if k > 7 { "#1a1f1c" } else { "#232833" });
    }
    for i in 0..4 {
        let (a, b) = (frame[i], frame[(i + 1) % 4]);
        p.line(a.0, a.1, b.0, b.1, "#4a3a24");
    }

    // A wall clock, stopped at 3:13.
    p.rect(211.0, 25.0, 11.0, 11.0, "#2a2620");
    p.rect(212.0, 26.0, 9.0, 9.0, "#4a463a");
    let (ccx, ccy) = (216.0, 30.0);
    let hour = (3.0 + 13.0 / 60.0) / 12.0 * TAU - TAU / 4.0;
    let minute = 13.0 / 60.0 * TAU - TAU / 4.0;
    p.line(ccx, ccy, ccx + hour.cos() * 2.5, ccy + hour.sin() * 2.5, "#0e0c0a");
    p.line(ccx, ccy, ccx + minute.cos() * 4.0, ccy + minute.sin() * 4.0, "#0e0c0a");

    // Dresser and an old TV with its screen smashed in.
    p.rect(196.0, 66.0, 40.0, 22.0, "#1d1510");
    p.rect(198.0, 73.0, 36.0, 1.0, "#120d09");
    p.rect(198.0, 80.0, 36.0, 1.0, "#120d09");
    p.rect(202.0, 48.0, 26.0, 18.0, "#161618");
    p.line(206.0, 48.0, 202.0, 41.0, "#3a3a3a"); // rabbit ears
    p.line(222.0, 48.0, 227.0, 42.0, "#3a3a3a");
    p.rect(205.0, 51.0, 20.0, 12.0, "#08090a");
    p.rect(212.0, 54.0, 5.0, 4.0, "#000");
    for (dx, dy) in [(-6.0, -3.0), (7.0, -2.0), (5.0, 5.0), (-5.0, 4.0)] {
        p.line(214.0, 56.0, 214.0 + dx, 56.0 + dy, "#3a3c40");
    }

    // Standing fan under the window, turning by itself.
    let (fx, fy) = (178.0, 76.0);
    p.rect(171.0, 114.0, 15.0, 2.0, "#262420");
    p.rect(177.0, 84.0, 2.0, 30.0, "#2a2824");
    p.rect(175.0, 82.0, 6.0, 4.0, "#22201c");
    let spin = t * 2.2 + (t * 0.7).sin() * 1.5;
    for k in 0..3 {
        let a = spin + k as f64 * TAU / 3.0;
        for da in [0.0, 0.25, 0.5] {
            p.line(fx, fy, fx + (a + da).cos() * 6.0, fy + (a + da).sin() * 6.0, "#4a4a42");
        }
    }
    ring(p, fx, fy, 8.0, "#5a5a52");
    for k in 0..8 {
        let a = k as f64 * TAU / 8.0;
        p.line(fx, fy, fx + a.cos() * 8.0, fy + a.sin() * 8.0, "rgba(90,90,82,0.35)");
    }
    p.rect(fx - 1.0, fy - 1.0, 2.0, 2.0, "#6a6a60");

    // The bed flipped over, mattress leaning on the wall.
    for r in 0..34 {
        p.rect(28.0 + r as f64 * 0.5, 54.0 + r as f64, 44.0, 1.0, "#4e4a3c");
    }
    p.rect(40.0, 62.0, 9.0, 5.0, "#3a3426");
    p.line(36.0, 72.0, 60.0, 80.0, "#24201a");
    p.px(48.0, 76.0, "#6a6458");
    p.px(52.0, 77.0, "#6a6458");
    p.rect(8.0, 98.0, 64.0, 6.0, "#2a1c14");
    p.rect(10.0, 90.0, 3.0, 8.0, "#2a1c14");
    p.rect(66.0, 90.0, 3.0, 8.0, "#2a1c14");

    // Lamp, nightstand, scattered papers.
    p.rect(92.0, 112.0, 10.0, 6.0, "#3a3020");
    p.line(102.0, 115.0, 114.0, 121.0, "#5a4a30");
    p.rect(113.0, 120.0, 5.0, 2.0, "#2a2014");
    p.rect(124.0, 100.0, 18.0, 12.0, "#20160f");
    p.rect(143.0, 106.0, 9.0, 6.0, "#1a120c");

    // Phone knocked to the floor, off the hook.
    p.rect(156.0, 118.0, 7.0, 3.0, "#141414");
    p.rect(157.0, 117.0, 5.0, 1.0, "#141414");
    p.rect(146.0, 125.0, 8.0, 2.0, "#141414");
    for k in 0..6 {
        p.px(156.0 - k as f64, 121.0 + (k / 2) as f64 + (k % 2) as f64, "#262626");
    }

    // Suitcase burst open, clothes spilled.
    p.rect(38.0, 106.0, 22.0, 6.0, "#3a2418");
    p.rect(38.0, 112.0, 22.0, 8.0, "#4a2e1c");
    p.rect(42.0, 113.0, 7.0, 3.0, "#4a4a5a");
    p.rect(52.0, 118.0, 10.0, 3.0, "#5a2e30");
    p.rect(30.0, 121.0, 6.0, 2.0, "#4a4a5a");

    // Armchair on its side.
    p.rect(198.0, 102.0, 28.0, 4.0, "#2e1416");
    p.rect(200.0, 106.0, 22.0, 14.0, "#3a1a1c");
    p.rect(203.0, 109.0, 16.0, 1.0, "#2a1012");
    p.rect(222.0, 108.0, 4.0, 1.0, "#1a120c");
    p.rect(222.0, 117.0, 4.0, 1.0, "#1a120c");
    for i in 0..16 {
        p.rect(14.0 + hash(i) * 210.0, 92.0 + hash(i + 20) * 40.0, 3.0, 2.0, "#6f6a5e");
    }

    // The ceiling bulb, still swinging.
    let bx = 84.0 + (t * 1.3).sin() * 2.0;
    p.line(84.0, 0.0, bx, 20.0, "#0a0a0a");
    p.rect(bx - 1.0, 20.0, 3.0, 2.0, "#2a2a24");
    p.px(bx + 1.0, 22.0, "#4a4a40");

    // Sulfur: slow yellow haze over everything.
    for i in 0..9 {
        let x = (hash(i + 300) * W + t * (3.0 + hash(i) * 4.0)).rem_euclid(W + 40.0) - 20.0;
        let y = 28.0 + hash(i + 310) * 90.0 + (t * 0.5 + i as f64).sin() * 3.0;
        p.rect(x - 14.0, y - 3.0, 28.0, 6.0, "rgba(160,150,50,0.05)");
        p.rect(x - 9.0, y - 6.0, 18.0, 12.0, "rgba(160,150,50,0.05)");
    }
    p.rect(0.0, 0.0, W, H, "rgba(110,100,30,0.07)");

    // No power: only a flashlight sweeping the room.
    let (lx, ly) = (120.0 + (t * 0.42).sin() * 78.0, 62.0 + (t * 0.9).sin() * 14.0);
    p.light(lx, ly, 80.0, 0.9);
    tv_sparks(p, t);
    // Something in the wardrobe, only when the beam looks away.
    if t > 4.0 && (lx - 18.0).abs() > 70.0 && (t * 0.7).fract() > 0.05 {
        p.px(15.0, 40.0, "#b0261c");
        p.px(19.0, 40.0, "#b0261c");
    }
}

/// Sparks spitting from the TV's smashed screen in irregular bursts.
/// Drawn over the darkness, since each burst briefly lights the set.
fn tv_sparks(p: &Pen, t: f64) {
    let Some(f) = spark(t) else { return };
    let flash = 0.3 * (1.0 - f);
    p.glow(214.0, 57.0, 30.0, "255,190,110", flash);
    p.rect(202.0, 48.0, 26.0, 18.0, &format!("rgba(80,70,60,{flash:.3})"));
    p.rect(212.0, 54.0, 5.0, 4.0, "#000");
    let frame = (t * 20.0) as u32;
    for i in 0..7 {
        let x = 214.0 + (hash(frame + i * 13) - 0.5) * 12.0 * f;
        let y = 56.0 + hash(frame + i * 29) * 4.0 + f * f * 16.0;
        p.px(x, y, if i % 2 == 0 { "#fff3c0" } else { "#ffb040" });
    }
}

/// Progress (0..1) through the TV's current spark burst in the room shot, if one is happening.
pub fn spark(t: f64) -> Option<f64> {
    const PERIOD: f64 = 1.1;
    const BURST: f64 = 0.2;
    let k = t / PERIOD;
    let at = k.fract() * PERIOD;
    (hash(k as u32 + 600) < 0.55 && at < BURST).then_some(at / BURST)
}

const G: [&str; 6] = ["#070d09", "#0f1a13", "#18281e", "#2a3f32", "#4e6a58", "#93b29c"];

/// Street camera footage of the man who rented the room. Choppy, green, noisy.
pub(super) fn cctv(p: &Pen, t: f64) {
    let t = (t * 8.0).floor() / 8.0;
    let frame = (t * 8.0) as u32;

    // Brick wall, back door, barred window.
    p.rect(0.0, 0.0, W, 88.0, G[1]);
    p.ink(G[0]);
    for (row, y) in (2..86).step_by(6).enumerate() {
        p.ctx.fill_rect(0.0, y as f64, W, 1.0);
        for x in ((row % 2) * 6..W as usize).step_by(12) {
            p.ctx.fill_rect(x as f64, y as f64 + 1.0, 1.0, 5.0);
        }
    }
    p.rect(33.0, 51.0, 20.0, 37.0, G[2]);
    p.rect(34.0, 52.0, 18.0, 36.0, G[0]);
    p.rect(150.0, 30.0, 24.0, 20.0, G[0]);
    for x in (152..174).step_by(4) {
        p.rect(x as f64, 30.0, 1.0, 20.0, G[2]);
    }
    p.rect(56.0, 80.0, 8.0, 8.0, G[0]);

    // Sidewalk, curb, road.
    p.rect(0.0, 88.0, W, 18.0, G[2]);
    for x in (10..W as i32).step_by(22) {
        p.rect(x as f64, 88.0, 1.0, 18.0, G[1]);
    }
    p.rect(0.0, 106.0, W, 2.0, G[3]);
    p.rect(0.0, 108.0, W, H - 108.0, G[0]);
    for x in (6..W as i32).step_by(24) {
        p.rect(x as f64, 121.0, 10.0, 1.0, G[2]);
    }
    p.rect(120.0, 98.0, 22.0, 3.0, G[3]);

    // Streetlamp.
    p.rect(206.0, 18.0, 2.0, 88.0, G[0]);
    p.rect(196.0, 18.0, 10.0, 1.0, G[0]);
    p.rect(193.0, 18.0, 6.0, 2.0, G[5]);
    for y in 20..106 {
        let hw = (y - 20) as f64 * 0.32;
        p.rect(196.0 - hw, y as f64, hw * 2.0, 1.0, "rgba(147,178,156,0.06)");
    }

    // He walks, stops, looks back over his shoulder, then hurries off.
    let (x, looking) = if t < 5.0 {
        (18.0 + t * 14.0, false)
    } else if t < 7.6 {
        (88.0, true)
    } else {
        (88.0 + (t - 7.6) * 34.0, false)
    };
    if x < W {
        walker(p, x.round(), 100.0, if looking { 1 } else { frame / 2 }, looking);
    }
    // What he saw: there for three frames, then gone.
    if (6.3..6.7).contains(&t) {
        stranger(p, 40.0, 100.0);
    }
    // Later it waits in the middle of the frame, looking at the camera.
    if t > 13.5 {
        stranger(p, 117.0, 100.0);
    }

    // Tape noise, scanlines, a rolling tracking band, vignette.
    p.ink("rgba(147,178,156,0.35)");
    for i in 0..140 {
        p.dot(hash(frame * 211 + i) * W, hash(frame * 157 + i + 3) * H);
    }
    if hash(frame / 6 + 900) < 0.35 {
        let y = (t * 40.0).rem_euclid(H + 20.0) - 10.0;
        p.rect(0.0, y, W, 4.0, "rgba(147,178,156,0.10)");
    }
    p.ink("rgba(0,0,0,0.22)");
    for y in (0..H as i32).step_by(2) {
        p.ctx.fill_rect(0.0, y as f64, W, 1.0);
    }
    p.light(W / 2.0, H / 2.0, 150.0, 0.6);

    // HUD.
    p.text("CAM 04", 6.0, 6.0, G[5]);
    if (t * 2.0).fract() < 0.5 {
        p.rect(W - 26.0, 7.0, 3.0, 1.0 + 2.0, "#d0302a");
        p.rect(W - 27.0, 8.0, 5.0, 1.0, "#d0302a");
    }
    p.text("REC", W - 19.0, 6.0, G[5]);
    let secs = 7 + t as u32;
    let stamp = format!("13/10 03:{:02}:{:02}", 13 + secs / 60, secs % 60);
    p.text(&stamp, 6.0, H - 11.0, G[5]);
}

/// A hunched man in a long coat and hat, standing on `ground`.
fn walker(p: &Pen, x: f64, ground: f64, step: u32, looking: bool) {
    let c = "#030604";
    let top = ground - 28.0;
    p.rect(x + 1.0, top + 2.0, 7.0, 1.0, c);
    p.rect(x + 2.0, top, 5.0, 2.0, c);
    let head = if looking { x + 1.0 } else { x + 2.0 };
    p.rect(head, top + 3.0, 4.0, 4.0, c);
    // The lamp catches his face on the side he's facing.
    p.rect(if looking { head } else { head + 3.0 }, top + 4.0, 1.0, 2.0, G[3]);
    p.rect(x + 1.0, top + 7.0, 7.0, 13.0, c);
    p.rect(x + 1.0, top + 6.0, 2.0, 1.0, c);
    p.rect(x + 6.0, top + 6.0, 2.0, 1.0, c);
    p.rect(x, top + 18.0, 9.0, 2.0, c);
    p.rect(x + 8.0, top + 8.0, 1.0, 10.0, G[3]);
    if step.is_multiple_of(2) {
        p.rect(x + 3.0, ground - 8.0, 3.0, 8.0, c);
    } else {
        p.rect(x + 1.0, ground - 8.0, 2.0, 8.0, c);
        p.rect(x + 6.0, ground - 8.0, 2.0, 8.0, c);
    }
}

/// Too tall, too thin, darker than the dark.
fn stranger(p: &Pen, x: f64, ground: f64) {
    let c = "#010201";
    p.rect(x + 1.0, ground - 46.0, 4.0, 5.0, c);
    p.rect(x, ground - 41.0, 6.0, 26.0, c);
    p.rect(x + 1.0, ground - 15.0, 1.0, 15.0, c);
    p.rect(x + 4.0, ground - 15.0, 1.0, 15.0, c);
    p.rect(x - 1.0, ground - 40.0, 1.0, 22.0, c);
    p.rect(x + 6.0, ground - 40.0, 1.0, 22.0, c);
    p.px(x + 1.0, ground - 44.0, G[5]);
    p.px(x + 4.0, ground - 44.0, G[5]);
}

/// The precinct at night: you, from behind, watching the footage.
fn office(p: &Pen, feed: &Pen, t: f64) {
    p.rect(0.0, 0.0, W, H, "#0d0b0e");
    p.ink("#0a080b");
    for x in (0..W as i32).step_by(20) {
        p.ctx.fill_rect(x as f64, 0.0, 1.0, 92.0);
    }

    // Window blinds, rain shadows sliding down the slats.
    p.rect(14.0, 8.0, 60.0, 58.0, "#07080b");
    for y in (9..65).step_by(3) {
        p.rect(15.0, y as f64, 58.0, 2.0, "#161a20");
    }
    for i in 0..16 {
        let x = 16.0 + hash(i) * 56.0;
        let y = 9.0 + (hash(i + 50) * 56.0 + t * 20.0 * (0.5 + hash(i + 90))).rem_euclid(54.0);
        p.rect(x, y, 1.0, 3.0, "rgba(0,0,0,0.55)");
    }

    // Calendar on the 13th; the evidence board with its red string.
    p.rect(150.0, 14.0, 15.0, 16.0, "#4a463e");
    p.rect(150.0, 14.0, 15.0, 4.0, "#7a2420");
    p.text("13", 154.0, 21.0, "#1a1714");
    p.rect(176.0, 12.0, 52.0, 44.0, "#17110c");
    p.rect(177.0, 13.0, 50.0, 42.0, "#2a1f16");
    let notes = [
        (180.0, 16.0, 10.0, 8.0, "#5a5446"),
        (200.0, 14.0, 8.0, 10.0, "#4a4a48"),
        (216.0, 20.0, 9.0, 7.0, "#5a5446"),
        (184.0, 36.0, 12.0, 9.0, "#3e3e3c"),
        (206.0, 38.0, 10.0, 8.0, "#5a5446"),
    ];
    for &(x, y, w, h, c) in &notes {
        p.rect(x, y, w, h, c);
    }
    for pair in [(0, 1), (1, 2), (0, 3), (3, 4), (1, 4)] {
        let (a, b) = (notes[pair.0], notes[pair.1]);
        p.line(a.0 + a.2 / 2.0, a.1, b.0 + b.2 / 2.0, b.1, "#8a2a22");
    }

    // Desk, keyboard, folder, mug.
    p.rect(0.0, 92.0, W, H - 92.0, "#17110d");
    p.rect(0.0, 92.0, W, 1.0, "#2a2018");
    p.rect(98.0, 96.0, 44.0, 6.0, "#151514");
    p.ink("#22221f");
    for x in (100..140).step_by(3) {
        p.dot(x as f64, 98.0);
        p.dot(x as f64 + 1.0, 100.0);
    }
    p.rect(142.0, 98.0, 14.0, 2.0, "#6a5838");
    p.rect(140.0, 100.0, 46.0, 18.0, "#6a5838");
    p.rect(140.0, 117.0, 46.0, 1.0, "#3a2f1d");
    p.rect(176.0, 98.0, 9.0, 12.0, "#3a3a38");
    p.text("CASE", 145.0, 103.0, "#2a2216");
    p.text("358", 145.0, 110.0, "#a02a20");
    p.rect(64.0, 90.0, 7.0, 8.0, "#2c2724");
    p.rect(71.0, 92.0, 2.0, 4.0, "#2c2724");
    p.rect(65.0, 90.0, 5.0, 1.0, "#120b07");
    for k in 0..5 {
        let rise = (t * 6.0 + k as f64 * 3.0).rem_euclid(14.0);
        let x = 67.0 + ((88.0 - rise) * 0.5 + t).sin() * 1.5;
        let c = format!("rgba(200,200,200,{:.3})", 0.15 * (1.0 - rise / 14.0));
        p.px(x, 88.0 - rise, &c);
    }

    // The monitor, showing the feed.
    p.glow(120.0, 60.0, 50.0, "120,180,140", 0.12);
    p.rect(86.0, 36.0, 68.0, 48.0, "#151413");
    p.rect(86.0, 36.0, 68.0, 1.0, "#24221f");
    p.rect(94.0, 44.0, 52.0, 31.0, "#050605");
    let (mx, my, mw, mh) = MONITOR;
    let _ = p.ctx.draw_image_with_html_canvas_element_and_dw_and_dh(&feed.canvas, mx, my, mw, mh);
    p.px(148.0, 80.0, "#4adf6a");
    p.rect(114.0, 84.0, 12.0, 5.0, "#161514");
    p.rect(106.0, 89.0, 28.0, 3.0, "#1d1c1b");

    // Banker's lamp and its pool of light.
    for (i, y) in (64..72).enumerate() {
        p.rect(200.0 - i as f64, y as f64, 16.0 + 2.0 * i as f64, 1.0, "#1d3a28");
    }
    p.rect(194.0, 72.0, 28.0, 1.0, "#e6cf8f");
    p.line(208.0, 73.0, 214.0, 89.0, "#2a2a26");
    p.rect(206.0, 89.0, 16.0, 3.0, "#22221f");
    for y in 73..H as i32 {
        let hw = (y - 72) as f64 * 0.9;
        p.rect(208.0 - hw, y as f64, hw * 2.0, 1.0, "rgba(230,190,120,0.04)");
    }

    p.light(120.0, 60.0, 150.0, 0.55);

    // You, over the shoulder. Deliberately featureless.
    let (cx, cy, r) = (34.0, 100.0, 12.0);
    for y in (cy - r) as i32..H as i32 {
        let dy = y as f64 - cy;
        let hw = if dy < 8.0 {
            (r * r - dy * dy).max(0.0).sqrt()
        } else if dy < 12.0 {
            7.0
        } else {
            (8.0 + (dy - 12.0) * 1.6).min(56.0)
        };
        p.rect(cx - hw, y as f64, hw * 2.0, 1.0, "#040305");
        let rim = if dy < 4.0 && dy > -r + 1.0 { "#2b3d31" } else { "#18241c" };
        p.px(cx + hw - 1.0, y as f64, rim);
    }
}


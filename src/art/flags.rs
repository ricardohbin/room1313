//! Pixel-art flags for the language screen (flag emoji don't render on Windows).

use super::*;
use crate::i18n::Lang;

/// Each pixel of the flag is drawn `FLAG_SCALE`×`FLAG_SCALE` CSS pixels.
pub const FLAG_SCALE: f64 = 3.0;

/// The flag of the language's country, on its own canvas.
pub fn flag(doc: &Document, lang: Lang) -> Result<HtmlCanvasElement, JsValue> {
    let pen = match lang {
        Lang::EnUs => united_states(doc)?,
        Lang::PtBr => brazil(doc)?,
    };
    let canvas = pen.canvas;
    let style = canvas.style();
    style.set_property("width", &format!("{}px", canvas.width() as f64 * FLAG_SCALE))?;
    style.set_property("height", &format!("{}px", canvas.height() as f64 * FLAG_SCALE))?;
    Ok(canvas)
}

/// 25×13: thirteen stripes, one pixel each, and a canton of stars.
fn united_states(doc: &Document) -> Result<Pen, JsValue> {
    let (w, h) = (25.0, 13.0);
    let p = Pen::sized(doc, w, h)?;
    for y in 0..13 {
        p.rect(0.0, y as f64, w, 1.0, if y % 2 == 0 { "#b22234" } else { "#eee8dc" });
    }
    p.rect(0.0, 0.0, 10.0, 7.0, "#3c3b6e");
    for y in 0..7 {
        for x in 0..10 {
            if (x + y) % 2 == 1 && x < 9 && y < 6 && x > 0 && y > 0 {
                p.px(x as f64, y as f64, "#eee8dc");
            }
        }
    }
    Ok(p)
}

/// 21×15: the yellow rhombus on green, the blue globe and its white band.
fn brazil(doc: &Document) -> Result<Pen, JsValue> {
    let (w, h) = (21.0, 15.0);
    let p = Pen::sized(doc, w, h)?;
    p.rect(0.0, 0.0, w, h, "#009c3b");
    let (cx, cy) = (10.0, 7.0);
    for y in 0..15 {
        for x in 0..21 {
            let (dx, dy) = (x as f64 - cx, y as f64 - cy);
            if dx.abs() / 9.0 + dy.abs() / 6.0 <= 1.0 {
                p.px(x as f64, y as f64, "#f2d000");
            }
            let r = (dx * dx + dy * dy).sqrt();
            if r <= 3.6 {
                // The band: an arc of a larger circle centred below the globe.
                let band = (dx * dx + (dy - 8.0) * (dy - 8.0)).sqrt();
                let color = if (7.6..8.7).contains(&band) { "#eee8dc" } else { "#1b3a8c" };
                p.px(x as f64, y as f64, color);
            }
        }
    }
    // A few stars under the band.
    for (x, y) in [(9.0, 9.0), (11.0, 8.0), (12.0, 9.0), (8.0, 8.0)] {
        p.px(x, y, "#eee8dc");
    }
    Ok(p)
}

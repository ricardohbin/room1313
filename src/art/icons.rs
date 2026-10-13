//! Small pixel icons for the card game's HUD: attributes and inventory items.
//! Each icon is a grid of characters, one per pixel, coloured by `ink`.

use super::*;
use crate::cards::Item;
use crate::engine::Stat;

/// '#' bone, 'd' dull grey, 'r' title red, 'y' brass; anything else is empty.
fn ink(c: u8) -> Option<&'static str> {
    match c {
        b'#' => Some("#cfc4b6"),
        b'd' => Some("#5a5450"),
        b'r' => Some("#c0392b"),
        b'y' => Some("#c8a040"),
        _ => None,
    }
}

fn paint(doc: &Document, rows: &[&str]) -> Result<HtmlCanvasElement, JsValue> {
    let size = rows.len() as f64;
    let pen = Pen::sized(doc, size, size)?;
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.bytes().enumerate() {
            if let Some(color) = ink(c) {
                pen.px(x as f64, y as f64, color);
            }
        }
    }
    Ok(pen.canvas)
}

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
    paint(doc, &rows)
}

/// A 12×12 icon for an inventory item.
pub fn item_icon(doc: &Document, item: Item) -> Result<HtmlCanvasElement, JsValue> {
    let rows: [&str; 12] = match item {
        // The broken e-reader, the sigil's eye on its screen.
        Item::EReader => [
            ".##########.",
            ".#dddddddd#.",
            ".#d.####.d#.",
            ".#d#....#d#.",
            ".#d#.rr.#d#.",
            ".#d#.rr.#d#.",
            ".#d#....#d#.",
            ".#d.####.d#.",
            ".#dddddddd#.",
            ".#dddddddd#.",
            ".#####d####.",
            ".##########.",
        ],
        // A brass key on a red tag.
        Item::RoomKey => [
            "............",
            ".yyy........",
            "y...y.......",
            "y...yyyyyyyy",
            "y...y...y.y.",
            ".yyy....y.y.",
            "..r.........",
            ".rrr........",
            ".r#r........",
            ".r#r........",
            ".rrr........",
            "............",
        ],
        // A strip of film; something red in every frame.
        Item::Negatives => [
            "............",
            "############",
            "#.#.#.#.#.#.",
            "############",
            "#dddd#dddd##",
            "#drdd#ddrd##",
            "#drdd#ddrd##",
            "############",
            "#.#.#.#.#.#.",
            "############",
            "............",
            "............",
        ],
        // A cassette with a red label.
        Item::Tapes => [
            "............",
            "############",
            "#dddddddddd#",
            "#d########d#",
            "#d#rrrrrr#d#",
            "#d########d#",
            "#dd.dddd.dd#",
            "#d.#.dd.#.d#",
            "#dd.dddd.dd#",
            "#ddd####ddd#",
            "############",
            "............",
        ],
        // The eye amulet on its chain.
        Item::Amulet => [
            "y..........y",
            ".y........y.",
            "..y......y..",
            "...y....y...",
            "....yyyy....",
            "...y....y...",
            "..y.####.y..",
            "..y#.rr.#y..",
            "..y.####.y..",
            "...y....y...",
            "....yyyy....",
            "............",
        ],
        // A blank business card: a seal and a phone number, nothing else.
        Item::AgentCard => [
            "............",
            "............",
            "############",
            "#..........#",
            "#.yy.......#",
            "#yyyy......#",
            "#.yy.......#",
            "#..........#",
            "#.....dddd.#",
            "############",
            "............",
            "............",
        ],
    };
    paint(doc, &rows)
}

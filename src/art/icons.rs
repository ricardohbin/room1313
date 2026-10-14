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
        // A crescent moon with a closed eye: what the shadow showed you.
        Item::Dream => [
            "....####....",
            "..##........",
            ".#..........",
            ".#..........",
            "#...........",
            "#....d...d..",
            "#.....ddd...",
            "#...........",
            ".#..........",
            ".#..........",
            "..##........",
            "....####....",
        ],
        // An on-air microphone.
        Item::Fame => [
            "....####....",
            "...#dddd#...",
            "...#d#d##...",
            "...#dddd#...",
            "...#d#d##...",
            "...#dddd#...",
            "..#.####.#..",
            "..#......#..",
            "...##..##...",
            ".....##.....",
            ".....##.....",
            "..rr####rr..",
        ],
        // A trapdoor with stairs going down into the dark.
        Item::Underworld => [
            "############",
            "#dddddddddd#",
            "#d########d#",
            "#d#......#d#",
            "#d####...#d#",
            "#d#..###.#d#",
            "#d#....###d#",
            "#d#......#d#",
            "#d#..rr..#d#",
            "#d########d#",
            "#dddddddddd#",
            "############",
        ],
        // Water closing over a head: one ring, then two.
        Item::FirstImmersion => [
            "............",
            "............",
            "............",
            ".....##.....",
            "....####....",
            "############",
            "#dddddddddd#",
            ".##########.",
            "............",
            "...######...",
            "............",
            "............",
        ],
        Item::SecondImmersion => [
            "............",
            "............",
            "............",
            "............",
            "############",
            "#dddddddddd#",
            ".##########.",
            "............",
            "...######...",
            "............",
            "....rrrr....",
            "............",
        ],
        // A white feather, the quill's tip in red.
        Item::Redemption => [
            "..........##",
            ".........###",
            "........###.",
            ".......####.",
            "......####..",
            ".....####...",
            "....####....",
            "...####.....",
            "..d###......",
            "..dd........",
            ".r..........",
            "r...........",
        ],
        // Mrs. Pell's cat, sitting, staring.
        Item::Cat => [
            ".#.....#....",
            ".##...##....",
            ".#######....",
            ".#y###y#....",
            ".#######....",
            "..#####.....",
            ".#######....",
            "#########...",
            "#########.##",
            "#########..#",
            ".#######..#.",
            "..##.##.##..",
        ],
        // A dented hip flask.
        Item::Flask => [
            "....##......",
            "....dd......",
            "..######....",
            ".#dddddd#...",
            ".#d####d#...",
            ".#d#..#d#...",
            ".#d####d#...",
            ".#dddddd#...",
            ".#ddd.dd#...",
            ".#dddddd#...",
            "..######....",
            "............",
        ],
        // A microcassette recorder, the red light on.
        Item::Recorder => [
            "............",
            "..########..",
            "..#dddddd#..",
            "..#d####d#..",
            "..#d#..#d#..",
            "..#d####d#..",
            "..#dddddd#..",
            "..#d.dd.d#..",
            "..#dddddd#..",
            "..#dd##rd#..",
            "..########..",
            "............",
        ],
        // A stack of tracts, a red cross on the top one.
        Item::Pamphlets => [
            "............",
            "...######...",
            "..#..r...#..",
            "..#.rrr..#..",
            "..#..r...#..",
            "..#.####.#..",
            "..#......#..",
            "..########..",
            ".#########..",
            ".##########.",
            "..##########",
            "............",
        ],
        // A brass plaque on a wooden board.
        Item::Plaque => [
            "............",
            "dddddddddddd",
            "d##########d",
            "d#yyyyyyyy#d",
            "d#y......y#d",
            "d#y.yyyy.y#d",
            "d#y......y#d",
            "d#y.yy.y.y#d",
            "d#yyyyyyyy#d",
            "d##########d",
            "dddddddddddd",
            "............",
        ],
        // A hotel voucher with a torn edge, 1313 stamped in red.
        Item::Voucher => [
            "............",
            "............",
            "###########.",
            "#.........#.",
            "#.r.r.r.r.##",
            "#.r.r.r.r.#.",
            "#.........##",
            "#.dddddd..#.",
            "#.........##",
            "###########.",
            "............",
            "............",
        ],
        // A long bill with a big red total at the bottom.
        Item::Invoice => [
            "..########..",
            "..#......#..",
            "..#.dddd.#..",
            "..#......#..",
            "..#.ddd..#..",
            "..#.dd.d.#..",
            "..#.ddd..#..",
            "..#......#..",
            "..#.rrrr.#..",
            "..#......#..",
            "..#.#.#.##..",
            "..#.#.#.#...",
        ],
    };
    paint(doc, &rows)
}

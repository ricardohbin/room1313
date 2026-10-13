# Project memory: swipe-engine

Context for picking this project back up in a new session.

## Goal

A Reigns / Tinder-style narrative card game engine that runs in the browser.
Written in Rust and compiled to WebAssembly. The player drags a card left or
right to disagree or agree with a scenario, and that choice leads to another scene.

## Status (as of 2026-10-13): first draft done

Working and tested in headless Chrome:
- Drag a card with mouse or touch (pointer events). The card tilts as it moves,
  and the label for the side it leans to fades in at the top of the card.
- Releasing past 100px commits the choice. The card flies off-screen, the engine
  moves to the next scene, and the new card animates in. Shorter drags snap back.
- Input is ignored while the animation plays.
- 6 native unit tests pass (`cargo test`). Clippy is clean for the wasm target.

Unverified: after the browser test, the hint label was changed to sit on the
trailing edge (lean right → text on the left) so it stays on screen while
dragging. That change hasn't been rebuilt or retested.

## Title screen (added 2026-10-13, draft)

`src/splash.rs` runs before the game: a "press any key" gate (browsers only allow
audio after a user gesture), then a black hold with a rumble, then ROOM 1313
flickers in on a boom, then the menu (New Game / Continue (disabled) / Exit)
fades in over the looping theme. New Game fades to black and calls `view::mount`.
Exit can't close a tab, so it whispers "There is no exit."
`src/audio.rs` synthesizes all sound with Web Audio (no asset files): a detuned
drone plus an out-of-tune music-box lullaby, scheduled one loop ahead on the
audio clock. The intro timings (`*_MS` in splash.rs) mirror durations in style.css.

## Intro cinematic (added 2026-10-13, draft)

`src/intro.rs` plays between New Game and the first card: six pixel-art shots
with typed captions (rainy city → Hotel Lumen → e-reader with the sigil →
wrecked room in sulfur haze, lit only by a flashlight → green CCTV of the renter
looking back → camera pulls back from the monitor to the new officer's desk,
"What happened in room 1313?"). Click/Space/Enter finishes a caption or moves
on; Escape skips. `src/intro_art.rs` paints each shot as a pure function of
time on a 240×135 canvas (CSS `image-rendering: pixelated`), with its own 3×5
pixel font. The office shot renders the CCTV feed and the office offscreen and
zooms out from the monitor. The intro creates a new `Music` (rain, thunder, hum,
static, typewriter ticks, drone) since the title's fades out. `FADE_MS` in
intro.rs mirrors `.intro-caption` in style.css. Verified by screenshots in
headless Chrome; Escape-to-skip and the audio weren't checked by ear.
Full recipe and conventions: `CINEMATICS.md`.

## Architecture decisions

- **All logic in Rust.** The JS in `www/index.html` only calls `init()`.
  The DOM is built and driven from Rust via `web-sys`.
- **Pure core, thin browser layer.** `engine.rs` and `swipe.rs` have no browser
  dependencies, so they're tested with plain `cargo test`. Only `view.rs` touches the DOM.
- **The scene graph is validated once on load** (duplicate ids, missing start
  scene, choices pointing to missing scenes). After that, `Engine::choose` can't fail.
- **The engine records history** as `Vec<(scene_id, Side)>`. It isn't used yet;
  it's there for future game state and conditions.
- **One card element is reused** for every scene (content swapped, CSS classes
  toggled), so event listeners are attached only once.
- Crate type is `cdylib` (for wasm) plus `rlib` (for native tests).

## Files

- `src/engine.rs`: `Scene { id, speaker, text, left, right }`, `Choice { label, next }`, `Engine`, `EngineError`
- `src/swipe.rs`: `SwipeConfig` (threshold, rotation, vertical damping), `Drag`
- `src/view.rs`: DOM, pointer handlers, `commit()` animation sequence
- `src/demo.rs`: 4 hard-coded placeholder scenes (intro, riot, feast, war)
- `src/splash.rs`, `src/intro.rs`, `src/intro_art.rs`, `src/audio.rs`: title, intro, sound
- `src/lib.rs`: `#[wasm_bindgen(start)]` entry point (splash → intro → view)
- `www/index.html`, `www/style.css`; `www/pkg/` is generated (gitignored)

## Gotchas

- The animation time (300ms) appears in both `www/style.css` (`.card` transition)
  and `TRANSITION_MS` in `src/view.rs`. Keep them in sync.
- `rust-toolchain.toml` pins the stable channel plus `wasm32-unknown-unknown`,
  so rustup installs the wasm target automatically. The global Rust is stable
  1.99 (upgraded from 1.67 on 2026-10-13).
- `wasm-pack` was installed with `cargo install wasm-pack` (v0.15.0).
- Build: `wasm-pack build --target web --out-dir www/pkg --no-pack`
- Serve: `python3 -m http.server 8765 -d www`
- Browser testing approach that worked: headless Chrome with
  `--remote-debugging-port`, driven over CDP from a Node script, using
  `Input.dispatchMouseEvent` to simulate drags.

## Next steps (agreed to postpone; not started)

1. **Event/scene format.** Load scenes from data (JSON or RON?) instead of
   `demo.rs`. Format not decided yet.
2. **Story/history design.** The narrative itself and how events chain together.
3. Likely after that: Reigns-style game state (e.g. 4 stats changed by each
   choice), choices that only appear under certain conditions, and game over / endings.
4. Polish ideas: real portraits instead of the speaker's first letter, arrow-key input.

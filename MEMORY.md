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

`src/language.rs` comes first: pick a language (flag + native name; it doubles
as the user gesture browsers want before audio), then `src/splash.rs`: a black hold with a rumble, then ROOM 1313
flickers in on a boom, then the menu (New Game / Continue (disabled) / Exit)
fades in over the looping theme. New Game fades to black and calls `view::mount`.
Exit can't close a tab, so it whispers "There is no exit."
`src/audio/` synthesizes all sound with Web Audio (no asset files): a detuned
drone plus an out-of-tune music-box lullaby, scheduled one loop ahead on the
audio clock. The intro timings (`*_MS` in splash.rs) mirror durations in style.css.

## Cinematics (added 2026-10-13, draft)

All cutscenes run on one player, `src/cinematic.rs` (`Film` = scenes + score;
`Scene` builder with `cue`/`sync` sound hooks). Shots are pure functions of
time painted on a 240×135 canvas (`src/art/`), sound is procedural
(`src/audio/`: `score.rs` is a step sequencer + synth band, `foley.rs` effects).
Full recipe and conventions: `CINEMATICS.md`.

- **Intro** (`src/intro.rs`): six shots between New Game and the first card,
  ending "What happened in room 1313?" (case number on the folder is 358).
- **Endings** (`src/endings.rs`): six game-over films, one per attribute
  extreme in GAME.md, each with its own score (lament, thrash, psychobilly,
  asylum, requiem, hardcore) and a closing document. Played when an attribute hits 0 or 100
  (`Ending::of`). Preview with `#ending=<name>` + New Game;
  an ending reloads back to the title.

- **Credits** (`endings::credits`): roll after the true ending only, over the
  rainy hotel with the title lullaby: "A game by bin & zed", "Made in Sao Paulo,
  SP. Brazil." (the Room 13 font has no accents), "Built 100% with Claude
  Opus 5.5". Dev link `#credits`.

Verified by screenshots in headless Chrome (no console errors). Not verified:
audio by ear, Escape-to-skip.

## Main quest (added 2026-10-13, draft)

The road to the truth is in GAME.md ("The Road to the Truth") and `cards.rs`.
New cast: Sgt. Otis Holloway (gives the e-reader on the first card; dull
recurring news), a shadow, Mrs. Celeste Moreau (prophet). New items: dream →
occult subcelebrity → access to the underworld → first/second immersion →
redemption → the shadow's finale choice → `Ending::Truth`. Engine support:
`Scene::excludes`, `after(item, rounds)`, `weight`, `until(item)` (repeat until
accepted), `Choice::finale`, `Engine::outcome() -> Outcome::{Loss, Truth}`.
START_ITEMS is now empty. Balance is checked by a bot simulation test
(`the_quest_can_be_won`): about 73% wins, about 113 cards (with ~45
filler cards and 7 side items: cat, flask, recorder, pamphlets, plaque,
voucher, invoice; see GAME.md). When adding cards, check the sim and keep the
mean Arousal delta per choice near 0: Arousal-high is what kills most runs. Dev link: `#card=<id>`.

## Cards (added 2026-10-13, draft)

- **Attributes** live in `engine.rs`: `Stat` (Reputation, Sensitivity, Arousal),
  `Stats` (start at 50, clamped 0..100), `Effect` per `Choice`, `Condition`
  (GAME.md thresholds). `Engine::preview(side)` gives the stats a choice would
  produce; `choose` applies it. 0 or 100 is a loss: `view` waits for the gauge,
  then `lib.rs` plays `Ending::of(stat, extreme)`.
- **Deck** (`src/cards.rs`): six characters (hotel owner Mr. Ostrowski, tabloid
  reporter Vera Kask, occultist Ambrose Quill, conspiracist Dale Pruitt,
  fanatic Brother Josiah, the Agent), each card `Summoned` (testimony) or
  `Unannounced`. Reporter, Pruitt and Josiah are recurring (`Character::recurring`);
  the owner, Quill and the Agent are unique: each of their cards shows once.
  Cards don't vary by attribute state yet.
- **Inventory and draws** (added 2026-10-13): `Item` (e-reader, key to 1313,
  negatives, tapes, amulet, agent's card) in `cards.rs`; `Inventory` bitset in
  `engine.rs`. A card `requires` items; a choice `gives`/`takes` them. After the
  fixed first card, the next is drawn at random (seeded xorshift, seed from
  `Math.random` in `lib.rs`) from the unspent cards whose items you hold, never
  the same card twice in a row unless nothing else fits. `Engine::new` rejects a
  deck without a recurring, item-free card, so draws can't run dry. The HUD
  shows held items along the bottom (`art/icons.rs`); dragging shows "+ item /
  - item" under the choice and marks items that would be taken.
- **Portraits** (`src/art/portraits.rs`): one still 96x96 pixel-art bust per
  character with the `Pen`, painted when the card is dealt. The card is an
  evidence photo: portrait, role strip, red arrival stamp.
- **HUD**: a gauge per attribute (zones per GAME.md). While dragging, arrows
  (1-3 by size, red if moving away from 50) and a ghost needle preview the
  change; on release the needles move and jolt.
- `#cards` skips the intro (after New Game).

## Architecture decisions

- **All logic in Rust.** The JS in `www/index.html` only calls `init()`.
  The DOM is built and driven from Rust via `web-sys`.
- **Pure core, thin browser layer.** `engine.rs` and `swipe.rs` have no browser
  dependencies, so they're tested with plain `cargo test`. Only `view.rs` touches the DOM.
- **The deck is validated once on load** (duplicate ids, missing start card,
  no fallback card). After that, `Engine::choose` can't fail.
- **The engine records history** as `Vec<(scene_id, Side)>`. It isn't used yet.
- **One card element is reused** for every scene (content swapped, CSS classes
  toggled), so event listeners are attached only once.
- Crate type is `cdylib` (for wasm) plus `rlib` (for native tests).

## Files

- `src/engine.rs`: `Scene { id, who, arrival, text, left, right }`, `Choice { label, effect, gives, takes }`, `Scene.requires`, `Stat`, `Stats`, `Effect`, `Inventory`, `Condition`, `Engine` (random draws), `EngineError`
- `src/swipe.rs`: `SwipeConfig` (threshold, rotation, vertical damping), `Drag`
- `src/view.rs`: DOM, pointer handlers, `commit()` animation sequence
- `src/cards.rs`: `Character`, `Arrival`, `Item`, `START_ITEMS`, the deck
- `src/splash.rs`: title screen; `src/cinematic.rs`, `src/intro.rs`, `src/endings.rs`, `src/art/`, `src/audio/`: cinematics and sound (see CINEMATICS.md)
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
- If `wasm-pack` isn't on PATH, its cached `wasm-bindgen` works: `cargo build --release
  --target wasm32-unknown-unknown`, then `~/AppData/Local/.wasm-pack/wasm-bindgen-*/wasm-bindgen
  --target web --out-dir www/pkg target/wasm32-unknown-unknown/release/swipe_engine.wasm`.
- Browser testing approach that worked: headless Chrome with
  `--remote-debugging-port`, driven over CDP from a Node script, using
  `Input.dispatchMouseEvent` to simulate drags.

## Languages (added 2026-10-13)

US English and Brazilian Portuguese. `src/i18n.rs`: English text is the key,
`i18n/pt_br.rs` is the table, `t()` translates at display time (view, cinematic
captions, splash, pixel-art signs/documents), so the deck and tests stay English.
Tests fail if any shown text lacks a translation or a table entry is stale, and
check that pixel-font text fits. The 3×5 pixel font drops accents (`art::fold`);
the Room 13 web font got accented capitals (ÁÀÂÃÄ ÉÊ… Ç Ñ) composed in
`design/font-draft/glyphs.py` (rebuild: `.venv` with fonttools, skia-pathops, brotli).
New player-visible text must go through `t()` and get a pt-BR entry.
Choice is saved in localStorage `room1313.lang`, else browser language.

## Next steps

1. **Event/scene format.** Load the deck from data (JSON or RON?) instead of
   `cards.rs`. Format not decided yet.
2. **Story/history design.** More cards, and cards chosen by attribute state
   (GAME.md "Card & Narrative Integration") instead of a fixed loop.
3. Polish ideas: arrow-key input, a card-game ambience, sounds on swipe.

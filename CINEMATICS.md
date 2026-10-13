# Cinematics

How the intro cinematic of Room 13 is built, and the steps we followed to make
it. Use this as the recipe for new cutscenes (endings, chapter breaks, etc.).

## The intro at a glance

Flow: **title screen → intro cinematic → card game**. It starts when the player
picks *New Game* (`src/lib.rs`: `splash::mount` → `intro::play` → `view::mount`).

Style: minimalist dark pixel art, "Dave the Diver in a Cthulhu movie". The
palette is mostly near-black, with a few accent colours: the title red
`#c0392b`, sickly sulfur yellow, and CCTV green.

| # | Shot (`Shot::`) | What we see | Caption |
|---|---|---|---|
| 1 | `City` | Grey city at night, heavy rain, street lamps reflected on wet asphalt, two lightning strikes | *October 13th. It had been raining for eleven days straight. / That morning, a crime shocked the whole country.* |
| 2 | `Hotel` | Hotel Lumen: 13 floors, dying red "HOTEL" neon, patrol car lights, police tape. Room 1313's window pulses red | *The Hotel Lumen. Old, huge, cheap. Nobody asks questions. / Two guests checked into room 1313. Then they vanished.* |
| 3 | `Kindle` | Broken e-reader on hotel carpet with a red halo. Cracked screen shows the sigil (Lucifer's seal + tentacles + an eye); e-ink refresh flashes; the eye turns red | *All they left behind was a broken e-reader. / Its screen would not turn off. It showed only this.* |
| 4 | `Room` | Room 1313 wrecked, sulfur haze, only a flashlight sweeping. Wardrobe (red eyes inside when the beam looks away), crooked picture, clock stopped at 3:13, self-turning standing fan, smashed TV spitting sparks, flipped bed, armchair, suitcase, phone off the hook, sigil on the wall | *The room was torn apart. The air reeked of sulfur. / Every device was fried. No light would come on.* |
| 5 | `Cctv` | Green, choppy street-camera footage (CAM 04, REC, 03:13). The renter walks, stops, looks back; a tall thin figure appears behind him for 3 frames; he hurries off | *Hours earlier, a street camera caught the man who rented it. / He kept looking back over his shoulder.* |
| 6 | `Office` | Hard cut, then the camera pulls back from the footage to a monitor on a police desk: folder "CASE 358", calendar on 13, evidence board with red string. The player is a featureless silhouette from behind (no gender cues). The figure now stands in the footage, facing the camera | *The case lands on your desk. Your first. / **What happened in room 1313?*** (red, lands on a boom) |

Controls: click / Space / Enter finishes the typing caption, or moves on if
it's done. Escape skips the whole intro. Shots also advance by themselves.

## Files

| File | What lives there |
|---|---|
| `src/intro.rs` | The `SCENES` table (shot, caption lines, timing), the frame loop, the typewriter, fades, input, and the sound cues per shot |
| `src/intro_art.rs` | The pixel art: the `Pen` painter, a 3×5 pixel font, shared pieces (sky, rain, skyline, sigil…), and one function per shot |
| `src/audio.rs` | Procedural sound (no asset files): rain, thunder, hum, static, spark crackle, typewriter tick, drone, boom, sting |
| `www/style.css` | The *Intro cinematic* section: canvas scaling, caption, hint, grain and vignette |

## How it works

### Rendering

- Each shot is drawn on a **240×135 canvas** (`W`, `H`) and CSS scales it up
  with `image-rendering: pixelated`. All coordinates are in those low-res pixels.
- Each shot is a **pure function of time**: `fn shot(p: &Pen, t: f64)`, where
  `t` is seconds since the shot began. It repaints the whole frame every
  animation frame. There's no state, so animations are formulas of `t`
  (`sin` for sways, `rem_euclid` for loops, step functions for choreography).
- Randomness comes from `hash(n)`, which is deterministic. The same seed gives
  the same rain, windows and debris every time. For per-frame flicker or noise,
  hash a frame counter: `let frame = (t * 20.0) as u32; hash(frame + i)`.
- Paint back to front: background, props, haze, then darkness (`p.light`),
  and last anything that emits its own light (TV sparks, eyes in the dark, HUD).

### `Pen` toolbox (`src/intro_art.rs`)

| Method | Use |
|---|---|
| `rect(x, y, w, h, color)` / `px(x, y, color)` | Filled boxes and single pixels. Coordinates are floored, so pixels stay crisp |
| `ink(color)` + `dot(x, y)` | Fast path for thousands of pixels in one colour (carpet, noise) |
| `line(x0, y0, x1, y1, color)` | Bresenham line, always 1px wide (cracks, rain, claw marks) |
| `text(s, x, y, color)` | 3×5 pixel font with a 4px advance. Only the glyphs in `glyph()` exist; add more there if you need new letters |
| `light(cx, cy, r, dark)` | Darkens everything except a soft circle: flashlights and vignettes |
| `glow(cx, cy, r, "r,g,b", alpha)` | Soft radial light: TV, monitor, sparks |

Shared pieces: `sky`, `clouds`, `bolt`, `lightning`, `skyline`, `rain`,
`splashes`, `streetlamp`, `sigil`, `ring`, `curl`.

Use radial `glow`/`light` for light sources. Stacked semi-transparent
rectangles look like hard boxes at this resolution; we tried that first and
replaced it.

### Timeline (`src/intro.rs`)

Each `Scene` has:

- `shot`: which art function to draw.
- `lines`: caption lines, typed one after another and kept on screen.
  Keep each line **at most ~60 characters**, or it wraps at the caption width.
- `text_at`: ms after the shot starts before typing begins. Give the image a
  moment first, longer if something happens first (the office pull-back takes
  ~5s).
- `cut_in`: hard cut from the previous shot instead of fading through black.

Timing constants: `CHAR_MS` (typing speed), `LINE_PAUSE_MS`, `HOLD_MS` (how
long a finished caption stays), `FINAL_HOLD_MS`, `FADE_MS`, `END_FADE_MS`.
**`FADE_MS` must match the `.intro-caption` transition in `www/style.css`.**

The last line of the last scene is the hook. It's styled `.accent` and plays
`boom()` when it starts typing.

While a caption types, the untyped rest is kept in an invisible span, so the
text never reflows.

### Sound

The intro creates its own `Music`, because the title screen's fades out. Browsers allow it
because the player already interacted with the page. Cues live in
`Intro::cue(shot)` and fire when a shot starts:

- City: loud rain, plus thunder timed to `LIGHTNING` (+0.6s, sound after light).
- Hotel: rain. Kindle: rain muffled (indoors, low lowpass) + sting.
- Room: muffled rain. Each spark burst plays `crackle()`, synced by
  checking `spark(t)` every frame and playing on the burst's first frame.
- Cctv: rain off, camera hum, static burst. Office: soft rain on the window, low hum.

For sounds synced to something on screen, expose the timing from the art
(`LIGHTNING`, `spark(t)`) and react to it in `intro.rs`. Don't duplicate the numbers.

### Special tricks

- **Camera pull-back (shot 6):** the CCTV footage and the office are both drawn
  on offscreen canvases. The office canvas is copied to the screen with a
  shrinking source rectangle (`MONITOR` → full frame). The footage is then
  redrawn at full resolution over the monitor so it stays crisp. The monitor
  is exactly 1/5 of the frame, so the first frame matches the CCTV shot exactly.
- **Continuity:** the monitor plays `cctv(FEED_OFFSET + t)`, so the footage
  continues from where shot 5 left off (the man is gone, the figure appears).
- **Choppy footage:** CCTV time is quantised, `t = (t * 8).floor() / 8`, for 8 fps.
- **Things in the dark:** the eyes in the wardrobe are drawn *after* the darkness,
  and only when the flashlight is pointing far away.

## Steps we followed

1. **Brief.** Story beats from the design (date, hotel, the e-reader, the room,
   the CCTV, the new officer) and a visual reference (Dave the Diver × Cthulhu).
2. **Captions.** Rewrote the beats as short lines (≤ ~60 chars), in English
   like the rest of the UI. Avoid gender cues for the player ("you", the
   silhouette).
3. **Engine.** Canvas + `Pen` + one pure function per shot. A `SCENES` table
   drives the timeline, typing, fades and input. Wire it in between title and game.
4. **Art pass.** Paint each shot, using shared pieces where possible. Keep a
   small palette per shot.
5. **Sound pass.** One ambience per shot plus a few synced effects.
6. **Build and look.** Build, then take headless-Chrome screenshots of every shot
   (see below). Review them and fix what reads badly. Our fixes from review:
   - captions too long → shortened
   - room too dark → bigger, softer flashlight
   - cracks drawn outside the screen → clipped to the glass
   - boxy glows → radial `glow`
   - lamp cones looked like trees → narrower and fainter
7. **Iterate on feedback.** Examples: case number 1313 → 358; more props in the
   room; TV changed from static to sparks, with the caption changed to match the story.
8. **Check:** `cargo clippy --target wasm32-unknown-unknown` clean,
   `cargo test` passing, every shot screenshotted, and the hand-off to the card game works.

## Build, run, check

```sh
wasm-pack build --target web --out-dir www/pkg --no-pack
python3 -m http.server 8765 -d www        # open http://localhost:8765
cargo clippy --target wasm32-unknown-unknown
```

Screenshots: run headless Chrome with `--remote-debugging-port`, drive it over CDP
from a Node script, and save `Page.captureScreenshot` output. The script presses a key
for the gate, Enter for *New Game*, then waits about 2–3s into each shot and
presses Space twice (finish caption, next shot). Also collect console
messages and exceptions over CDP to catch Rust panics.

Tips:
- To see a rare or short event (a spark burst, a 3-frame apparition),
  temporarily force it on, build, take the screenshot, then **revert and rebuild**.
- Captures right after a shot change land in the fade-in; wait at least `FADE_MS`.

## Adding a new cinematic shot

1. Add a variant to `Shot` and a `fn my_shot(p: &Pen, t: f64)` in
   `src/intro_art.rs`. Hook it up in `Art::draw`.
2. Add a `Scene { shot, lines, text_at, cut_in }` to `SCENES` in `src/intro.rs`
   (bump the array length).
3. Add its sound cue in `Intro::cue`.
4. Build, take screenshots, and review against the style notes above.

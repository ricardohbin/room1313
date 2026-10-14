# Cinematics

How the cinematics of Room 13 are built (the intro and the six game-over
endings), and the steps we followed to make them. Use this as the recipe for
new cutscenes (chapter breaks, new endings, etc.).

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

## The endings at a glance

One tragedy per attribute extreme from `GAME.md`. Each ending has its own score,
and each closes on a document (newspaper, police report or hospital record) whose
caption ends with the name of the loss in red.

| Attribute | Ending (`Ending::`) | Shots | Score (`Score::`) |
|---|---|---|---|
| Reputation low | `Dismissal` | Your hand lays the badge, then the gun, on the captain's desk; he never looks up → you leave the precinct in the rain with a box → *THE EVENING STAR: DETECTIVE DISMISSED* | `Lament`: lonely noir horn over slow piano, D minor |
| Reputation high | `Assassination` | Night street; a motorcycle passes and the passenger opens fire; the shop window shatters; you fall → *DETECTIVE SHOT DEAD* | `Thrash`: palm-muted E riff with a tritone, skank beat, 190 bpm |
| Sensitivity low | `FatalHaunting` | You run down your own hallway; doors slam, lights die behind you, handprints appear → you trip at the top of the dark stairs and fall all the way down → police report *ACCIDENTAL DEATH*, stamped CLOSED | `Psychobilly`: slap-bass boogie, backbeat, creepy surf lick, 180 bpm |
| Sensitivity high | `Institutionalization` | Two orderlies drag you to a van while red eyes watch from the windows → you rock in a padded cell scratched with the sigil and tally marks of 13 → *INVOLUNTARY COMMITMENT*, stamped NO VISITS | `Asylum`: detuned music box over a dissonant cluster, breath swells |
| Arousal low | `Coma` | You sway in the bullpen as your vision closes in, collapse, officers rush over → hospital bed, heart monitor, empty chair → *PATIENT IN COMA* | `Requiem`: piano and strings, A minor with a dramatic E major |
| Arousal high | `FatalOverload` | Home alone, hand on the wall, heart racing (the screen shakes and flushes red on each beat) until you fall, and **the music stops dead** → the same room, still; the phone rings and rings → coroner's *CARDIAC ARREST*, stamped DECEASED | `Hardcore`: d-beat, power chords on every 8th, 210 bpm |

The card game plays an ending when an attribute reaches 0 or 100
(`Ending::of(stat, extreme)` in `src/lib.rs`). The seventh ending, **`Truth`**,
is the win: choosing "Tell me" on the shadow's last card (GAME.md, "The Road to
the Truth"). Its film: the 13th-floor corridor and door 1313 open on the light
→ the golden serpent biting its tail, you facing your own shape in light → the
newspaper *DETECTIVE VANISHES*. Its score is `Revelation`: major-seventh chords
that turn dark on B-flat, with bells and a breath. Any ending reloads back to
the title screen.

**Dev links** (then New Game):
- `#ending=<name>` plays an ending: `dismissal`, `assassination`, `haunting`,
  `institution`, `coma`, `overload`, `truth`.
- `#cards` skips the intro.
- `#card=<id>` skips the intro and deals that card first, with the items it
  requires in hand (e.g. `#card=prophet_first`, `#card=shadow_truth`).

## Files

| File | What lives there |
|---|---|
| `src/cinematic.rs` | The player: `Film` (scenes + score), `Scene` (shot, caption, timing, cue, sync), the frame loop, typewriter, fades, input |
| `src/intro.rs` | The intro's `Film`: its scene table, cues and syncs |
| `src/endings.rs` | `Ending`, the six endings' `Film`s, preview names |
| `src/art/mod.rs` | `Shot`, `Art`, the `Pen` painter, the 3×5 pixel font (A–Z, 0–9, some punctuation), shared pieces (sky, rain, skyline, sigil…) |
| `src/art/intro.rs` | The intro's shots |
| `src/art/endings.rs` | The endings' shots, the detective sprite (`agent` with poses), the document, and the timing constants sound syncs to |
| `src/audio/mod.rs` | `Music`: title theme, rain, thunder, hum, static, ticks, drone, boom, sting |
| `src/audio/score.rs` | `Score`: a tiny synth band (drums, distorted guitar, bass, slap bass, surf guitar, piano, strings, horn, music box) and the six endings' scores |
| `src/audio/foley.rs` | Ending effects: thuds, gunshots, motorcycle pass, heart monitor, telephone bell |
| `www/style.css` | The *Cinematics* section: canvas scaling, caption, hint, grain and vignette (`.cine-*`) |

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

### `Pen` toolbox (`src/art/mod.rs`)

| Method | Use |
|---|---|
| `rect(x, y, w, h, color)` / `px(x, y, color)` | Filled boxes and single pixels. Coordinates are floored, so pixels stay crisp |
| `ink(color)` + `dot(x, y)` | Fast path for thousands of pixels in one colour (carpet, noise) |
| `line(x0, y0, x1, y1, color)` | Bresenham line, always 1px wide (cracks, rain, claw marks) |
| `text(s, x, y, color)` / `text_at_scale(s, x, y, scale, color)` | 3×5 pixel font with a 4px advance (×`scale` for headlines). Glyphs live in `glyph()` |
| `light(cx, cy, r, dark)` | Darkens everything except a soft circle: flashlights and vignettes |
| `glow(cx, cy, r, "r,g,b", alpha)` | Soft radial light: TV, monitor, sparks |

Shared pieces: `sky`, `clouds`, `bolt`, `lightning`, `skyline`, `rain`,
`splashes`, `streetlamp`, `sigil`, `ring`, `curl`. In `art/endings.rs`:
`agent(p, x, ground, pose, coat, rim)` draws the detective (`Stand`, `Walk`,
`Run`, `Lean`, `Kneel`, `Lie`, `Slump`, `Dragged`). Keep the coat `COAT` and give it a
rim light: a dark figure on a dark street disappears (we hit this twice).

Use radial `glow`/`light` for light sources. Stacked semi-transparent
rectangles look like hard boxes at this resolution; we tried that first and
replaced it.

### Timeline (`src/cinematic.rs`)

A `Film` is a list of scenes plus a `score` function that starts the music.
Scenes are built with a small builder:

```rust
Scene::new(Shot::Corridor, &["First line.", "Second line."], 1200.0) // text_at, ms
    .cut_in()                       // hard cut instead of fading through black
    .cue(|m| m.rain(0.2, 800.0, 1.0)) // sounds when the shot starts
    .sync(|m, prev, now| {          // every frame, shot time in seconds
        if crossed(prev, now, DOOR_SLAMS[0]) { m.knock(0.9); }
    })
```

- Caption lines are typed one after another and stay on screen. Keep each one
  **at most ~60 characters**, or it wraps.
- `text_at`: how long the image plays before typing starts. Make it longer when
  the action comes first (drive-by, collapse, office pull-back).
- A scene auto-advances `HOLD_MS` after its caption finishes (`FINAL_HOLD_MS`
  for the last). If an event must happen on screen, make sure the caption timing
  lasts past it.
- The last line of the last scene is the hook: styled `.accent`, lands on `boom()`.
  For endings, it's the name of the loss.

Timing constants: `CHAR_MS`, `LINE_PAUSE_MS`, `HOLD_MS`, `FINAL_HOLD_MS`,
`FADE_MS`, `END_FADE_MS`. **`FADE_MS` must match the `.cine-caption` transition
in `www/style.css`.**

### Sound

Each film creates its own `Music` (the title's fades out); the browser allows it
because the player already clicked. `Film.score` starts the music (the intro's
drone, or an ending's `Score`), `Scene.cue` plays sounds when a shot starts, and
`Scene.sync` hits moments in the picture.

**Syncing:** the art exports the moment (`BADGE_DROP`, `FIRE`, `STAIRS_HITS`,
`OVERLOAD_FALL`, `spark(t)`…) and the scene's `sync` fires when the shot clock
crosses it (`crossed(prev, now, at)`). Never copy the numbers into the sound.

**Scores** (`src/audio/score.rs`) are sequenced on a 16th-note grid: each score
is a function that schedules one loop of notes on the audio clock, and
`Music::score` re-runs it every loop, always one loop ahead. To write one:
pick `tempo()` (bpm, bars), then call instruments per step: `kick`, `snare`,
`hat`, `crash`, `bass`, `power` (distorted power chord, palm-muted or open),
`pluck`, `slap`, `piano`, `pad`, `horn`, `music_box`, `breath`.

`Music::stop_score()` cuts the score instantly, notes already scheduled included
(the fatal-overload fall). If a later shot must be silent, call it in that shot's
`cue` too, in case the player skipped past the moment.

Intro cues: city loud rain + thunder after `LIGHTNING`; hotel rain; e-reader
muffled rain + sting; room muffled rain + a crackle per spark burst; CCTV hum and
static; office soft rain and hum.

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

### Intro

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

### Endings

1. **Brief.** One loss per attribute extreme from `GAME.md` (the "Loss"
   bullets), plus an animation idea and a music genre per ending.
2. **Refactor first.** The intro's player became `cinematic.rs` (`Film` +
   `Scene` builder, `cue`/`sync` hooks), the art moved to `art/`, audio to
   `audio/`, so endings are only data plus art.
3. **Structure.** 2–3 action shots per ending, closing on a document (GAME.md:
   "expressed through a short epilogue, police report, newspaper clipping, or
   hospital record"). The caption ends with the loss name.
4. **Music.** A small step sequencer and synth band, one score per ending
   matching the requested genre.
5. **Art.** A shared detective sprite with poses, then one function per shot.
   Moments sound must hit are exported constants.
6. **Review.** Screenshot every ending while it auto-plays (see below). Fixes from
   review:
   - the detective vanished on dark backgrounds → lighter coat and rim light
   - the motorcycle was black on a black road → lighter body, rim light, headlight glow
   - the hospital patient was lost in the sheets → head drawn on the pillow
   - the hand stopped short of the wall → moved the sprite

## Build, run, check

```sh
make run                                 # build + serve at http://localhost:8765
make check                               # clippy (wasm) + tests
```

Endings: `http://localhost:8765/#ending=<name>`, then New Game.

Screenshots: headless Chrome with `--remote-debugging-port`, driven over CDP
from a Node script that saves `Page.captureScreenshot` output and collects
console messages and exceptions (catches Rust panics and audio warnings).
- **Intro:** press a key for the gate, Enter for New Game, then wait 2–3s into
  each shot and press Space twice (finish caption, next shot).
- **Endings:** don't press keys during playback; key presses drift out of step
  with auto-advance and skip shots. Instead, let it play and poll the caption
  text (`.cine-caption`) with `Runtime.evaluate`. A change means a new shot; take
  frames at fixed offsets after it (1.5s, 3.5s, 5.5s, 8s). Run several endings in
  parallel on different debugging ports, then put the frames side by side on
  one HTML contact sheet, screenshotted by headless Chrome.

Tips:
- To see a rare or short event (a spark burst, a 3-frame apparition),
  temporarily force it on, build, take the screenshot, then **revert and rebuild**.
- Captures right after a shot change land in the fade-in; wait at least `FADE_MS`.
- Music and effects can't be judged from screenshots. Listen to them.

## Adding a new cinematic shot

1. Add a variant to `Shot` (`src/art/mod.rs`), a `pub(super) fn my_shot(p: &Pen, t: f64)`
   in `art/intro.rs` or `art/endings.rs`, and hook it up in `Art::draw`.
2. Export any moment the sound must hit as a `pub const`.
3. Add a `Scene::new(...)` with `.cue`/`.sync` to the film's table (bump the array length).
4. Build, screenshot, review against the style notes above.

## Adding a new ending or score

1. Add the `Ending` variant, its `name()` for previews, a `static` `Film` and a
   scene table in `src/endings.rs`, and its document as a `Doc` variant.
2. For new music: add a `Score` variant with its `tempo()` and a function in
   `audio/score.rs` that schedules one loop.

//! Music for the endings: a tiny synth band (drums, distorted guitar, bass,
//! piano, strings, horn, music box) and six looping scores, sequenced on a
//! 16th-note grid and scheduled one loop ahead on the audio clock.

use wasm_bindgen::prelude::*;
use web_sys::{AudioNode, BiquadFilterType, GainNode, OscillatorType};

use super::{chain, midi, play, rand, warn, Bus, Music};

#[derive(Clone, Copy, PartialEq)]
pub enum Score {
    /// Dismissal: a lonely noir horn over slow piano, D minor.
    Lament,
    /// Assassination: palm-muted thrash riff with a skank beat.
    Thrash,
    /// Fatal haunting: slap-bass boogie, backbeat and a surf-guitar lick.
    Psychobilly,
    /// Institutionalization: a broken music box over a dissonant cluster.
    Asylum,
    /// Coma: dramatic piano and strings in A minor.
    Requiem,
    /// Fatal overload: d-beat hardcore, cut dead when the body hits the floor.
    Hardcore,
}

impl Score {
    /// Beats per minute and bars per loop (4/4).
    fn tempo(self) -> (f64, u32) {
        match self {
            Score::Lament => (60.0, 4),
            Score::Thrash => (190.0, 2),
            Score::Psychobilly => (180.0, 4),
            Score::Asylum => (60.0, 2),
            Score::Requiem => (66.0, 4),
            Score::Hardcore => (210.0, 4),
        }
    }
}

/// Where a score plays: `out` is its own volume (to cut it dead), `amp` the
/// input of a distortion stage for guitars.
#[derive(Clone)]
struct Rig {
    out: GainNode,
    amp: GainNode,
}

impl Music {
    /// Loops `score` until `stop_score` or `fade_out`. Replaces any theme or score.
    pub fn score(&mut self, score: Score) {
        self.stop_score();
        let rig = match self.bus.rig() {
            Ok(rig) => rig,
            Err(e) => return web_sys::console::warn_1(&e),
        };
        self.score_out = Some(rig.out.clone());
        let (bpm, bars) = score.tempo();
        let step = 60.0 / bpm / 4.0;
        let loop_s = step * 16.0 * bars as f64;
        let mut next = self.bus.ctx.current_time() + 0.15;
        let bus = self.bus.clone();
        self.start_loop(loop_s, move || {
            warn(bus.score(score, &rig, next, step));
            next += loop_s;
        });
    }

    /// Cuts the score off instantly, notes already scheduled included.
    pub fn stop_score(&mut self) {
        self.stop_loop();
        if let Some(out) = self.score_out.take() {
            let t = self.bus.ctx.current_time();
            let _ = out.gain().cancel_scheduled_values(t);
            let _ = out.gain().set_value_at_time(0.0, t);
        }
    }
}

impl Bus {
    fn rig(&self) -> Result<Rig, JsValue> {
        let out = self.ctx.create_gain()?;
        out.gain().set_value(0.7);
        out.connect_with_audio_node(&self.master)?;

        let amp = self.ctx.create_gain()?;
        let shaper = self.ctx.create_wave_shaper()?;
        let mut curve: Vec<f32> = (0..1024)
            .map(|i| {
                let x = i as f32 / 512.0 - 1.0;
                (1.0 + 40.0) * x / (1.0 + 40.0 * x.abs())
            })
            .collect();
        shaper.set_curve_opt_f32_slice(Some(&mut curve));
        let tone = self.filter(BiquadFilterType::Lowpass, 3200.0, 0.7)?;
        let tight = self.filter(BiquadFilterType::Highpass, 90.0, 0.7)?;
        let level = self.ctx.create_gain()?;
        level.gain().set_value(0.3);
        chain(&[&amp, &shaper, &tone, &tight, &level, &out])?;
        Ok(Rig { out, amp })
    }

    /// Schedules one loop of `score` starting at `t0`; `step` is a 16th note.
    fn score(&self, score: Score, rig: &Rig, t0: f64, step: f64) -> Result<(), JsValue> {
        match score {
            Score::Lament => self.lament(rig, t0, step),
            Score::Thrash => self.thrash(rig, t0, step),
            Score::Psychobilly => self.psychobilly(rig, t0, step),
            Score::Asylum => self.asylum(rig, t0, step),
            Score::Requiem => self.requiem(rig, t0, step),
            Score::Hardcore => self.hardcore(rig, t0, step),
        }
    }

    fn lament(&self, rig: &Rig, t0: f64, step: f64) -> Result<(), JsValue> {
        let out: &AudioNode = &rig.out;
        let bar = step * 16.0;
        let beat = step * 4.0;
        // Dm, Bb, Gm, A.
        let chords: [[u8; 3]; 4] = [[50, 53, 57], [46, 50, 53], [43, 46, 50], [45, 49, 52]];
        // (beat, note, beats) per bar for the horn.
        let horn: [&[(f64, u8, f64)]; 4] = [
            &[(0.0, 74, 3.0), (3.0, 72, 1.0)],
            &[(0.0, 70, 2.0), (2.0, 69, 2.0)],
            &[(0.0, 67, 1.0), (1.0, 70, 1.0), (2.0, 69, 2.0)],
            &[(0.0, 73, 2.0), (2.0, 69, 2.0)],
        ];
        for (b, chord) in chords.iter().enumerate() {
            let t = t0 + b as f64 * bar;
            self.pad(t, chord, bar, out, 0.05)?;
            self.piano(t, chord[0] - 12, bar, out, 0.2)?;
            for k in 0..4 {
                let tone = chord[[0, 1, 2, 1][k]] + 12;
                self.piano(t + k as f64 * beat, tone, beat, out, 0.1)?;
                if k % 2 == 1 {
                    self.hat(t + k as f64 * beat, out, 0.03, true)?;
                }
            }
            for &(at, note, beats) in horn[b] {
                self.horn(t + at * beat, note, beats * beat, out, 0.12)?;
            }
        }
        Ok(())
    }

    fn thrash(&self, rig: &Rig, t0: f64, step: f64) -> Result<(), JsValue> {
        let out: &AudioNode = &rig.out;
        // e: palm-muted E chug, E: open E, f: F, g: G, b: B flat (the tritone).
        const RIFF: &[u8; 32] = b"eeeEeeeEeeefeeeGeeeEeeeEeeebggfE";
        self.crash(t0, out, 0.25)?;
        for (i, &c) in RIFF.iter().enumerate() {
            let t = t0 + i as f64 * step;
            let (note, muted) = match c {
                b'E' => (40, false),
                b'f' => (41, false),
                b'g' => (43, false),
                b'b' => (46, false),
                _ => (40, true),
            };
            let len = if muted { step * 0.8 } else { step * 1.8 };
            self.power(t, note, len, rig, if muted { 0.45 } else { 0.6 }, muted)?;
            if i % 2 == 0 {
                self.bass(t, note - 12, step * 1.8, out, 0.3, OscillatorType::Sawtooth, 500.0)?;
                self.hat(t, out, 0.1, false)?;
            }
            match i % 4 {
                0 => self.kick(t, out, 0.9)?,
                2 => self.snare(t, out, 0.55)?,
                _ => {}
            }
            // Double-kick run into the next loop.
            if i >= 24 && i % 4 != 0 {
                self.kick(t, out, 0.6)?;
            }
        }
        Ok(())
    }

    fn psychobilly(&self, rig: &Rig, t0: f64, step: f64) -> Result<(), JsValue> {
        let out: &AudioNode = &rig.out;
        // Walking slap bass per bar (quarter notes): E, E, A, B.
        let walks: [[u8; 4]; 4] = [[40, 44, 47, 49], [50, 49, 47, 44], [45, 49, 52, 54], [47, 51, 54, 52]];
        // A creepy surf lick, transposed to each bar's root: (16th, note).
        let lick: [(usize, u8); 6] = [(0, 64), (3, 67), (6, 70), (8, 71), (11, 70), (14, 67)];
        for (b, walk) in walks.iter().enumerate() {
            let shift = walk[0] - 40;
            for i in 0..16 {
                let t = t0 + (b * 16 + i) as f64 * step;
                match i % 4 {
                    0 => self.slap(t, Some(walk[i / 4]), out, 0.45)?,
                    2 => self.slap(t, None, out, 0.25)?,
                    _ => {}
                }
                match i {
                    0 | 8 => self.kick(t, out, 0.6)?,
                    4 | 12 => self.snare(t, out, 0.5)?,
                    _ => {}
                }
                if i % 2 == 0 {
                    self.hat(t, out, 0.05, false)?;
                }
            }
            for &(i, note) in &lick {
                self.pluck(t0 + (b * 16 + i) as f64 * step, note + shift, out, 0.13)?;
            }
        }
        Ok(())
    }

    fn asylum(&self, rig: &Rig, t0: f64, step: f64) -> Result<(), JsValue> {
        let out: &AudioNode = &rig.out;
        let eighth = step * 2.0;
        // Whole-tone and tritone steps: nothing ever resolves.
        const TUNE: [u8; 16] = [72, 0, 78, 0, 77, 71, 0, 0, 72, 0, 66, 0, 65, 71, 0, 0];
        let mut rng = t0.to_bits() | 1;
        for (i, &note) in TUNE.iter().enumerate() {
            if note != 0 {
                let late = rand(&mut rng) * 0.08;
                let cents = (rand(&mut rng) - 0.5) * 80.0;
                self.music_box(note, cents, t0 + i as f64 * eighth + late, out)?;
            }
        }
        let loop_s = step * 32.0;
        self.pad(t0, &[36, 37, 42], loop_s, out, 0.07)?;
        self.breath(t0 + loop_s * 0.45, loop_s * 0.4, out)
    }

    fn requiem(&self, rig: &Rig, t0: f64, step: f64) -> Result<(), JsValue> {
        let out: &AudioNode = &rig.out;
        let bar = step * 16.0;
        let beat = step * 4.0;
        // Am, F, C, E: harmonic minor for the drama.
        let chords: [[u8; 3]; 4] = [[57, 60, 64], [53, 57, 60], [48, 52, 55], [52, 56, 59]];
        let melody: [[u8; 4]; 4] = [[76, 74, 72, 71], [72, 0, 69, 72], [79, 76, 74, 72], [71, 68, 64, 0]];
        const ARP: [usize; 8] = [0, 1, 2, 3, 2, 1, 2, 1];
        for (b, chord) in chords.iter().enumerate() {
            let t = t0 + b as f64 * bar;
            let tones = [chord[0], chord[1], chord[2], chord[0] + 12];
            self.pad(t, chord, bar, out, 0.06)?;
            self.piano(t, chord[0] - 12, bar, out, 0.22)?;
            self.kick(t, out, 0.35)?;
            for (k, &a) in ARP.iter().enumerate() {
                self.piano(t + k as f64 * step * 2.0, tones[a], step * 2.0, out, 0.08)?;
            }
            for (k, &note) in melody[b].iter().enumerate() {
                if note != 0 {
                    self.piano(t + k as f64 * beat, note, beat, out, 0.16)?;
                }
            }
        }
        Ok(())
    }

    fn hardcore(&self, rig: &Rig, t0: f64, step: f64) -> Result<(), JsValue> {
        let out: &AudioNode = &rig.out;
        // A, A, F, G power chords on every 8th, d-beat underneath.
        let roots: [u8; 4] = [45, 45, 41, 43];
        for (b, &root) in roots.iter().enumerate() {
            let tb = t0 + b as f64 * step * 16.0;
            self.crash(tb, out, if b == 0 { 0.3 } else { 0.15 })?;
            for e in 0..8 {
                let t = tb + e as f64 * step * 2.0;
                self.power(t, root, step * 1.9, rig, 0.6, false)?;
                self.bass(t, root - 12, step * 1.9, out, 0.3, OscillatorType::Square, 400.0)?;
                self.hat(t, out, 0.1, false)?;
                match e {
                    0 | 3 | 4 => self.kick(t, out, 0.9)?,
                    2 | 6 => self.snare(t, out, 0.6)?,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    // ---------- Instruments ----------

    fn kick(&self, t: f64, out: &AudioNode, level: f32) -> Result<(), JsValue> {
        let o = self.osc(OscillatorType::Sine, 150.0)?;
        o.frequency().set_value_at_time(150.0, t)?;
        o.frequency().exponential_ramp_to_value_at_time(40.0, t + 0.12)?;
        let env = self.envelope(t, level)?;
        env.gain().exponential_ramp_to_value_at_time(0.001, t + 0.3)?;
        chain(&[&o, &env, out])?;
        play(&o, t, t + 0.32)
    }

    fn snare(&self, t: f64, out: &AudioNode, level: f32) -> Result<(), JsValue> {
        let noise = self.grit()?;
        let hp = self.filter(BiquadFilterType::Highpass, 1200.0, 0.7)?;
        let env = self.envelope(t, level)?;
        env.gain().exponential_ramp_to_value_at_time(0.001, t + 0.16)?;
        chain(&[&noise, &hp, &env, out])?;
        play(&noise, t, t + 0.18)?;
        let body = self.osc(OscillatorType::Triangle, 190.0)?;
        let body_env = self.envelope(t, level * 0.5)?;
        body_env.gain().exponential_ramp_to_value_at_time(0.001, t + 0.08)?;
        chain(&[&body, &body_env, out])?;
        play(&body, t, t + 0.1)
    }

    fn hat(&self, t: f64, out: &AudioNode, level: f32, open: bool) -> Result<(), JsValue> {
        let len = if open { 0.25 } else { 0.04 };
        let noise = self.grit()?;
        let hp = self.filter(BiquadFilterType::Highpass, 7000.0, 0.7)?;
        let env = self.envelope(t, level)?;
        env.gain().exponential_ramp_to_value_at_time(0.001, t + len)?;
        chain(&[&noise, &hp, &env, out])?;
        play(&noise, t, t + len + 0.02)
    }

    fn crash(&self, t: f64, out: &AudioNode, level: f32) -> Result<(), JsValue> {
        let noise = self.grit()?;
        let hp = self.filter(BiquadFilterType::Highpass, 4000.0, 0.5)?;
        let env = self.envelope(t, level)?;
        env.gain().exponential_ramp_to_value_at_time(0.001, t + 1.4)?;
        chain(&[&noise, &hp, &env, out])?;
        play(&noise, t, t + 1.5)
    }

    #[allow(clippy::too_many_arguments)]
    fn bass(
        &self,
        t: f64,
        note: u8,
        len: f64,
        out: &AudioNode,
        level: f32,
        ty: OscillatorType,
        cutoff: f32,
    ) -> Result<(), JsValue> {
        let o = self.osc(ty, midi(note as f64))?;
        let lp = self.filter(BiquadFilterType::Lowpass, cutoff, 1.0)?;
        let env = self.envelope(t, level)?;
        env.gain().set_value_at_time(level, t + len * 0.8)?;
        env.gain().linear_ramp_to_value_at_time(0.0, t + len)?;
        chain(&[&o, &lp, &env, out])?;
        play(&o, t, t + len + 0.01)
    }

    /// Root, fifth and octave into the distortion. Palm-muted chords are short and dark.
    fn power(&self, t: f64, note: u8, len: f64, rig: &Rig, level: f32, muted: bool) -> Result<(), JsValue> {
        let lp = self.filter(BiquadFilterType::Lowpass, if muted { 900.0 } else { 5000.0 }, 0.7)?;
        let env = self.envelope(t, level)?;
        env.gain().set_value_at_time(level, t + len * 0.85)?;
        env.gain().linear_ramp_to_value_at_time(0.0, t + len)?;
        chain(&[&lp, &env, &rig.amp])?;
        for (semis, detune) in [(0.0, -0.06), (0.0, 0.06), (7.0, 0.0), (12.0, 0.0)] {
            let o = self.osc(OscillatorType::Sawtooth, midi(note as f64 + semis + detune))?;
            o.connect_with_audio_node(&lp)?;
            play(&o, t, t + len + 0.01)?;
        }
        Ok(())
    }

    /// Twangy surf guitar: a bright saw whose filter closes fast, into the echo.
    fn pluck(&self, t: f64, note: u8, out: &AudioNode, level: f32) -> Result<(), JsValue> {
        let o = self.osc(OscillatorType::Sawtooth, midi(note as f64))?;
        let lp = self.filter(BiquadFilterType::Lowpass, 3500.0, 4.0)?;
        lp.frequency().set_value_at_time(3500.0, t)?;
        lp.frequency().exponential_ramp_to_value_at_time(500.0, t + 0.3)?;
        let env = self.envelope(t, level)?;
        env.gain().exponential_ramp_to_value_at_time(0.001, t + 0.6)?;
        chain(&[&o, &lp, &env, out])?;
        env.connect_with_audio_node(&self.echo)?;
        play(&o, t, t + 0.62)
    }

    /// Upright bass slap: a short low note plus the click of the string on the
    /// fingerboard. `None` is just the click.
    fn slap(&self, t: f64, note: Option<u8>, out: &AudioNode, level: f32) -> Result<(), JsValue> {
        if let Some(note) = note {
            let o = self.osc(OscillatorType::Triangle, midi(note as f64 - 12.0))?;
            let env = self.envelope(t, level)?;
            env.gain().exponential_ramp_to_value_at_time(0.001, t + 0.3)?;
            chain(&[&o, &env, out])?;
            play(&o, t, t + 0.32)?;
        }
        let click = self.grit()?;
        let bp = self.filter(BiquadFilterType::Bandpass, 2500.0, 1.5)?;
        let env = self.envelope(t, level * 0.6)?;
        env.gain().exponential_ramp_to_value_at_time(0.001, t + 0.03)?;
        chain(&[&click, &bp, &env, out])?;
        play(&click, t, t + 0.04)
    }

    fn piano(&self, t: f64, note: u8, len: f64, out: &AudioNode, level: f32) -> Result<(), JsValue> {
        let hz = midi(note as f64);
        let env = self.envelope(t, 0.0)?;
        env.gain().linear_ramp_to_value_at_time(level, t + 0.005)?;
        env.gain().exponential_ramp_to_value_at_time(level * 0.3, t + 0.4)?;
        env.gain().exponential_ramp_to_value_at_time(0.0005, t + len + 1.2)?;
        env.connect_with_audio_node(out)?;
        for (mult, partial, ty) in [
            (1.0, 1.0, OscillatorType::Triangle),
            (2.0, 0.3, OscillatorType::Sine),
            (3.0, 0.1, OscillatorType::Sine),
        ] {
            let o = self.osc(ty, hz * mult)?;
            let g = self.ctx.create_gain()?;
            g.gain().set_value(partial);
            chain(&[&o, &g, &env])?;
            play(&o, t, t + len + 1.25)?;
        }
        Ok(())
    }

    /// Slow string pad: two detuned saws per note under a soft filter.
    fn pad(&self, t: f64, notes: &[u8], len: f64, out: &AudioNode, level: f32) -> Result<(), JsValue> {
        let lp = self.filter(BiquadFilterType::Lowpass, 900.0, 0.7)?;
        let env = self.envelope(t, 0.0)?;
        env.gain().linear_ramp_to_value_at_time(level, t + len * 0.3)?;
        env.gain().set_value_at_time(level, t + len * 0.8)?;
        env.gain().linear_ramp_to_value_at_time(0.0, t + len + 0.6)?;
        chain(&[&lp, &env, out])?;
        for &note in notes {
            for cents in [-8.0, 8.0] {
                let o = self.osc(OscillatorType::Sawtooth, midi(note as f64 + cents / 100.0))?;
                o.connect_with_audio_node(&lp)?;
                play(&o, t, t + len + 0.65)?;
            }
        }
        Ok(())
    }

    /// Muted noir trumpet: a filtered saw with a late, slow vibrato.
    fn horn(&self, t: f64, note: u8, len: f64, out: &AudioNode, level: f32) -> Result<(), JsValue> {
        let hz = midi(note as f64);
        let o = self.osc(OscillatorType::Sawtooth, hz)?;
        let vibrato = self.osc(OscillatorType::Sine, 5.0)?;
        let depth = self.envelope(t, 0.0)?;
        depth.gain().linear_ramp_to_value_at_time(0.0, t + 0.3)?;
        depth.gain().linear_ramp_to_value_at_time((hz * 0.012) as f32, t + 0.8)?;
        vibrato.connect_with_audio_node(&depth)?;
        depth.connect_with_audio_param(&o.frequency())?;
        let lp = self.filter(BiquadFilterType::Lowpass, 1400.0, 2.0)?;
        let env = self.envelope(t, 0.0)?;
        env.gain().linear_ramp_to_value_at_time(level, t + 0.08)?;
        env.gain().set_value_at_time(level, t + len * 0.85)?;
        env.gain().linear_ramp_to_value_at_time(0.0, t + len)?;
        chain(&[&o, &lp, &env, out])?;
        env.connect_with_audio_node(&self.echo)?;
        play(&vibrato, t, t + len + 0.01)?;
        play(&o, t, t + len + 0.01)
    }

    /// A swell of breathy noise that stops at its loudest.
    fn breath(&self, t: f64, len: f64, out: &AudioNode) -> Result<(), JsValue> {
        let noise = self.grit()?;
        noise.set_loop(true);
        let bp = self.filter(BiquadFilterType::Bandpass, 700.0, 3.0)?;
        let env = self.envelope(t, 0.0001)?;
        env.gain().exponential_ramp_to_value_at_time(0.2, t + len)?;
        env.gain().set_value_at_time(0.0, t + len)?;
        chain(&[&noise, &bp, &env, out])?;
        play(&noise, t, t + len + 0.01)
    }
}

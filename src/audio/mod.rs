//! Procedural audio (Web Audio, no asset files) for the title screen and the
//! cinematics: rumble and boom, a detuned drone, an out-of-tune music-box
//! lullaby, rain, thunder, camera hum and typewriter clicks.
//! `score` holds the endings' music, `foley` their sound effects.

mod foley;
mod score;
mod volume;

pub use score::Score;
pub use volume::{set_volume, volume, MAX as MAX_VOLUME};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    AudioBuffer, AudioBufferSourceNode, AudioContext, AudioNode, AudioParam,
    AudioScheduledSourceNode, BiquadFilterNode, BiquadFilterType, GainNode, OscillatorNode,
    OscillatorType,
};

use crate::view::{after, window};

/// Seconds per lullaby step.
const STEP_S: f64 = 0.42;
const REST: u8 = 0;
/// MIDI notes: an A-minor lullaby that keeps landing on the wrong note (D#).
const LULLABY: [u8; 32] = [
    76, 69, 72, 71, 76, 69, 72, 75, 76, 72, 69, 68, 69, REST, REST, REST, //
    77, 72, 69, 68, 77, 72, 69, 75, 74, 71, 68, 65, 63, REST, REST, REST,
];

/// Cheap handles to the graph; cloned into the theme's interval callback.
#[derive(Clone)]
struct Bus {
    ctx: AudioContext,
    master: GainNode,
    /// Input of a dark feedback delay that feeds `master`.
    echo: GainNode,
    /// Two seconds of white noise, shared by drums and effects.
    grit: AudioBuffer,
}

pub struct Music {
    bus: Bus,
    /// Interval that schedules the looping theme or score, one loop ahead.
    theme: Option<(i32, Closure<dyn FnMut()>)>,
    /// Output of the playing score, so it can be cut dead.
    score_out: Option<GainNode>,
    /// Looping rain (lowpass, gain), built on first use.
    rain: Option<(BiquadFilterNode, GainNode)>,
    /// Electrical hum of the CCTV monitor, built on first use.
    hum: Option<GainNode>,
}

impl Music {
    /// Must be called from a user-gesture handler, or the browser keeps it muted.
    pub fn new() -> Result<Self, JsValue> {
        let ctx = AudioContext::new()?;
        let master = ctx.create_gain()?;
        volume::track(&master);
        master.connect_with_audio_node(&ctx.destination())?;

        let echo = ctx.create_gain()?;
        let delay = ctx.create_delay_with_max_delay_time(2.0)?;
        delay.delay_time().set_value(0.63);
        let feedback = ctx.create_gain()?;
        feedback.gain().set_value(0.45);
        let wet = ctx.create_gain()?;
        wet.gain().set_value(0.5);

        let grit = noise_buffer(&ctx, 2.0)?;
        let bus = Bus { ctx, master, echo, grit };
        let tone = bus.filter(BiquadFilterType::Lowpass, 1800.0, 0.7)?;
        bus.echo.connect_with_audio_node(&delay)?;
        chain(&[&delay, &tone, &feedback, &delay])?;
        chain(&[&tone, &wet, &bus.master])?;
        Ok(Self { bus, theme: None, score_out: None, rain: None, hum: None })
    }

    /// Low noise swelling for `secs`, cut off right where `boom` lands.
    pub fn rumble(&self, secs: f64) {
        warn(self.bus.rumble(secs));
    }

    pub fn boom(&self) {
        warn(self.bus.boom());
    }

    /// Dissonant low cluster, for the "no exit" whisper.
    pub fn sting(&self) {
        warn(self.bus.sting());
    }

    /// Soft tick for moving through the menu.
    pub fn blip(&self) {
        warn(self.bus.blip());
    }

    /// Just the drone, without the lullaby. Runs until `fade_out`.
    pub fn drone(&self) {
        warn(self.bus.drone());
    }

    /// Ramps the rain to `level` over `secs`, muffled through a lowpass at
    /// `cutoff` Hz (lower sounds like it's heard from indoors).
    pub fn rain(&mut self, level: f32, cutoff: f32, secs: f64) {
        if self.rain.is_none() {
            match self.bus.rain_loop() {
                Ok(nodes) => self.rain = Some(nodes),
                Err(e) => return web_sys::console::warn_1(&e),
            }
        }
        if let Some((lowpass, gain)) = &self.rain {
            let t = self.bus.ctx.current_time();
            ramp(&lowpass.frequency(), cutoff, t, secs);
            ramp(&gain.gain(), level, t, secs);
        }
    }

    pub fn hum(&mut self, level: f32, secs: f64) {
        if self.hum.is_none() {
            match self.bus.hum_loop() {
                Ok(gain) => self.hum = Some(gain),
                Err(e) => return web_sys::console::warn_1(&e),
            }
        }
        if let Some(gain) = &self.hum {
            ramp(&gain.gain(), level, self.bus.ctx.current_time(), secs);
        }
    }

    /// A thunderclap `delay` seconds from now.
    pub fn thunder(&self, delay: f64) {
        warn(self.bus.thunder(delay));
    }

    /// A burst of tape static, for cutting to camera footage.
    pub fn static_burst(&self) {
        warn(self.bus.static_burst());
    }

    /// An electrical spark: a few sharp snaps.
    pub fn crackle(&self) {
        warn(self.bus.crackle());
    }

    /// One typewriter key.
    pub fn tick(&self) {
        warn(self.bus.tick());
    }

    /// Starts the drone and loops the lullaby until `fade_out`.
    pub fn start_theme(&mut self) {
        if self.theme.is_some() {
            return;
        }
        warn(self.bus.drone());

        let loop_s = STEP_S * LULLABY.len() as f64;
        // The lullaby enters after the drone has had a few seconds to swell.
        let mut next = self.bus.ctx.current_time() + 3.0;
        let bus = self.bus.clone();
        self.start_loop(loop_s, move || {
            warn(bus.lullaby(next));
            next += loop_s;
        });
    }

    /// Calls `tick` now and then every `loop_s`. Each tick schedules one whole
    /// loop on the audio clock, a loop ahead, so timer jitter never reaches the music.
    fn start_loop(&mut self, loop_s: f64, mut tick: impl FnMut() + 'static) {
        self.stop_loop();
        tick();
        tick();
        let cb = Closure::<dyn FnMut()>::new(tick);
        if let Ok(id) = window().set_interval_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            (loop_s * 1000.0) as i32,
        ) {
            self.theme = Some((id, cb));
        }
    }

    fn stop_loop(&mut self) {
        if let Some((id, _cb)) = self.theme.take() {
            window().clear_interval_with_handle(id);
        }
    }

    /// Fades everything out, then closes the audio context.
    pub fn fade_out(mut self, secs: f64) {
        self.stop_loop();
        ramp(&self.bus.master.gain(), 0.0, self.bus.ctx.current_time(), secs);
        let ctx = self.bus.ctx.clone();
        after((secs * 1000.0) as i32 + 100, move || {
            let _ = ctx.close();
        });
    }
}

impl Bus {
    fn rumble(&self, secs: f64) -> Result<(), JsValue> {
        let t = self.ctx.current_time();
        let noise = self.noise(secs + 0.2)?;
        let lp = self.filter(BiquadFilterType::Lowpass, 90.0, 1.0)?;
        let sub = self.osc(OscillatorType::Sine, 28.0)?;
        sub.frequency().linear_ramp_to_value_at_time(46.0, t + secs)?;
        let out = self.envelope(t, 0.0001)?;
        out.gain().exponential_ramp_to_value_at_time(0.9, t + secs)?;
        out.gain().linear_ramp_to_value_at_time(0.0, t + secs + 0.15)?;

        chain(&[&noise, &lp, &out, &self.master])?;
        sub.connect_with_audio_node(&out)?;
        play(&noise, t, t + secs + 0.2)?;
        play(&sub, t, t + secs + 0.2)
    }

    fn boom(&self) -> Result<(), JsValue> {
        let t = self.ctx.current_time();

        // Pitch-dropping sub hit.
        let sub = self.osc(OscillatorType::Sine, 110.0)?;
        sub.frequency().exponential_ramp_to_value_at_time(30.0, t + 1.4)?;
        let sub_env = self.envelope(t, 0.9)?;
        sub_env.gain().exponential_ramp_to_value_at_time(0.0001, t + 3.0)?;
        chain(&[&sub, &sub_env, &self.master])?;
        play(&sub, t, t + 3.0)?;

        // Muffled noise burst, partly into the echo.
        let noise = self.noise(2.0)?;
        let lp = self.filter(BiquadFilterType::Lowpass, 600.0, 0.7)?;
        let noise_env = self.envelope(t, 0.7)?;
        noise_env.gain().exponential_ramp_to_value_at_time(0.0001, t + 1.8)?;
        chain(&[&noise, &lp, &noise_env, &self.master])?;
        noise_env.connect_with_audio_node(&self.echo)?;
        play(&noise, t, t + 2.0)?;

        // A thin E/F semitone ringing over the top.
        self.ting(t + 0.05, 0.05)
    }

    fn sting(&self) -> Result<(), JsValue> {
        let t = self.ctx.current_time();
        let lp = self.filter(BiquadFilterType::Lowpass, 400.0, 2.0)?;
        let env = self.envelope(t, 0.0001)?;
        env.gain().exponential_ramp_to_value_at_time(0.25, t + 0.1)?;
        env.gain().exponential_ramp_to_value_at_time(0.0001, t + 2.5)?;
        chain(&[&lp, &env, &self.master])?;
        env.connect_with_audio_node(&self.echo)?;
        for hz in [61.7, 65.4, 69.3] {
            let o = self.osc(OscillatorType::Sawtooth, hz)?;
            o.connect_with_audio_node(&lp)?;
            play(&o, t, t + 2.6)?;
        }
        self.ting(t, 0.03)
    }

    fn rain_loop(&self) -> Result<(BiquadFilterNode, GainNode), JsValue> {
        let noise = self.noise(3.0)?;
        noise.set_loop(true);
        let hp = self.filter(BiquadFilterType::Highpass, 300.0, 0.5)?;
        let lp = self.filter(BiquadFilterType::Lowpass, 2000.0, 0.5)?;
        let gain = self.ctx.create_gain()?;
        gain.gain().set_value(0.0);
        chain(&[&noise, &hp, &lp, &gain, &self.master])?;
        noise.start()?;
        Ok((lp, gain))
    }

    fn hum_loop(&self) -> Result<GainNode, JsValue> {
        let gain = self.ctx.create_gain()?;
        gain.gain().set_value(0.0);
        let lp = self.filter(BiquadFilterType::Lowpass, 320.0, 1.0)?;
        chain(&[&lp, &gain, &self.master])?;
        for (ty, hz) in [(OscillatorType::Square, 50.0), (OscillatorType::Sine, 100.0)] {
            let o = self.osc(ty, hz)?;
            o.connect_with_audio_node(&lp)?;
            o.start()?;
        }
        Ok(gain)
    }

    fn thunder(&self, delay: f64) -> Result<(), JsValue> {
        let t = self.ctx.current_time() + delay;
        let noise = self.noise(6.0)?;
        let lp = self.filter(BiquadFilterType::Lowpass, 900.0, 0.8)?;
        lp.frequency().set_value_at_time(900.0, t)?;
        lp.frequency().exponential_ramp_to_value_at_time(90.0, t + 4.0)?;
        let env = self.envelope(t, 0.0001)?;
        env.gain().exponential_ramp_to_value_at_time(0.8, t + 0.08)?;
        env.gain().exponential_ramp_to_value_at_time(0.35, t + 0.6)?;
        env.gain().exponential_ramp_to_value_at_time(0.0001, t + 5.5)?;
        chain(&[&noise, &lp, &env, &self.master])?;
        env.connect_with_audio_node(&self.echo)?;
        play(&noise, t, t + 6.0)
    }

    fn static_burst(&self) -> Result<(), JsValue> {
        let t = self.ctx.current_time();
        let noise = self.noise(0.5)?;
        let bp = self.filter(BiquadFilterType::Bandpass, 2500.0, 0.7)?;
        let env = self.envelope(t, 0.3)?;
        env.gain().exponential_ramp_to_value_at_time(0.0001, t + 0.45)?;
        chain(&[&noise, &bp, &env, &self.master])?;
        play(&noise, t, t + 0.5)
    }

    fn crackle(&self) -> Result<(), JsValue> {
        let t = self.ctx.current_time();
        let hp = self.filter(BiquadFilterType::Highpass, 2000.0, 0.7)?;
        chain(&[&hp, &self.master])?;
        let mut rng = t.to_bits() | 1;
        let mut at = t;
        for _ in 0..5 {
            let noise = self.noise(0.03)?;
            let env = self.envelope(at, 0.25)?;
            env.gain().exponential_ramp_to_value_at_time(0.0001, at + 0.025)?;
            chain(&[&noise, &env, &hp])?;
            play(&noise, at, at + 0.03)?;
            at += 0.02 + rand(&mut rng) * 0.05;
        }
        Ok(())
    }

    fn tick(&self) -> Result<(), JsValue> {
        let t = self.ctx.current_time();
        let noise = self.noise(0.02)?;
        let hp = self.filter(BiquadFilterType::Highpass, 3000.0, 0.7)?;
        let env = self.envelope(t, 0.06)?;
        env.gain().exponential_ramp_to_value_at_time(0.0001, t + 0.02)?;
        chain(&[&noise, &hp, &env, &self.master])?;
        play(&noise, t, t + 0.02)
    }

    fn blip(&self) -> Result<(), JsValue> {
        let t = self.ctx.current_time();
        let o = self.osc(OscillatorType::Sine, 1760.0)?;
        let env = self.envelope(t, 0.04)?;
        env.gain().exponential_ramp_to_value_at_time(0.0001, t + 0.12)?;
        chain(&[&o, &env, &self.master])?;
        play(&o, t, t + 0.15)
    }

    fn ting(&self, t: f64, level: f32) -> Result<(), JsValue> {
        let env = self.envelope(t, level)?;
        env.gain().exponential_ramp_to_value_at_time(0.0001, t + 4.0)?;
        env.connect_with_audio_node(&self.master)?;
        env.connect_with_audio_node(&self.echo)?;
        for hz in [1318.5, 1396.9] {
            let o = self.osc(OscillatorType::Sine, hz)?;
            o.connect_with_audio_node(&env)?;
            play(&o, t, t + 4.0)?;
        }
        Ok(())
    }

    /// Two beating saws under a slowly breathing filter, plus a tritone hum.
    /// Never stopped: `Music::fade_out` closes the whole context.
    fn drone(&self) -> Result<(), JsValue> {
        let t = self.ctx.current_time();
        let out = self.envelope(t, 0.0001)?;
        out.gain().exponential_ramp_to_value_at_time(0.18, t + 6.0)?;
        out.connect_with_audio_node(&self.master)?;

        let lp = self.filter(BiquadFilterType::Lowpass, 260.0, 5.0)?;
        lp.connect_with_audio_node(&out)?;
        let lfo = self.osc(OscillatorType::Sine, 0.06)?;
        let depth = self.ctx.create_gain()?;
        depth.gain().set_value(140.0);
        lfo.connect_with_audio_node(&depth)?;
        depth.connect_with_audio_param(&lp.frequency())?;
        lfo.start()?;

        for hz in [55.0, 55.37] {
            let o = self.osc(OscillatorType::Sawtooth, hz)?;
            o.connect_with_audio_node(&lp)?;
            o.start()?;
        }
        for (hz, level) in [(110.0, 0.3), (77.78, 0.25)] {
            let o = self.osc(OscillatorType::Sine, hz)?;
            let g = self.ctx.create_gain()?;
            g.gain().set_value(level);
            chain(&[&o, &g, &out])?;
            o.start()?;
        }
        Ok(())
    }

    fn lullaby(&self, start: f64) -> Result<(), JsValue> {
        let mut rng = start.to_bits() | 1;
        for (i, &note) in LULLABY.iter().enumerate() {
            if note == REST {
                continue;
            }
            // A worn-out box: slightly late, slightly out of tune.
            let jitter = (rand(&mut rng) - 0.5) * 0.03;
            let cents = (rand(&mut rng) - 0.5) * 24.0;
            self.music_box(note, cents, start + i as f64 * STEP_S + jitter, &self.master)?;
        }
        Ok(())
    }

    fn music_box(&self, note: u8, cents: f64, t: f64, out: &AudioNode) -> Result<(), JsValue> {
        let hz = midi(note as f64 + cents / 100.0);
        let env = self.envelope(t, 0.0)?;
        env.gain().linear_ramp_to_value_at_time(0.13, t + 0.005)?;
        env.gain().exponential_ramp_to_value_at_time(0.0001, t + 2.2)?;
        env.connect_with_audio_node(out)?;
        env.connect_with_audio_node(&self.echo)?;
        // Fundamental, octave and a slightly inharmonic tine partial.
        for (mult, level) in [(1.0, 1.0), (2.0, 0.35), (4.17, 0.12)] {
            let o = self.osc(OscillatorType::Sine, hz * mult)?;
            let g = self.ctx.create_gain()?;
            g.gain().set_value(level);
            chain(&[&o, &g, &env])?;
            play(&o, t, t + 2.3)?;
        }
        Ok(())
    }

    fn osc(&self, ty: OscillatorType, hz: f64) -> Result<OscillatorNode, JsValue> {
        let o = self.ctx.create_oscillator()?;
        o.set_type(ty);
        o.frequency().set_value(hz as f32);
        Ok(o)
    }

    fn filter(&self, ty: BiquadFilterType, hz: f32, q: f32) -> Result<BiquadFilterNode, JsValue> {
        let f = self.ctx.create_biquad_filter()?;
        f.set_type(ty);
        f.frequency().set_value(hz);
        f.q().set_value(q);
        Ok(f)
    }

    /// A gain node pinned to `level` at time `t`, ready for ramps.
    fn envelope(&self, t: f64, level: f32) -> Result<GainNode, JsValue> {
        let g = self.ctx.create_gain()?;
        g.gain().set_value_at_time(level, t)?;
        Ok(g)
    }

    fn noise(&self, secs: f64) -> Result<AudioBufferSourceNode, JsValue> {
        let src = self.ctx.create_buffer_source()?;
        src.set_buffer(Some(&noise_buffer(&self.ctx, secs)?));
        Ok(src)
    }

    /// A source on the shared noise buffer (up to 2s), cheaper than `noise` for drums.
    fn grit(&self) -> Result<AudioBufferSourceNode, JsValue> {
        let src = self.ctx.create_buffer_source()?;
        src.set_buffer(Some(&self.grit));
        Ok(src)
    }
}

fn noise_buffer(ctx: &AudioContext, secs: f64) -> Result<AudioBuffer, JsValue> {
    let rate = ctx.sample_rate();
    let len = (secs * rate as f64) as u32;
    let buffer = ctx.create_buffer(1, len, rate)?;
    let mut rng = 0x9E37_79B9_7F4A_7C15;
    let samples: Vec<f32> = (0..len).map(|_| (rand(&mut rng) * 2.0 - 1.0) as f32).collect();
    buffer.copy_to_channel(&samples, 0)?;
    Ok(buffer)
}

/// Frequency of a MIDI note (fractional notes detune).
fn midi(note: f64) -> f64 {
    440.0 * 2f64.powf((note - 69.0) / 12.0)
}

fn chain(nodes: &[&AudioNode]) -> Result<(), JsValue> {
    for pair in nodes.windows(2) {
        pair[0].connect_with_audio_node(pair[1])?;
    }
    Ok(())
}

/// Glides `param` from wherever it is now to `to` over `secs`.
fn ramp(param: &AudioParam, to: f32, t: f64, secs: f64) {
    let _ = param.cancel_scheduled_values(t);
    let _ = param.set_value_at_time(param.value(), t);
    let _ = param.linear_ramp_to_value_at_time(to, t + secs);
}

fn play(node: &AudioScheduledSourceNode, from: f64, to: f64) -> Result<(), JsValue> {
    node.start_with_when(from)?;
    node.stop_with_when(to)
}

/// xorshift64, uniform in [0, 1).
fn rand(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state >> 11) as f64 / (1u64 << 53) as f64
}

/// Audio is decoration: log failures instead of breaking the menu.
fn warn(result: Result<(), JsValue>) {
    if let Err(e) = result {
        web_sys::console::warn_1(&e);
    }
}

//! Procedural audio for the title screen (Web Audio, no asset files):
//! intro rumble and boom, a detuned drone and an out-of-tune music-box lullaby.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    AudioBufferSourceNode, AudioContext, AudioNode, AudioScheduledSourceNode, BiquadFilterNode,
    BiquadFilterType, GainNode, OscillatorNode, OscillatorType,
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
}

pub struct Music {
    bus: Bus,
    theme: Option<(i32, Closure<dyn FnMut()>)>,
}

impl Music {
    /// Must be called from a user-gesture handler, or the browser keeps it muted.
    pub fn new() -> Result<Self, JsValue> {
        let ctx = AudioContext::new()?;
        let master = ctx.create_gain()?;
        master.gain().set_value(0.8);
        master.connect_with_audio_node(&ctx.destination())?;

        let echo = ctx.create_gain()?;
        let delay = ctx.create_delay_with_max_delay_time(2.0)?;
        delay.delay_time().set_value(0.63);
        let feedback = ctx.create_gain()?;
        feedback.gain().set_value(0.45);
        let wet = ctx.create_gain()?;
        wet.gain().set_value(0.5);

        let bus = Bus { ctx, master, echo };
        let tone = bus.filter(BiquadFilterType::Lowpass, 1800.0, 0.7)?;
        bus.echo.connect_with_audio_node(&delay)?;
        chain(&[&delay, &tone, &feedback, &delay])?;
        chain(&[&tone, &wet, &bus.master])?;
        Ok(Self { bus, theme: None })
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
        // Each tick schedules one whole loop ahead on the audio clock, so
        // timer jitter never reaches the music.
        let mut tick = move || {
            warn(bus.lullaby(next));
            next += loop_s;
        };
        tick();
        let cb = Closure::<dyn FnMut()>::new(tick);
        if let Ok(id) = window().set_interval_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            (loop_s * 1000.0) as i32,
        ) {
            self.theme = Some((id, cb));
        }
    }

    /// Fades everything out, then closes the audio context.
    pub fn fade_out(mut self, secs: f64) {
        if let Some((id, _cb)) = self.theme.take() {
            window().clear_interval_with_handle(id);
        }
        let t = self.bus.ctx.current_time();
        let gain = self.bus.master.gain();
        let _ = gain.cancel_scheduled_values(t);
        let _ = gain.set_value_at_time(gain.value(), t);
        let _ = gain.linear_ramp_to_value_at_time(0.0, t + secs);
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
            self.music_box(note, cents, start + i as f64 * STEP_S + jitter)?;
        }
        Ok(())
    }

    fn music_box(&self, note: u8, cents: f64, t: f64) -> Result<(), JsValue> {
        let hz = 440.0 * 2f64.powf((note as f64 - 69.0) / 12.0 + cents / 1200.0);
        let env = self.envelope(t, 0.0)?;
        env.gain().linear_ramp_to_value_at_time(0.13, t + 0.005)?;
        env.gain().exponential_ramp_to_value_at_time(0.0001, t + 2.2)?;
        env.connect_with_audio_node(&self.master)?;
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
        let rate = self.ctx.sample_rate();
        let len = (secs * rate as f64) as u32;
        let buffer = self.ctx.create_buffer(1, len, rate)?;
        let mut rng = 0x9E37_79B9_7F4A_7C15;
        let samples: Vec<f32> = (0..len).map(|_| (rand(&mut rng) * 2.0 - 1.0) as f32).collect();
        buffer.copy_to_channel(&samples, 0)?;
        let src = self.ctx.create_buffer_source()?;
        src.set_buffer(Some(&buffer));
        Ok(src)
    }
}

fn chain(nodes: &[&AudioNode]) -> Result<(), JsValue> {
    for pair in nodes.windows(2) {
        pair[0].connect_with_audio_node(pair[1])?;
    }
    Ok(())
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

//! Sound effects for the endings: thuds, gunfire, a passing motorcycle,
//! a heart monitor and an old telephone bell.

use wasm_bindgen::prelude::*;
use web_sys::{BiquadFilterType, OscillatorType};

use super::{chain, play, warn, Bus, Music};

impl Music {
    /// A dull thud: a badge on a desk (quiet), a footstep, a body hitting the floor (loud).
    pub fn knock(&self, level: f32) {
        warn(self.bus.knock(level));
    }

    pub fn gunshot(&self) {
        warn(self.bus.gunshot());
    }

    /// A motorcycle coming from far away, roaring past at `delay + secs / 2`, then gone.
    pub fn engine_pass(&self, delay: f64, secs: f64) {
        warn(self.bus.engine_pass(delay, secs));
    }

    /// Heart-monitor beeps every `every` seconds for `secs`.
    pub fn monitor(&self, every: f64, secs: f64) {
        warn(self.bus.monitor(every, secs));
    }

    /// An old telephone ringing `times` times, starting after `delay`.
    pub fn ring(&self, delay: f64, times: u32) {
        warn(self.bus.ring(delay, times));
    }
}

impl Bus {
    fn knock(&self, level: f32) -> Result<(), JsValue> {
        let t = self.ctx.current_time();
        let o = self.osc(OscillatorType::Sine, 110.0)?;
        o.frequency().exponential_ramp_to_value_at_time(45.0, t + 0.15)?;
        let env = self.envelope(t, level)?;
        env.gain().exponential_ramp_to_value_at_time(0.001, t + 0.35)?;
        chain(&[&o, &env, &self.master])?;
        play(&o, t, t + 0.4)?;
        let noise = self.grit()?;
        let lp = self.filter(BiquadFilterType::Lowpass, 700.0, 0.7)?;
        let thump = self.envelope(t, level * 0.6)?;
        thump.gain().exponential_ramp_to_value_at_time(0.001, t + 0.12)?;
        chain(&[&noise, &lp, &thump, &self.master])?;
        play(&noise, t, t + 0.15)
    }

    fn gunshot(&self) -> Result<(), JsValue> {
        let t = self.ctx.current_time();
        let noise = self.grit()?;
        let bp = self.filter(BiquadFilterType::Bandpass, 1600.0, 0.6)?;
        let env = self.envelope(t, 0.7)?;
        env.gain().exponential_ramp_to_value_at_time(0.001, t + 0.18)?;
        chain(&[&noise, &bp, &env, &self.master])?;
        env.connect_with_audio_node(&self.echo)?;
        play(&noise, t, t + 0.2)?;
        let o = self.osc(OscillatorType::Sine, 90.0)?;
        o.frequency().exponential_ramp_to_value_at_time(40.0, t + 0.1)?;
        let thump = self.envelope(t, 0.6)?;
        thump.gain().exponential_ramp_to_value_at_time(0.001, t + 0.15)?;
        chain(&[&o, &thump, &self.master])?;
        play(&o, t, t + 0.16)
    }

    fn engine_pass(&self, delay: f64, secs: f64) -> Result<(), JsValue> {
        let t = self.ctx.current_time() + delay;
        let mid = t + secs / 2.0;
        let lp = self.filter(BiquadFilterType::Lowpass, 300.0, 2.0)?;
        lp.frequency().set_value_at_time(300.0, t)?;
        lp.frequency().exponential_ramp_to_value_at_time(1800.0, mid)?;
        lp.frequency().exponential_ramp_to_value_at_time(250.0, t + secs)?;
        let env = self.envelope(t, 0.001)?;
        env.gain().exponential_ramp_to_value_at_time(0.35, mid)?;
        env.gain().exponential_ramp_to_value_at_time(0.001, t + secs)?;
        chain(&[&lp, &env, &self.master])?;
        // Pitch rises as it comes, drops as it goes (Doppler).
        for (hz, ty) in [(62.0, OscillatorType::Sawtooth), (93.0, OscillatorType::Square)] {
            let o = self.osc(ty, hz)?;
            o.frequency().set_value_at_time(hz as f32, t)?;
            o.frequency().linear_ramp_to_value_at_time((hz * 1.5) as f32, mid)?;
            o.frequency().linear_ramp_to_value_at_time((hz * 0.9) as f32, t + secs)?;
            o.connect_with_audio_node(&lp)?;
            play(&o, t, t + secs)?;
        }
        Ok(())
    }

    fn monitor(&self, every: f64, secs: f64) -> Result<(), JsValue> {
        let start = self.ctx.current_time() + 0.5;
        let mut t = start;
        while t < start + secs {
            let o = self.osc(OscillatorType::Sine, 988.0)?;
            let env = self.envelope(t, 0.08)?;
            env.gain().set_value_at_time(0.08, t + 0.1)?;
            env.gain().linear_ramp_to_value_at_time(0.0, t + 0.12)?;
            chain(&[&o, &env, &self.master])?;
            play(&o, t, t + 0.13)?;
            t += every;
        }
        Ok(())
    }

    fn ring(&self, delay: f64, times: u32) -> Result<(), JsValue> {
        let start = self.ctx.current_time() + delay;
        for k in 0..times {
            let t = start + k as f64 * 4.0;
            // Two tones through a gain that a fast square wave chops into a bell rattle.
            let rattle = self.ctx.create_gain()?;
            rattle.gain().set_value(0.0);
            let lfo = self.osc(OscillatorType::Square, 20.0)?;
            let depth = self.ctx.create_gain()?;
            depth.gain().set_value(0.5);
            chain(&[&lfo, &depth])?;
            depth.connect_with_audio_param(&rattle.gain())?;
            let env = self.envelope(t, 0.12)?;
            env.gain().set_value_at_time(0.0, t + 1.6)?;
            chain(&[&rattle, &env, &self.master])?;
            play(&lfo, t, t + 1.6)?;
            for hz in [440.0, 480.0] {
                let o = self.osc(OscillatorType::Triangle, hz)?;
                o.connect_with_audio_node(&rattle)?;
                play(&o, t, t + 1.6)?;
            }
        }
        Ok(())
    }
}

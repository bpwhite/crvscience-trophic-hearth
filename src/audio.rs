// Copyright (C) 2026 CRV Science Network (https://crvscience.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://gnu.org>.

use crate::events::{SoundCue, SoundCueKind};
use std::f32::consts::PI;
use std::io::Write;
use std::path::Path;

pub struct BioacousticEngine {
    pub muted: bool,
    pub master_volume: f32,
    #[cfg(feature = "live-audio")]
    stream_and_sink: Option<(rodio::OutputStream, rodio::OutputStreamHandle)>,
}

impl Default for BioacousticEngine {
    fn default() -> Self {
        Self::new(true)
    }
}

impl BioacousticEngine {
    pub fn new(enable_hardware: bool) -> Self {
        #[cfg(feature = "live-audio")]
        let stream_and_sink = if enable_hardware {
            rodio::OutputStream::try_default().ok()
        } else {
            None
        };

        #[cfg(not(feature = "live-audio"))]
        let _ = enable_hardware;

        Self {
            muted: false,
            master_volume: 0.55,
            #[cfg(feature = "live-audio")]
            stream_and_sink,
        }
    }

    pub fn trigger(&self, cue: SoundCue) {
        if self.muted || self.master_volume <= 0.001 {
            return;
        }
        #[cfg(feature = "live-audio")]
        if let Some((_, ref handle)) = self.stream_and_sink {
            let samples = synthesize_cue_stereo(cue, 44_100, self.master_volume);
            let source = StereoVecSource {
                samples,
                cursor: 0,
                sample_rate: 44_100,
            };
            let _ = handle.play_raw(source);
        }
    }
}

#[cfg(feature = "live-audio")]
struct StereoVecSource {
    samples: Vec<f32>,
    cursor: usize,
    sample_rate: u32,
}

#[cfg(feature = "live-audio")]
impl Iterator for StereoVecSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor < self.samples.len() {
            let v = self.samples[self.cursor];
            self.cursor += 1;
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(feature = "live-audio")]
impl rodio::Source for StereoVecSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some((self.samples.len().saturating_sub(self.cursor)) / 2)
    }

    fn channels(&self) -> u16 {
        2
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        let frames = self.samples.len() / 2;
        Some(std::time::Duration::from_secs_f64(
            frames as f64 / self.sample_rate as f64,
        ))
    }
}

#[inline]
fn hash_noise(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    x = ((x >> ((x >> 28).wrapping_add(4))) ^ x).wrapping_mul(277_803_737);
    let y = (x >> 22) ^ x;
    (y as f32 / u32::MAX as f32) * 2.0 - 1.0
}

pub fn synthesize_cue_stereo(cue: SoundCue, sample_rate: u32, master_gain: f32) -> Vec<f32> {
    let duration_sec = match cue.kind {
        SoundCueKind::MonkeyChatter => 0.62,
        SoundCueKind::MacawSquawk => 0.58,
        SoundCueKind::ToucanCroak => 0.42,
        SoundCueKind::BoaHiss => 0.85,
        SoundCueKind::PantherLowRumble => 1.10,
        SoundCueKind::UnderbrushRustle | SoundCueKind::KelpFrondSwish => 0.52,
        SoundCueKind::FruitDropThud => 0.26,
        SoundCueKind::TreefrogCoqui => 0.38,
        SoundCueKind::CicadaStridulation => 0.95,
        SoundCueKind::RainShower | SoundCueKind::OceanSwellSurge => 1.65,
        SoundCueKind::DistantThunder | SoundCueKind::DistantWhaleHydrophone => 2.25,
        SoundCueKind::OtterChestStoneClick => 0.66,
        SoundCueKind::UnderwaterBubbleStream => 0.88,
        SoundCueKind::ReefUrchinCrunch | SoundCueKind::ParrotfishCoralCrunch => 0.38,
        SoundCueKind::SnappingShrimpCrackle => 1.25,
        SoundCueKind::LoonTremoloWail => 1.85,
        SoundCueKind::MallardQuack => 0.55,
        SoundCueKind::KingfisherRattle => 0.72,
        SoundCueKind::BullfrogJugORum => 0.95,
        SoundCueKind::WaterPlungeSplash => 0.68,
        SoundCueKind::HydrothermalVentRumble => 1.85,
        SoundCueKind::TectonicFissureCrack => 0.78,
        SoundCueKind::AbyssalBiolumPing => 1.25,
        SoundCueKind::SpottedOwlFourNoteHoot => 1.85,
        SoundCueKind::VariedThrushFlute => 1.35,
        SoundCueKind::GravelCreekRiffle => 1.60,
        SoundCueKind::SalmonWeirSplash => 0.65,
        SoundCueKind::SpermWhaleClickCoda => 1.75,
        SoundCueKind::DslBiolumShimmer => 1.50,
        SoundCueKind::PelagicCanyonCurrent => 1.80,
        SoundCueKind::ArchiteuthisMantleJet => 0.95,
        SoundCueKind::SurfSurgeBreaker => 1.75,
        SoundCueKind::BlackOystercatcherPipe => 0.92,
        SoundCueKind::TidepoolBubbleTrickle => 1.25,
        SoundCueKind::MethaneClathrateBubbles => 1.45,
        SoundCueKind::BrinePoolDensityHum => 1.85,
    };

    let frames = (duration_sec * sample_rate as f32) as usize;
    let mut out = Vec::with_capacity(frames * 2);

    let pan = (cue.world_pos.x / 15.0).clamp(-0.85, 0.85);
    let left_gain = ((1.0 - pan) * 0.5).sqrt();
    let right_gain = ((1.0 + pan) * 0.5).sqrt();
    let depth_dist = (4.0 - cue.world_pos.z).max(0.0);
    let dist_atten = (1.0 / (1.0 + 0.065 * depth_dist)).clamp(0.18, 1.0);
    let lp_alpha = (0.88 - 0.032 * depth_dist).clamp(0.16, 0.88);

    let mut lp_state = 0.0_f32;
    let mut phase = 0.0_f32;
    let p_mul = cue.pitch_mul.clamp(0.65, 1.45);
    let amp_scale = (cue.gain * master_gain * dist_atten).clamp(0.0, 0.95);

    for i in 0..frames {
        let t = i as f32 / sample_rate as f32;
        let u = t / duration_sec;
        let n = hash_noise(i as u32 ^ ((cue.kind as u32) << 16));

        let raw = match cue.kind {
            SoundCueKind::MonkeyChatter => {
                let syllable = (t * 7.5).fract();
                let env = (syllable * PI).sin().max(0.0).powf(1.8) * (1.0 - u * 0.35);
                let f = (1250.0 + 680.0 * (syllable * 2.0 * PI).sin()) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.72 + (phase * 2.04).sin() * 0.28 + n * 0.08) * env * 0.45
            }
            SoundCueKind::MacawSquawk => {
                let env = (u * PI).sin().powf(0.65);
                let vibrato = (t * 48.0 * 2.0 * PI).sin() * 65.0;
                let f = (720.0 - 180.0 * u + vibrato) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.45 + (phase * 2.0).sin() * 0.32 + (phase * 3.0).sin() * 0.18 + n * 0.16)
                    * env
                    * 0.40
            }
            SoundCueKind::ToucanCroak => {
                let pulse = (t * 5.2).fract();
                let env = (-pulse * 12.0).exp() * (1.0 - 0.3 * u);
                let f = (460.0 + 90.0 * (-pulse * 18.0).exp()) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() + 0.55 * (phase * 1.98).sin()) * env * 0.42
            }
            SoundCueKind::BoaHiss => {
                let env = (u * PI).sin().powf(0.8);
                let mod_f = (2900.0 + 400.0 * (t * 9.0).sin()) * p_mul;
                phase += 2.0 * PI * mod_f / sample_rate as f32;
                (n * 0.75 + n * phase.sin() * 0.25) * env * 0.28
            }
            SoundCueKind::PantherLowRumble => {
                let env = (u * PI).sin().powf(0.9);
                let flutter = 1.0 + 0.32 * (t * 24.0 * 2.0 * PI).sin();
                let f = (58.0 - 8.0 * u) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                ((phase.sin() * 0.65 + (phase * 2.0).sin() * 0.25) * flutter + n * 0.10) * env * 0.52
            }
            SoundCueKind::UnderbrushRustle => {
                let env = (-u * 3.8).exp() * (u * 14.0).min(1.0);
                let grain = 0.5 + 0.5 * (t * 34.0).sin();
                n * grain * env * 0.24
            }
            SoundCueKind::FruitDropThud => {
                let env = (-t * 22.0).exp();
                let f = (135.0 * (-t * 14.0).exp() + 48.0) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.85 + n * 0.15) * env * 0.46
            }
            SoundCueKind::TreefrogCoqui => {
                if u < 0.38 {
                    let env = ((u / 0.38) * PI).sin();
                    phase += 2.0 * PI * (1180.0 * p_mul) / sample_rate as f32;
                    phase.sin() * env * 0.32
                } else if u > 0.48 {
                    let v = (u - 0.48) / 0.52;
                    let env = (v * PI).sin();
                    phase += 2.0 * PI * ((2050.0 + 380.0 * v) * p_mul) / sample_rate as f32;
                    phase.sin() * env * 0.36
                } else {
                    0.0
                }
            }
            SoundCueKind::CicadaStridulation => {
                let env = (u * PI).sin();
                let pulse = 0.55 + 0.45 * (t * 110.0 * 2.0 * PI).sin();
                phase += 2.0 * PI * (4350.0 * p_mul) / sample_rate as f32;
                (phase.sin() * 0.6 + n * 0.4) * pulse * env * 0.16
            }
            SoundCueKind::RainShower => {
                let env = (u * PI).sin().powf(0.45);
                let drop = if hash_noise(i as u32 ^ 0x9E37_79B9) > 0.996 { 0.55 } else { 0.0 };
                (n * 0.22 + drop) * env * 0.30
            }
            SoundCueKind::DistantThunder => {
                let env = (-u * 2.1).exp() * (u * 8.0).min(1.0);
                let rumble_f = 42.0 + 18.0 * (t * 3.1).sin();
                phase += 2.0 * PI * rumble_f / sample_rate as f32;
                (phase.sin() * 0.55 + n * 0.45) * env * 0.48
            }
            // --- Marine Kelp Forest Cues ---
            SoundCueKind::OtterChestStoneClick => {
                // 3 crisp stone-on-urchin taps at the water surface ("tap-tap-tap!")
                let tap_u = (t * 4.6).fract();
                let env = (-tap_u * 28.0).exp();
                let f = (1680.0 + 420.0 * (-tap_u * 40.0).exp()) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.72 + (phase * 2.41).sin() * 0.28) * env * 0.48
            }
            SoundCueKind::UnderwaterBubbleStream => {
                let grain = (t * 13.0).fract();
                let env = (grain * PI).sin() * (1.0 - 0.4 * u);
                let f = (480.0 + 620.0 * grain + 120.0 * (t * 7.0).sin()) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                phase.sin() * env * 0.34
            }
            SoundCueKind::OceanSwellSurge => {
                let env = (u * PI).sin().powf(0.7);
                let f = 92.0 + 28.0 * (t * 1.4).sin();
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.45 + n * 0.22) * env * 0.32
            }
            SoundCueKind::KelpFrondSwish => {
                let env = (u * PI).sin();
                let f = 210.0 + 80.0 * u;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.35 + n * 0.25) * env * 0.25
            }
            SoundCueKind::ReefUrchinCrunch | SoundCueKind::ParrotfishCoralCrunch => {
                let crackle = if hash_noise(i as u32 ^ 0x517C_C1B7) > 0.92 { 0.85 } else { 0.18 };
                let env = (-u * 4.2).exp();
                n * crackle * env * 0.38
            }
            SoundCueKind::SnappingShrimpCrackle => {
                // Warm reef "crackling fireplace" cavitation snaps from Alpheus snapping shrimp!
                let snap = if hash_noise(i as u32 ^ 0xA5A5_5A5A) > 0.993 { 0.92 } else { 0.04 };
                let swell = 0.08 * (2.0 * PI * 95.0 * t).sin();
                (n * snap + swell) * (u * PI).sin() * 0.36
            }
            SoundCueKind::DistantWhaleHydrophone => {
                let env = (u * PI).sin().powf(0.85);
                let f = (195.0 - 68.0 * u + 12.0 * (t * 3.2).sin()) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.78 + (phase * 2.0).sin() * 0.22) * env * 0.38
            }
            SoundCueKind::LoonTremoloWail => {
                // Haunting North-Woods Gavia immer rising-and-falling wail with soft vibrato
                let env = (u * PI).sin().powf(0.75);
                let arch = (u * PI).sin();
                let vib = (t * 8.5 * 2.0 * PI).sin() * 18.0;
                let f = (520.0 + 340.0 * arch + vib) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.84 + (phase * 2.0).sin() * 0.16) * env * 0.40
            }
            SoundCueKind::MallardQuack => {
                let syl = (t * 5.6).fract();
                let env = (-syl * 8.5).exp() * (1.0 - 0.35 * u);
                let f = (390.0 - 75.0 * syl) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.45 + (phase * 2.0).sin() * 0.35 + (phase * 3.0).sin() * 0.20) * env * 0.38
            }
            SoundCueKind::KingfisherRattle => {
                let click = (t * 21.0).fract();
                let env = (-click * 14.0).exp() * (u * PI).sin();
                let f = 2250.0 * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.65 + n * 0.35) * env * 0.34
            }
            SoundCueKind::BullfrogJugORum => {
                let env = (u * PI).sin().powf(0.8);
                let f = (108.0 - 16.0 * u + 6.0 * (t * 26.0 * 2.0 * PI).sin()) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.68 + (phase * 2.0).sin() * 0.32) * env * 0.42
            }
            SoundCueKind::WaterPlungeSplash => {
                let env = (-t * 5.8).exp();
                let bubble = (2.0 * PI * (320.0 + 480.0 * t) * t).sin() * 0.28;
                (n * 0.72 + bubble) * env * 0.46
            }
            SoundCueKind::HydrothermalVentRumble => {
                // Warm sub-bass black-smoker chimney roar + mineral percolation ("abyssal fireplace")
                let env = (u * PI).sin().powf(0.65);
                let f = (54.0 + 14.0 * (t * 2.2).sin()) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                let perc = if hash_noise(i as u32 ^ 0x7F4A_7C15) > 0.992 { 0.48 } else { 0.0 };
                (phase.sin() * 0.62 + n * 0.24 + perc) * env * 0.48
            }
            SoundCueKind::TectonicFissureCrack => {
                let env = (-t * 4.5).exp();
                let f = (95.0 - 40.0 * u) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.55 + n * 0.45) * env * 0.50
            }
            SoundCueKind::AbyssalBiolumPing => {
                let env = (-t * 2.6).exp() * (t * 22.0).min(1.0);
                let f = 980.0 * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.75 + (phase * 1.5).sin() * 0.25) * env * 0.30
            }
            SoundCueKind::SpottedOwlFourNoteHoot => {
                // Iconic Strix occidentalis 4-note cadence: "hoo ... hoo-hoo ... hoooo" (~520 Hz)
                let win = if (0.04..0.22).contains(&u) || (0.36..0.50).contains(&u) || (0.54..0.68).contains(&u) || (0.74..0.98).contains(&u) {
                    1.0
                } else {
                    0.0
                };
                let droop = if u > 0.74 { (u - 0.74) * 140.0 } else { 0.0 };
                let f = (535.0 - droop) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.85 + (phase * 2.0).sin() * 0.15) * win * 0.38
            }
            SoundCueKind::VariedThrushFlute => {
                // Ethereal PNW Ixoreus naevius dual-pitched harmonic whistle in damp old-growth
                let env = (u * PI).sin().powf(0.65);
                let trill = (t * 32.0 * 2.0 * PI).sin() * 18.0;
                let f = (2420.0 + trill) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.72 + (phase * 1.035).sin() * 0.28) * env * 0.28
            }
            SoundCueKind::GravelCreekRiffle => {
                let env = (u * PI).sin().powf(0.55);
                let babble = (2.0 * PI * (420.0 + 180.0 * (t * 7.5).sin()) * t).sin() * 0.22;
                (n * 0.26 + babble) * env * 0.30
            }
            SoundCueKind::SalmonWeirSplash => {
                let env = (-t * 6.2).exp();
                let plop = (2.0 * PI * (280.0 + 380.0 * t) * t).sin() * 0.32;
                (n * 0.68 + plop) * env * 0.45
            }
            SoundCueKind::SpermWhaleClickCoda => {
                // Authentic 5-pulse Physeter spermaceti click coda ("1+1+3" rhythm) + canyon wall echo
                let click_times = [0.06_f32, 0.34, 0.68, 0.82, 0.96];
                let mut acc = 0.0_f32;
                for &ct in &click_times {
                    let dt_c = t - ct;
                    if (0.0..0.055).contains(&dt_c) {
                        let env = (-dt_c * 85.0).exp();
                        acc += ((2.0 * PI * 1450.0 * p_mul * dt_c).sin() * 0.7 + n * 0.3) * env;
                    }
                    let dt_echo = t - (ct + 0.14);
                    if (0.0..0.065).contains(&dt_echo) {
                        let env = (-dt_echo * 65.0).exp() * 0.35;
                        acc += (2.0 * PI * 920.0 * p_mul * dt_echo).sin() * env;
                    }
                }
                acc * 0.46
            }
            SoundCueKind::DslBiolumShimmer => {
                // Calm non-strobing mesopelagic harmonic chord (220 / 330 / 440 Hz)
                let env = (u * PI).sin().powf(0.75);
                let f = 220.0 * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.55 + (phase * 1.5).sin() * 0.30 + (phase * 2.0).sin() * 0.15) * env * 0.28
            }
            SoundCueKind::PelagicCanyonCurrent => {
                let env = (u * PI).sin().powf(0.6);
                let hum = (2.0 * PI * 92.0 * p_mul * t).sin() * 0.38;
                (n * 0.20 + hum) * env * 0.30
            }
            SoundCueKind::ArchiteuthisMantleJet => {
                let env = (-t * 3.8).exp() * (t * 18.0).min(1.0);
                let jet = (2.0 * PI * (150.0 - 55.0 * u) * t).sin() * 0.45;
                (n * 0.35 + jet) * env * 0.40
            }
            SoundCueKind::SurfSurgeBreaker => {
                let env = (u * PI).sin().powf(0.62);
                let foam = (2.0 * PI * (135.0 + 45.0 * (t * 2.2).sin()) * t).sin() * 0.28;
                (n * 0.44 + foam) * env * 0.38
            }
            SoundCueKind::BlackOystercatcherPipe => {
                // Sharp piping Haematopus bachmani contact notes (~2.7 kHz)
                let win = if (0.05..0.24).contains(&u) || (0.34..0.52).contains(&u) || (0.62..0.85).contains(&u) { 1.0 } else { 0.0 };
                let f = (2680.0 - 240.0 * u) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                phase.sin() * win * 0.34
            }
            SoundCueKind::TidepoolBubbleTrickle | SoundCueKind::MethaneClathrateBubbles => {
                let env = (u * PI).sin().powf(0.65);
                let f = (520.0 + 280.0 * (t * 16.0).sin()) * p_mul;
                phase += 2.0 * PI * f / sample_rate as f32;
                (phase.sin() * 0.55 + n * 0.18) * env * 0.28
            }
            SoundCueKind::BrinePoolDensityHum => {
                let env = (u * PI).sin().powf(0.70);
                let f1 = (2.0 * PI * 68.0 * p_mul * t).sin() * 0.55;
                let f2 = (2.0 * PI * 102.0 * p_mul * t).sin() * 0.30;
                (f1 + f2 + n * 0.12) * env * 0.32
            }
        };

        lp_state += lp_alpha * (raw - lp_state);
        let sample = (lp_state * amp_scale).clamp(-0.98, 0.98);
        out.push(sample * left_gain);
        out.push(sample * right_gain);
    }

    out
}

pub fn export_ambient_soundscape_wav(path: &Path, duration_sec: f32) -> std::io::Result<()> {
    let sr = 44_100_u32;
    let total_frames = (duration_sec.max(1.0) * sr as f32) as usize;
    let mut mix = vec![0.0_f32; total_frames * 2];

    let mut lp_l = 0.0_f32;
    let mut lp_r = 0.0_f32;
    for i in 0..total_frames {
        let t = i as f32 / sr as f32;
        let swell = 0.045 + 0.025 * (t * 0.42).sin();
        lp_l += 0.06 * (hash_noise(i as u32) * swell - lp_l);
        lp_r += 0.06 * (hash_noise((i as u32).wrapping_add(99_991)) * swell - lp_r);
        mix[i * 2] += lp_l;
        mix[i * 2 + 1] += lp_r;
    }

    let script: &[(f32, SoundCueKind, f32, f32)] = &[
        (0.8, SoundCueKind::OtterChestStoneClick, -3.5, -1.0),
        (2.4, SoundCueKind::UnderwaterBubbleStream, -3.0, -1.2),
        (4.2, SoundCueKind::DistantWhaleHydrophone, 5.0, -12.0),
        (7.2, SoundCueKind::MonkeyChatter, -6.5, -1.5),
        (9.4, SoundCueKind::PantherLowRumble, -1.8, 1.6),
        (12.2, SoundCueKind::TreefrogCoqui, -8.0, -2.5),
        (14.8, SoundCueKind::OceanSwellSurge, 0.0, -4.0),
    ];

    for &(t_sec, kind, x, z) in script {
        let start_frame = ((t_sec % duration_sec.max(1.0)) * sr as f32) as usize;
        let cue = SoundCue {
            kind,
            world_pos: glam::Vec3::new(x, 3.0, z),
            gain: 0.85,
            pitch_mul: 1.0,
        };
        let clip = synthesize_cue_stereo(cue, sr, 0.85);
        for f in 0..(clip.len() / 2) {
            let dst = start_frame + f;
            if dst >= total_frames {
                break;
            }
            mix[dst * 2] = (mix[dst * 2] + clip[f * 2]).clamp(-0.98, 0.98);
            mix[dst * 2 + 1] = (mix[dst * 2 + 1] + clip[f * 2 + 1]).clamp(-0.98, 0.98);
        }
    }

    write_pcm16_stereo_wav(path, sr, &mix)
}

fn write_pcm16_stereo_wav(path: &Path, sample_rate: u32, interleaved_lr: &[f32]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    let num_channels: u16 = 2;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_bytes = (interleaved_lr.len() * 2) as u32;
    let riff_size = 36_u32.saturating_add(data_bytes);

    file.write_all(b"RIFF")?;
    file.write_all(&riff_size.to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16_u32.to_le_bytes())?;
    file.write_all(&1_u16.to_le_bytes())?;
    file.write_all(&num_channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;

    let mut pcm_bytes = Vec::with_capacity(interleaved_lr.len() * 2);
    for &s in interleaved_lr {
        let q = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        pcm_bytes.extend_from_slice(&q.to_le_bytes());
    }
    file.write_all(&pcm_bytes)?;
    Ok(())
}

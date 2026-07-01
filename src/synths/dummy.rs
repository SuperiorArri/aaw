use crate::{
    audio::block::AudioBlock,
    midi::{ControlChange, MidiEvent, MidiEventKind},
};
use std::f32::consts::TAU;

const MAX_VOICES: usize = 32;
const OUTPUT_GAIN: f32 = 0.07;
const ATTACK_TIME_SECONDS: f32 = 0.025;
const RELEASE_TIME_SECONDS: f32 = 0.5;

struct SineVoice {
    key: u8,
    freq: f32,
    phase: f32,
    amp: f32,
    attack: f32,
    release: f32,
    releasing: bool,
}

pub struct Dummy {
    sample_rate: f32,
    voices: Vec<SineVoice>,
}

impl Dummy {
    pub fn new() -> Self {
        Self {
            sample_rate: 48_000.0,
            voices: Vec::with_capacity(MAX_VOICES),
        }
    }
}

impl Default for Dummy {
    fn default() -> Self {
        Self::new()
    }
}

impl Dummy {
    pub fn prepare(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate as f32;
        self.voices.clear();
    }

    pub fn handle_midi_event(&mut self, event: MidiEvent) {
        match event.kind {
            MidiEventKind::NoteOn { key, velocity } if velocity > 0 => {
                self.voices.retain(|voice| voice.key != key);
                if self.voices.len() == MAX_VOICES {
                    self.voices.remove(0);
                }
                let freq = 440.0 * 2.0_f32.powf((key as f32 - 69.0) / 12.0);
                self.voices.push(SineVoice {
                    key,
                    freq,
                    phase: 0.0,
                    amp: velocity as f32 / 127.0,
                    attack: 0.0,
                    release: 1.0,
                    releasing: false,
                });
            }
            MidiEventKind::NoteOff { key, .. } => {
                for voice in &mut self.voices {
                    if voice.key == key {
                        voice.releasing = true;
                    }
                }
            }
            MidiEventKind::ControlChange { controller, value: _ } => {
                if controller == ControlChange::AllNotesOff as u8 {
                    self.voices.clear();
                }
            }
            _ => {}
        }
    }

    pub fn process(&mut self, output: &mut AudioBlock) {
        let frame_count = output.frame_count();
        let channel_count = output.channel_count();
        let samples = output.samples_mut();

        for frame in 0..frame_count {
            let sample = self.next_frame();

            for channel_samples in samples.iter_mut().take(channel_count) {
                channel_samples[frame] = sample;
            }
        }

        self.voices
            .retain(|voice| !voice.releasing || voice.release > 0.001);
    }

    fn next_frame(&mut self) -> f32 {
        let attack_step = attack_step_per_sample(self.sample_rate);
        let release_gain = release_gain_per_sample(self.sample_rate);

        let mut sample = 0.0_f32;
        for voice in &mut self.voices {
            sample += voice.phase.sin() * voice.amp * voice.attack * voice.release * OUTPUT_GAIN;
            voice.phase = (voice.phase + TAU * voice.freq / self.sample_rate) % TAU;
            if voice.attack < 1.0 {
                voice.attack = (voice.attack + attack_step).min(1.0);
            }
            if voice.releasing {
                voice.release *= release_gain;
            }
        }
        sample
    }
}

fn release_gain_per_sample(sample_rate: f32) -> f32 {
    let samples = (RELEASE_TIME_SECONDS * sample_rate).max(1.0);
    0.001_f32.powf(1.0 / samples)
}

fn attack_step_per_sample(sample_rate: f32) -> f32 {
    1.0 / (ATTACK_TIME_SECONDS * sample_rate).max(1.0)
}

use serde::{Deserialize, Serialize};

use crate::{
    audio::block::AudioBlock,
    engine::effect::{self, EffectError, EffectRequest, EffectResult, ParamValue},
};

pub struct Gain {
    value: f32,
}

impl Gain {
    pub fn request(&mut self, request: EffectRequest) -> EffectResult {
        match request {
            EffectRequest::SetParam { id, value } => self.set_param(id, value),
        }
    }

    pub fn process(&mut self, output: &mut AudioBlock) {
        let channel_count = output.channel_count();
        let frame_count = output.frame_count();

        for channel_samples in output.samples_mut().iter_mut().take(channel_count) {
            for frame in channel_samples.iter_mut().take(frame_count) {
                *frame *= self.value;
            }
        }
    }

    fn set_param(&mut self, id: u8, value: ParamValue) -> EffectResult {
        if id == 0 {
            self.value = effect::expect_f32(value)?;
            return Ok(());
        }

        Err(EffectError::InvalidParamId)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GainDoc {
    value: f32,
}

impl GainDoc {
    pub fn set_param(&mut self, id: u8, value: ParamValue) {
        if id == 0
            && let Ok(value) = effect::expect_f32(value)
        {
            self.value = value;
        }
    }

    pub fn prepare_effect(&self) -> Gain {
        Gain { value: self.value }
    }
}

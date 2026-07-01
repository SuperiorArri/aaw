use serde::{Deserialize, Serialize};

use crate::{audio::block::AudioBlock, effects, midi::MidiEvent};

#[derive(Debug, Clone)]
pub enum EffectRequest {
    SetParam { id: u8, value: ParamValue },
}

#[derive(Debug, Clone)]
pub enum ParamValue {
    Bool(bool),
    Float32(f32),
    VecFloat32(Vec<f32>),
    Usize(usize),
}

#[derive(Debug, thiserror::Error)]
pub enum EffectError {
    #[error("invalid parameter id")]
    InvalidParamId,

    #[error("invalid parameter value")]
    InvalidParamValue,

    #[error("invalid value type")]
    InvalidValueType,
}

pub type EffectResult = Result<(), EffectError>;

pub enum Effect {
    Gain(effects::Gain),
}

impl Effect {
    pub fn prepare(&mut self, _sample_rate: u32, _max_frame_count: usize, _channel_count: usize) {
        match self {
            Effect::Gain(_) => {}
        }
    }

    pub fn request(&mut self, request: EffectRequest) -> EffectResult {
        match self {
            Effect::Gain(effect) => effect.request(request),
        }
    }

    pub fn handle_midi(&mut self, _event: MidiEvent) {
        match self {
            Effect::Gain(_) => {}
        }
    }

    pub fn process(&mut self, output: &mut AudioBlock) {
        match self {
            Effect::Gain(effect) => effect.process(output),
        }
    }

    pub fn panic(&mut self) {
        match self {
            Effect::Gain(_) => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectDoc {
    Gain(effects::GainDoc),
}

impl EffectDoc {
    pub fn prepare_effect(&self) -> Effect {
        match self {
            EffectDoc::Gain(doc) => Effect::Gain(doc.prepare_effect()),
        }
    }
}

pub fn expect_bool(value: ParamValue) -> Result<bool, EffectError> {
    if let ParamValue::Bool(value) = value {
        Ok(value)
    } else {
        Err(EffectError::InvalidValueType)
    }
}

pub fn expect_f32(value: ParamValue) -> Result<f32, EffectError> {
    if let ParamValue::Float32(value) = value {
        Ok(value)
    } else {
        Err(EffectError::InvalidValueType)
    }
}

pub fn expect_vec_f32(value: ParamValue) -> Result<Vec<f32>, EffectError> {
    if let ParamValue::VecFloat32(value) = value {
        Ok(value)
    } else {
        Err(EffectError::InvalidValueType)
    }
}

pub fn expect_usize(value: ParamValue) -> Result<usize, EffectError> {
    if let ParamValue::Usize(value) = value {
        Ok(value)
    } else {
        Err(EffectError::InvalidValueType)
    }
}

use serde::{Deserialize, Serialize};

use crate::midi::event::{MidiEvent, MidiEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KeyMask {
    pub mask: u128,
}

impl KeyMask {
    pub const ALL: Self = Self { mask: u128::MAX };

    pub fn contains(&self, key: u8) -> bool {
        self.mask & (1 << key) > 0
    }
}

impl Default for KeyMask {
    fn default() -> Self {
        Self::ALL
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ChannelMask {
    pub mask: u16,
}

impl ChannelMask {
    pub const ALL: Self = Self { mask: u16::MAX };

    pub fn contains(&self, key: u8) -> bool {
        self.mask & (1 << key) > 0
    }
}

impl Default for ChannelMask {
    fn default() -> Self {
        Self::ALL
    }
}

pub struct MidiFilter {
    pub key_mask: KeyMask,
    pub channel_mask: ChannelMask,
}

impl MidiFilter {
    pub fn filter(&self, event: MidiEvent) -> Option<MidiEvent> {
        if !self.channel_mask.contains(event.channel) {
            return None;
        }

        if let MidiEventKind::NoteOn { key, velocity: _ } = event.kind
            && !self.key_mask.contains(key)
        {
            return None;
        }

        if let MidiEventKind::NoteOff { key, velocity: _ } = event.kind
            && !self.key_mask.contains(key)
        {
            return None;
        }

        Some(event)
    }
}

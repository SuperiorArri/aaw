use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MidiEvent {
    pub timestamp_samples: Option<u64>,
    pub channel: u8,
    pub kind: MidiEventKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MidiEventKind {
    NoteOn { key: u8, velocity: u8 },
    NoteOff { key: u8, velocity: u8 },
    ControlChange { controller: u8, value: u8 },
    PitchBend { value: i16 },
    ProgramChange { program: u8 },
    Aftertouch { value: u8 },
}

pub fn parse_midi_message(message: &[u8]) -> Option<MidiEvent> {
    let status = *message.first()?;
    let channel = status & 0x0f;
    let kind = match status & 0xf0 {
        0x80 if message.len() >= 3 => MidiEventKind::NoteOff {
            key: message[1],
            velocity: message[2],
        },
        0x90 if message.len() >= 3 && message[2] == 0 => MidiEventKind::NoteOff {
            key: message[1],
            velocity: 0,
        },
        0x90 if message.len() >= 3 => MidiEventKind::NoteOn {
            key: message[1],
            velocity: message[2],
        },
        0xb0 if message.len() >= 3 => MidiEventKind::ControlChange {
            controller: message[1],
            value: message[2],
        },
        0xc0 if message.len() >= 2 => MidiEventKind::ProgramChange {
            program: message[1],
        },
        0xd0 if message.len() >= 2 => MidiEventKind::Aftertouch { value: message[1] },
        0xe0 if message.len() >= 3 => {
            let raw = ((message[2] as i16) << 7) | message[1] as i16;
            MidiEventKind::PitchBend { value: raw - 8192 }
        }
        _ => return None,
    };

    Some(MidiEvent {
        timestamp_samples: None,
        channel,
        kind,
    })
}

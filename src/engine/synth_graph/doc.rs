use crate::midi::ChannelMask;
use serde::{Deserialize, Serialize};

pub enum Command {
    Append,
    Remove { slot_id: u8 },
    SetSource { slot_id: u8, source_id: Option<u8> },
    SetEnabled { slot_id: u8, flag: bool },
    SetChannelMask { slot_id: u8, mask: ChannelMask },
    SetTransposition { slot_id: u8, transposition: i8 },
    SetUseGlobalTransposition { slot_id: u8, flag: bool },
    SetName { slot_id: u8, name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    pub source_id: Option<u8>,
    pub enabled: bool,
    pub channel_mask: ChannelMask,
    pub transposition: i8,
    pub use_global_transposition: bool,
    pub name: String,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            source_id: None,
            enabled: true,
            channel_mask: ChannelMask::ALL,
            transposition: 0,
            use_global_transposition: true,
            name: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub slots: Vec<Slot>,
}

impl Graph {
    pub fn process_command(&mut self, cmd: Command) {
        match cmd {
            Command::Append => {
                if self.slots.len() < self.slots.capacity() {
                    self.slots.push(Slot::default());
                }
            }
            Command::Remove { slot_id } => {
                if (slot_id as usize) < self.slots.len() {
                    self.slots.remove(slot_id as usize);
                }
            }
            Command::SetSource { slot_id, source_id } => {
                if (slot_id as usize) < self.slots.len() {
                    self.slots[slot_id as usize].source_id = source_id;
                }
            }
            Command::SetEnabled { slot_id, flag } => {
                if (slot_id as usize) < self.slots.len() {
                    self.slots[slot_id as usize].enabled = flag;
                }
            }
            Command::SetChannelMask { slot_id, mask } => {
                if (slot_id as usize) < self.slots.len() {
                    self.slots[slot_id as usize].channel_mask = mask;
                }
            }
            Command::SetTransposition {
                slot_id,
                transposition,
            } => {
                if (slot_id as usize) < self.slots.len() {
                    self.slots[slot_id as usize].transposition = transposition;
                }
            }
            Command::SetUseGlobalTransposition { slot_id, flag } => {
                if (slot_id as usize) < self.slots.len() {
                    self.slots[slot_id as usize].use_global_transposition = flag;
                }
            }
            Command::SetName { slot_id, name } => {
                if (slot_id as usize) < self.slots.len() {
                    self.slots[slot_id as usize].name = name;
                }
            }
        }
    }
}

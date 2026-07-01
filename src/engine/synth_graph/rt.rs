use crate::{
    midi::{ChannelMask, MidiEvent, MidiEventKind},
    rt_channels,
};
use tracing::error;

pub enum Command {
    SetGlobalTransposition { transposition: i8 },
    RemoveSource { source_id: u8 },
    PushSlot,
    RemoveSlot { slot_id: u8 },
    SetSource { slot_id: u8, source_id: Option<u8> },
    SetEnabled { slot_id: u8, flag: bool },
    SetChannelMask { slot_id: u8, mask: ChannelMask },
    SetTransposition { slot_id: u8, transposition: i8 },
    SetUseGlobalTransposition { slot_id: u8, flag: bool },
}

pub struct Slot {
    pub source_id: Option<u8>,
    pub enabled: bool,
    pub channel_mask: ChannelMask,
    pub transposition: i8,
    pub use_global_transposition: bool,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            source_id: None,
            enabled: true,
            channel_mask: ChannelMask::ALL,
            transposition: 0,
            use_global_transposition: true,
        }
    }
}

pub struct MidiIter<'a> {
    graph: &'a Graph,
    next_id: u8,
    event: MidiEvent,
}

impl<'a> MidiIter<'a> {
    fn new(graph: &'a Graph, event: MidiEvent) -> Self {
        Self {
            graph,
            next_id: 0,
            event,
        }
    }
}

impl<'a> Iterator for MidiIter<'a> {
    type Item = (u8, MidiEvent);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_id = self.next_id as usize;
            self.next_id += 1;

            if next_id >= self.graph.slots.len() {
                return None;
            }

            let slot = &self.graph.slots[next_id];
            let source_id = slot.source_id?;

            if !slot.channel_mask.contains(self.event.channel) {
                continue;
            }

            if !slot.enabled
                && let MidiEventKind::NoteOn {
                    key: _,
                    velocity: _,
                } = self.event.kind
            {
                continue;
            }

            let Some(event) = self.event.transpose(slot.transposition) else {
                continue;
            };

            let Some(event) = (if slot.use_global_transposition {
                event.transpose(self.graph.global_transposition)
            } else {
                Some(event)
            }) else {
                continue;
            };

            return Some((source_id, event));
        }
    }
}

pub struct AudioIter<'a> {
    graph: &'a Graph,
    next_id: u8,
}

impl<'a> AudioIter<'a> {
    fn new(graph: &'a Graph) -> Self {
        Self { graph, next_id: 0 }
    }
}

impl<'a> Iterator for AudioIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_id = self.next_id as usize;
            self.next_id += 1;

            if next_id >= self.graph.slots.len() {
                return None;
            }

            let slot = &self.graph.slots[next_id];
            let source_id = slot.source_id?;

            if !slot.enabled {
                continue;
            }

            return Some(source_id);
        }
    }
}

pub struct Graph {
    pub cmd_rx: rt_channels::RtConsumer<(u32, Command)>,
    pub ack_tx: rt_channels::RtProducer<(u32, bool)>,
    pub slots: Vec<Slot>,
    pub global_transposition: i8,
}

impl Graph {
    pub fn drain_commands(&mut self) {
        while let Ok((id, cmd)) = self.cmd_rx.pop() {
            let res = self.process_command(cmd);
            if self.ack_tx.push((id, res)).is_err() {
                error!("failed to ack cmd #{id}")
            }
        }
    }

    pub fn iter_midi_event(&'_ self, event: MidiEvent) -> MidiIter<'_> {
        MidiIter::new(self, event)
    }

    pub fn iter_audio(&'_ self) -> AudioIter<'_> {
        AudioIter::new(self)
    }

    fn process_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::SetGlobalTransposition { transposition } => {
                self.global_transposition = transposition;
                true
            }
            Command::RemoveSource { source_id } => {
                for slot in &mut self.slots {
                    if let Some(slot_source_id) = slot.source_id {
                        if slot_source_id == source_id {
                            slot.source_id = None;
                        } else if slot_source_id > source_id {
                            slot.source_id = Some(slot_source_id - 1);
                        }
                    }
                }
                true
            }
            Command::PushSlot => {
                if self.slots.len() == self.slots.capacity() {
                    return false;
                }
                self.slots.push(Slot::default());
                true
            }
            Command::RemoveSlot { slot_id } => {
                if slot_id as usize >= self.slots.len() {
                    return false;
                }
                self.slots.remove(slot_id as usize);
                true
            }
            Command::SetSource { slot_id, source_id } => {
                if slot_id as usize >= self.slots.len() {
                    return false;
                }
                self.slots[slot_id as usize].source_id = source_id;
                true
            }
            Command::SetEnabled { slot_id, flag } => {
                if slot_id as usize >= self.slots.len() {
                    return false;
                }
                self.slots[slot_id as usize].enabled = flag;
                true
            }
            Command::SetChannelMask { slot_id, mask } => {
                if slot_id as usize >= self.slots.len() {
                    return false;
                }
                self.slots[slot_id as usize].channel_mask = mask;
                true
            }
            Command::SetTransposition {
                slot_id,
                transposition,
            } => {
                if slot_id as usize >= self.slots.len() {
                    return false;
                }
                self.slots[slot_id as usize].transposition = transposition;
                true
            }
            Command::SetUseGlobalTransposition { slot_id, flag } => {
                if slot_id as usize >= self.slots.len() {
                    return false;
                }
                self.slots[slot_id as usize].use_global_transposition = flag;
                true
            }
        }
    }
}

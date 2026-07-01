use crate::{audio::block::AudioBlock, midi::MidiEvent, rt_channels, synths};
use tracing::error;

pub enum Command {
    Push(synths::Dummy),
    Remove(u8),
}

pub struct Graph {
    pub cmd_rx: rt_channels::RtConsumer<(u32, Command)>,
    pub ack_tx: rt_channels::RtProducer<(u32, bool)>,
    pub retire_tx: rt_channels::RtProducer<synths::Dummy>,
    pub synths: Vec<synths::Dummy>,
}

impl Graph {
    pub fn process_incoming_commands(&mut self) {
        while let Ok((id, cmd)) = self.cmd_rx.pop() {
            let res = self.process_command(cmd);
            if self.ack_tx.push((id, res)).is_err() {
                error!("failed to ack cmd #{id}")
            }
        }
    }

    pub fn handle_midi_event(&mut self, id: u8, event: MidiEvent) {
        if let Some(synth) = self.synths.get_mut(id as usize) {
            synth.handle_midi_event(event);
        }
    }

    pub fn process(&mut self, id: u8, output: &mut AudioBlock) {
        if let Some(synth) = self.synths.get_mut(id as usize) {
            synth.process(output);
        }
    }

    fn process_command(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::Push(synth) => {
                if self.synths.len() == self.synths.capacity() {
                    retire_synth(&mut self.retire_tx, synth);
                    return false;
                }
                self.synths.push(synth);
                true
            }
            Command::Remove(id) => {
                if id as usize >= self.synths.len() {
                    return false;
                }
                let synth = self.synths.remove(id as usize);
                retire_synth(&mut self.retire_tx, synth);
                true
            }
        }
    }
}

fn retire_synth(tx: &mut rt_channels::RtProducer<synths::Dummy>, synth: synths::Dummy) {
    if tx.push(synth).is_err() {
        error!(concat!(
            "failed to retire synth: retirement queue full;",
            " synth deallocated in the audio thread"
        ));
    }
}

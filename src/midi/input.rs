use crate::{midi::event::MidiEvent, rt_channels::RtSharedProducer};
use midir::{
    ConnectError, ConnectErrorKind, InitError, MidiInput, MidiInputConnection, PortInfoError,
};
use std::collections::HashMap;
use thiserror::Error;
use tracing::warn;

#[derive(Debug, Error)]
pub enum InputSlotError {
    #[error("slot already disconnected")]
    AlreadyDisconnected,

    #[error("slot already connected")]
    AlreadyConnected,

    #[error("slot not connected")]
    NotConnected,

    #[error("slot not disconnected")]
    NotDisconnected,

    #[error("invalid slot id: {0}")]
    InvalidId(usize),

    #[error("failed to connect input: {0}")]
    Connect(ConnectErrorKind),

    #[error("failed to get MIDI port information: {0}")]
    PortInfo(#[from] PortInfoError),
}

pub enum InputSlot {
    Connected(ConnectedInputSlot),
    Disconnected(DisconnectedInputSlot),
}

pub struct ConnectedInputSlot {
    inner: MidiInputConnection<()>,
    tx: RtSharedProducer<MidiEvent>,
    name: String,
    port_id: String,
}

impl ConnectedInputSlot {
    pub fn disconnect(self) -> DisconnectedInputSlot {
        DisconnectedInputSlot {
            inner: self.inner.close().0,
            tx: self.tx,
            name: self.name,
        }
    }

    pub fn port_id(&self) -> &String {
        &self.port_id
    }
}

pub struct DisconnectedInputSlot {
    inner: MidiInput,
    tx: RtSharedProducer<MidiEvent>,
    name: String,
}

impl DisconnectedInputSlot {
    pub fn new(
        name: impl Into<String>,
        tx: RtSharedProducer<MidiEvent>,
    ) -> Result<Self, InitError> {
        let name = name.into();

        Ok(Self {
            inner: MidiInput::new(&name)?,
            tx,
            name,
        })
    }

    pub fn connect(
        self,
        port_id: &str,
    ) -> Result<ConnectedInputSlot, ConnectError<DisconnectedInputSlot>> {
        let ports = self.inner.ports();
        let Some(port) = ports.iter().find(|&p| p.id() == port_id) else {
            return Err(ConnectError::new(ConnectErrorKind::InvalidPort, self));
        };

        let tx = self.tx.clone();

        match self.inner.connect(
            port,
            &self.name,
            move |_, msg, _| {
                if let Some(event) = super::event::parse_midi_message(msg)
                    && let Err(event) = tx.push(event)
                {
                    warn!(
                        target = "midi-runtime",
                        "failed to push midi event: {event:#?}"
                    );
                }
            },
            (),
        ) {
            Ok(con) => Ok(ConnectedInputSlot {
                inner: con,
                tx: self.tx,
                name: self.name,
                port_id: port_id.into(),
            }),
            Err(err) => {
                let kind = err.kind();
                let inner = err.into_inner();
                Err(ConnectError::new(
                    kind,
                    DisconnectedInputSlot {
                        inner,
                        tx: self.tx,
                        name: self.name,
                    },
                ))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputPortInfo {
    pub id: String,
    pub name: String,
}

pub struct MidiRuntime {
    inputs: Vec<InputSlot>,
    port_discovery: MidiInput,
    available_ports: Vec<InputPortInfo>,
}

impl MidiRuntime {
    pub fn new(
        input_count: usize,
        client_name: impl Into<String>,
        tx: RtSharedProducer<MidiEvent>,
    ) -> Result<Self, InitError> {
        let client_name = client_name.into();

        let inputs = (0..input_count)
            .map(|i| {
                DisconnectedInputSlot::new(format!("{client_name}-{i}"), tx.clone())
                    .map(InputSlot::Disconnected)
            })
            .collect::<Result<_, _>>()?;

        let port_discovery = MidiInput::new(&format!("{client_name}-port-discovery"))?;
        let available_ports = Default::default();

        Ok(Self {
            inputs,
            port_discovery,
            available_ports,
        })
    }

    pub fn connect_input(&mut self, slot_id: usize, port_id: &str) -> Result<(), InputSlotError> {
        if slot_id >= self.inputs.len() {
            return Err(InputSlotError::InvalidId(slot_id));
        }

        let slot = self.inputs.remove(slot_id);

        let (slot, result) = match slot {
            InputSlot::Connected(slot) => (
                InputSlot::Connected(slot),
                Err(InputSlotError::AlreadyConnected),
            ),
            InputSlot::Disconnected(slot) => match slot.connect(port_id) {
                Ok(connected) => (InputSlot::Connected(connected), Ok(())),

                Err(error) => {
                    let kind = error.kind();
                    let disconnected = error.into_inner();

                    (
                        InputSlot::Disconnected(disconnected),
                        Err(InputSlotError::Connect(kind)),
                    )
                }
            },
        };

        self.inputs.insert(slot_id, slot);

        result
    }

    pub fn disconnect_input(&mut self, slot_id: usize) -> Result<(), InputSlotError> {
        if slot_id >= self.inputs.len() {
            return Err(InputSlotError::InvalidId(slot_id));
        }

        let slot = self.inputs.remove(slot_id);

        let (slot, result) = match slot {
            InputSlot::Connected(slot) => (InputSlot::Disconnected(slot.disconnect()), Ok(())),
            InputSlot::Disconnected(slot) => (
                InputSlot::Disconnected(slot),
                Err(InputSlotError::AlreadyDisconnected),
            ),
        };

        self.inputs.insert(slot_id, slot);

        result
    }

    pub fn refresh_input_ports(&mut self) -> Result<(), PortInfoError> {
        let mut current = HashMap::new();

        for port in self.port_discovery.ports() {
            let id = port.id();
            let name = self.port_discovery.port_name(&port)?;
            current.insert(id.clone(), InputPortInfo { id, name });
        }

        self.available_ports =self.port_discovery.ports().into_iter()
            .map(|port| {
                let id = port.id();
                let name = self.port_discovery.port_name(&port)?;
                Ok(InputPortInfo { id, name })
            })
            .collect::<Result<_, _>>()?;

        Ok(())
    }

    pub fn available_input_ports(&self) -> &Vec<InputPortInfo> {
        &self.available_ports
    }
}

use crate::{midi::event::MidiEvent, rt_channels::RtSharedProducer};
use midir::{
    ConnectError, ConnectErrorKind, InitError, MidiInput, MidiInputConnection, PortInfoError,
};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, warn};

const LOG_TARGET: &str = "midi-runtime";

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
    Empty,
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
                    && let Err(event) = tx.blocking_push(event)
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

pub struct AutoConnectInput {
    pub slot_id: usize,
    pub slot_id_fixed: bool,
}

struct AutoConnectInputState {
    preferred_slot_id: usize,
    slot_id_fixed: bool,
}

impl From<AutoConnectInput> for AutoConnectInputState {
    fn from(value: AutoConnectInput) -> Self {
        Self {
            preferred_slot_id: value.slot_id,
            slot_id_fixed: value.slot_id_fixed,
        }
    }
}

pub struct MidiRuntime {
    inputs: Vec<InputSlot>,
    port_discovery: MidiInput,
    available_ports: Vec<InputPortInfo>,
    auto_connect_inputs: HashMap<String, AutoConnectInputState>,
}

impl MidiRuntime {
    pub fn new(
        input_count: usize,
        client_name: impl Into<String>,
        tx: RtSharedProducer<MidiEvent>,
        auto_connect_inputs: HashMap<String, AutoConnectInput>,
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
        let auto_connect_inputs = auto_connect_inputs
            .into_iter()
            .map(|(i, x)| (i, x.into()))
            .collect();

        Ok(Self {
            inputs,
            port_discovery,
            available_ports,
            auto_connect_inputs,
        })
    }

    pub fn connect_input(&mut self, slot_id: usize, port_id: &str) -> Result<(), InputSlotError> {
        let slot = self
            .inputs
            .get_mut(slot_id)
            .ok_or(InputSlotError::InvalidId(slot_id))?;
        connect_input_slot(slot, port_id)
    }

    pub fn disconnect_input(&mut self, slot_id: usize) -> Result<(), InputSlotError> {
        let slot = self
            .inputs
            .get_mut(slot_id)
            .ok_or(InputSlotError::InvalidId(slot_id))?;
        disconnect_input_slot(slot)
    }

    pub fn port_id(&self, slot_id: usize) -> Result<&str, InputSlotError> {
        let slot = self
            .inputs
            .get(slot_id)
            .ok_or(InputSlotError::InvalidId(slot_id))?;

        match slot {
            InputSlot::Connected(slot) => Ok(&slot.port_id),
            InputSlot::Disconnected(_) => Err(InputSlotError::NotConnected),
            InputSlot::Empty => unreachable!("temporary empty input slot leaked"),
        }
    }

    pub fn refresh_input_ports(&mut self) -> Result<(), PortInfoError> {
        let mut current = HashMap::new();

        for port in self.port_discovery.ports() {
            let id = port.id();
            let name = self.port_discovery.port_name(&port)?;
            current.insert(id.clone(), InputPortInfo { id, name });
        }

        self.available_ports = self
            .port_discovery
            .ports()
            .into_iter()
            .map(|port| {
                let id = port.id();
                let name = self.port_discovery.port_name(&port)?;
                Ok(InputPortInfo { id, name })
            })
            .collect::<Result<_, _>>()?;

        Ok(())
    }

    pub fn update_input_connections(&mut self) -> Result<(), InputSlotError> {
        self.disconnect_non_existent_inputs()?;
        self.try_autoconnect_inputs()?;
        Ok(())
    }

    pub fn available_input_ports(&self) -> &Vec<InputPortInfo> {
        &self.available_ports
    }

    fn disconnect_non_existent_inputs(&mut self) -> Result<(), InputSlotError> {
        for slot_id in 0..self.inputs.len() {
            if let Ok(port_id) = self.port_id(slot_id)
                && !is_input_port_available(&self.available_ports, port_id)
            {
                info!(
                    target = LOG_TARGET,
                    "auto disconnecting midi input: {} from slot {}", port_id, slot_id
                );
            } else {
                continue;
            }
            self.disconnect_input(slot_id)?;
        }
        Ok(())
    }

    fn try_autoconnect_inputs(&mut self) -> Result<(), InputSlotError> {
        for (port_id, state) in &mut self.auto_connect_inputs {
            if input_port_connected(&self.inputs, port_id)
                || !is_input_port_available(&self.available_ports, port_id)
            {
                continue;
            }

            let Some((slot_id, slot)) = get_new_slot_mut(&mut self.inputs, state) else {
                info!(
                    target = LOG_TARGET,
                    "no available slots for auto-connect of port: {}", port_id
                );
                continue;
            };

            connect_input_slot(slot, port_id)?;
            state.preferred_slot_id = slot_id;
            info!(
                target = LOG_TARGET,
                "auto connected midi input: {} to slot {}", port_id, slot_id
            );
        }

        Ok(())
    }
}

fn is_input_port_available(available_ports: &[InputPortInfo], port_id: &str) -> bool {
    available_ports.iter().find(|p| p.id == port_id).is_some()
}

fn find_first_disconnected_input_slot_mut(
    inputs: &mut [InputSlot],
) -> Option<(usize, &mut InputSlot)> {
    for (slot_id, slot) in inputs.iter_mut().enumerate() {
        if let InputSlot::Disconnected(_) = slot {
            return Some((slot_id, slot));
        }
    }
    None
}

fn input_port_connected(inputs: &[InputSlot], port_id: &str) -> bool {
    inputs
        .iter()
        .any(|slot| matches!(slot, InputSlot::Connected(slot) if slot.port_id() == port_id))
}

fn input_slot_connected(inputs: &[InputSlot], slot_id: usize) -> bool {
    inputs
        .iter()
        .enumerate()
        .any(|(id, slot)| id == slot_id && matches!(slot, InputSlot::Connected(_)))
}

fn get_new_slot_mut<'a>(
    inputs: &'a mut [InputSlot],
    state: &AutoConnectInputState,
) -> Option<(usize, &'a mut InputSlot)> {
    if input_slot_connected(inputs, state.preferred_slot_id) {
        if state.slot_id_fixed {
            return None;
        }

        let search_result = find_first_disconnected_input_slot_mut(inputs);
        let (slot_id, slot) = search_result?;
        return Some((slot_id, slot));
    }

    let slot = inputs.get_mut(state.preferred_slot_id)?;
    Some((state.preferred_slot_id, slot))
}

fn connect_input_slot(slot: &mut InputSlot, port_id: &str) -> Result<(), InputSlotError> {
    let current = std::mem::replace(slot, InputSlot::Empty);

    let (next, result) = match current {
        InputSlot::Connected(slot) => (
            InputSlot::Connected(slot),
            Err(InputSlotError::AlreadyConnected),
        ),

        InputSlot::Disconnected(slot) => match slot.connect(port_id) {
            Ok(slot) => (InputSlot::Connected(slot), Ok(())),
            Err(error) => {
                let kind = error.kind();
                let disconnected = error.into_inner();

                (
                    InputSlot::Disconnected(disconnected),
                    Err(InputSlotError::Connect(kind)),
                )
            }
        },

        InputSlot::Empty => unreachable!("temporary empty input slot leaked"),
    };

    *slot = next;
    result
}

fn disconnect_input_slot(slot: &mut InputSlot) -> Result<(), InputSlotError> {
    let current = std::mem::replace(slot, InputSlot::Empty);

    let (next, result) = match current {
        InputSlot::Connected(slot) => (InputSlot::Disconnected(slot.disconnect()), Ok(())),

        InputSlot::Disconnected(slot) => (
            InputSlot::Disconnected(slot),
            Err(InputSlotError::AlreadyDisconnected),
        ),

        InputSlot::Empty => unreachable!("temporary empty input slot leaked"),
    };

    *slot = next;
    result
}

use aaw::{
    midi::{event::MidiEventKind, runtime::MidiRuntime},
    rt_channels,
};
use std::{thread, time::Duration};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aaw=info,tower_http=info".into()),
        )
        .init();

    info!("** Arri's Audio Workstation **");

    let (tx, mut rx) = rt_channels::create_mpsc(256);
    let mut midi_runtime = MidiRuntime::new(16, "aaw", tx)?;
    midi_runtime.refresh_input_ports()?;

    for (i, info) in midi_runtime.available_input_ports().iter().enumerate() {
        info!("[{i}] {info:?}");
    }

    midi_runtime.connect_input(0, "129:0")?;

    loop {
        if let Ok(event) = rx.pop() {
            info!("[event] {event:?}");

            if let MidiEventKind::NoteOn { key, velocity: _ } = event.kind
                && key < 64
            {
                break;
            }
        } else {
            thread::sleep(Duration::from_millis(100));
        }
    }

    return Ok(());
}

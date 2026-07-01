use aaw::{
    audio::{self, params::DeviceCfgSelection},
    engine::Engine,
    midi::{AutoConnectInput, ControlChange, MidiEventKind, MidiRuntime},
    rt_channels,
};
use std::{collections::HashMap, sync::Arc, thread, time::Duration};
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("AAW_LOG")
                .unwrap_or_else(|_| "aaw=info,midi-runtime=info,tower_http=info".into()),
        )
        .init();

    info!("** Arri's Audio Workstation **");

    let (tx, mut rx) = rt_channels::create_mpsc(256);
    let auto_connect_inputs = HashMap::from([
        (
            "129:0".into(),
            AutoConnectInput {
                slot_id: 0,
                slot_id_fixed: false,
            },
        ),
        (
            "148:0".into(),
            AutoConnectInput {
                slot_id: 0,
                slot_id_fixed: false,
            },
        ),
    ]);
    let mut midi_runtime = MidiRuntime::new(16, "aaw", tx, auto_connect_inputs)?;
    midi_runtime.refresh_input_ports()?;

    println!("--- MIDI Info ---");
    for (i, info) in midi_runtime.available_input_ports().iter().enumerate() {
        println!("[{i}] {info:?}");
    }

    println!("===================");
    println!("--- Audio Hosts ---");
    for (i, host) in audio::info::list_hosts().iter().enumerate() {
        println!("[{i}] {}: {}", host.id, host.name);
    }

    println!("=====================");
    println!("--- Audio Outputs ---");
    for (i, info) in audio::info::list_output_devices("pulseaudio")?
        .iter()
        .enumerate()
    {
        println!("[{i:>2}] {} {}", info.host_id, info.id);
    }

    let device_config = audio::output::ResolvedConfig::find(&DeviceCfgSelection {
        host: audio::params::HostSelection::Id("pulseaudio".into()),
        device: audio::params::DeviceSelection::Id(
            "alsa_output.usb-HP__Inc_HyperX_Cloud_Alpha_Wireless_00000001-00.analog-stereo".into(),
        ),
        latency: audio::params::LatencySelection::Low,
        sample_rate: audio::params::SampleRateSelection::Automatic,
    })?;

    println!("=====================");
    println!("--- Stream Info ---");
    info!("Sample format: {}", device_config.sample_format());
    let stream_cfg = device_config.config();
    info!("Buffer size: {:?}", stream_cfg.buffer_size);
    info!("Channel count: {}", stream_cfg.channels);
    info!("Sample rate: {} Hz", stream_cfg.sample_rate);

    let engine = Arc::new(std::sync::Mutex::new(Engine));
    let _stream = device_config.start_stream(Arc::clone(&engine));

    println!("=====================");

    spawn_midi_runtime_task(midi_runtime);

    loop {
        if let Ok(event) = rx.pop() {
            info!(target: "midi-events", "[event] {event:?}");

            if let MidiEventKind::ControlChange {
                controller,
                value: _,
            } = event.kind
                && controller == ControlChange::AllNotesOff as u8
            {
                info!("[midi panic]");
                break;
            }
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }

    return Ok(());
}

fn spawn_midi_runtime_task(mut midi_runtime: MidiRuntime) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = midi_runtime.refresh_input_ports() {
                error!("Error refreshing MIDI input ports: {}", err);
            } else if let Err(err) = midi_runtime.update_input_connections() {
                error!("Error updating MIDI input connections: {}", err);
            }
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    });
}

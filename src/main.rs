use aaw::{
    audio::{self, params::DeviceCfgSelection}, instruments::engine::Engine, midi::{event::MidiEventKind, runtime::MidiRuntime}, rt_channels
};
use std::{sync::Arc, thread, time::Duration};
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
        // println!("[{i}] {info:#?}");
        // println!("------------------------------")
        println!("[{i}] {} {}", info.host_id, info.id);
    }

    let device_config = audio::output::ResolvedConfig::find(&DeviceCfgSelection {
        host: audio::params::HostSelection::Id("pulseaudio".into()),
        device: audio::params::DeviceSelection::Id("alsa_output.usb-HP__Inc_HyperX_Cloud_Alpha_Wireless_00000001-00.analog-stereo".into()),
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

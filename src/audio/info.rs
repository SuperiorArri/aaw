use cpal::{
    SampleFormat,
    traits::{DeviceTrait, HostTrait},
};
use std::str::FromStr;

type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = StdResult<T, cpal::Error>;

#[derive(Debug, Clone)]
pub struct Host {
    pub id: String,
    pub name: &'static str,
}

#[derive(Debug, Clone)]
pub struct OutputDeviceInfo {
    pub host_id: String,
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub stream_configs: Vec<OutputStreamConfigInfo>,
}

#[derive(Debug, Clone)]
pub struct OutputStreamConfigInfo {
    pub sample_format: SampleFormat,
    pub channels: u16,
    pub sample_rate: (u32, u32),
    pub buffer_size: BufferSizeInfo,
}

#[derive(Debug, Clone)]
pub enum BufferSizeInfo {
    Range { min: u32, max: u32 },
    Unknown,
}

impl From<cpal::SupportedBufferSize> for BufferSizeInfo {
    fn from(value: cpal::SupportedBufferSize) -> Self {
        match value {
            cpal::SupportedBufferSize::Range { min, max } => Self::Range { min, max },
            cpal::SupportedBufferSize::Unknown => Self::Unknown,
        }
    }
}

pub fn list_hosts() -> Vec<Host> {
    cpal::available_hosts()
        .iter()
        .map(|h| Host {
            id: h.to_string(),
            name: h.name(),
        })
        .collect()
}

pub fn list_output_devices(host_id: &str) -> Result<Vec<OutputDeviceInfo>> {
    let host = host_by_id(host_id)?;

    let default_id = host
        .default_output_device()
        .and_then(|device| device_id(&device).ok());

    let cpal_devices = host.output_devices()?;
    let mut devices = Vec::new();

    for device in cpal_devices {
        let id = device_id(&device)?;
        let name = device_display_name(&device).unwrap_or_else(|_| id.clone());
        let is_default = default_id.as_deref() == Some(id.as_str());
        let stream_configs = list_output_stream_configs(&device)?;

        devices.push(OutputDeviceInfo {
            host_id: host_id.into(),
            id,
            name,
            is_default,
            stream_configs,
        });
    }

    Ok(devices)
}

pub fn default_output_device(host_id: &str) -> Result<Option<OutputDeviceInfo>> {
    let host = host_by_id(host_id)?;
    let host_id = host.id().to_string();

    let Some(device) = host.default_output_device() else {
        return Ok(None);
    };

    let id = device_id(&device)?;
    let name = device_display_name(&device)?;
    let stream_configs = list_output_stream_configs(&device)?;

    Ok(Some(OutputDeviceInfo {
        host_id: host_id.clone(),
        id,
        name,
        is_default: true,
        stream_configs,
    }))
}

pub fn list_output_stream_configs(device: &cpal::Device) -> Result<Vec<OutputStreamConfigInfo>> {
    let configs = device.supported_output_configs()?;
    let configs = configs
        .into_iter()
        .map(|config| OutputStreamConfigInfo {
            sample_format: config.sample_format(),
            channels: config.channels(),
            sample_rate: (config.min_sample_rate(), config.max_sample_rate()),
            buffer_size: (*config.buffer_size()).into(),
        })
        .collect();
    Ok(configs)
}

pub fn host_by_id(host_id: &str) -> Result<cpal::Host> {
    let cpal_host_id = cpal::HostId::from_str(host_id)?;
    cpal::host_from_id(cpal_host_id)
}

fn device_id(device: &cpal::Device) -> Result<String> {
    device.id().map(|id| id.id().into())
}

fn device_display_name(device: &cpal::Device) -> Result<String> {
    device
        .description()
        .map(|description| description.name().to_string())
}

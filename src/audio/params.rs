#[derive(Debug, Clone)]
pub enum HostSelection {
    Default,
    Id(String),
}

#[derive(Debug, Clone)]
pub enum DeviceSelection {
    Default,
    Id(String),
}

#[derive(Debug, Clone)]
pub enum LatencySelection {
    // buffer size:
    Automatic, //  device default, then 256
    Low,       //  64, 128, then 256
    Stable,    //  256, 512, then 1024
}

#[derive(Debug, Clone)]
pub enum SampleRateSelection {
    Automatic,
    Fixed(u32),
}

#[derive(Debug, Clone)]
pub struct DeviceCfgSelection {
    pub host: HostSelection,
    pub device: DeviceSelection,
    pub latency: LatencySelection,
    pub sample_rate: SampleRateSelection,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct OutputProfileSelection {
//     pub sample_format: String, // TODO: change to the actual format type
//     pub channels: u16,
//     pub sample_rate: u32,
// }

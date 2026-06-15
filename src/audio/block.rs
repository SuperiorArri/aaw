pub struct AudioBlock {
    channel_count: usize,
    max_frame_count: usize,
    frame_count: usize,
    samples: Vec<f32>,
}

impl AudioBlock {
    pub fn new(channel_count: usize, max_frame_count: usize) -> Self {
        Self {
            channel_count,
            max_frame_count,
            frame_count: 0,
            samples: vec![0.0; channel_count * max_frame_count],
        }
    }

    pub fn resize_for_callback(&mut self, frame_count: usize) -> bool {
        if frame_count > self.max_frame_count {
            self.frame_count = 0;
            return false;
        }
        self.frame_count = frame_count;
        true
    }

    pub fn clear(&mut self) {
        for channel in 0..self.channel_count {
            let start = channel * self.max_frame_count;
            self.samples[start..start + self.frame_count].fill(0.0);
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn channel_count(&self) -> usize {
        self.channel_count
    }

    pub fn channel_mut(&mut self, channel: usize) -> &mut [f32] {
        let start = channel * self.max_frame_count;
        &mut self.samples[start..start + self.frame_count]
    }

    pub fn add_sample(&mut self, channel: usize, frame: usize, value: f32) {
        let index = channel * self.max_frame_count + frame;
        self.samples[index] += value;
    }

    pub fn write_interleaved_f32(&self, output: &mut [f32]) {
        for frame in 0..self.frame_count {
            for channel in 0..self.channel_count {
                output[frame * self.channel_count + channel] =
                    self.samples[channel * self.max_frame_count + frame].clamp(-1.0, 1.0);
            }
        }
    }
}

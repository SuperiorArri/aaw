pub struct AudioBlock {
    channel_count: usize,
    max_frame_count: usize,
    frame_count: usize,
    samples: Vec<Vec<f32>>,
}

impl AudioBlock {
    pub fn new(channel_count: usize, max_frame_count: usize) -> Self {
        Self {
            channel_count,
            max_frame_count,
            frame_count: 0,
            samples: vec![vec![0.0; max_frame_count]; channel_count],
        }
    }

    pub fn mirror(&self) -> Self {
        Self::new(self.channel_count, self.max_frame_count)
    }

    pub fn prepare(&mut self, frame_count: usize) -> bool {
        if frame_count > self.max_frame_count {
            self.frame_count = 0;
            return false;
        }
        self.frame_count = frame_count;
        true
    }

    pub fn clear(&mut self) {
        for channel in 0..self.channel_count {
            self.samples[channel].fill(0.0);
        }
    }

    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    pub fn channel_count(&self) -> usize {
        self.channel_count
    }

    pub fn samples_mut(&mut self) -> &mut Vec<Vec<f32>> {
        &mut self.samples
    }

    pub fn add_samples_from_unchecked(&mut self, other: &AudioBlock) {
        for channel in 0..self.channel_count {
            let local_samples = &mut self.samples[channel];
            let other_samples = &other.samples[channel];

            for i in 0..self.frame_count {
                local_samples[i] += other_samples[i];
            }
        }
    }

    pub fn write_interleaved_f32(&self, output: &mut [f32]) {
        for frame in 0..self.frame_count {
            for channel in 0..self.channel_count {
                output[frame * self.channel_count + channel] =
                    self.samples[channel][frame].clamp(-1.0, 1.0);
            }
        }
    }
}

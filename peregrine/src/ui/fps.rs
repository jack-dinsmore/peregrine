use std::time::Instant;

const N_DATA: usize = 10;

pub struct FpsCounter {
    instant: Instant,
    rolling_summary: [f32; N_DATA],
    data_index: usize,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            instant: Instant::now(),
            rolling_summary: [f32::NAN; N_DATA],
            data_index: 0,
        }
    }

    pub fn update(&mut self) {
        let time = self.instant.elapsed().as_secs_f32();
        self.instant = Instant::now();
        self.rolling_summary[self.data_index] = time;
        self.data_index = (self.data_index + 1) % N_DATA;
    }

    pub fn invalidate(&mut self) {
        for item in &mut self.rolling_summary {
            *item = f32::NAN;
        }
        self.data_index = 0;

    }

    pub fn get_fps(&self) -> f32 {
        let mut sum = 0.;
        let mut n = 0;
        for item in &self.rolling_summary {
            if item.is_nan() {continue;}
            sum += item;
            n += 1;
        }
        n as f32 / sum
    }
}
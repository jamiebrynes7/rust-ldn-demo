use spatialos_sdk::worker::connection::{Connection, WorkerConnection};
use spatialos_sdk::worker::metrics::Metrics;
use std::time::{Duration, SystemTime};

pub struct FpsTracker {
    measurements: Vec<Duration>,
    max_measurements: usize,
    last: SystemTime,

    metrics: Metrics,
}

impl FpsTracker {
    pub fn new(max: usize) -> Self {
        FpsTracker {
            measurements: Vec::new(),
            max_measurements: max,
            last: SystemTime::now(),
            metrics: Metrics::new(),
        }
    }

    pub fn tick(&mut self, connection: &mut WorkerConnection) -> f64 {
        self.record();

        self.metrics.load = Some(self.get_fps());
        connection.send_metrics(&self.metrics);

        self.measurements.last().unwrap().as_micros() as f64 / 1000.0
    }

    fn record(&mut self) {
        let now = SystemTime::now();
        let diff = now.duration_since(self.last).expect("Error");
        self.measurements.push(diff);

        if self.measurements.len() > self.max_measurements {
            self.measurements.remove(0);
        }

        self.last = now;
    }

    fn get_fps(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }

        let sum = self
            .measurements
            .iter()
            .map(|duration| 1.0 / (f64::from(duration.subsec_micros()) / 1_000_000.0))
            .fold(0.0, |sum, next| sum + next);

        sum / self.measurements.len() as f64
    }
}

pub struct FpsLimiter {
    target_frame_ms: f64
}

impl FpsLimiter {
    pub fn new(target_framerate: f64) -> Self {
        FpsLimiter {
            target_frame_ms: 1000.0 / target_framerate
        }
    }

    pub fn tick(&self, frame_time_ms: f64) {
        let diff = self.target_frame_ms - frame_time_ms;

        if diff > 0.0 {
            ::std::thread::sleep(Duration::from_micros((diff * 1000.0).round() as i64 as u64));
        }
    }
}



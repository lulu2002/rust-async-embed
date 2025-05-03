use crate::app::ticker::TickDuration;
use fugit::ExtU64;

#[derive(Copy, Clone)]
pub struct Sound {
    freq_hz: u32,
    duration_ms: u32,
}

impl Sound {
    pub fn new(freq_hz: u32, duration_ms: u32) -> Self {
        Self {
            freq_hz,
            duration_ms,
        }
    }

    pub fn period(&self) -> u32 {
        let clk = 1_000_000;
        clk / self.freq_hz
    }

    pub fn duration(&self) -> TickDuration {
        (self.duration_ms as u64).millis()
    }
}

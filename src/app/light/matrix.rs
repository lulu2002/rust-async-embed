use crate::app::btn::types::ButtonDirection;
use crate::app::light::operator::LedOperator;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use microbit::gpio::NUM_COLS;
use microbit::hal::gpio::{Output, Pin, PushPull};
use rtt_target::rprintln;

pub struct LedMatrix {
    col: [Pin<Output<PushPull>>; NUM_COLS],
    active_col: usize,
}

impl LedMatrix {
    pub fn new(col: [Pin<Output<PushPull>>; 5]) -> Self {
        Self { col, active_col: 0 }
    }
}

impl LedOperator for LedMatrix {
    fn toggle(&mut self) {
        rprintln!("Blinking LED {}", self.active_col);

        #[cfg(feature = "trigger_overflow")]
        {
            let time = self.ticker.now();
            rprintln!(
                "time: 0x{:x} ticks, {} ms",
                time.ticks(),
                time.duration_since_epoch().to_millis()
            )
        }

        self.col[self.active_col].toggle().ok();
    }
    fn shift(&mut self, direction: ButtonDirection) {
        self.col[self.active_col].set_high().ok();
        self.active_col = match direction {
            ButtonDirection::Left => match self.active_col {
                0 => 4,
                _ => self.active_col - 1,
            },
            ButtonDirection::Right => match self.active_col {
                4 => 0,
                _ => self.active_col + 1,
            },
        };

        self.col[self.active_col].set_high().ok();
    }
}

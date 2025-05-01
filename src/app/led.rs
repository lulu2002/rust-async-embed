use crate::app::button::ButtonDirection;
use crate::app::channel::Receiver;
use crate::app::future::{OurFuture, Poll};
use crate::app::time::Timer;
use embedded_hal::digital::{OutputPin, StatefulOutputPin};
use fugit::ExtU64;
use microbit::{
    gpio::NUM_COLS,
    hal::gpio::{Output, Pin, PushPull},
};
use rtt_target::rprintln;
use crate::app::ticker::Ticker;

pub enum LedState<'a> {
    Toggle,
    Wait(Timer<'a>),
}

pub struct LedTask<'a> {
    col: [Pin<Output<PushPull>>; NUM_COLS],
    active_col: usize,
    ticker: &'a Ticker,
    state: LedState<'a>,
    receiver: Receiver<'a, ButtonDirection>,
}

impl<'a> OurFuture for LedTask<'a> {
    type Output = ();

    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        loop {
            match self.state {
                LedState::Toggle => {
                    self.toggle();
                    self.state = LedState::Wait(Timer::new(500.millis(), &self.ticker))
                }
                LedState::Wait(ref timer) => {
                    if timer.is_ready() {
                        self.state = LedState::Toggle;
                        continue;
                    }

                    if let Some(direction) = self.receiver.receive() {
                        self.shift(direction);
                        self.state = LedState::Toggle;
                        continue;
                    }
                    break;
                }
            }
        }
        Poll::Pending
    }
}

impl<'a> LedTask<'a> {
    pub fn new(
        col: [Pin<Output<PushPull>>; 5],
        ticker: &'a Ticker,
        receiver: Receiver<'a, ButtonDirection>,
    ) -> Self {
        Self {
            col,
            active_col: 0,
            ticker,
            state: LedState::Toggle,
            receiver,
        }
    }

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
        rprintln!("Button press detected..");
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

use crate::channel::Sender;
use crate::task::Task;
use crate::time::{Ticker, Timer};
use embedded_hal::digital::InputPin;
use fugit::ExtU64;
use microbit::hal::gpio::{Floating, Input, Pin};

#[derive(Copy, Clone)]
pub enum ButtonDirection {
    Left,
    Right,
}

pub enum ButtonState {
    WaitForPress,
    Debounce(Timer),
}

pub struct ButtonTask<'a> {
    pin: Pin<Input<Floating>>,
    ticker: &'a Ticker,
    direction: ButtonDirection,
    state: ButtonState,
    sender: Sender<'a, ButtonDirection>,
}

impl<'a> ButtonTask<'a> {
    pub fn new(
        pin: Pin<Input<Floating>>,
        ticker: &'a Ticker,
        direction: ButtonDirection,
        sender: Sender<'a, ButtonDirection>,
    ) -> Self {
        Self {
            pin,
            ticker,
            direction,
            state: ButtonState::WaitForPress,
            sender,
        }
    }
}

impl<'a> Task for ButtonTask<'a> {
    fn poll(&mut self) {
        match self.state {
            ButtonState::WaitForPress => {
                if self.pin.is_low().unwrap() {
                    self.sender.send(self.direction);
                    self.state = ButtonState::Debounce(Timer::new(100.millis(), &self.ticker))
                }
            }
            ButtonState::Debounce(ref timer) => {
                if timer.is_ready() && self.pin.is_high().unwrap() {
                    self.state = ButtonState::WaitForPress;
                }
            }
        }
    }
}

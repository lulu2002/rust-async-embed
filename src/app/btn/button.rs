use crate::app::btn::types::{ButtonDirection, ButtonState};
use crate::app::channel::Sender;
use crate::app::future::{OurFuture, Poll};
use crate::app::gpiote::InputChannel;
use crate::app::ticker::Ticker;
use crate::app::time::Timer;
use embedded_hal::digital::PinState;
use fugit::ExtU64;

pub struct ButtonTask<'a> {
    ticker: &'a Ticker,
    input: InputChannel,
    direction: ButtonDirection,
    state: ButtonState<'a>,
    sender: Sender<'a, ButtonDirection>,
}

impl<'a> ButtonTask<'a> {
    pub fn new(
        channel: InputChannel,
        ticker: &'a Ticker,
        direction: ButtonDirection,
        sender: Sender<'a, ButtonDirection>,
    ) -> Self {
        Self {
            input: channel,
            ticker,
            direction,
            state: ButtonState::WaitForPress,
            sender,
        }
    }
}

impl<'a> OurFuture for ButtonTask<'a> {
    type Output = ();

    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        loop {
            match self.state {
                ButtonState::WaitForPress => {
                    self.input.set_ready_state(PinState::Low);
                    if let Poll::Ready(_) = self.input.poll(task_id) {
                        self.sender.send(self.direction);
                        self.state = ButtonState::Debounce(Timer::new(100.millis(), &self.ticker));
                        continue;
                    }
                }
                ButtonState::Debounce(ref mut timer) => {
                    if let Poll::Ready(_) = timer.poll(task_id) {
                        self.state = ButtonState::WaitForRelease;
                        continue;
                    }
                }
                ButtonState::WaitForRelease => {
                    self.input.set_ready_state(PinState::High);
                    if let Poll::Ready(_) = self.input.poll(task_id) {
                        self.state = ButtonState::WaitForPress;
                        continue;
                    }
                }
            }
            break;
        }
        Poll::Pending
    }
}

use crate::app::button::ButtonDirection;
use crate::app::future::{OurFuture, Poll};
use crate::app::light::matrix::LedMatrix;
use crate::app::light::types::LedState;
use crate::app::ticker::Ticker;
use crate::app::time::Timer;
use fugit::ExtU64;

pub struct LedController<'a> {
    matrix: LedMatrix,
    ticker: &'a Ticker,
    state: LedState<'a>,
}

impl<'a> LedController<'a> {
    pub fn new(matrix: LedMatrix, ticker: &'a Ticker) -> Self {
        Self {
            matrix,
            ticker,
            state: LedState::Toggle,
        }
    }

    pub fn poll(&mut self, task_id: usize) -> Poll<()> {
        loop {
            match self.state {
                LedState::Toggle => {
                    self.matrix.toggle();
                    self.state = LedState::Wait(Timer::new(500.millis(), self.ticker));
                    continue;
                }
                LedState::Wait(ref mut timer) => {
                    if let Poll::Ready(_) = timer.poll(task_id) {
                        self.state = LedState::Toggle;
                        continue;
                    }
                    break;
                }
            }
        }
        Poll::Pending
    }

    pub fn shift(&mut self, dir: ButtonDirection) {
        self.matrix.shift(dir);
    }
}

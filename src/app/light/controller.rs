use crate::app::button::ButtonDirection;
use crate::app::future::{OurFuture, Poll};
use crate::app::light::operator::LedOperator;
use crate::app::light::types::LedState;
use crate::app::ticker::Ticker;
use crate::app::time::Timer;
use fugit::ExtU64;

pub struct LedController<'a> {
    operator: &'a mut dyn LedOperator,
    ticker: &'a Ticker,
    state: LedState<'a>,
}

impl<'a> LedController<'a> {
    pub fn new(operator: &'a mut dyn LedOperator, ticker: &'a Ticker) -> Self {
        Self {
            operator,
            ticker,
            state: LedState::Toggle,
        }
    }

    pub fn poll(&mut self, task_id: usize) -> Poll<()> {
        loop {
            match self.state {
                LedState::Toggle => {
                    self.operator.toggle();
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
        self.operator.shift(dir);
        self.state = LedState::Toggle;
    }
}

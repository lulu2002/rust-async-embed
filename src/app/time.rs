use crate::app::ticker::{TickDuration, TickInstant, Ticker};
use crate::executor::ExtWaker;
use crate::WAKEUP_MANAGER;
use core::pin::Pin;
use core::task::{Context, Poll};

pub struct Timer<'a> {
    ticker: &'a Ticker,
    end_time: TickInstant,
    state: TimerState,
}

impl<'a> Timer<'a> {
    pub fn new(duration: TickDuration, ticker: &'a Ticker) -> Self {
        Self {
            end_time: ticker.now() + duration,
            ticker,
            state: TimerState::Init,
        }
    }

    fn register(&self, task_id: usize) {
        let new_deadline: u64 = self.end_time.ticks();
        WAKEUP_MANAGER.add_deadline(new_deadline, task_id);
    }
}

enum TimerState {
    Init,
    Wait,
}

impl<'a> Future for Timer<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state {
            TimerState::Init => {
                self.register(ctx.waker().task_id());
                self.state = TimerState::Wait;
                Poll::Pending
            }
            TimerState::Wait => {
                if self.ticker.now() >= self.end_time {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            }
        }
    }
}

pub async fn delay(duration: TickDuration, ticker: &Ticker) {
    Timer::new(duration, ticker).await
}

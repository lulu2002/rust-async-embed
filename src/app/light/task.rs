use crate::app::button::ButtonDirection;
use crate::app::channel::Receiver;
use crate::app::future::{OurFuture, Poll};
use crate::app::light::controller::LedController;
use rtt_target::rprintln;

pub struct LedTask<'a> {
    controller: LedController<'a>,
    receiver: Receiver<'a, ButtonDirection>,
}

impl<'a> LedTask<'a> {
    pub fn new(controller: LedController<'a>, receiver: Receiver<'a, ButtonDirection>) -> Self {
        Self {
            controller,
            receiver,
        }
    }
}

impl<'a> OurFuture for LedTask<'a> {
    type Output = ();

    fn poll(&mut self, task_id: usize) -> Poll<Self::Output> {
        if let Poll::Ready(direction) = self.receiver.poll(task_id) {
            rprintln!("Button press detected..");
            self.controller.shift(direction);
        }

        self.controller.poll(task_id)
    }
}

use crate::executor::{ExtWaker, wake_task};
use core::future::poll_fn;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::Poll;
use cortex_m::peripheral::NVIC;
use embedded_hal::digital::{InputPin, PinState};
use microbit::hal::gpio::{Floating, Input, Pin};
use microbit::hal::gpiote::Gpiote;
use microbit::pac::{Interrupt, interrupt};

const MAX_CHANNELS_USED: usize = 2;
static NEXT_CHANNEL: AtomicUsize = AtomicUsize::new(0);

pub struct InputChannel {
    pin: Pin<Input<Floating>>,
    channel_id: usize,
}

impl InputChannel {
    pub fn new(pin: Pin<Input<Floating>>, gpiote: &Gpiote) -> Self {
        let channel_id = NEXT_CHANNEL.fetch_add(1, Ordering::Relaxed);
        let channel = match channel_id {
            0 => gpiote.channel0(),
            1 => gpiote.channel1(),
            MAX_CHANNELS_USED.. => todo!("Setup more channels"),
        };
        channel.input_pin(&pin).toggle().enable_interrupt();
        unsafe { NVIC::unmask(Interrupt::GPIOTE) }

        Self { pin, channel_id }
    }

    pub async fn wait_for(&mut self, ready_state: PinState) {
        poll_fn(|ctx| {
            if ready_state == PinState::from(self.pin.is_high().unwrap()) {
                return Poll::Ready(());
            }

            WAKE_TASKS[self.channel_id].store(ctx.waker().task_id(), Ordering::Relaxed);
            Poll::Pending
        })
        .await
    }
}

const INVALID_TASK_ID: usize = 0xFFFF_FFFF;
const DEFAULT_TASK: AtomicUsize = AtomicUsize::new(INVALID_TASK_ID);
static WAKE_TASKS: [AtomicUsize; MAX_CHANNELS_USED] = [DEFAULT_TASK; MAX_CHANNELS_USED];

#[interrupt]
fn GPIOTE() {
    let gpiote = unsafe { &*microbit::pac::GPIOTE::ptr() };

    for (channel, task) in WAKE_TASKS.iter().enumerate() {
        if gpiote.events_in[channel].read().bits() != 0 {
            gpiote.events_in[channel].write(|w| w);
            let task_id = task.swap(INVALID_TASK_ID, Ordering::Relaxed);
            if task_id != INVALID_TASK_ID {
                wake_task(task_id);
            }
        }
    }

    let _ = gpiote.events_in[0].read().bits();
}

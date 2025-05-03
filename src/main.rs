#![no_main]
#![no_std]

mod app;
mod executor;
mod wakeup;

use crate::wakeup::WakeupManager;
use app::channel::Channel;
use app::gpiote::InputChannel;
use app::ticker::Ticker;
use cortex_m_rt::entry;
use embedded_hal::digital::{OutputPin, PinState};
use microbit::hal::gpiote::Gpiote;
use microbit::hal::rtc::RtcInterrupt;
use microbit::pac::interrupt;
use microbit::Board;
use rtt_target::{rprintln, rtt_init_print};

pub static TICKER: Ticker = Ticker::new_raw();
pub static WAKEUP_MANAGER: WakeupManager = WakeupManager::new(&TICKER);

use crate::app::channel::{Receiver, Sender};
use crate::app::light::matrix::LedMatrix;
use crate::app::light::operator::LedOperator;
use crate::app::time::delay;
use app::btn::types::ButtonDirection;
use core::panic::PanicInfo;
use core::pin::pin;
use fugit::ExtU64;
use futures::{select_biased, FutureExt};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rprintln!("panic: {}", _info);
    loop {}
}

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board = Board::take().unwrap();
    TICKER.init(board.RTC0, &mut board.NVIC);

    let gpiote = Gpiote::new(board.GPIOTE);

    let (col, mut row) = board.display_pins.degrade();

    row[0].set_high().ok();

    let channel: Channel<ButtonDirection> = Channel::new();
    let button_l = board.buttons.button_a.degrade();
    let button_r = board.buttons.button_b.degrade();

    let mut matrix = LedMatrix::new(col);

    executor::run_tasks(&mut [
        pin!(led_task(&mut matrix, channel.get_receiver())),
        pin!(button_task(
            InputChannel::new(button_l, &gpiote),
            ButtonDirection::Left,
            channel.get_sender(),
        )),
        pin!(button_task(
            InputChannel::new(button_r, &gpiote),
            ButtonDirection::Right,
            channel.get_sender(),
        )),
    ]);
}

async fn button_task<'a>(
    mut input: InputChannel,
    direction: ButtonDirection,
    sender: Sender<'a, ButtonDirection>,
) {
    loop {
        input.wait_for(PinState::Low).await;
        sender.send(direction);
        delay(100.millis(), &TICKER).await;
        input.wait_for(PinState::High).await;
    }
}

async fn led_task<'a>(operator: &mut dyn LedOperator, mut receiver: Receiver<'a, ButtonDirection>) {
    loop {
        operator.toggle();
        select_biased! {
            direction = receiver.receive().fuse() => {
                operator.shift(direction)
            }
            _ = delay(500.millis(), &TICKER).fuse() => {}
        }
    }
}

#[interrupt]
fn RTC0() {
    critical_section::with(|cs| {
        TICKER.with_rtc_in_cs(cs, |rtc| {
            if rtc.is_event_triggered(RtcInterrupt::Overflow) {
                rtc.reset_event(RtcInterrupt::Overflow);
                TICKER.ovf_once();
            }

            if rtc.is_event_triggered(RtcInterrupt::Compare0) {
                rtc.reset_event(RtcInterrupt::Compare0);
            }

            WAKEUP_MANAGER.check_and_schedule_wakeups(cs, rtc);
        });
    })
}

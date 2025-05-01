#![no_main]
#![no_std]

mod app;
mod executor;
mod wakeup;

use crate::wakeup::WakeupManager;
use app::button::{ButtonDirection, ButtonTask};
use app::channel::Channel;
use app::future::OurFuture;
use app::gpiote::InputChannel;
use app::led::LedTask;
use app::ticker::Ticker;
use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use microbit::Board;
use microbit::hal::gpiote::Gpiote;
use microbit::hal::rtc::RtcInterrupt;
use microbit::pac::interrupt;
use rtt_target::{rprintln, rtt_init_print};

pub static TICKER: Ticker = Ticker::new_raw();
pub static WAKEUP_MANAGER: WakeupManager = WakeupManager::new(&TICKER);

use core::panic::PanicInfo;

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

    let mut tasks: [&mut dyn OurFuture<Output = ()>; 3] = [
        &mut LedTask::new(col, &TICKER, channel.get_receiver()),
        &mut ButtonTask::new(
            InputChannel::new(button_l, &gpiote),
            &TICKER,
            ButtonDirection::Left,
            channel.get_sender(),
        ),
        &mut ButtonTask::new(
            InputChannel::new(button_r, &gpiote),
            &TICKER,
            ButtonDirection::Right,
            channel.get_sender(),
        ),
    ];

    executor::run_tasks(&mut tasks);
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

#![no_main]
#![no_std]

extern crate panic_halt;

mod app;
mod executor;

use app::button::{ButtonDirection, ButtonTask};
use app::channel::Channel;
use app::future::OurFuture;
use app::gpiote::InputChannel;
use app::led::LedTask;
use app::ticker::{Ticker, TICKER};
use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use microbit::hal::gpiote::Gpiote;
use microbit::Board;
use rtt_target::rtt_init_print;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let mut board = Board::take().unwrap();
    Ticker::init(board.RTC0, &mut board.NVIC);

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

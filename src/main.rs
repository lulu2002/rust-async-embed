#![no_main]
#![no_std]

extern crate panic_halt;

mod button;
mod channel;
mod led;
mod task;
mod time;

use crate::button::{ButtonDirection, ButtonTask};
use crate::channel::Channel;
use crate::led::LedTask;
use crate::task::Task;
use crate::time::Ticker;
use cortex_m_rt::entry;
use embedded_hal::digital::OutputPin;
use microbit::Board;
use rtt_target::rtt_init_print;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let (col, mut row) = board.display_pins.degrade();

    row[0].set_high().ok();

    let channel: Channel<ButtonDirection> = Channel::new();
    let mut button_l = board.buttons.button_a.degrade();
    let mut button_r = board.buttons.button_b.degrade();

    let mut tasks: [&mut dyn Task; 3] = [
        &mut LedTask::new(col, &ticker, channel.get_receiver()),
        &mut ButtonTask::new(
            button_l,
            &ticker,
            ButtonDirection::Left,
            channel.get_sender(),
        ),
        &mut ButtonTask::new(
            button_r,
            &ticker,
            ButtonDirection::Right,
            channel.get_sender(),
        ),
    ];

    loop {
        for task in tasks.iter_mut() {
            task.poll();
        }
    }
}

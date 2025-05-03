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
use microbit::Board;
use microbit::hal::gpiote::Gpiote;
use microbit::hal::pwm::{Channel as PwmChannel, Pwm};
use microbit::hal::rtc::RtcInterrupt;
use microbit::pac::{PWM0, interrupt};
use rtt_target::{rprintln, rtt_init_print};

pub static TICKER: Ticker = Ticker::new_raw();
pub static WAKEUP_MANAGER: WakeupManager = WakeupManager::new(&TICKER);

use crate::app::channel::{Receiver, Sender};
use crate::app::light::matrix::LedMatrix;
use crate::app::light::operator::LedOperator;
use crate::app::sound::Sound;
use crate::app::time::delay;
use app::btn::types::ButtonDirection;
use core::panic::PanicInfo;
use core::pin::pin;
use fugit::ExtU64;
use futures::{FutureExt, select_biased};
use microbit::hal::gpio::Level;
use microbit::hal::time::U32Ext;

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

    let pwm = Pwm::new(board.PWM0);
    let speaker = board.speaker_pin.into_push_pull_output(Level::Low);
    pwm.set_output_pin(PwmChannel::C0, speaker.degrade());

    let led_channel: Channel<ButtonDirection> = Channel::new();
    let button_l = board.buttons.button_a.degrade();
    let button_r = board.buttons.button_b.degrade();

    let sound_channel: Channel<Sound> = Channel::new();

    let mut matrix = LedMatrix::new(col);

    executor::run_tasks(&mut [
        pin!(led_task(&mut matrix, led_channel.get_receiver())),
        pin!(button_task(
            InputChannel::new(button_l, &gpiote),
            ButtonDirection::Left,
            led_channel.get_sender(),
            sound_channel.get_sender(),
            Sound::new(500, 50)
        )),
        pin!(button_task(
            InputChannel::new(button_r, &gpiote),
            ButtonDirection::Right,
            led_channel.get_sender(),
            sound_channel.get_sender(),
            Sound::new(300, 50)
        )),
        pin!(sound_task(sound_channel.get_receiver(), pwm)),
    ]);
}

async fn button_task<'a>(
    mut input: InputChannel,
    direction: ButtonDirection,
    direction_sender: Sender<'a, ButtonDirection>,
    sound_sender: Sender<'_, Sound>,
    sound: Sound,
) {
    loop {
        input.wait_for(PinState::Low).await;
        sound_sender.send(sound);
        direction_sender.send(direction);
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

async fn sound_task<'a>(mut receiver: Receiver<'a, Sound>, mut pwm: Pwm<PWM0>) {
    loop {
        let sound = receiver.receive().await;
        play_tone_async(&mut pwm, sound, &TICKER).await;
    }
}

pub async fn play_tone_async(pwm: &mut Pwm<microbit::pac::PWM0>, sound: Sound, ticker: &Ticker) {
    pwm.set_period(sound.period().hz());
    pwm.set_duty_on(PwmChannel::C0, 0);
    pwm.set_duty_off(PwmChannel::C0, (sound.period() / 2) as u16); // 50% duty

    pwm.enable();
    delay(sound.duration(), ticker).await;
    pwm.disable();
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

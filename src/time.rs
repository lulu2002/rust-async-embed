use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::asm::semihosting_syscall;
use cortex_m::peripheral::NVIC;
use critical_section::Mutex;
use fugit::{Duration, Instant};
use microbit::hal::Rtc;
use microbit::hal::rtc::RtcInterrupt;
use microbit::pac::{RTC0, interrupt};

type TickInstant = Instant<u64, 1, 32768>;
type TickDuration = Duration<u64, 1, 32768>;

pub static TICKER: Ticker = Ticker {
    ovf_count: AtomicU32::new(0),
    rtc: Mutex::new(RefCell::new(None)),
};

pub struct Timer<'a> {
    end_time: TickInstant,
    ticker: &'a Ticker,
}

impl<'a> Timer<'a> {
    pub fn new(duration: TickDuration, ticker: &'a Ticker) -> Self {
        Self {
            end_time: ticker.now() + duration,
            ticker,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ticker.now() >= self.end_time
    }
}

pub struct Ticker {
    ovf_count: AtomicU32,
    rtc: Mutex<RefCell<Option<Rtc<RTC0>>>>,
}

impl Ticker {
    pub fn init(rtc0: RTC0, nvic: &mut NVIC) {
        let mut rtc = Rtc::new(rtc0, 0).unwrap();
        rtc.enable_counter();

        #[cfg(feature = "trigger_overflow")]
        {
            rtc.trigger_overflow();
            while rtc.get_counter() == 0 {}
        }

        rtc.enable_event(RtcInterrupt::Overflow);
        rtc.enable_interrupt(RtcInterrupt::Overflow, Some(nvic));
        critical_section::with(|cs| TICKER.rtc.replace(cs, Some(rtc)));
    }

    pub fn now(&self) -> TickInstant {
        let ticks: u64 = {
            loop {
                let ovf_before: u32 = self.get_ovf_count_from_ticker();
                let counter = critical_section::with(|cs| {
                    TICKER.rtc.borrow_ref(cs).as_ref().unwrap().get_counter()
                });
                let ovf: u32 = self.get_ovf_count_from_ticker();

                if ovf == ovf_before {
                    break ((ovf as u64) << 24 | counter as u64);
                }
            }
        };
        TickInstant::from_ticks(ticks)
    }

    fn get_ovf_count_from_ticker(&self) -> u32 {
        TICKER.ovf_count.load(Ordering::SeqCst)
    }
}

#[interrupt]
fn RTC0() {
    critical_section::with(|cs| {
        let mut rm_rtc = TICKER.rtc.borrow_ref_mut(cs);
        let rtc = rm_rtc.as_mut().unwrap();

        if rtc.is_event_triggered(RtcInterrupt::Overflow) {
            rtc.reset_event(RtcInterrupt::Overflow);
            TICKER.ovf_count.fetch_add(1, Ordering::Relaxed);
        }

        let _ = rtc.is_event_triggered(RtcInterrupt::Overflow);
    })
}

use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::peripheral::NVIC;
use critical_section::{CriticalSection, Mutex};
use fugit::{Duration, Instant};
use microbit::hal::Rtc;
use microbit::hal::rtc::RtcInterrupt;
use microbit::pac::RTC0;

pub type TickInstant = Instant<u64, 1, 32768>;
pub type TickDuration = Duration<u64, 1, 32768>;

pub static TICKER: Ticker = Ticker {
    ovf_count: AtomicU32::new(0),
    rtc: Mutex::new(RefCell::new(None)),
};

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
        rtc.enable_interrupt(RtcInterrupt::Compare0, Some(nvic));
        critical_section::with(|cs| TICKER.rtc.replace(cs, Some(rtc)));
    }

    pub fn now(&self) -> TickInstant {
        loop {
            let result = self.get_counter_result();
            if result.is_valid() {
                return TickInstant::from_ticks(result.get_count());
            }
        }
    }

    pub fn get_ovf_count(&self) -> u32 {
        self.ovf_count.load(Ordering::Relaxed)
    }

    pub fn with_rtc_in_cs<F, R>(&self, cs: CriticalSection, f: F) -> R
    where
        F: FnOnce(&mut Rtc<RTC0>) -> R,
    {
        let mut opt_ref = self.rtc.borrow_ref_mut(cs);
        let rtc = opt_ref.as_mut().expect("RTC not initialized");
        f(rtc)
    }

    pub fn ovf_once(&self) {
        self.ovf_count.fetch_add(1, Ordering::Relaxed);
    }

    fn get_counter_result(&self) -> CounterResult {
        let ovf_before: u32 = self.get_ovf_count_from_ticker();
        let counter =
            critical_section::with(|cs| self.rtc.borrow_ref(cs).as_ref().unwrap().get_counter());
        let ovf: u32 = self.get_ovf_count_from_ticker();
        CounterResult {
            counter,
            ovf,
            ovf_before,
        }
    }

    fn get_ovf_count_from_ticker(&self) -> u32 {
        self.ovf_count.load(Ordering::SeqCst)
    }
}

struct CounterResult {
    ovf_before: u32,
    ovf: u32,
    counter: u32,
}

impl CounterResult {
    fn is_valid(&self) -> bool {
        self.ovf_before == self.ovf
    }

    fn get_count(&self) -> u64 {
        (self.ovf as u64) << 24 | self.counter as u64
    }
}

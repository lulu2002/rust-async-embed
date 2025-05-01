use crate::app::ticker::{TickDuration, TickInstant, Ticker, TICKER};
use crate::executor::wake_task;
use core::cell::{RefCell, RefMut};
use critical_section::Mutex;
use heapless::binary_heap::Min;
use heapless::BinaryHeap;
use microbit::hal::rtc::{RtcCompareReg, RtcInterrupt};
use microbit::hal::Rtc;
use microbit::pac::{interrupt, RTC0};

const MAX_DEADLINES: usize = 4;
static WAKEUP_DEADLINES: Mutex<RefCell<BinaryHeap<(u64, usize), Min, MAX_DEADLINES>>> =
    Mutex::new(RefCell::new(BinaryHeap::new()));

fn schedule_wakeup(
    mut rm_deadlines: RefMut<BinaryHeap<(u64, usize), Min, MAX_DEADLINES>>,
    rtc: &mut Rtc<RTC0>,
) {
    while let Some((deadline, task_id)) = rm_deadlines.peek() {
        let ovf_count = (*deadline >> 24) as u32;
        if ovf_count == TICKER.get_ovf_count() {
            let counter = (*deadline & 0xFF_FF_FF) as u32;
            if counter > (rtc.get_counter() + 1) {
                rtc.set_compare(RtcCompareReg::Compare0, counter).ok();
                rtc.enable_event(RtcInterrupt::Compare0);
            } else {
                wake_task(*task_id);
                rm_deadlines.pop();
                continue;
            }
        }
        break;
    }

    if rm_deadlines.is_empty() {
        rtc.disable_event(RtcInterrupt::Compare0)
    }
}

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

    fn register(&self, task_id: usize) {
        let new_deadline: u64 = self.end_time.ticks();
        critical_section::with(|cs| {
            let mut rm_deadlines = WAKEUP_DEADLINES.borrow_ref_mut(cs);
            let is_earliest: bool = if let Some((next_deadline, _)) = rm_deadlines.peek() {
                new_deadline < *next_deadline
            } else {
                true
            };

            if rm_deadlines.push((new_deadline, task_id)).is_err() {
                panic!("Deadline dropped for task {}", task_id)
            }

            if is_earliest {
                self.ticker
                    .with_rtc_in_cs(cs, |rtc| schedule_wakeup(rm_deadlines, rtc))
            }
        })
    }

    pub fn is_ready(&self) -> bool {
        self.ticker.now() >= self.end_time
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

            schedule_wakeup(WAKEUP_DEADLINES.borrow_ref_mut(cs), rtc);
        });
    })
}

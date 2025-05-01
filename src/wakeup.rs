use crate::app::ticker::Ticker;
use crate::executor::wake_task;
use core::cell::{RefCell, RefMut};
use critical_section::{CriticalSection, Mutex};
use heapless::BinaryHeap;
use heapless::binary_heap::Min;
use microbit::hal::Rtc;
use microbit::hal::rtc::{RtcCompareReg, RtcInterrupt};
use microbit::pac::RTC0;

const MAX_DEADLINES: usize = 4;
pub type WakeupDeadline = (u64, usize);

pub struct WakeupManager<'a> {
    ticker: &'a Ticker,
    deadlines: Mutex<RefCell<BinaryHeap<WakeupDeadline, Min, MAX_DEADLINES>>>,
}

impl<'a> WakeupManager<'a> {
    pub const fn new(ticker: &'a Ticker) -> Self {
        Self {
            ticker,
            deadlines: Mutex::new(RefCell::new(BinaryHeap::new())),
        }
    }

    pub fn add_deadline(&self, deadline: u64, task_id: usize) {
        critical_section::with(|cs| {
            let mut rm_deadlines = self.deadlines.borrow_ref_mut(cs);

            let is_earliest: bool = if let Some((next_deadline, _)) = rm_deadlines.peek() {
                deadline < *next_deadline
            } else {
                true
            };

            if rm_deadlines.push((deadline, task_id)).is_err() {
                panic!("Deadline dropped for task {}", task_id);
            }

            if is_earliest {
                self.ticker.with_rtc_in_cs(cs, |rtc| {
                    self.schedule_wakeup(rm_deadlines, rtc);
                });
            }
        });
    }

    pub fn check_and_schedule_wakeups(&self, cs: CriticalSection, rtc: &mut Rtc<RTC0>) {
        let deadlines = self.deadlines.borrow_ref_mut(cs);
        self.schedule_wakeup(deadlines, rtc);
    }

    fn schedule_wakeup(
        &self,
        mut rm_deadlines: RefMut<BinaryHeap<WakeupDeadline, Min, MAX_DEADLINES>>,
        rtc: &mut Rtc<RTC0>,
    ) {
        while let Some((deadline, task_id)) = rm_deadlines.peek() {
            let ovf_count = (*deadline >> 24) as u32;
            if ovf_count == self.ticker.get_ovf_count() {
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
            rtc.disable_event(RtcInterrupt::Compare0);
        }
    }
}

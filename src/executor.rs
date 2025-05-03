use core::pin::Pin;
use core::task::{Context, RawWaker, RawWakerVTable, Waker};
use cortex_m::asm;
use heapless::mpmc::Q4;
use rtt_target::rprintln;

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

pub trait ExtWaker {
    fn task_id(&self) -> usize;
}

impl ExtWaker for Waker {
    fn task_id(&self) -> usize {
        self.data() as usize
    }
}

fn get_waker(task_id: usize) -> Waker {
    unsafe { Waker::from_raw(RawWaker::new(task_id as *const (), &VTABLE)) }
}

unsafe fn clone(p: *const ()) -> RawWaker {
    RawWaker::new(p, &VTABLE)
}

unsafe fn drop(p: *const ()) {}

unsafe fn wake(p: *const ()) {
    wake_task(p as usize)
}

unsafe fn wake_by_ref(p: *const ()) {
    wake_task(p as usize)
}

pub fn wake_task(task_id: usize) {
    rprintln!("Waking task {}", task_id);

    if TASK_ID_READY.enqueue(task_id).is_err() {
        panic!("Task queue full: can't add task {}", task_id)
    }
}

static TASK_ID_READY: Q4<usize> = Q4::new();

pub fn run_tasks(tasks: &mut [Pin<&mut dyn Future<Output = ()>>]) -> ! {
    for task_id in 0..tasks.len() {
        TASK_ID_READY.enqueue(task_id).ok();
    }

    loop {
        while let Some(task_id) = TASK_ID_READY.dequeue() {
            if task_id >= tasks.len() {
                rprintln!("Bad task id {}!", task_id);
                continue;
            }
            rprintln!("Running task {}", task_id);
            let _ = tasks[task_id]
                .as_mut()
                .poll(&mut Context::from_waker(&get_waker(task_id)));
        }

        rprintln!("No tasks ready, going to sleep...");
        asm::wfi();
    }
}

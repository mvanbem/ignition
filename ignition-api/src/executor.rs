use alloc::collections::VecDeque;
use alloc::sync::Arc;
use core::future::Future;
use core::task::Context;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::executor::task_waker::TaskWaker;
use crate::task::Task;

lazy_static! {
    static ref EXECUTOR: Arc<Mutex<Executor>> = Default::default();
}

pub(crate) fn run() {
    executor_run(&EXECUTOR);
}

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    EXECUTOR.lock().spawn(Task::new(future));
}

#[derive(Default)]
struct Executor {
    awake: VecDeque<Task>,
}

fn executor_run(arc: &Arc<Mutex<Executor>>) {
    loop {
        let mut inner = arc.lock();
        let task = match inner.awake.pop_front() {
            Some(task) => task,
            None => break,
        };
        drop(inner);

        let arc_waker = Arc::new(TaskWaker::new(task));
        let waker = arc_waker::new(Arc::clone(&arc_waker));
        let mut context = Context::from_waker(&waker);
        // The result from poll() may be discarded without action. All clones of the waker will
        // eventually be dropped, dropping the task only after it is no longer needed. If
        // incomplete, it will eventually be respawned when wakened.
        arc_waker.start_deferred_wake();
        drop(arc_waker.task_mut().as_mut().unwrap().poll(&mut context));
        arc_waker.end_deferred_wake();
    }
}

impl Executor {
    fn spawn(&mut self, task: Task) {
        self.awake.push_back(task);
    }
}

mod task_waker {
    use core::ops::DerefMut;
    use core::sync::atomic::{AtomicU8, Ordering};

    use spin::Mutex;

    use crate::executor::arc_waker::WakerTrait;
    use crate::executor::EXECUTOR;
    use crate::task::Task;

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum DeferredWake {
        NotDeferred,
        DeferredIdle,
        DeferredPending,
    }

    impl DeferredWake {
        fn as_u8(self) -> u8 {
            match self {
                DeferredWake::NotDeferred => 0,
                DeferredWake::DeferredIdle => 1,
                DeferredWake::DeferredPending => 2,
            }
        }
    }

    pub struct TaskWaker {
        deferred_wake: AtomicU8,
        task: Mutex<Option<Task>>,
    }

    impl TaskWaker {
        pub fn new(task: Task) -> Self {
            Self {
                deferred_wake: AtomicU8::new(DeferredWake::NotDeferred.as_u8()),
                task: Mutex::new(Some(task)),
            }
        }

        pub fn start_deferred_wake(&self) {
            self.deferred_wake
                .compare_exchange(
                    DeferredWake::NotDeferred.as_u8(),
                    DeferredWake::DeferredIdle.as_u8(),
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
                .unwrap();
        }

        pub fn end_deferred_wake(&self) {
            match self
                .deferred_wake
                .swap(DeferredWake::NotDeferred.as_u8(), Ordering::SeqCst)
            {
                x if x == DeferredWake::NotDeferred.as_u8() => panic!(),
                x if x == DeferredWake::DeferredIdle.as_u8() => (),
                x if x == DeferredWake::DeferredPending.as_u8() => self.wake(),
                _ => unreachable!(),
            }
        }

        pub fn task_mut(&self) -> impl DerefMut<Target = Option<Task>> + '_ {
            self.task.lock()
        }
    }

    impl WakerTrait for TaskWaker {
        fn wake(&self) {
            loop {
                // Attempt to register a deferred wake.
                match self.deferred_wake.compare_exchange(
                    DeferredWake::DeferredIdle.as_u8(),
                    DeferredWake::DeferredPending.as_u8(),
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_idle) => {
                        // Successfully registered a deferred wake.
                        return;
                    }
                    Err(x) if x == DeferredWake::DeferredPending.as_u8() => {
                        // A deferred wake was already registered. Nothing needs to be done.
                        return;
                    }
                    Err(x) if x == DeferredWake::NotDeferred.as_u8() => {
                        // Attempt an immediate wake below.
                    }
                    Err(_) => unreachable!(),
                }

                // Falling through means we need to attempt an immediate wake. If the lock is
                // unavailable, assume the task switched to a deferred wake and try again.
                if let Some(mut guard) = self.task.try_lock() {
                    if let Some(task) = guard.take() {
                        EXECUTOR.lock().spawn(task);
                    } else {
                        crate::log("*** duplicate wake!");
                    }
                    return;
                } else {
                    crate::log(
                        "TaskWaker::wake() failed to acquire the lock (this should be rare)",
                    );
                }
            }
        }
    }
}

mod arc_waker {
    use core::task::{RawWaker, RawWakerVTable, Waker};

    use alloc::sync::Arc;

    pub trait WakerTrait {
        fn wake(&self);
    }

    fn raw_waker_clone<T: WakerTrait>(ptr: *const ()) -> RawWaker {
        unsafe { Arc::increment_strong_count(ptr) }
        new_raw_waker(ptr as *const T)
    }

    fn raw_waker_wake<T: WakerTrait>(ptr: *const ()) {
        raw_waker_wake_by_ref::<T>(ptr);
        raw_waker_drop::<T>(ptr);
    }

    fn raw_waker_wake_by_ref<T: WakerTrait>(ptr: *const ()) {
        let data = unsafe { (ptr as *const T).as_ref() }.unwrap();
        data.wake();
    }

    fn raw_waker_drop<T: WakerTrait>(ptr: *const ()) {
        unsafe { Arc::decrement_strong_count(ptr) }
    }

    fn new_raw_waker<T: WakerTrait>(data: *const T) -> RawWaker {
        RawWaker::new(
            data as *const (),
            &RawWakerVTable::new(
                raw_waker_clone::<T>,
                raw_waker_wake::<T>,
                raw_waker_wake_by_ref::<T>,
                raw_waker_drop::<T>,
            ),
        )
    }

    pub fn new<T: WakerTrait>(data: Arc<T>) -> Waker {
        unsafe { Waker::from_raw(new_raw_waker(Arc::into_raw(data))) }
    }
}

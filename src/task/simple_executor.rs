#![allow(dead_code, unused_imports)]
use core::{
    future::{self, Future},
    task::{Context, Poll, RawWakerVTable},
};

use super::Task;
use alloc::collections::VecDeque;

pub struct SimpleExecutor {
    tasks: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        SimpleExecutor {
            tasks: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.tasks.push_back(task);
    }

    pub fn start(&mut self) {
        while let Some(mut task) = self.tasks.pop_front() {
            let mut waker = dummy_waker();
            let mut context = Context::from_waker(&mut waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.tasks.push_back(task),
            }
        }
    }
}

use core::task::{RawWaker, Waker};

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

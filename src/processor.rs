use crate::id::TaskId;
use alloc::{boxed::Box, collections::VecDeque};
use core::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};
use queue::AtomicCell;
use queue::LockFreeQueue;
use spin::Lazy;

/// 这个数据结构只能使用无锁的数据结构，因为在内核和用户态使用的锁不一样
/// 此外，还需要额外的结构来存放每个 CPU 上使用的数据，因为内核有自己重新定义的数据
/// 可以将 percpu 的初始化放在这里进行，其他的包中不需要使用 percpu 数据
#[derive(Debug)]
#[repr(C, align(64))]
pub struct Processor {
    /// Processor ready_queue
    ready_queue: LockFreeQueue<TaskId>,
    ///
    current_task: AtomicCell<Option<TaskId>>,
}

unsafe impl Sync for Processor {}
unsafe impl Send for Processor {}

impl Processor {
    pub fn new() -> Self {
        let queue = LockFreeQueue::new();
        Processor {
            ready_queue: queue,
            current_task: AtomicCell::new(None),
        }
    }

    #[inline]
    /// Pick one task from processor
    pub(crate) fn pick_next_task(&self) -> Option<TaskId> {
        self.ready_queue.pop()
    }

    #[inline]
    /// Add curr task to Processor, it ususally add to back
    pub(crate) fn put_prev_task(&self, task: TaskId, _front: bool) {
        self.ready_queue.push(task);
    }

    #[inline]
    /// Add task to processor, now just put it to own processor
    /// TODO: support task migrate on differ processor
    pub(crate) fn add_task(&self, task: TaskId) {
        self.ready_queue.push(task);
    }

    #[inline]
    /// First add task to processor
    pub(crate) fn first_add_task(task: TaskId) {
        let p = Processor::select_processor();
        p.ready_queue.push(task);
    }

    #[inline]
    fn select_processor() -> &'static Processor {
        crate::percpu::percpus()
            .iter()
            .min_by_key(|p| p.ready_queue.len())
            .unwrap()
    }
}

use alloc::collections::btree_map::BTreeMap;
use spinlock::SpinRaw;

use crate::prio_queue::{KtaskInfo, Scheduler};

pub struct VdsoData {
    /// 存储所有调度器
    pub(crate) schedulers: SpinRaw<BTreeMap<usize, Scheduler>>,
    /// 存储用户调度器和内核任务的对应关系（用于优先级更新）
    pub(crate) uscheduler_ktask: SpinRaw<BTreeMap<usize, KtaskInfo>>,
    /// 用于GlobalAllocator进行动态内存分配的区域
    pub(crate) alloc_area: AllocArea
}

const PAGE_SIZE: usize = 0x1000;
#[repr(align(1024))]
pub struct AllocArea(pub(crate) [u8; PAGE_SIZE]);

impl VdsoData {
    pub const fn new() -> Self {
        Self {
            schedulers: SpinRaw::new(BTreeMap::new()),
            uscheduler_ktask: SpinRaw::new(BTreeMap::new()),
            alloc_area: AllocArea([0; PAGE_SIZE])
        }
    }
}

unsafe extern "C" { 
    unsafe fn vdso_data();
}

// SAFETY: 用于vdso中的函数（这样才有vdso_data符号），且需在vdso初始化之后调用
unsafe fn get_vdso_data() -> &'static mut VdsoData {
    &mut *(vdso_data as *mut () as *mut VdsoData)
}

// SAFETY: 用于vdso中的函数（这样才有vdso_data符号），且需在vdso初始化之后调用
pub(crate) fn get_schedulers() -> &'static mut SpinRaw<BTreeMap<usize, Scheduler>> {
    unsafe { &mut get_vdso_data().schedulers }
}

// SAFETY: 用于vdso中的函数（这样才有vdso_data符号），且需在vdso初始化之后调用
pub(crate) fn get_uscheduler_ktask() -> &'static mut SpinRaw<BTreeMap<usize, KtaskInfo>> {
    unsafe { &mut get_vdso_data().uscheduler_ktask }
}

// SAFETY: 用于vdso中的函数（这样才有vdso_data符号），且需在vdso初始化之后调用
pub(crate) fn get_alloc_area() -> &'static mut AllocArea {
    unsafe { &mut get_vdso_data().alloc_area }
}
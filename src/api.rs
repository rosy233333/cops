use crate::{get_schedulers, get_uscheduler_ktask, prio_queue::{get_uscheduler_from_task, update_ktask_priority, KtaskInfo, Scheduler}};

/// 初始化数据段
/// 只在初始化vdso时调用一次。映射到用户空间时不需再调用

/// 新增一个调度器。
/// 对于内核调度器，scheduler_id为cpu_id（每个CPU使用不同调度器）或0（每个CPU使用相同调度器）。对于用户调度器，scheduler_id为0x80000000+thread_id（每个线程使用不同调度器）。
/// 代码逻辑需要通过scheduler_id & 0x80000000 == 0判断调度器属于用户或是内核。因此，需要保证cpu_id和thread_id均小于0x80000000。
/// 每个用户调度器会关联一个内核线程，通过ktask_info传入。元组的第0、1个元素分别代表内核任务地址和内核任务的scheduler_id。
#[no_mangle]
pub extern "C" fn __vdso_add_scheduler(scheduler_id: usize, ktask_info: Option<(usize, usize)>) -> bool {
    if get_schedulers().lock().contains_key(&scheduler_id) {
        false
    }
    else {
        assert!(get_schedulers().lock().insert(scheduler_id, Scheduler::new()).is_none());
        if let Some(ktask_info) = ktask_info {
            assert!(get_uscheduler_ktask().lock().insert(scheduler_id, KtaskInfo { ktask_ptr: ktask_info.0, cpu_id: ktask_info.1 }).is_none());
        }
        true
    }
}

#[no_mangle]
pub extern "C" fn __vdso_delete_scheduler(scheduler_id: usize) -> bool {
    if get_schedulers().lock().remove(&scheduler_id).is_some() {
        get_uscheduler_ktask().lock().remove(&scheduler_id);
        true
    }
    else {
        false
    }
}

// 在以下三个对任务的操作中，需要保证传入的scheduler_id是当前调度器；
// 否则，可能导致获取的当前优先级不正确，进而导致对“是否需要重新调度”的判断不正确。

/// 在添加任务时，不一定会使用default_task_prio中指定的优先级：
/// 如果添加的内核任务已经对应了一个用户调度器，则使用该用户调度器的最高优先级代替default_task_prio中指定的优先级。
/// 这样安排是为了解决当内核任务不在调度器中时，无法根据其对应的用户调度器及时更新其优先级的问题。
/// 
/// 返回值代表当前调度器是否需要重新调度。
/// 用户调度器只在clear_current时检查内核是否需要重新调度。
/// 因此，虽然此时update_ktask_priority可能返回true，但我们不使用其结果。
#[no_mangle]
pub extern "C" fn __vdso_add_task(scheduler_id: usize, task_ptr: usize, default_task_prio: usize) -> bool {
    let mut task_prio = default_task_prio;
    if let Some(uscheduler) = get_uscheduler_from_task(task_ptr) {
        // 将内核任务加入调度器时，它（及其对应的用户调度器）一定不在运行，
        // 因此不需考虑用户调度器的current_prio。
        task_prio = get_schedulers().lock().get(&uscheduler).unwrap().highest_prio;
    }
    let res = get_schedulers().lock().get_mut(&scheduler_id).as_mut().unwrap().add_task(task_ptr, task_prio);
    if !is_kernel(scheduler_id) {
        update_ktask_priority(scheduler_id);
    }
    res
}

/// 在当前任务执行完成或中止、下一个任务还未取出时调用，此时调度器会将current_priority设置为最低。
/// 由于与用户调度器关联的内核任务的优先级取决于current_priority和highest_priority中较高者，因此该操作会改变用户调度器对应的内核任务的优先级，可能导致内核的重新调度。
/// 用户态调用时，返回值代表内核是否需要重新调度，此时用户态需要主动陷入内核；
/// 内核态调用时，返回值恒为false。
#[no_mangle]
pub extern "C" fn __vdso_clear_current(scheduler_id: usize) -> bool {
    get_schedulers().lock().get_mut(&scheduler_id).as_mut().unwrap().clear_current();
    if !is_kernel(scheduler_id) {
        update_ktask_priority(scheduler_id)
    }
    else {
        false
    }
}

/// 由于内核重新调度已在调用clear_current后触发，因此调用此函数不会触发内核重新调度。
#[no_mangle]
pub extern "C" fn __vdso_pick_next_task(scheduler_id: usize) -> Option<usize> {
    get_schedulers().lock().get_mut(&scheduler_id).as_mut().unwrap().pick_next_task()
}

/// 判断调度器id属于用户态还是内核态。
fn is_kernel(scheduler_id: usize) -> bool {
    scheduler_id & 0x80000000 == 0
}
use crate::{allocator, id::TaskId, percpu, processor::Processor};

#[no_mangle]
pub extern "C" fn init_primary(max_cpu: usize, cpu_id: usize) {
    allocator::init();
    percpu::init_percpu_primary(max_cpu, cpu_id);
    percpu::setup_percpu(cpu_id);
}

#[no_mangle]
pub extern "C" fn init_secondary(max_cpu: usize, cpu_id: usize) {
    percpu::init_percpu_secondary(max_cpu, cpu_id);
    percpu::setup_percpu(cpu_id);
}

#[no_mangle]
pub extern "C" fn pick_next_task() -> TaskId {
    percpu::current_processor()
        .pick_next_task()
        .unwrap_or(TaskId::NULL)
}

#[no_mangle]
pub extern "C" fn add_task(task: TaskId) {
    percpu::current_processor().add_task(task);
}

#[no_mangle]
pub extern "C" fn first_add_task(task: TaskId) {
    Processor::first_add_task(task);
}

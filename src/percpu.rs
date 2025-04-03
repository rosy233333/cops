use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use alloc::vec::Vec;

/// 这里定义了每个 CPU 的局部数据，因为非 PIC 的代码在共享库中不能正常工作，所以 percpu 库的实现方式在这里不能继续使用
///
use crate::{processor::Processor, PAGE_SIZE};

#[repr(C, align(64))]
pub struct PerCPU {
    processor: Processor,
    cpu_id: AtomicUsize,
    is_bsp: AtomicBool,
    cpu_count: AtomicUsize,
}

pub fn current_processor() -> &'static Processor {
    &get_percpu().processor
}

const fn align_up_64(val: usize) -> usize {
    const SIZE_64BIT: usize = 0x40;
    (val + SIZE_64BIT - 1) & !(SIZE_64BIT - 1)
}

static PERCPU_AREA_BASE: spin::once::Once<usize> = spin::once::Once::new();

pub(crate) fn init_percpu_primary(max_cpu: usize, cpu_id: usize) {
    let size = core::mem::size_of::<PerCPU>();
    let total_size = align_up_64(size) * max_cpu;
    let layout = core::alloc::Layout::from_size_align(total_size, 0x1000).unwrap();
    PERCPU_AREA_BASE.call_once(|| unsafe { alloc::alloc::alloc(layout) as usize });
    let base = PERCPU_AREA_BASE.get().unwrap();
    let percpu = unsafe { &mut *(*base as *mut PerCPU) };
    *percpu = PerCPU {
        processor: Processor::new(),
        cpu_id: AtomicUsize::new(cpu_id),
        is_bsp: AtomicBool::new(true),
        cpu_count: AtomicUsize::new(max_cpu),
    };
}

pub(crate) fn init_percpu_secondary(max_cpu: usize, cpu_id: usize) {
    let size = core::mem::size_of::<PerCPU>();
    let base = PERCPU_AREA_BASE.get().unwrap();
    let percpu = unsafe { &mut *((*base + cpu_id * align_up_64(size)) as *mut PerCPU) };
    *percpu = PerCPU {
        processor: Processor::new(),
        cpu_id: AtomicUsize::new(cpu_id),
        is_bsp: AtomicBool::new(false),
        cpu_count: AtomicUsize::new(max_cpu),
    };
}

pub(crate) fn percpus() -> Vec<&'static Processor> {
    let size = core::mem::size_of::<PerCPU>();
    let base = PERCPU_AREA_BASE.get().unwrap();
    let mut processors = Vec::new();
    let cpus = unsafe { &mut *((*base) as *mut PerCPU) }
        .cpu_count
        .load(Ordering::Relaxed);
    for i in 0..cpus {
        let percpu = unsafe { &mut *((*base + i * align_up_64(size)) as *mut PerCPU) };
        processors.push(&percpu.processor);
    }
    processors
}

/// Set the architecture-specific thread pointer register to the per-CPU data
/// area base on the current CPU.
///
/// `cpu_id` indicates which per-CPU data area to use.
pub fn setup_percpu(cpu_id: usize) {
    let tp = PERCPU_AREA_BASE.get().unwrap() + cpu_id * align_up_64(core::mem::size_of::<PerCPU>());
    unsafe {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                if cfg!(target_os = "linux") {
                    const ARCH_SET_GS: u32 = 0x1001;
                    const SYS_ARCH_PRCTL: u32 = 158;
                    core::arch::asm!(
                        "syscall",
                        in("eax") SYS_ARCH_PRCTL,
                        in("edi") ARCH_SET_GS,
                        in("rsi") tp,
                    );
                } else if cfg!(target_os = "none") {
                    x86::msr::wrmsr(x86::msr::IA32_GS_BASE, tp as u64);
                } else {
                    unimplemented!()
                }
                SELF_PTR.write_current_raw(tp);
            } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                core::arch::asm!("mv gp, {}", in(reg) tp)
            } else if #[cfg(target_arch = "aarch64")] {
                core::arch::asm!("msr TPIDR_EL1, {}", in(reg) tp)
            }
        }
    }
}

/// Read the architecture-specific thread pointer register on the current CPU.
pub fn get_percpu() -> &'static PerCPU {
    let tp: usize;
    unsafe {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "x86_64")] {
                tp = if cfg!(target_os = "linux") {
                    SELF_PTR.read_current_raw()
                } else if cfg!(target_os = "none") {
                    x86::msr::rdmsr(x86::msr::IA32_GS_BASE) as usize
                } else {
                    unimplemented!()
                };
            } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
                core::arch::asm!("mv {}, gp", out(reg) tp)
            } else if #[cfg(target_arch = "aarch64")] {
                core::arch::asm!("mrs {}, TPIDR_EL1", out(reg) tp)
            }
        }
    }
    unsafe { &*(tp as *const PerCPU) }
}

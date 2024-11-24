//! 提供获取全局位图的方法

#[no_mangle]
pub extern "C" fn __vdso_rt_sigreturn() -> usize {
    0xdeadbeef
}
#[no_mangle]
pub extern "C" fn __vdso_gettimeofday() -> usize {
    0xdeadbeef
}
#[no_mangle]
pub extern "C" fn __vdso_clock_gettime() -> usize {
    0xdeadbeef
}
#[no_mangle]
pub extern "C" fn __vdso_clock_getres() -> usize {
    0xdeadbeef
}
#[no_mangle]
pub extern "C" fn __vdso_getcpu() -> usize {
    0xdeadbeef
}

#[no_mangle]
pub extern "C" fn __vdso_flush_icache() -> usize {
    0xdeadbeef
}

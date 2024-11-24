//!
#![no_std]
#![feature(lang_items)]
#![allow(internal_features)]

mod getbitmap;
pub use getbitmap::__vdso_gettimeofday;

mod lang_item {

    #[lang = "eh_personality"]
    #[no_mangle]
    fn rust_eh_personality() {}

    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> ! {
        unreachable!()
    }
}

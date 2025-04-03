//!
#![no_std]
#![no_main]
#![feature(lang_items)]
#![allow(internal_features, unused, non_snake_case)]
mod allocator;
mod api;
mod id;
mod percpu;
mod processor;
pub use api::*;

extern crate alloc;

pub(crate) const PAGE_SIZE: usize = 0x1000;

#[cfg(feature = "no_std")]
mod lang_item {

    #[lang = "eh_personality"]
    #[no_mangle]
    fn rust_eh_personality() {}

    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> ! {
        unreachable!()
    }

    #[no_mangle]
    fn _Unwind_Resume() {
        unreachable!()
    }
}

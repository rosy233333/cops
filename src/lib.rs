//!
#![no_std]
#![feature(lang_items)]
#![allow(internal_features)]

// mod getbitmap;
mod api;
mod prio_queue;
mod allocator;
mod data;

extern crate alloc;

pub use api::*;
pub use data::*;

mod lang_item {

    #[lang = "eh_personality"]
    #[no_mangle]
    fn rust_eh_personality() {}

    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> ! {
        unreachable!()
    }
}

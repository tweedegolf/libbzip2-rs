#![no_std]

extern crate libbzip2_rs_sys;

use core::ffi::c_int;
use core::panic::PanicInfo;
pub use libbzip2_rs_sys::*;

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    extern "C" {
        fn bz_internal_error(errcode: c_int);
    }

    unsafe { bz_internal_error(-1) }
    loop {}
}

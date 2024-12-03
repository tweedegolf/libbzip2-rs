#![no_std]

extern crate libbzip2_rs_sys;

use core::panic::PanicInfo;
pub use libbzip2_rs_sys::*;

#[cfg(feature = "stdio")]
struct StderrWritter;

#[cfg(feature = "stdio")]
impl core::fmt::Write for StderrWritter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        use core::ffi::c_void;
        use libc::write;

        unsafe {
            write(2, s.as_ptr() as *const c_void, s.len());
        }
        Ok(())
    }
}

#[panic_handler]
fn panic_handler(_info: &PanicInfo) -> ! {
    #[cfg(feature = "stdio")]
    {
        use core::fmt::Write;
        use libc::exit;

        let _ = StderrWritter.write_str("libbzip2-rs: internal error:\n");
        let _ = StderrWritter.write_fmt(format_args!("{}", _info.message()));

        unsafe {
            exit(3);
        }
    }

    #[cfg(not(feature = "stdio"))]
    {
        extern "C" {
            fn bz_internal_error(errcode: core::ffi::c_int);
        }

        unsafe { bz_internal_error(-1) }
        loop {}
    }
}

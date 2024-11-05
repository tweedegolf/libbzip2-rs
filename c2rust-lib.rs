#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_safety_doc)] // FIXME remove once everything has safety docs
#![allow(clippy::needless_range_loop)] // FIXME remove once all instances are fixed

extern crate libc;
pub mod blocksort;
pub mod bzlib;
pub mod compress;
pub mod crctable;
pub mod decompress;
pub mod huffman;
pub mod randtable;

#[macro_export]
macro_rules! assert_h {
    ($condition:expr, $errcode:expr) => {{
        if !$condition {
            $crate::bzlib::BZ2_bz__AssertH__fail($errcode);
        }
    }};
}

#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_safety_doc)] // FIXME remove once everything has safety docs
#![allow(clippy::needless_range_loop)] // FIXME remove once all instances are fixed

//! A drop-in compatible rust implementation of bzip2

use core::ffi::c_int;

extern crate libc;

mod blocksort;
mod bzlib;
mod compress;
mod crctable;
mod decompress;
mod huffman;
mod randtable;

#[macro_export]
macro_rules! assert_h {
    ($condition:expr, $errcode:expr) => {{
        if !$condition {
            $crate::bzlib::BZ2_bz__AssertH__fail(3001);
        }
    }};
}

pub(crate) use bzlib::{Action, ReturnCode};

pub const BZ_OK: c_int = ReturnCode::BZ_OK as c_int;
pub const BZ_RUN_OK: c_int = ReturnCode::BZ_RUN_OK as c_int;
pub const BZ_FLUSH_OK: c_int = ReturnCode::BZ_FLUSH_OK as c_int;
pub const BZ_FINISH_OK: c_int = ReturnCode::BZ_FINISH_OK as c_int;
pub const BZ_STREAM_END: c_int = ReturnCode::BZ_STREAM_END as c_int;
pub const BZ_SEQUENCE_ERROR: c_int = ReturnCode::BZ_SEQUENCE_ERROR as c_int;
pub const BZ_PARAM_ERROR: c_int = ReturnCode::BZ_PARAM_ERROR as c_int;
pub const BZ_MEM_ERROR: c_int = ReturnCode::BZ_MEM_ERROR as c_int;
pub const BZ_DATA_ERROR: c_int = ReturnCode::BZ_DATA_ERROR as c_int;
pub const BZ_DATA_ERROR_MAGIC: c_int = ReturnCode::BZ_DATA_ERROR_MAGIC as c_int;
pub const BZ_IO_ERROR: c_int = ReturnCode::BZ_IO_ERROR as c_int;
pub const BZ_UNEXPECTED_EOF: c_int = ReturnCode::BZ_UNEXPECTED_EOF as c_int;
pub const BZ_OUTBUFF_FULL: c_int = ReturnCode::BZ_OUTBUFF_FULL as c_int;
pub const BZ_CONFIG_ERROR: c_int = ReturnCode::BZ_CONFIG_ERROR as c_int;

pub const BZ_RUN: c_int = Action::Run as c_int;
pub const BZ_FLUSH: c_int = Action::Flush as c_int;
pub const BZ_FINISH: c_int = Action::Finish as c_int;

// types
pub use bzlib::bz_stream;

// the low-level interface
pub use bzlib::{BZ2_bzCompress, BZ2_bzCompressEnd, BZ2_bzCompressInit};
pub use bzlib::{BZ2_bzDecompress, BZ2_bzDecompressEnd, BZ2_bzDecompressInit};

// utility functions
pub use bzlib::{BZ2_bzBuffToBuffCompress, BZ2_bzBuffToBuffDecompress};

// the high-level interface
pub use bzlib::{BZ2_bzRead, BZ2_bzReadClose, BZ2_bzReadGetUnused, BZ2_bzReadOpen};
pub use bzlib::{BZ2_bzWrite, BZ2_bzWriteClose, BZ2_bzWriteClose64, BZ2_bzWriteOpen};

// zlib compatibility functions
pub use bzlib::{
    BZ2_bzclose, BZ2_bzdopen, BZ2_bzerror, BZ2_bzflush, BZ2_bzlibVersion, BZ2_bzopen, BZ2_bzread,
    BZ2_bzwrite,
};

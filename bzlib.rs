use core::ffi::{c_char, c_int, c_uint, c_void};

use libc::FILE;
use libc::{
    exit, fclose, fdopen, ferror, fflush, fgetc, fopen, fread, free, fwrite, malloc, strcat,
    strcmp, ungetc,
};

use crate::compress::BZ2_compressBlock;
use crate::crctable::BZ2_CRC32TABLE;
use crate::decompress::{self, BZ2_decompress};
use crate::randtable::BZ2_RNUMS;

extern "C" {
    static stdin: *mut FILE;
    static stdout: *mut FILE;
}

pub(crate) const BZ_MAX_ALPHA_SIZE: usize = 258;
pub(crate) const BZ_MAX_CODE_LEN: usize = 23;

pub(crate) const BZ_N_GROUPS: usize = 6;
pub(crate) const BZ_G_SIZE: usize = 50;
pub(crate) const BZ_N_ITERS: usize = 4;

pub(crate) const BZ_MAX_SELECTORS: usize = 2 + (900000 / BZ_G_SIZE);

pub(crate) const BZ_RUNA: u16 = 0;
pub(crate) const BZ_RUNB: u16 = 1;

#[cfg(feature = "custom-prefix")]
macro_rules! prefix {
    ($name:expr) => {
        concat!(env!("LIBBZIP2_RS_SYS_PREFIX"), stringify!($name))
    };
}

#[cfg(all(
    not(feature = "custom-prefix"),
    not(any(test, feature = "testing-prefix"))
))]
macro_rules! prefix {
    ($name:expr) => {
        stringify!($name)
    };
}

#[cfg(all(not(feature = "custom-prefix"), any(test, feature = "testing-prefix")))]
macro_rules! prefix {
    ($name:expr) => {
        concat!("LIBBZIP2_RS_SYS_TEST_", stringify!($name))
    };
}

macro_rules! libbzip2_rs_sys_version {
    () => {
        concat!("1.1.0-libbzip2-rs-sys-", env!("CARGO_PKG_VERSION"))
    };
}

const LIBBZIP2_RS_SYS_VERSION: &str = concat!(libbzip2_rs_sys_version!(), "\0");

/// The version of the zlib library.
///
/// Its value is a pointer to a NULL-terminated sequence of bytes.
///
/// The version string for this release is `
#[doc = libbzip2_rs_sys_version!()]
/// `:
///
/// - The first component is the version of stock zlib that this release is compatible with
/// - The final component is the zlib-rs version used to build this release.
#[export_name = prefix!(BZ2_bzlibVersion)]
pub extern "C" fn BZ2_bzlibVersion() -> *const core::ffi::c_char {
    LIBBZIP2_RS_SYS_VERSION.as_ptr().cast::<core::ffi::c_char>()
}

type AllocFunc = unsafe extern "C" fn(*mut c_void, c_int, c_int) -> *mut c_void;
type FreeFunc = unsafe extern "C" fn(*mut c_void, *mut c_void) -> ();

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bz_stream {
    pub next_in: *mut c_char,
    pub avail_in: c_uint,
    pub total_in_lo32: c_uint,
    pub total_in_hi32: c_uint,
    pub next_out: *mut c_char,
    pub avail_out: c_uint,
    pub total_out_lo32: c_uint,
    pub total_out_hi32: c_uint,
    pub state: *mut c_void,
    pub bzalloc: Option<AllocFunc>,
    pub bzfree: Option<FreeFunc>,
    pub opaque: *mut c_void,
}

impl bz_stream {
    pub const fn zeroed() -> Self {
        Self {
            next_in: std::ptr::null_mut::<libc::c_char>(),
            avail_in: 0,
            total_in_lo32: 0,
            total_in_hi32: 0,
            next_out: std::ptr::null_mut::<libc::c_char>(),
            avail_out: 0,
            total_out_lo32: 0,
            total_out_hi32: 0,
            state: std::ptr::null_mut::<libc::c_void>(),
            bzalloc: None,
            bzfree: None,
            opaque: std::ptr::null_mut::<libc::c_void>(),
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
enum ReturnCode {
    BZ_OK = 0,
    BZ_RUN_OK = 1,
    BZ_FLUSH_OK = 2,
    BZ_FINISH_OK = 3,
    BZ_STREAM_END = 4,
    BZ_SEQUENCE_ERROR = -1,
    BZ_PARAM_ERROR = -2,
    BZ_MEM_ERROR = -3,
    BZ_DATA_ERROR = -4,
    // BZ_DATA_ERROR_MAGIC = -5,
    // BZ_IO_ERROR = -6,
    BZ_UNEXPECTED_EOF = -7,
    BZ_OUTBUFF_FULL = -8,
    BZ_CONFIG_ERROR = -9,
}

use ReturnCode::*;

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum Mode {
    Idle = 1,
    Running = 2,
    Flushing = 3,
    Finishing = 4,
}

#[repr(i32)]
#[derive(Copy, Clone)]
pub enum State {
    Output = 1,
    Input = 2,
}

pub const BZ_N_RADIX: i32 = 2;
pub const BZ_N_QSORT: i32 = 12;
pub const BZ_N_SHELL: i32 = 18;
pub const BZ_N_OVERSHOOT: i32 = BZ_N_RADIX + BZ_N_QSORT + BZ_N_SHELL + 2;
pub const BZ_N_OVERSHOOT2: usize = (BZ_N_RADIX + BZ_N_QSORT + BZ_N_SHELL + 2) as usize;

pub const FTAB_LEN: usize = u16::MAX as usize + 2;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct EState {
    pub strm: *mut bz_stream,
    pub mode: Mode,
    pub state: State,
    pub avail_in_expect: u32,
    pub arr1: *mut u32,
    pub arr2: *mut u32,
    pub ftab: *mut u32,
    pub origPtr: i32,
    pub ptr: *mut u32,
    pub block: *mut u8,
    pub mtfv: *mut u16,
    pub writer: crate::compress::EWriter,
    pub workFactor: i32,
    pub state_in_ch: u32,
    pub state_in_len: i32,
    pub rNToGo: i32,
    pub rTPos: i32,
    pub nblock: i32,
    pub nblockMAX: i32,
    pub state_out_pos: i32,
    pub nInUse: i32,
    pub inUse: [bool; 256],
    pub unseqToSeq: [u8; 256],
    pub blockCRC: u32,
    pub combinedCRC: u32,
    pub verbosity: i32,
    pub blockNo: i32,
    pub blockSize100k: i32,
    pub nMTF: i32,
    pub mtfFreq: [i32; 258],
    pub selector: [u8; 18002],
    pub selectorMtf: [u8; 18002],
    pub len: [[u8; BZ_MAX_ALPHA_SIZE]; BZ_N_GROUPS],
    pub code: [[i32; 258]; 6],
    pub rfreq: [[i32; 258]; 6],
    pub len_pack: [[u32; 4]; 258],
}
pub type Bool = libc::c_uchar;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DState {
    pub strm: *mut bz_stream,
    pub state: decompress::State,
    pub state_out_ch: u8,
    pub state_out_len: i32,
    pub blockRandomised: bool,
    pub rNToGo: i32,
    pub rTPos: i32,
    pub bsBuff: u32,
    pub bsLive: i32,
    pub blockSize100k: i32,
    pub smallDecompress: bool,
    pub currBlockNo: i32,
    pub verbosity: i32,
    pub origPtr: i32,
    pub tPos: u32,
    pub k0: i32,
    pub unzftab: [i32; 256],
    pub nblock_used: i32,
    pub cftab: [i32; 257],
    pub cftabCopy: [i32; 257],
    pub tt: *mut u32,
    pub ll16: *mut u16,
    pub ll4: *mut u8,
    pub storedBlockCRC: u32,
    pub storedCombinedCRC: u32,
    pub calculatedBlockCRC: u32,
    pub calculatedCombinedCRC: u32,
    pub nInUse: i32,
    pub inUse: [Bool; 256],
    pub inUse16: [Bool; 16],
    pub seqToUnseq: [u8; 256],
    pub mtfa: [u8; 4096],
    pub mtfbase: [i32; 16],
    pub selector: [u8; 18002],
    pub selectorMtf: [u8; 18002],
    pub len: [[u8; 258]; 6],
    pub limit: [[i32; 258]; 6],
    pub base: [[i32; 258]; 6],
    pub perm: [[i32; 258]; 6],
    pub minLens: [i32; 6],
    pub save_i: i32,
    pub save_j: i32,
    pub save_t: i32,
    pub save_alphaSize: i32,
    pub save_nGroups: i32,
    pub save_nSelectors: i32,
    pub save_EOB: i32,
    pub save_groupNo: i32,
    pub save_groupPos: i32,
    pub save_nextSym: i32,
    pub save_nblockMAX: i32,
    pub save_nblock: i32,
    pub save_es: i32,
    pub save_N: i32,
    pub save_curr: i32,
    pub save_zt: i32,
    pub save_zn: i32,
    pub save_zvec: i32,
    pub save_zj: i32,
    pub save_gSel: i32,
    pub save_gMinlen: i32,
    pub save_gLimit: *mut i32,
    pub save_gBase: *mut i32,
    pub save_gPerm: *mut i32,
}
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bzFile {
    pub handle: *mut FILE,
    pub buf: [i8; 5000],
    pub bufN: i32,
    pub writing: Bool,
    pub strm: bz_stream,
    pub lastErr: i32,
    pub initialisedOk: Bool,
}

pub fn BZ2_bz__AssertH__fail(errcode: libc::c_int) {
    eprint!(
        concat!(
            "\n",
            "\n",
            "bzip2/libbzip2: internal error number {}.\n",
            "This is a bug in bzip2/libbzip2, {}.\n",
            "Please report it at: https://gitlab.com/bzip2/bzip2/-/issues\n",
            "If this happened when you were using some program which uses\n",
            "libbzip2 as a component, you should also report this bug to\n",
            "the author(s) of that program.\n",
            "Please make an effort to report this bug;\n",
            "timely and accurate bug reports eventually lead to higher\n",
            "quality software.  Thanks.\n",
            "\n"
        ),
        errcode,
        libbzip2_rs_sys_version!(),
    );
    if errcode == 1007 as libc::c_int {
        eprint!(concat!(
            "\n",
            "*** A special note about internal error number 1007 ***\n",
            "\n",
            "Experience suggests that a common cause of i.e. 1007\n",
            "is unreliable memory or other hardware.  The 1007 assertion\n",
            "just happens to cross-check the results of huge numbers of\n",
            "memory reads/writes, and so acts (unintendedly) as a stress\n",
            "test of your memory system.\n",
            "\n",
            "I suggest the following: try compressing the file again,\n",
            "possibly monitoring progress in detail with the -vv flag.\n",
            "\n",
            "* If the error cannot be reproduced, and/or happens at different\n",
            "  points in compression, you may have a flaky memory system.\n",
            "  Try a memory-test program.  I have used Memtest86\n",
            "  (www.memtest86.com).  At the time of writing it is free (GPLd).\n",
            "  Memtest86 tests memory much more thorougly than your BIOSs\n",
            "  power-on test, and may find failures that the BIOS doesn't.\n",
            "\n",
            "* If the error can be repeatably reproduced, this is a bug in\n",
            "  bzip2, and I would very much like to hear about it.  Please\n",
            "  let me know, and, ideally, save a copy of the file causing the\n",
            "  problem -- without which I will be unable to investigate it.\n",
            "\n"
        ));
    }
    unsafe {
        exit(3 as libc::c_int);
    }
}

const fn bz_config_ok() -> bool {
    if core::mem::size_of::<core::ffi::c_int>() != 4 {
        return false;
    }
    if core::mem::size_of::<core::ffi::c_short>() != 2 {
        return false;
    }
    if core::mem::size_of::<core::ffi::c_char>() != 1 {
        return false;
    }

    true
}

unsafe extern "C" fn default_bzalloc(
    _opaque: *mut libc::c_void,
    items: i32,
    size: i32,
) -> *mut libc::c_void {
    let v: *mut libc::c_void = malloc((items * size) as usize);
    v
}
unsafe extern "C" fn default_bzfree(_opaque: *mut libc::c_void, addr: *mut libc::c_void) {
    if !addr.is_null() {
        free(addr);
    }
}
unsafe fn prepare_new_block(s: &mut EState) {
    s.nblock = 0;
    s.writer.num_z = 0;
    s.state_out_pos = 0;
    s.blockCRC = 0xffffffff;
    s.inUse.fill(false);
    s.blockNo += 1;
}

fn init_RL(s: &mut EState) {
    s.state_in_ch = 256 as libc::c_int as u32;
    s.state_in_len = 0 as libc::c_int;
}

fn isempty_RL(s: &mut EState) -> bool {
    !(s.state_in_ch < 256 && s.state_in_len > 0)
}

#[export_name = prefix!(BZ2_bzCompressInit)]
pub unsafe extern "C" fn BZ2_bzCompressInit(
    strm: *mut bz_stream,
    blockSize100k: libc::c_int,
    verbosity: libc::c_int,
    mut workFactor: libc::c_int,
) -> libc::c_int {
    if !bz_config_ok() {
        return BZ_CONFIG_ERROR as libc::c_int;
    }

    if strm.is_null()
        || blockSize100k < 1
        || blockSize100k > 9
        || workFactor < 0
        || workFactor > 250
    {
        return BZ_PARAM_ERROR as c_int;
    }

    if workFactor == 0 {
        workFactor = 30;
    }

    let bzalloc = (*strm).bzalloc.get_or_insert(default_bzalloc);
    let bzfree = (*strm).bzfree.get_or_insert(default_bzfree);

    let s = (bzalloc)((*strm).opaque, core::mem::size_of::<EState>() as i32, 1) as *mut EState;
    if s.is_null() {
        return BZ_MEM_ERROR as c_int;
    }

    (*s).strm = strm;

    (*s).arr1 = std::ptr::null_mut::<u32>();
    (*s).arr2 = std::ptr::null_mut::<u32>();
    (*s).ftab = std::ptr::null_mut::<u32>();

    let n = 100000 * blockSize100k;

    (*s).arr1 = (bzalloc)(
        (*strm).opaque,
        (n as u64).wrapping_mul(::core::mem::size_of::<u32>() as u64) as i32,
        1,
    ) as *mut u32;
    (*s).arr2 = (bzalloc)(
        (*strm).opaque,
        ((n + (2 + 12 + 18 + 2)) as u64).wrapping_mul(::core::mem::size_of::<u32>() as u64) as i32,
        1,
    ) as *mut u32;
    (*s).ftab = (bzalloc)(
        (*strm).opaque,
        (FTAB_LEN * core::mem::size_of::<u32>()) as i32,
        1,
    ) as *mut u32;

    if ((*s).arr1).is_null() || ((*s).arr2).is_null() || ((*s).ftab).is_null() {
        if !((*s).arr1).is_null() {
            (bzfree)((*strm).opaque, (*s).arr1 as *mut libc::c_void);
        }
        if !((*s).arr2).is_null() {
            (bzfree)((*strm).opaque, (*s).arr2 as *mut libc::c_void);
        }
        if !((*s).ftab).is_null() {
            (bzfree)((*strm).opaque, (*s).ftab as *mut libc::c_void);
        }
        if !s.is_null() {
            (bzfree)((*strm).opaque, s as *mut libc::c_void);
        }
        return BZ_MEM_ERROR as c_int;
    }

    (*s).blockNo = 0;
    (*s).state = State::Output;
    (*s).mode = Mode::Running;
    (*s).combinedCRC = 0;
    (*s).blockSize100k = blockSize100k;
    (*s).nblockMAX = 100000 * blockSize100k - 19;
    (*s).verbosity = verbosity;
    (*s).workFactor = workFactor;

    (*s).block = (*s).arr2 as *mut u8;
    (*s).mtfv = (*s).arr1 as *mut u16;
    (*s).writer.zbits = std::ptr::null_mut::<u8>();
    (*s).ptr = (*s).arr1;

    (*strm).state = s as *mut libc::c_void;

    (*strm).total_in_lo32 = 0;
    (*strm).total_in_hi32 = 0;
    (*strm).total_out_lo32 = 0;
    (*strm).total_out_hi32 = 0;

    init_RL(&mut *s);
    prepare_new_block(&mut *s);

    0
}

macro_rules! BZ_UPDATE_CRC {
    ($crcVar:expr, $cha:expr) => {
        let index = ($crcVar >> 24) ^ ($cha as core::ffi::c_uint);
        $crcVar = ($crcVar << 8) ^ BZ2_CRC32TABLE[index as usize];
    };
}

unsafe fn add_pair_to_block(s: &mut EState) {
    let ch: u8 = s.state_in_ch as u8;

    for _ in 0..s.state_in_len {
        BZ_UPDATE_CRC!(s.blockCRC, ch);
    }

    s.inUse[s.state_in_ch as usize] = true;
    match s.state_in_len {
        1 => {
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
        }
        2 => {
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
        }
        3 => {
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
        }
        _ => {
            s.inUse[(s.state_in_len - 4) as usize] = true;
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
            *(s.block).offset(s.nblock as isize) = ch;
            s.nblock += 1;
            *(s.block).offset(s.nblock as isize) = (s.state_in_len - 4) as u8;
            s.nblock += 1;
        }
    };
}

unsafe fn flush_RL(s: &mut EState) {
    if s.state_in_ch < 256 {
        add_pair_to_block(s);
    }
    init_RL(s);
}

macro_rules! ADD_CHAR_TO_BLOCK {
    ($zs:expr, $zchh0:expr) => {
        let zchh: u32 = $zchh0 as u32;

        if zchh != $zs.state_in_ch && $zs.state_in_len == 1 {
            /*-- fast track the common case --*/

            let ch: u8 = $zs.state_in_ch as u8;
            BZ_UPDATE_CRC!($zs.blockCRC, ch);
            $zs.inUse[$zs.state_in_ch as usize] = true;
            *($zs.block).offset($zs.nblock as isize) = ch;
            $zs.nblock += 1;
            $zs.nblock;
            $zs.state_in_ch = zchh;
        } else if zchh != $zs.state_in_ch || $zs.state_in_len == 255 {
            /*-- general, uncommon cases --*/

            if $zs.state_in_ch < 256 {
                add_pair_to_block($zs);
            }
            $zs.state_in_ch = zchh;
            $zs.state_in_len = 1;
        } else {
            $zs.state_in_len += 1;
        }
    };
}

unsafe fn copy_input_until_stop(strm: &mut bz_stream, s: &mut EState) -> bool {
    let mut progress_in = false;

    match (*s).mode {
        Mode::Running => loop {
            if s.nblock >= s.nblockMAX {
                break;
            }
            if (*strm).avail_in == 0 {
                break;
            }
            progress_in = true;
            ADD_CHAR_TO_BLOCK!(s, *(strm.next_in as *mut u8) as u32);
            strm.next_in = (strm.next_in).offset(1);
            strm.avail_in = (strm.avail_in).wrapping_sub(1);
            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
            if strm.total_in_lo32 == 0 {
                strm.total_in_hi32 = ((*strm).total_in_hi32).wrapping_add(1);
            }
        },
        _ => loop {
            if s.nblock >= s.nblockMAX {
                break;
            }
            if (*strm).avail_in == 0 {
                break;
            }
            if s.avail_in_expect == 0 {
                break;
            }
            progress_in = true;
            ADD_CHAR_TO_BLOCK!(s, *(strm.next_in as *mut u8) as u32);
            strm.next_in = (strm.next_in).offset(1);
            strm.avail_in = (strm.avail_in).wrapping_sub(1);
            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
            if strm.total_in_lo32 == 0 {
                strm.total_in_hi32 = ((*strm).total_in_hi32).wrapping_add(1);
            }
            s.avail_in_expect = (s.avail_in_expect).wrapping_sub(1);
        },
    }
    progress_in
}

unsafe fn copy_output_until_stop(strm: &mut bz_stream, s: &mut EState) -> bool {
    let mut progress_out = false;

    loop {
        if strm.avail_out == 0 {
            break;
        }
        if s.state_out_pos >= s.writer.num_z as i32 {
            break;
        }
        progress_out = true;
        *strm.next_out = *(s.writer.zbits).offset(s.state_out_pos as isize) as libc::c_char;
        s.state_out_pos += 1;
        strm.avail_out = (strm.avail_out).wrapping_sub(1);
        strm.next_out = (strm.next_out).offset(1);
        strm.total_out_lo32 = (strm.total_out_lo32).wrapping_add(1);
        if strm.total_out_lo32 == 0 {
            strm.total_out_hi32 = (strm.total_out_hi32).wrapping_add(1);
        }
    }
    progress_out
}

unsafe fn handle_compress(strm: &mut bz_stream, s: &mut EState) -> bool {
    let mut progress_in = false;
    let mut progress_out = false;

    loop {
        if let State::Input = s.state {
            progress_out |= copy_output_until_stop(strm, s);
            if s.state_out_pos < s.writer.num_z as i32 {
                break;
            }
            if matches!(s.mode, Mode::Finishing) && s.avail_in_expect == 0 && isempty_RL(&mut *s) {
                break;
            }
            prepare_new_block(&mut *s);
            s.state = State::Output;
            if matches!(s.mode, Mode::Flushing) && s.avail_in_expect == 0 && isempty_RL(&mut *s) {
                break;
            }
        }
        if let State::Input = s.state {
            continue;
        }
        progress_in |= copy_input_until_stop(strm, s);
        if !matches!(s.mode, Mode::Running) && s.avail_in_expect == 0 {
            flush_RL(s);
            let is_last_block = matches!(s.mode, Mode::Finishing);
            BZ2_compressBlock(s, is_last_block);
            s.state = State::Input;
        } else if s.nblock >= s.nblockMAX {
            BZ2_compressBlock(s, false);
            s.state = State::Input;
        } else if (*strm).avail_in == 0 {
            break;
        }
    }

    progress_in || progress_out
}

enum Action {
    Run = 0,
    Flush = 1,
    Finish = 2,
}

impl TryFrom<i32> for Action {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Run),
            1 => Ok(Self::Flush),
            2 => Ok(Self::Finish),
            _ => Err(()),
        }
    }
}

#[export_name = prefix!(BZ2_bzCompress)]
pub unsafe extern "C" fn BZ2_bzCompress(strm: *mut bz_stream, action: c_int) -> c_int {
    let Some(strm) = strm.as_mut() else {
        return BZ_PARAM_ERROR as c_int;
    };

    let Some(s) = ((*strm).state as *mut EState).as_mut() else {
        return BZ_PARAM_ERROR as c_int;
    };

    if s.strm != strm {
        return BZ_PARAM_ERROR as c_int;
    }

    BZ2_bzCompressHelp(strm, s, action) as c_int
}

unsafe fn BZ2_bzCompressHelp(strm: &mut bz_stream, s: &mut EState, action: i32) -> ReturnCode {
    loop {
        match s.mode {
            Mode::Idle => return BZ_SEQUENCE_ERROR,
            Mode::Running => match Action::try_from(action) {
                Ok(Action::Run) => {
                    let progress = handle_compress(strm, s);
                    return if progress { BZ_RUN_OK } else { BZ_PARAM_ERROR };
                }
                Ok(Action::Flush) => {
                    s.avail_in_expect = strm.avail_in;
                    s.mode = Mode::Flushing;
                }
                Ok(Action::Finish) => {
                    s.avail_in_expect = strm.avail_in;
                    s.mode = Mode::Finishing;
                }
                Err(()) => {
                    return BZ_PARAM_ERROR;
                }
            },
            Mode::Flushing => {
                let Ok(Action::Flush) = Action::try_from(action) else {
                    return BZ_SEQUENCE_ERROR;
                };
                if s.avail_in_expect != strm.avail_in {
                    return BZ_SEQUENCE_ERROR;
                }
                handle_compress(strm, s);
                if s.avail_in_expect > 0
                    || !isempty_RL(&mut *s)
                    || s.state_out_pos < s.writer.num_z as i32
                {
                    return BZ_FLUSH_OK;
                }
                s.mode = Mode::Running;
                return BZ_RUN_OK;
            }
            Mode::Finishing => {
                let Ok(Action::Finish) = Action::try_from(action) else {
                    return BZ_SEQUENCE_ERROR;
                };
                if s.avail_in_expect != strm.avail_in {
                    return BZ_SEQUENCE_ERROR;
                }
                let progress = handle_compress(strm, s);
                if !progress {
                    return BZ_SEQUENCE_ERROR;
                }
                if s.avail_in_expect > 0
                    || !isempty_RL(s)
                    || s.state_out_pos < s.writer.num_z as i32
                {
                    return BZ_FINISH_OK;
                }
                s.mode = Mode::Idle;
                return BZ_STREAM_END;
            }
        }
    }
}

#[export_name = prefix!(BZ2_bzCompressEnd)]
pub unsafe extern "C" fn BZ2_bzCompressEnd(strm: *mut bz_stream) -> c_int {
    let Some(strm) = strm.as_mut() else {
        return BZ_PARAM_ERROR as c_int;
    };

    let Some(s) = ((*strm).state as *mut EState).as_mut() else {
        return BZ_PARAM_ERROR as c_int;
    };

    if s.strm != strm {
        return BZ_PARAM_ERROR as c_int;
    }

    let Some(bzfree) = (*strm).bzfree else {
        return BZ_PARAM_ERROR as c_int;
    };

    if !(s.arr1).is_null() {
        (bzfree)(strm.opaque, s.arr1.cast::<c_void>());
    }
    if !(s.arr2).is_null() {
        (bzfree)(strm.opaque, s.arr2.cast::<c_void>());
    }
    if !(s.ftab).is_null() {
        (bzfree)(strm.opaque, s.ftab.cast::<c_void>());
    }

    (bzfree)(strm.opaque, strm.state);
    strm.state = std::ptr::null_mut::<libc::c_void>();

    0 as libc::c_int
}

#[export_name = prefix!(BZ2_bzDecompressInit)]
pub unsafe extern "C" fn BZ2_bzDecompressInit(
    strm: *mut bz_stream,
    verbosity: c_int,
    small: c_int,
) -> libc::c_int {
    let s: *mut DState;
    if !bz_config_ok() {
        return BZ_CONFIG_ERROR as libc::c_int;
    }
    if strm.is_null() {
        return BZ_PARAM_ERROR as c_int;
    }
    if small != 0 && small != 1 {
        return BZ_PARAM_ERROR as c_int;
    }
    if verbosity < 0 || verbosity > 4 {
        return BZ_PARAM_ERROR as c_int;
    }
    let bzalloc = (*strm).bzalloc.get_or_insert(default_bzalloc);
    let _bzfree = (*strm).bzfree.get_or_insert(default_bzfree);

    s = (bzalloc)((*strm).opaque, core::mem::size_of::<DState>() as i32, 1) as *mut DState;
    if s.is_null() {
        return BZ_MEM_ERROR as c_int;
    }
    (*s).strm = strm;
    (*strm).state = s as *mut libc::c_void;
    (*s).state = decompress::State::BZ_X_MAGIC_1;
    (*s).bsLive = 0;
    (*s).bsBuff = 0;
    (*s).calculatedCombinedCRC = 0;
    (*strm).total_in_lo32 = 0;
    (*strm).total_in_hi32 = 0;
    (*strm).total_out_lo32 = 0;
    (*strm).total_out_hi32 = 0;
    (*s).smallDecompress = small != 0;
    (*s).ll4 = std::ptr::null_mut::<u8>();
    (*s).ll16 = std::ptr::null_mut::<u16>();
    (*s).tt = std::ptr::null_mut::<u32>();
    (*s).currBlockNo = 0;
    (*s).verbosity = verbosity;

    BZ_OK as libc::c_int
}

unsafe fn unRLE_obuf_to_output_FAST(strm: &mut bz_stream, s: &mut DState) -> bool {
    let mut current_block: u64;
    let mut k1: u8;
    if s.blockRandomised {
        loop {
            loop {
                if strm.avail_out == 0 as libc::c_int as libc::c_uint {
                    return false;
                }
                if s.state_out_len == 0 as libc::c_int {
                    break;
                }
                *(strm.next_out as *mut u8) = s.state_out_ch;
                BZ_UPDATE_CRC!(s.calculatedBlockCRC, s.state_out_ch);
                s.state_out_len -= 1;
                strm.next_out = (strm.next_out).offset(1);
                strm.avail_out = (strm.avail_out).wrapping_sub(1);
                strm.total_out_lo32 = (strm.total_out_lo32).wrapping_add(1);
                if strm.total_out_lo32 == 0 as libc::c_int as libc::c_uint {
                    strm.total_out_hi32 = (strm.total_out_hi32).wrapping_add(1);
                }
            }
            if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                return false;
            }
            if s.nblock_used > s.save_nblock + 1 as libc::c_int {
                return true;
            }
            s.state_out_len = 1 as libc::c_int;
            s.state_out_ch = s.k0 as u8;
            if s.tPos >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32) {
                return true;
            }
            s.tPos = *(s.tt).offset(s.tPos as isize);
            k1 = (s.tPos & 0xff as libc::c_int as libc::c_uint) as u8;
            s.tPos >>= 8 as libc::c_int;
            if s.rNToGo == 0 as libc::c_int {
                s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                s.rTPos += 1;
                if s.rTPos == 512 as libc::c_int {
                    s.rTPos = 0 as libc::c_int;
                }
            }
            s.rNToGo -= 1;
            k1 = (k1 as libc::c_int
                ^ if s.rNToGo == 1 as libc::c_int {
                    1 as libc::c_int
                } else {
                    0 as libc::c_int
                }) as u8;
            s.nblock_used += 1;
            if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                continue;
            }
            if k1 as libc::c_int != s.k0 {
                s.k0 = k1 as i32;
            } else {
                s.state_out_len = 2 as libc::c_int;
                if s.tPos >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32) {
                    return true;
                }
                s.tPos = *(s.tt).offset(s.tPos as isize);
                k1 = (s.tPos & 0xff as libc::c_int as libc::c_uint) as u8;
                s.tPos >>= 8 as libc::c_int;
                if s.rNToGo == 0 as libc::c_int {
                    s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                    s.rTPos += 1;
                    if s.rTPos == 512 as libc::c_int {
                        s.rTPos = 0 as libc::c_int;
                    }
                }
                s.rNToGo -= 1;
                k1 = (k1 as libc::c_int
                    ^ if s.rNToGo == 1 as libc::c_int {
                        1 as libc::c_int
                    } else {
                        0 as libc::c_int
                    }) as u8;
                s.nblock_used += 1;
                if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                    continue;
                }
                if k1 as libc::c_int != s.k0 {
                    s.k0 = k1 as i32;
                } else {
                    s.state_out_len = 3 as libc::c_int;
                    if s.tPos >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32)
                    {
                        return true;
                    }
                    s.tPos = *(s.tt).offset(s.tPos as isize);
                    k1 = (s.tPos & 0xff as libc::c_int as libc::c_uint) as u8;
                    s.tPos >>= 8 as libc::c_int;
                    if s.rNToGo == 0 as libc::c_int {
                        s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                        s.rTPos += 1;
                        if s.rTPos == 512 as libc::c_int {
                            s.rTPos = 0 as libc::c_int;
                        }
                    }
                    s.rNToGo -= 1;
                    k1 = (k1 as libc::c_int
                        ^ if s.rNToGo == 1 as libc::c_int {
                            1 as libc::c_int
                        } else {
                            0 as libc::c_int
                        }) as u8;
                    s.nblock_used += 1;
                    if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                        continue;
                    }
                    if k1 as libc::c_int != s.k0 {
                        s.k0 = k1 as i32;
                    } else {
                        if s.tPos
                            >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32)
                        {
                            return true;
                        }
                        s.tPos = *(s.tt).offset(s.tPos as isize);
                        k1 = (s.tPos & 0xff as libc::c_int as libc::c_uint) as u8;
                        s.tPos >>= 8 as libc::c_int;
                        if s.rNToGo == 0 as libc::c_int {
                            s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                            s.rTPos += 1;
                            if s.rTPos == 512 as libc::c_int {
                                s.rTPos = 0 as libc::c_int;
                            }
                        }
                        s.rNToGo -= 1;
                        k1 = (k1 as libc::c_int
                            ^ if s.rNToGo == 1 as libc::c_int {
                                1 as libc::c_int
                            } else {
                                0 as libc::c_int
                            }) as u8;
                        s.nblock_used += 1;
                        s.state_out_len = k1 as i32 + 4 as libc::c_int;
                        if s.tPos
                            >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32)
                        {
                            return true;
                        }
                        s.tPos = *(s.tt).offset(s.tPos as isize);
                        s.k0 = (s.tPos & 0xff as libc::c_int as libc::c_uint) as u8 as i32;
                        s.tPos >>= 8 as libc::c_int;
                        if s.rNToGo == 0 as libc::c_int {
                            s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                            s.rTPos += 1;
                            if s.rTPos == 512 as libc::c_int {
                                s.rTPos = 0 as libc::c_int;
                            }
                        }
                        s.rNToGo -= 1;
                        s.k0 ^= if s.rNToGo == 1 as libc::c_int {
                            1 as libc::c_int
                        } else {
                            0 as libc::c_int
                        };
                        s.nblock_used += 1;
                    }
                }
            }
        }
    } else {
        let mut c_calculatedBlockCRC: u32 = s.calculatedBlockCRC;
        let mut c_state_out_ch: u8 = s.state_out_ch;
        let mut c_state_out_len: i32 = s.state_out_len;
        let mut c_nblock_used: i32 = s.nblock_used;
        let mut c_k0: i32 = s.k0;
        let c_tt: *mut u32 = s.tt;
        let mut c_tPos: u32 = s.tPos;
        let mut cs_next_out: *mut libc::c_char = strm.next_out;
        let mut cs_avail_out: libc::c_uint = strm.avail_out;
        let ro_blockSize100k: i32 = s.blockSize100k;
        let avail_out_INIT: u32 = cs_avail_out;
        let s_save_nblockPP: i32 = s.save_nblock + 1 as libc::c_int;
        let total_out_lo32_old: libc::c_uint;
        's_453: while 1 as Bool != 0 {
            if c_state_out_len > 0 as libc::c_int {
                loop {
                    if cs_avail_out == 0 as libc::c_int as libc::c_uint {
                        break 's_453;
                    }
                    if c_state_out_len == 1 as libc::c_int {
                        break;
                    }
                    *(cs_next_out as *mut u8) = c_state_out_ch;
                    c_calculatedBlockCRC = c_calculatedBlockCRC << 8 as libc::c_int
                        ^ BZ2_CRC32TABLE[(c_calculatedBlockCRC >> 24 as libc::c_int
                            ^ c_state_out_ch as libc::c_uint)
                            as usize];
                    c_state_out_len -= 1;
                    cs_next_out = cs_next_out.offset(1);
                    cs_avail_out = cs_avail_out.wrapping_sub(1);
                }
                current_block = 1417769144978639029;
            } else {
                current_block = 14483658890531361756;
            }
            loop {
                match current_block {
                    1417769144978639029 => {
                        if cs_avail_out == 0 as libc::c_int as libc::c_uint {
                            c_state_out_len = 1 as libc::c_int;
                            break 's_453;
                        } else {
                            *(cs_next_out as *mut u8) = c_state_out_ch;
                            c_calculatedBlockCRC = c_calculatedBlockCRC << 8 as libc::c_int
                                ^ BZ2_CRC32TABLE[(c_calculatedBlockCRC >> 24 as libc::c_int
                                    ^ c_state_out_ch as libc::c_uint)
                                    as usize];
                            cs_next_out = cs_next_out.offset(1);
                            cs_avail_out = cs_avail_out.wrapping_sub(1);
                            current_block = 14483658890531361756;
                        }
                    }
                    _ => {
                        if c_nblock_used > s_save_nblockPP {
                            return true;
                        }
                        if c_nblock_used == s_save_nblockPP {
                            c_state_out_len = 0 as libc::c_int;
                            break 's_453;
                        } else {
                            c_state_out_ch = c_k0 as u8;
                            if c_tPos
                                >= (100000 as libc::c_int as u32)
                                    .wrapping_mul(ro_blockSize100k as u32)
                            {
                                return true;
                            }
                            c_tPos = *c_tt.offset(c_tPos as isize);
                            k1 = (c_tPos & 0xff as libc::c_int as libc::c_uint) as u8;
                            c_tPos >>= 8 as libc::c_int;
                            c_nblock_used += 1;
                            if k1 as libc::c_int != c_k0 {
                                c_k0 = k1 as i32;
                                current_block = 1417769144978639029;
                            } else {
                                if c_nblock_used == s_save_nblockPP {
                                    current_block = 1417769144978639029;
                                    continue;
                                }
                                c_state_out_len = 2 as libc::c_int;
                                if c_tPos
                                    >= (100000 as libc::c_int as u32)
                                        .wrapping_mul(ro_blockSize100k as u32)
                                {
                                    return true;
                                }
                                c_tPos = *c_tt.offset(c_tPos as isize);
                                k1 = (c_tPos & 0xff as libc::c_int as libc::c_uint) as u8;
                                c_tPos >>= 8 as libc::c_int;
                                c_nblock_used += 1;
                                if c_nblock_used == s_save_nblockPP {
                                    continue 's_453;
                                }
                                if k1 as libc::c_int != c_k0 {
                                    current_block = 6897179874198677617;
                                    break;
                                } else {
                                    current_block = 13256895345714485905;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            match current_block {
                6897179874198677617 => {
                    c_k0 = k1 as i32;
                }
                _ => {
                    c_state_out_len = 3 as libc::c_int;
                    if c_tPos
                        >= (100000 as libc::c_int as u32).wrapping_mul(ro_blockSize100k as u32)
                    {
                        return true;
                    }
                    c_tPos = *c_tt.offset(c_tPos as isize);
                    k1 = (c_tPos & 0xff as libc::c_int as libc::c_uint) as u8;
                    c_tPos >>= 8 as libc::c_int;
                    c_nblock_used += 1;
                    if c_nblock_used == s_save_nblockPP {
                        continue;
                    }
                    if k1 as libc::c_int != c_k0 {
                        c_k0 = k1 as i32;
                    } else {
                        if c_tPos
                            >= (100000 as libc::c_int as u32).wrapping_mul(ro_blockSize100k as u32)
                        {
                            return true;
                        }
                        c_tPos = *c_tt.offset(c_tPos as isize);
                        k1 = (c_tPos & 0xff as libc::c_int as libc::c_uint) as u8;
                        c_tPos >>= 8 as libc::c_int;
                        c_nblock_used += 1;
                        c_state_out_len = k1 as i32 + 4 as libc::c_int;
                        if c_tPos
                            >= (100000 as libc::c_int as u32).wrapping_mul(ro_blockSize100k as u32)
                        {
                            return true;
                        }
                        c_tPos = *c_tt.offset(c_tPos as isize);
                        c_k0 = (c_tPos & 0xff as libc::c_int as libc::c_uint) as u8 as i32;
                        c_tPos >>= 8 as libc::c_int;
                        c_nblock_used += 1;
                    }
                }
            }
        }
        total_out_lo32_old = strm.total_out_lo32;
        strm.total_out_lo32 =
            (strm.total_out_lo32).wrapping_add(avail_out_INIT.wrapping_sub(cs_avail_out));
        if strm.total_out_lo32 < total_out_lo32_old {
            strm.total_out_hi32 = (strm.total_out_hi32).wrapping_add(1);
        }
        s.calculatedBlockCRC = c_calculatedBlockCRC;
        s.state_out_ch = c_state_out_ch;
        s.state_out_len = c_state_out_len;
        s.nblock_used = c_nblock_used;
        s.k0 = c_k0;
        s.tt = c_tt;
        s.tPos = c_tPos;
        strm.next_out = cs_next_out;
        strm.avail_out = cs_avail_out;
    }

    false
}

#[inline]
pub fn BZ2_indexIntoF(indx: i32, cftab: &mut [i32]) -> i32 {
    let mut nb = 0;
    let mut na = 256;
    loop {
        let mid = (nb + na) >> 1;
        if indx >= cftab[mid as usize] {
            nb = mid;
        } else {
            na = mid;
        }
        if na - nb == 1 {
            break;
        }
    }
    nb
}

unsafe fn unRLE_obuf_to_output_SMALL(strm: &mut bz_stream, s: &mut DState) -> bool {
    let mut k1: u8;
    if s.blockRandomised {
        loop {
            loop {
                if strm.avail_out == 0 as libc::c_int as libc::c_uint {
                    return false;
                }
                if s.state_out_len == 0 as libc::c_int {
                    break;
                }
                *(strm.next_out as *mut u8) = s.state_out_ch;
                s.calculatedBlockCRC = s.calculatedBlockCRC << 8 as libc::c_int
                    ^ BZ2_CRC32TABLE[(s.calculatedBlockCRC >> 24 as libc::c_int
                        ^ s.state_out_ch as libc::c_uint)
                        as usize];
                s.state_out_len -= 1;
                strm.next_out = (strm.next_out).offset(1);
                strm.avail_out = (strm.avail_out).wrapping_sub(1);
                strm.total_out_lo32 = (strm.total_out_lo32).wrapping_add(1);
                if strm.total_out_lo32 == 0 as libc::c_int as libc::c_uint {
                    strm.total_out_hi32 = (strm.total_out_hi32).wrapping_add(1);
                }
            }
            if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                return false;
            }
            if s.nblock_used > s.save_nblock + 1 as libc::c_int {
                return true;
            }
            s.state_out_len = 1 as libc::c_int;
            s.state_out_ch = s.k0 as u8;
            if s.tPos >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32) {
                return true;
            }
            k1 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab) as u8;
            s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                    >> (s.tPos << 2 as libc::c_int & 0x4 as libc::c_int as libc::c_uint)
                    & 0xf as libc::c_int as libc::c_uint)
                    << 16 as libc::c_int;
            if s.rNToGo == 0 as libc::c_int {
                s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                s.rTPos += 1;
                if s.rTPos == 512 as libc::c_int {
                    s.rTPos = 0 as libc::c_int;
                }
            }
            s.rNToGo -= 1;
            k1 = (k1 as libc::c_int
                ^ if s.rNToGo == 1 as libc::c_int {
                    1 as libc::c_int
                } else {
                    0 as libc::c_int
                }) as u8;
            s.nblock_used += 1;
            if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                continue;
            }
            if k1 as libc::c_int != s.k0 {
                s.k0 = k1 as i32;
            } else {
                s.state_out_len = 2 as libc::c_int;
                if s.tPos >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32) {
                    return true;
                }
                k1 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab) as u8;
                s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                    | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                        >> (s.tPos << 2 as libc::c_int & 0x4 as libc::c_int as libc::c_uint)
                        & 0xf as libc::c_int as libc::c_uint)
                        << 16 as libc::c_int;
                if s.rNToGo == 0 as libc::c_int {
                    s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                    s.rTPos += 1;
                    if s.rTPos == 512 as libc::c_int {
                        s.rTPos = 0 as libc::c_int;
                    }
                }
                s.rNToGo -= 1;
                k1 = (k1 as libc::c_int
                    ^ if s.rNToGo == 1 as libc::c_int {
                        1 as libc::c_int
                    } else {
                        0 as libc::c_int
                    }) as u8;
                s.nblock_used += 1;
                if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                    continue;
                }
                if k1 as libc::c_int != s.k0 {
                    s.k0 = k1 as i32;
                } else {
                    s.state_out_len = 3 as libc::c_int;
                    if s.tPos >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32)
                    {
                        return true;
                    }
                    k1 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab) as u8;
                    s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                        | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                            >> (s.tPos << 2 as libc::c_int & 0x4 as libc::c_int as libc::c_uint)
                            & 0xf as libc::c_int as libc::c_uint)
                            << 16 as libc::c_int;
                    if s.rNToGo == 0 as libc::c_int {
                        s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                        s.rTPos += 1;
                        if s.rTPos == 512 as libc::c_int {
                            s.rTPos = 0 as libc::c_int;
                        }
                    }
                    s.rNToGo -= 1;
                    k1 = (k1 as libc::c_int
                        ^ if s.rNToGo == 1 as libc::c_int {
                            1 as libc::c_int
                        } else {
                            0 as libc::c_int
                        }) as u8;
                    s.nblock_used += 1;
                    if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                        continue;
                    }
                    if k1 as libc::c_int != s.k0 {
                        s.k0 = k1 as i32;
                    } else {
                        if s.tPos
                            >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32)
                        {
                            return true;
                        }
                        k1 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab) as u8;
                        s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                            | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                                >> (s.tPos << 2 as libc::c_int
                                    & 0x4 as libc::c_int as libc::c_uint)
                                & 0xf as libc::c_int as libc::c_uint)
                                << 16 as libc::c_int;
                        if s.rNToGo == 0 as libc::c_int {
                            s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                            s.rTPos += 1;
                            if s.rTPos == 512 as libc::c_int {
                                s.rTPos = 0 as libc::c_int;
                            }
                        }
                        s.rNToGo -= 1;
                        k1 = (k1 as libc::c_int
                            ^ if s.rNToGo == 1 as libc::c_int {
                                1 as libc::c_int
                            } else {
                                0 as libc::c_int
                            }) as u8;
                        s.nblock_used += 1;
                        s.state_out_len = k1 as i32 + 4 as libc::c_int;
                        if s.tPos
                            >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32)
                        {
                            return true;
                        }
                        s.k0 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab);
                        s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                            | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                                >> (s.tPos << 2 as libc::c_int
                                    & 0x4 as libc::c_int as libc::c_uint)
                                & 0xf as libc::c_int as libc::c_uint)
                                << 16 as libc::c_int;
                        if s.rNToGo == 0 as libc::c_int {
                            s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                            s.rTPos += 1;
                            if s.rTPos == 512 as libc::c_int {
                                s.rTPos = 0 as libc::c_int;
                            }
                        }
                        s.rNToGo -= 1;
                        s.k0 ^= if s.rNToGo == 1 as libc::c_int {
                            1 as libc::c_int
                        } else {
                            0 as libc::c_int
                        };
                        s.nblock_used += 1;
                    }
                }
            }
        }
    } else {
        loop {
            loop {
                if strm.avail_out == 0 as libc::c_int as libc::c_uint {
                    return false;
                }
                if s.state_out_len == 0 as libc::c_int {
                    break;
                }
                *(strm.next_out as *mut u8) = s.state_out_ch;
                s.calculatedBlockCRC = s.calculatedBlockCRC << 8 as libc::c_int
                    ^ BZ2_CRC32TABLE[(s.calculatedBlockCRC >> 24 as libc::c_int
                        ^ s.state_out_ch as libc::c_uint)
                        as usize];
                s.state_out_len -= 1;
                strm.next_out = (strm.next_out).offset(1);
                strm.avail_out = (strm.avail_out).wrapping_sub(1);
                strm.total_out_lo32 = (strm.total_out_lo32).wrapping_add(1);
                if strm.total_out_lo32 == 0 as libc::c_int as libc::c_uint {
                    strm.total_out_hi32 = (strm.total_out_hi32).wrapping_add(1);
                }
            }
            if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                return false;
            }
            if s.nblock_used > s.save_nblock + 1 as libc::c_int {
                return true;
            }
            s.state_out_len = 1 as libc::c_int;
            s.state_out_ch = s.k0 as u8;
            if s.tPos >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32) {
                return true;
            }
            k1 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab) as u8;
            s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                    >> (s.tPos << 2 as libc::c_int & 0x4 as libc::c_int as libc::c_uint)
                    & 0xf as libc::c_int as libc::c_uint)
                    << 16 as libc::c_int;
            s.nblock_used += 1;
            if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                continue;
            }
            if k1 as libc::c_int != s.k0 {
                s.k0 = k1 as i32;
            } else {
                s.state_out_len = 2 as libc::c_int;
                if s.tPos >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32) {
                    return true;
                }
                k1 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab) as u8;
                s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                    | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                        >> (s.tPos << 2 as libc::c_int & 0x4 as libc::c_int as libc::c_uint)
                        & 0xf as libc::c_int as libc::c_uint)
                        << 16 as libc::c_int;
                s.nblock_used += 1;
                if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                    continue;
                }
                if k1 as libc::c_int != s.k0 {
                    s.k0 = k1 as i32;
                } else {
                    s.state_out_len = 3 as libc::c_int;
                    if s.tPos >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32)
                    {
                        return true;
                    }
                    k1 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab) as u8;
                    s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                        | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                            >> (s.tPos << 2 as libc::c_int & 0x4 as libc::c_int as libc::c_uint)
                            & 0xf as libc::c_int as libc::c_uint)
                            << 16 as libc::c_int;
                    s.nblock_used += 1;
                    if s.nblock_used == s.save_nblock + 1 as libc::c_int {
                        continue;
                    }
                    if k1 as libc::c_int != s.k0 {
                        s.k0 = k1 as i32;
                    } else {
                        if s.tPos
                            >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32)
                        {
                            return true;
                        }
                        k1 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab) as u8;
                        s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                            | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                                >> (s.tPos << 2 as libc::c_int
                                    & 0x4 as libc::c_int as libc::c_uint)
                                & 0xf as libc::c_int as libc::c_uint)
                                << 16 as libc::c_int;
                        s.nblock_used += 1;
                        s.state_out_len = k1 as i32 + 4 as libc::c_int;
                        if s.tPos
                            >= (100000 as libc::c_int as u32).wrapping_mul(s.blockSize100k as u32)
                        {
                            return true;
                        }
                        s.k0 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab);
                        s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                            | (*(s.ll4).offset((s.tPos >> 1 as libc::c_int) as isize) as u32
                                >> (s.tPos << 2 as libc::c_int
                                    & 0x4 as libc::c_int as libc::c_uint)
                                & 0xf as libc::c_int as libc::c_uint)
                                << 16 as libc::c_int;
                        s.nblock_used += 1;
                    }
                }
            }
        }
    }
}

#[export_name = prefix!(BZ2_bzDecompress)]
pub unsafe extern "C" fn BZ2_bzDecompress(strm: *mut bz_stream) -> c_int {
    let Some(strm) = strm.as_mut() else {
        return BZ_PARAM_ERROR as c_int;
    };

    let Some(s) = ((*strm).state as *mut DState).as_mut() else {
        return BZ_PARAM_ERROR as c_int;
    };

    if s.strm != strm {
        return BZ_PARAM_ERROR as c_int;
    }

    loop {
        if let decompress::State::BZ_X_IDLE = s.state {
            return -1 as libc::c_int;
        }
        if let decompress::State::BZ_X_OUTPUT = s.state {
            let corrupt = if s.smallDecompress {
                unRLE_obuf_to_output_SMALL(strm, s)
            } else {
                unRLE_obuf_to_output_FAST(strm, s)
            };
            if corrupt {
                return BZ_DATA_ERROR as c_int;
            }
            if s.nblock_used == s.save_nblock + 1 && s.state_out_len == 0 {
                s.calculatedBlockCRC = !s.calculatedBlockCRC;
                if s.verbosity >= 3 {
                    eprint!(
                        " {{{:#08x}, {:#08x}}}",
                        s.storedBlockCRC, s.calculatedBlockCRC,
                    );
                }
                if s.verbosity >= 2 {
                    eprint!("]");
                }
                if s.calculatedBlockCRC != s.storedBlockCRC {
                    return BZ_DATA_ERROR as c_int;
                }
                s.calculatedCombinedCRC =
                    s.calculatedCombinedCRC << 1 | s.calculatedCombinedCRC >> 31;
                s.calculatedCombinedCRC ^= s.calculatedBlockCRC;
                s.state = decompress::State::BZ_X_BLKHDR_1;
            } else {
                return BZ_OK as libc::c_int;
            }
        }
        if !matches!(
            (*s).state,
            decompress::State::BZ_X_IDLE | decompress::State::BZ_X_OUTPUT
        ) {
            let r: i32 = BZ2_decompress(strm, s);
            if r == 4 as libc::c_int {
                if (*s).verbosity >= 3 {
                    eprint!(
                        "\n    combined CRCs: stored = {:#08x}, computed = {:#08x}",
                        (*s).storedCombinedCRC,
                        (*s).calculatedCombinedCRC,
                    );
                }
                if (*s).calculatedCombinedCRC != (*s).storedCombinedCRC {
                    return BZ_DATA_ERROR as c_int;
                }
                return r;
            }
            if !matches!((*s).state, decompress::State::BZ_X_OUTPUT) {
                return r;
            }
        }
    }
}

#[export_name = prefix!(BZ2_bzDecompressEnd)]
pub unsafe extern "C" fn BZ2_bzDecompressEnd(strm: *mut bz_stream) -> libc::c_int {
    let Some(strm) = strm.as_mut() else {
        return BZ_PARAM_ERROR as c_int;
    };

    let Some(s) = ((*strm).state as *mut DState).as_mut() else {
        return BZ_PARAM_ERROR as c_int;
    };

    if s.strm != strm {
        return BZ_PARAM_ERROR as c_int;
    }

    let Some(bzfree) = (*strm).bzfree else {
        return BZ_PARAM_ERROR as c_int;
    };

    if !(s.tt).is_null() {
        (bzfree)(strm.opaque, s.tt.cast::<c_void>());
    }

    if !(s.ll16).is_null() {
        (bzfree)(strm.opaque, s.ll16.cast::<c_void>());
    }

    if !(s.ll4).is_null() {
        (bzfree)(strm.opaque, s.ll4.cast::<c_void>());
    }

    (bzfree)(strm.opaque, strm.state.cast::<c_void>());
    strm.state = std::ptr::null_mut::<libc::c_void>();

    BZ_OK as libc::c_int
}

unsafe fn myfeof(f: *mut FILE) -> bool {
    let c = fgetc(f);
    if c == -1 {
        return true;
    }

    ungetc(c, f);

    false
}

#[export_name = prefix!(BZ2_bzWriteOpen)]
pub unsafe extern "C" fn BZ2_bzWriteOpen(
    bzerror: *mut libc::c_int,
    f: *mut FILE,
    blockSize100k: libc::c_int,
    verbosity: libc::c_int,
    mut workFactor: libc::c_int,
) -> *mut libc::c_void {
    let ret: i32;
    let mut bzf: *mut bzFile = std::ptr::null_mut::<bzFile>();
    if !bzerror.is_null() {
        *bzerror = 0 as libc::c_int;
    }
    if !bzf.is_null() {
        (*bzf).lastErr = 0 as libc::c_int;
    }
    if f.is_null()
        || (blockSize100k < 1 as libc::c_int || blockSize100k > 9 as libc::c_int)
        || (workFactor < 0 as libc::c_int || workFactor > 250 as libc::c_int)
        || (verbosity < 0 as libc::c_int || verbosity > 4 as libc::c_int)
    {
        if !bzerror.is_null() {
            *bzerror = -2 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -2 as libc::c_int;
        }
        return std::ptr::null_mut::<libc::c_void>();
    }
    if ferror(f) != 0 {
        if !bzerror.is_null() {
            *bzerror = -6 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -6 as libc::c_int;
        }
        return std::ptr::null_mut::<libc::c_void>();
    }
    bzf = malloc(core::mem::size_of::<bzFile>() as libc::size_t) as *mut bzFile;
    if bzf.is_null() {
        if !bzerror.is_null() {
            *bzerror = -3 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -3 as libc::c_int;
        }
        return std::ptr::null_mut::<libc::c_void>();
    }
    if !bzerror.is_null() {
        *bzerror = 0 as libc::c_int;
    }
    if !bzf.is_null() {
        (*bzf).lastErr = 0 as libc::c_int;
    }
    (*bzf).initialisedOk = 0 as Bool;
    (*bzf).bufN = 0 as libc::c_int;
    (*bzf).handle = f;
    (*bzf).writing = 1 as Bool;
    (*bzf).strm.bzalloc = None;
    (*bzf).strm.bzfree = None;
    (*bzf).strm.opaque = std::ptr::null_mut::<libc::c_void>();
    if workFactor == 0 as libc::c_int {
        workFactor = 30 as libc::c_int;
    }
    ret = BZ2_bzCompressInit(&mut (*bzf).strm, blockSize100k, verbosity, workFactor);
    if ret != 0 as libc::c_int {
        if !bzerror.is_null() {
            *bzerror = ret;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = ret;
        }
        free(bzf as *mut libc::c_void);
        return std::ptr::null_mut::<libc::c_void>();
    }
    (*bzf).strm.avail_in = 0 as libc::c_int as libc::c_uint;
    (*bzf).initialisedOk = 1 as Bool;
    bzf as *mut libc::c_void
}
#[export_name = prefix!(BZ2_bzWrite)]
pub unsafe extern "C" fn BZ2_bzWrite(
    bzerror: *mut libc::c_int,
    b: *mut libc::c_void,
    buf: *mut libc::c_void,
    len: libc::c_int,
) {
    let mut n: i32;
    let mut n2: i32;
    let mut ret: i32;
    let bzf: *mut bzFile = b as *mut bzFile;
    if !bzerror.is_null() {
        *bzerror = 0 as libc::c_int;
    }
    if !bzf.is_null() {
        (*bzf).lastErr = 0 as libc::c_int;
    }
    if bzf.is_null() || buf.is_null() || len < 0 as libc::c_int {
        if !bzerror.is_null() {
            *bzerror = -2 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -2 as libc::c_int;
        }
        return;
    }
    if (*bzf).writing == 0 {
        if !bzerror.is_null() {
            *bzerror = -1 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -1 as libc::c_int;
        }
        return;
    }
    if ferror((*bzf).handle) != 0 {
        if !bzerror.is_null() {
            *bzerror = -6 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -6 as libc::c_int;
        }
        return;
    }
    if len == 0 as libc::c_int {
        if !bzerror.is_null() {
            *bzerror = 0 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = 0 as libc::c_int;
        }
        return;
    }
    (*bzf).strm.avail_in = len as libc::c_uint;
    (*bzf).strm.next_in = buf as *mut libc::c_char;
    loop {
        (*bzf).strm.avail_out = 5000 as libc::c_int as libc::c_uint;
        (*bzf).strm.next_out = ((*bzf).buf).as_mut_ptr();
        ret = BZ2_bzCompress(&mut (*bzf).strm, 0 as libc::c_int);
        if ret != 1 as libc::c_int {
            if !bzerror.is_null() {
                *bzerror = ret;
            }
            if !bzf.is_null() {
                (*bzf).lastErr = ret;
            }
            return;
        }
        if (*bzf).strm.avail_out < 5000 as libc::c_int as libc::c_uint {
            n = (5000 as libc::c_int as libc::c_uint).wrapping_sub((*bzf).strm.avail_out) as i32;
            n2 = fwrite(
                ((*bzf).buf).as_mut_ptr() as *mut libc::c_void,
                ::core::mem::size_of::<u8>() as libc::size_t,
                n as usize,
                (*bzf).handle,
            ) as i32;
            if n != n2 || ferror((*bzf).handle) != 0 {
                if !bzerror.is_null() {
                    *bzerror = -6 as libc::c_int;
                }
                if !bzf.is_null() {
                    (*bzf).lastErr = -6 as libc::c_int;
                }
                return;
            }
        }
        if (*bzf).strm.avail_in == 0 as libc::c_int as libc::c_uint {
            if !bzerror.is_null() {
                *bzerror = 0 as libc::c_int;
            }
            if !bzf.is_null() {
                (*bzf).lastErr = 0 as libc::c_int;
            }
            return;
        }
    }
}
#[export_name = prefix!(BZ2_bzWriteClose)]
pub unsafe extern "C" fn BZ2_bzWriteClose(
    bzerror: *mut libc::c_int,
    b: *mut libc::c_void,
    abandon: libc::c_int,
    nbytes_in: *mut libc::c_uint,
    nbytes_out: *mut libc::c_uint,
) {
    BZ2_bzWriteClose64(
        bzerror,
        b,
        abandon,
        nbytes_in,
        std::ptr::null_mut::<libc::c_uint>(),
        nbytes_out,
        std::ptr::null_mut::<libc::c_uint>(),
    );
}
#[export_name = prefix!(BZ2_bzWriteClose64)]
pub unsafe extern "C" fn BZ2_bzWriteClose64(
    bzerror: *mut libc::c_int,
    b: *mut libc::c_void,
    abandon: libc::c_int,
    nbytes_in_lo32: *mut libc::c_uint,
    nbytes_in_hi32: *mut libc::c_uint,
    nbytes_out_lo32: *mut libc::c_uint,
    nbytes_out_hi32: *mut libc::c_uint,
) {
    let mut n: i32;
    let mut n2: i32;
    let mut ret: i32;
    let bzf: *mut bzFile = b as *mut bzFile;
    if bzf.is_null() {
        if !bzerror.is_null() {
            *bzerror = 0 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = 0 as libc::c_int;
        }
        return;
    }
    if (*bzf).writing == 0 {
        if !bzerror.is_null() {
            *bzerror = -1 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -1 as libc::c_int;
        }
        return;
    }
    if ferror((*bzf).handle) != 0 {
        if !bzerror.is_null() {
            *bzerror = -6 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -6 as libc::c_int;
        }
        return;
    }
    if !nbytes_in_lo32.is_null() {
        *nbytes_in_lo32 = 0 as libc::c_int as libc::c_uint;
    }
    if !nbytes_in_hi32.is_null() {
        *nbytes_in_hi32 = 0 as libc::c_int as libc::c_uint;
    }
    if !nbytes_out_lo32.is_null() {
        *nbytes_out_lo32 = 0 as libc::c_int as libc::c_uint;
    }
    if !nbytes_out_hi32.is_null() {
        *nbytes_out_hi32 = 0 as libc::c_int as libc::c_uint;
    }
    if abandon == 0 && (*bzf).lastErr == 0 as libc::c_int {
        loop {
            (*bzf).strm.avail_out = 5000 as libc::c_int as libc::c_uint;
            (*bzf).strm.next_out = ((*bzf).buf).as_mut_ptr();
            ret = BZ2_bzCompress(&mut (*bzf).strm, 2 as libc::c_int);
            if ret != 3 as libc::c_int && ret != 4 as libc::c_int {
                if !bzerror.is_null() {
                    *bzerror = ret;
                }
                if !bzf.is_null() {
                    (*bzf).lastErr = ret;
                }
                return;
            }
            if (*bzf).strm.avail_out < 5000 as libc::c_int as libc::c_uint {
                n = (5000 as libc::c_int as libc::c_uint).wrapping_sub((*bzf).strm.avail_out)
                    as i32;
                n2 = fwrite(
                    ((*bzf).buf).as_mut_ptr() as *mut libc::c_void,
                    ::core::mem::size_of::<u8>() as libc::size_t,
                    n as usize,
                    (*bzf).handle,
                ) as i32;
                if n != n2 || ferror((*bzf).handle) != 0 {
                    if !bzerror.is_null() {
                        *bzerror = -6 as libc::c_int;
                    }
                    if !bzf.is_null() {
                        (*bzf).lastErr = -6 as libc::c_int;
                    }
                    return;
                }
            }
            if ret == 4 as libc::c_int {
                break;
            }
        }
    }
    if abandon == 0 && ferror((*bzf).handle) == 0 {
        fflush((*bzf).handle);
        if ferror((*bzf).handle) != 0 {
            if !bzerror.is_null() {
                *bzerror = -6 as libc::c_int;
            }
            if !bzf.is_null() {
                (*bzf).lastErr = -6 as libc::c_int;
            }
            return;
        }
    }
    if !nbytes_in_lo32.is_null() {
        *nbytes_in_lo32 = (*bzf).strm.total_in_lo32;
    }
    if !nbytes_in_hi32.is_null() {
        *nbytes_in_hi32 = (*bzf).strm.total_in_hi32;
    }
    if !nbytes_out_lo32.is_null() {
        *nbytes_out_lo32 = (*bzf).strm.total_out_lo32;
    }
    if !nbytes_out_hi32.is_null() {
        *nbytes_out_hi32 = (*bzf).strm.total_out_hi32;
    }
    if !bzerror.is_null() {
        *bzerror = 0 as libc::c_int;
    }
    if !bzf.is_null() {
        (*bzf).lastErr = 0 as libc::c_int;
    }
    BZ2_bzCompressEnd(&mut (*bzf).strm);
    free(bzf as *mut libc::c_void);
}
#[export_name = prefix!(BZ2_bzReadOpen)]
pub unsafe extern "C" fn BZ2_bzReadOpen(
    bzerror: *mut libc::c_int,
    f: *mut FILE,
    verbosity: libc::c_int,
    small: libc::c_int,
    mut unused: *mut libc::c_void,
    mut nUnused: libc::c_int,
) -> *mut libc::c_void {
    let mut bzf: *mut bzFile = std::ptr::null_mut::<bzFile>();
    let ret: libc::c_int;
    if !bzerror.is_null() {
        *bzerror = 0 as libc::c_int;
    }
    if !bzf.is_null() {
        (*bzf).lastErr = 0 as libc::c_int;
    }
    if f.is_null()
        || small != 0 as libc::c_int && small != 1 as libc::c_int
        || (verbosity < 0 as libc::c_int || verbosity > 4 as libc::c_int)
        || unused.is_null() && nUnused != 0 as libc::c_int
        || !unused.is_null() && (nUnused < 0 as libc::c_int || nUnused > 5000 as libc::c_int)
    {
        if !bzerror.is_null() {
            *bzerror = -2 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -2 as libc::c_int;
        }
        return std::ptr::null_mut::<libc::c_void>();
    }
    if ferror(f) != 0 {
        if !bzerror.is_null() {
            *bzerror = -6 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -6 as libc::c_int;
        }
        return std::ptr::null_mut::<libc::c_void>();
    }
    bzf = malloc(core::mem::size_of::<bzFile>() as libc::size_t) as *mut bzFile;
    if bzf.is_null() {
        if !bzerror.is_null() {
            *bzerror = -3 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -3 as libc::c_int;
        }
        return std::ptr::null_mut::<libc::c_void>();
    }
    if !bzerror.is_null() {
        *bzerror = 0 as libc::c_int;
    }
    if !bzf.is_null() {
        (*bzf).lastErr = 0 as libc::c_int;
    }
    (*bzf).initialisedOk = 0 as Bool;
    (*bzf).handle = f;
    (*bzf).bufN = 0 as libc::c_int;
    (*bzf).writing = 0 as Bool;
    (*bzf).strm.bzalloc = None;
    (*bzf).strm.bzfree = None;
    (*bzf).strm.opaque = std::ptr::null_mut::<libc::c_void>();
    while nUnused > 0 as libc::c_int {
        (*bzf).buf[(*bzf).bufN as usize] = *(unused as *mut u8) as i8;
        (*bzf).bufN += 1;
        unused = (unused as *mut u8).offset(1 as libc::c_int as isize) as *mut libc::c_void;
        nUnused -= 1;
    }
    ret = BZ2_bzDecompressInit(&mut (*bzf).strm, verbosity, small);
    if ret != 0 as libc::c_int {
        if !bzerror.is_null() {
            *bzerror = ret;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = ret;
        }
        free(bzf as *mut libc::c_void);
        return std::ptr::null_mut::<libc::c_void>();
    }
    (*bzf).strm.avail_in = (*bzf).bufN as libc::c_uint;
    (*bzf).strm.next_in = ((*bzf).buf).as_mut_ptr();
    (*bzf).initialisedOk = 1 as Bool;
    bzf as *mut libc::c_void
}
#[export_name = prefix!(BZ2_bzReadClose)]
pub unsafe extern "C" fn BZ2_bzReadClose(bzerror: *mut libc::c_int, b: *mut libc::c_void) {
    let bzf: *mut bzFile = b as *mut bzFile;
    if !bzerror.is_null() {
        *bzerror = 0 as libc::c_int;
    }
    if !bzf.is_null() {
        (*bzf).lastErr = 0 as libc::c_int;
    }
    if bzf.is_null() {
        if !bzerror.is_null() {
            *bzerror = 0 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = 0 as libc::c_int;
        }
        return;
    }
    if (*bzf).writing != 0 {
        if !bzerror.is_null() {
            *bzerror = -1 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -1 as libc::c_int;
        }
        return;
    }
    if (*bzf).initialisedOk != 0 {
        BZ2_bzDecompressEnd(&mut (*bzf).strm);
    }
    free(bzf as *mut libc::c_void);
}
#[export_name = prefix!(BZ2_bzRead)]
pub unsafe extern "C" fn BZ2_bzRead(
    bzerror: *mut libc::c_int,
    b: *mut libc::c_void,
    buf: *mut libc::c_void,
    len: libc::c_int,
) -> libc::c_int {
    let mut n: i32;
    let mut ret: i32;
    let bzf: *mut bzFile = b as *mut bzFile;
    if !bzerror.is_null() {
        *bzerror = 0 as libc::c_int;
    }
    if !bzf.is_null() {
        (*bzf).lastErr = 0 as libc::c_int;
    }
    if bzf.is_null() || buf.is_null() || len < 0 as libc::c_int {
        if !bzerror.is_null() {
            *bzerror = -2 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -2 as libc::c_int;
        }
        return 0 as libc::c_int;
    }
    if (*bzf).writing != 0 {
        if !bzerror.is_null() {
            *bzerror = -1 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -1 as libc::c_int;
        }
        return 0 as libc::c_int;
    }
    if len == 0 as libc::c_int {
        if !bzerror.is_null() {
            *bzerror = 0 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = 0 as libc::c_int;
        }
        return 0 as libc::c_int;
    }
    (*bzf).strm.avail_out = len as libc::c_uint;
    (*bzf).strm.next_out = buf as *mut libc::c_char;
    loop {
        if ferror((*bzf).handle) != 0 {
            if !bzerror.is_null() {
                *bzerror = -6 as libc::c_int;
            }
            if !bzf.is_null() {
                (*bzf).lastErr = -6 as libc::c_int;
            }
            return 0 as libc::c_int;
        }
        if (*bzf).strm.avail_in == 0 as libc::c_int as libc::c_uint && !myfeof((*bzf).handle) {
            n = fread(
                ((*bzf).buf).as_mut_ptr() as *mut libc::c_void,
                ::core::mem::size_of::<u8>() as libc::size_t,
                5000,
                (*bzf).handle,
            ) as i32;
            if ferror((*bzf).handle) != 0 {
                if !bzerror.is_null() {
                    *bzerror = -6 as libc::c_int;
                }
                if !bzf.is_null() {
                    (*bzf).lastErr = -6 as libc::c_int;
                }
                return 0 as libc::c_int;
            }
            (*bzf).bufN = n;
            (*bzf).strm.avail_in = (*bzf).bufN as libc::c_uint;
            (*bzf).strm.next_in = ((*bzf).buf).as_mut_ptr();
        }
        ret = BZ2_bzDecompress(&mut (*bzf).strm);
        if ret != 0 as libc::c_int && ret != 4 as libc::c_int {
            if !bzerror.is_null() {
                *bzerror = ret;
            }
            if !bzf.is_null() {
                (*bzf).lastErr = ret;
            }
            return 0 as libc::c_int;
        }
        if ret == 0 as libc::c_int
            && myfeof((*bzf).handle)
            && (*bzf).strm.avail_in == 0 as libc::c_int as libc::c_uint
            && (*bzf).strm.avail_out > 0 as libc::c_int as libc::c_uint
        {
            if !bzerror.is_null() {
                *bzerror = -7 as libc::c_int;
            }
            if !bzf.is_null() {
                (*bzf).lastErr = -7 as libc::c_int;
            }
            return 0 as libc::c_int;
        }
        if ret == 4 as libc::c_int {
            if !bzerror.is_null() {
                *bzerror = 4 as libc::c_int;
            }
            if !bzf.is_null() {
                (*bzf).lastErr = 4 as libc::c_int;
            }
            return (len as libc::c_uint).wrapping_sub((*bzf).strm.avail_out) as libc::c_int;
        }
        if (*bzf).strm.avail_out == 0 as libc::c_int as libc::c_uint {
            if !bzerror.is_null() {
                *bzerror = 0 as libc::c_int;
            }
            if !bzf.is_null() {
                (*bzf).lastErr = 0 as libc::c_int;
            }
            return len;
        }
    }
}
#[export_name = prefix!(BZ2_bzReadGetUnused)]
pub unsafe extern "C" fn BZ2_bzReadGetUnused(
    bzerror: *mut libc::c_int,
    b: *mut libc::c_void,
    unused: *mut *mut libc::c_void,
    nUnused: *mut libc::c_int,
) {
    let bzf: *mut bzFile = b as *mut bzFile;
    if bzf.is_null() {
        if !bzerror.is_null() {
            *bzerror = -2 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -2 as libc::c_int;
        }
        return;
    }
    if (*bzf).lastErr != 4 as libc::c_int {
        if !bzerror.is_null() {
            *bzerror = -1 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -1 as libc::c_int;
        }
        return;
    }
    if unused.is_null() || nUnused.is_null() {
        if !bzerror.is_null() {
            *bzerror = -2 as libc::c_int;
        }
        if !bzf.is_null() {
            (*bzf).lastErr = -2 as libc::c_int;
        }
        return;
    }
    if !bzerror.is_null() {
        *bzerror = 0 as libc::c_int;
    }
    if !bzf.is_null() {
        (*bzf).lastErr = 0 as libc::c_int;
    }
    *nUnused = (*bzf).strm.avail_in as libc::c_int;
    *unused = (*bzf).strm.next_in as *mut libc::c_void;
}
#[export_name = prefix!(BZ2_bzBuffToBuffCompress)]
pub unsafe extern "C" fn BZ2_bzBuffToBuffCompress(
    dest: *mut libc::c_char,
    destLen: *mut libc::c_uint,
    source: *mut libc::c_char,
    sourceLen: libc::c_uint,
    blockSize100k: libc::c_int,
    verbosity: libc::c_int,
    mut workFactor: libc::c_int,
) -> libc::c_int {
    let mut strm: bz_stream = bz_stream::zeroed();
    let mut ret: libc::c_int;
    if dest.is_null()
        || destLen.is_null()
        || source.is_null()
        || blockSize100k < 1 as libc::c_int
        || blockSize100k > 9 as libc::c_int
        || verbosity < 0 as libc::c_int
        || verbosity > 4 as libc::c_int
        || workFactor < 0 as libc::c_int
        || workFactor > 250 as libc::c_int
    {
        return BZ_PARAM_ERROR as c_int;
    }
    if workFactor == 0 as libc::c_int {
        workFactor = 30 as libc::c_int;
    }
    strm.bzalloc = None;
    strm.bzfree = None;
    strm.opaque = std::ptr::null_mut::<libc::c_void>();
    ret = BZ2_bzCompressInit(&mut strm, blockSize100k, verbosity, workFactor);
    if ret != 0 as libc::c_int {
        return ret;
    }
    strm.next_in = source;
    strm.next_out = dest;
    strm.avail_in = sourceLen;
    strm.avail_out = *destLen;
    ret = BZ2_bzCompress(&mut strm, 2 as libc::c_int);
    if ret == 3 as libc::c_int {
        BZ2_bzCompressEnd(&mut strm);
        -8 as libc::c_int
    } else if ret != 4 as libc::c_int {
        BZ2_bzCompressEnd(&mut strm);
        return ret;
    } else {
        *destLen = (*destLen).wrapping_sub(strm.avail_out);
        BZ2_bzCompressEnd(&mut strm);
        return 0 as libc::c_int;
    }
}
#[export_name = prefix!(BZ2_bzBuffToBuffDecompress)]
pub unsafe extern "C" fn BZ2_bzBuffToBuffDecompress(
    dest: *mut libc::c_char,
    destLen: *mut libc::c_uint,
    source: *mut libc::c_char,
    sourceLen: libc::c_uint,
    small: libc::c_int,
    verbosity: libc::c_int,
) -> libc::c_int {
    if dest.is_null()
        || destLen.is_null()
        || source.is_null()
        || small != 0 as libc::c_int && small != 1 as libc::c_int
        || verbosity < 0 as libc::c_int
        || verbosity > 4 as libc::c_int
    {
        return BZ_PARAM_ERROR as c_int;
    }

    let mut strm: bz_stream = bz_stream::zeroed();

    match BZ2_bzDecompressInit(&mut strm, verbosity, small) {
        0 => {}
        ret => return ret,
    }

    strm.next_in = source;
    strm.next_out = dest;
    strm.avail_in = sourceLen;
    strm.avail_out = *destLen;

    match BZ2_bzDecompress(&mut strm) {
        0 => {
            BZ2_bzDecompressEnd(&mut strm);

            match strm.avail_out {
                0 => BZ_OUTBUFF_FULL as c_int,
                _ => BZ_UNEXPECTED_EOF as c_int,
            }
        }
        4 => {
            *destLen = (*destLen).wrapping_sub(strm.avail_out);
            BZ2_bzDecompressEnd(&mut strm);

            BZ_OK as c_int
        }
        ret => {
            BZ2_bzDecompressEnd(&mut strm);

            ret
        }
    }
}

unsafe fn bzopen_or_bzdopen(
    path: *const libc::c_char,
    fd: libc::c_int,
    mut mode: *const libc::c_char,
    open_mode: libc::c_int,
) -> *mut libc::c_void {
    let mut bzerr: libc::c_int = 0;
    let mut unused: [libc::c_char; 5000] = [0; 5000];
    let mut blockSize100k: libc::c_int = 9 as libc::c_int;
    let mut writing: libc::c_int = 0 as libc::c_int;
    let mut mode2: [libc::c_char; 10] = [0; 10];
    let fp: *mut FILE;
    let bzfp: *mut libc::c_void;
    let verbosity: libc::c_int = 0 as libc::c_int;
    let workFactor: libc::c_int = 30 as libc::c_int;
    let mut smallMode: libc::c_int = 0 as libc::c_int;
    let nUnused: libc::c_int = 0 as libc::c_int;
    if mode.is_null() {
        return std::ptr::null_mut::<libc::c_void>();
    }
    while *mode != 0 {
        match *mode as libc::c_int {
            114 => {
                writing = 0 as libc::c_int;
            }
            119 => {
                writing = 1 as libc::c_int;
            }
            115 => {
                smallMode = 1 as libc::c_int;
            }
            _ => {
                if (*mode as u8 as char).is_ascii_digit() {
                    blockSize100k = (*mode as u8 - 0x30) as libc::c_int;
                }
            }
        }
        mode = mode.offset(1);
    }
    strcat(
        mode2.as_mut_ptr(),
        if writing != 0 {
            b"wb\0" as *const u8 as *const libc::c_char
        } else {
            b"rb\0" as *const u8 as *const libc::c_char
        },
    );
    if open_mode == 0 as libc::c_int {
        strcat(
            mode2.as_mut_ptr(),
            if writing != 0 {
                b"e\0" as *const u8 as *const libc::c_char
            } else {
                b"e\0" as *const u8 as *const libc::c_char
            },
        );
    }
    if open_mode == 0 as libc::c_int {
        if path.is_null()
            || strcmp(path, b"\0" as *const u8 as *const libc::c_char) == 0 as libc::c_int
        {
            fp = if writing != 0 { stdout } else { stdin };
        } else {
            fp = fopen(path, mode2.as_mut_ptr());
        }
    } else {
        fp = fdopen(fd, mode2.as_mut_ptr());
    }
    if fp.is_null() {
        return std::ptr::null_mut::<libc::c_void>();
    }
    if writing != 0 {
        if blockSize100k < 1 as libc::c_int {
            blockSize100k = 1 as libc::c_int;
        }
        if blockSize100k > 9 as libc::c_int {
            blockSize100k = 9 as libc::c_int;
        }
        bzfp = BZ2_bzWriteOpen(&mut bzerr, fp, blockSize100k, verbosity, workFactor);
    } else {
        bzfp = BZ2_bzReadOpen(
            &mut bzerr,
            fp,
            verbosity,
            smallMode,
            unused.as_mut_ptr() as *mut libc::c_void,
            nUnused,
        );
    }
    if bzfp.is_null() {
        if fp != stdin && fp != stdout {
            fclose(fp);
        }
        return std::ptr::null_mut::<libc::c_void>();
    }
    bzfp
}
#[export_name = prefix!(BZ2_bzopen)]
pub unsafe extern "C" fn BZ2_bzopen(
    path: *const libc::c_char,
    mode: *const libc::c_char,
) -> *mut libc::c_void {
    bzopen_or_bzdopen(path, -1 as libc::c_int, mode, 0 as libc::c_int)
}
#[export_name = prefix!(BZ2_bzdopen)]
pub unsafe extern "C" fn BZ2_bzdopen(
    fd: libc::c_int,
    mode: *const libc::c_char,
) -> *mut libc::c_void {
    bzopen_or_bzdopen(std::ptr::null::<libc::c_char>(), fd, mode, 1 as libc::c_int)
}
#[export_name = prefix!(BZ2_bzread)]
pub unsafe extern "C" fn BZ2_bzread(
    b: *mut libc::c_void,
    buf: *mut libc::c_void,
    len: libc::c_int,
) -> libc::c_int {
    let mut bzerr: libc::c_int = 0;
    let nread: libc::c_int;
    if (*(b as *mut bzFile)).lastErr == 4 as libc::c_int {
        return 0 as libc::c_int;
    }
    nread = BZ2_bzRead(&mut bzerr, b, buf, len);
    if bzerr == 0 as libc::c_int || bzerr == 4 as libc::c_int {
        nread
    } else {
        -1 as libc::c_int
    }
}

#[export_name = prefix!(BZ2_bzwrite)]
pub unsafe extern "C" fn BZ2_bzwrite(
    b: *mut libc::c_void,
    buf: *mut libc::c_void,
    len: libc::c_int,
) -> libc::c_int {
    let mut bzerr = 0;
    BZ2_bzWrite(&mut bzerr, b, buf, len);
    if bzerr == 0 {
        len
    } else {
        -1
    }
}

#[export_name = prefix!(BZ2_bzflush)]
pub unsafe extern "C" fn BZ2_bzflush(mut _b: *mut c_void) -> c_int {
    /* do nothing now... */
    0
}

#[export_name = prefix!(BZ2_bzclose)]
pub unsafe extern "C" fn BZ2_bzclose(b: *mut libc::c_void) {
    let mut bzerr: libc::c_int = 0;
    let fp: *mut FILE;
    if b.is_null() {
        return;
    }
    fp = (*(b as *mut bzFile)).handle;
    if (*(b as *mut bzFile)).writing != 0 {
        BZ2_bzWriteClose(
            &mut bzerr,
            b,
            0 as libc::c_int,
            std::ptr::null_mut::<libc::c_uint>(),
            std::ptr::null_mut::<libc::c_uint>(),
        );
        if bzerr != 0 as libc::c_int {
            BZ2_bzWriteClose(
                std::ptr::null_mut::<libc::c_int>(),
                b,
                1 as libc::c_int,
                std::ptr::null_mut::<libc::c_uint>(),
                std::ptr::null_mut::<libc::c_uint>(),
            );
        }
    } else {
        BZ2_bzReadClose(&mut bzerr, b);
    }
    if fp != stdin && fp != stdout {
        fclose(fp);
    }
}

const BZERRORSTRINGS: [&str; 16] = [
    "OK\0",
    "SEQUENCE_ERROR\0",
    "PARAM_ERROR\0",
    "MEM_ERROR\0",
    "DATA_ERROR\0",
    "DATA_ERROR_MAGIC\0",
    "IO_ERROR\0",
    "UNEXPECTED_EOF\0",
    "OUTBUFF_FULL\0",
    "CONFIG_ERROR\0",
    "???\0",
    "???\0",
    "???\0",
    "???\0",
    "???\0",
    "???\0",
];

#[export_name = prefix!(BZ2_bzerror)]
pub unsafe extern "C" fn BZ2_bzerror(b: *mut c_void, errnum: *mut c_int) -> *const c_char {
    let err = Ord::max(0, (*(b as *mut bzFile)).lastErr);
    *errnum = err;
    let msg = BZERRORSTRINGS[(err * -1) as usize];
    msg.as_ptr().cast::<c_char>()
}

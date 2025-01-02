#![forbid(unsafe_code)]

use core::ffi::{c_int, c_uint};

use crate::allocator::Allocator;
use crate::bzlib::{index_into_f, BzStream, DSlice, DState, DecompressMode, ReturnCode};
use crate::randtable::BZ2_RNUMS;
use crate::{debug_log, huffman};

/*-- Constants for the fast MTF decoder. --*/

const MTFA_SIZE: i32 = 4096;
const MTFL_SIZE: i32 = 16;

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub(crate) enum State {
    BZ_X_IDLE = 1,
    BZ_X_OUTPUT = 2,
    BZ_X_MAGIC_1 = 10,
    BZ_X_MAGIC_2 = 11,
    BZ_X_MAGIC_3 = 12,
    BZ_X_MAGIC_4 = 13,
    BZ_X_BLKHDR_1 = 14,
    BZ_X_BLKHDR_2 = 15,
    BZ_X_BLKHDR_3 = 16,
    BZ_X_BLKHDR_4 = 17,
    BZ_X_BLKHDR_5 = 18,
    BZ_X_BLKHDR_6 = 19,
    BZ_X_BCRC_1 = 20,
    BZ_X_BCRC_2 = 21,
    BZ_X_BCRC_3 = 22,
    BZ_X_BCRC_4 = 23,
    BZ_X_RANDBIT = 24,
    BZ_X_ORIGPTR_1 = 25,
    BZ_X_ORIGPTR_2 = 26,
    BZ_X_ORIGPTR_3 = 27,
    BZ_X_MAPPING_1 = 28,
    BZ_X_MAPPING_2 = 29,
    BZ_X_SELECTOR_1 = 30,
    BZ_X_SELECTOR_2 = 31,
    BZ_X_SELECTOR_3 = 32,
    BZ_X_CODING_1 = 33,
    BZ_X_CODING_2 = 34,
    BZ_X_CODING_3 = 35,
    BZ_X_MTF_1 = 36,
    BZ_X_MTF_2 = 37,
    BZ_X_MTF_3 = 38,
    BZ_X_MTF_4 = 39,
    BZ_X_MTF_5 = 40,
    BZ_X_MTF_6 = 41,
    BZ_X_ENDHDR_2 = 42,
    BZ_X_ENDHDR_3 = 43,
    BZ_X_ENDHDR_4 = 44,
    BZ_X_ENDHDR_5 = 45,
    BZ_X_ENDHDR_6 = 46,
    BZ_X_CCRC_1 = 47,
    BZ_X_CCRC_2 = 48,
    BZ_X_CCRC_3 = 49,
    BZ_X_CCRC_4 = 50,
}

#[allow(non_camel_case_types)]
#[derive(Eq, PartialEq)]
enum Block {
    BZ_X_MAGIC_2,
    BZ_X_MAGIC_3,
    BZ_X_MAGIC_4,
    BZ_X_BLKHDR_1,
    BZ_X_BLKHDR_2,
    BZ_X_BLKHDR_3,
    BZ_X_BLKHDR_4,
    BZ_X_BLKHDR_5,
    BZ_X_BLKHDR_6,
    BZ_X_BCRC_1,
    BZ_X_BCRC_2,
    BZ_X_BCRC_3,
    BZ_X_BCRC_4,
    BZ_X_RANDBIT,
    BZ_X_ORIGPTR_1,
    BZ_X_ORIGPTR_2,
    BZ_X_ORIGPTR_3,
    BZ_X_MAPPING_1,
    BZ_X_MAPPING_2,
    BZ_X_SELECTOR_1,
    BZ_X_SELECTOR_2,
    BZ_X_SELECTOR_3,
    BZ_X_CODING_1,
    BZ_X_CODING_2,
    BZ_X_CODING_3,
    BZ_X_MTF_1,
    BZ_X_MTF_2,
    BZ_X_MTF_3,
    BZ_X_MTF_4,
    BZ_X_MTF_5,
    BZ_X_MTF_6,
    BZ_X_ENDHDR_2,
    BZ_X_ENDHDR_3,
    BZ_X_ENDHDR_4,
    BZ_X_ENDHDR_5,
    BZ_X_ENDHDR_6,
    BZ_X_CCRC_1,
    BZ_X_CCRC_2,
    BZ_X_CCRC_3,
    BZ_X_CCRC_4,
    Block1,
    Block11,
    Block18,
    Block24,
    Block25,
    Block26,
    Block28,
    Block35,
    Block39,
    Block40,
    Block41,
    Block43,
    Block45,
    Block46,
    Block51,
    Block52,
    Block56,
    Block58,
}
use Block::*;

fn make_maps_d(s: &mut DState) {
    s.nInUse = 0;
    for (i, in_use) in s.inUse.iter().enumerate() {
        if *in_use {
            s.seqToUnseq[s.nInUse as usize] = i as u8;
            s.nInUse += 1;
        }
    }
}

trait GetBitsConvert {
    fn convert(x: u32) -> Self;
}

impl GetBitsConvert for bool {
    fn convert(x: u32) -> Self {
        x != 0
    }
}

impl GetBitsConvert for u8 {
    fn convert(x: u32) -> Self {
        x as u8
    }
}

impl GetBitsConvert for i32 {
    fn convert(x: u32) -> Self {
        x as i32
    }
}

pub(crate) fn decompress(
    strm: &mut BzStream<DState>,
    s: &mut DState,
    allocator: &Allocator,
) -> ReturnCode {
    let mut current_block: Block;
    let mut uc: u8;
    let mut minLen: i32;
    let mut maxLen: i32;
    let mut i: i32;
    let mut j: i32;
    let mut t: i32;
    let mut alphaSize: i32;
    let mut nGroups: i32;
    let mut nSelectors: i32;
    let mut EOB: i32;
    let mut groupNo: i32;
    let mut groupPos: i32;
    let mut nextSym: i32;
    let mut nblockMAX: i32;
    let mut nblock: i32;
    let mut es: i32;
    let mut N: i32;
    let mut curr: i32;

    let mut zn: i32;
    let mut zvec: i32;
    let mut zj: i32;
    let mut gSel: i32;
    let mut gMinlen: i32;
    let mut gLimit: i32;
    let mut gBase: i32;
    let mut gPerm: i32;

    if let State::BZ_X_MAGIC_1 = s.state {
        /*initialise the save area*/
        s.save_i = 0;
        s.save_j = 0;
        s.save_t = 0;
        s.save_alphaSize = 0;
        s.save_nGroups = 0;
        s.save_nSelectors = 0;
        s.save_EOB = 0;
        s.save_groupNo = 0;
        s.save_groupPos = 0;
        s.save_nextSym = 0;
        s.save_nblockMAX = 0;
        s.save_nblock = 0;
        s.save_es = 0;
        s.save_N = 0;
        s.save_curr = 0;
        s.save_zt = 0;
        s.save_zn = 0;
        s.save_zvec = 0;
        s.save_zj = 0;
        s.save_gSel = 0;
        s.save_gMinlen = 0;
        s.save_gLimit = 0;
        s.save_gBase = 0;
        s.save_gPerm = 0;
    }

    /*restore from the save area*/
    i = s.save_i;
    j = s.save_j;
    t = s.save_t;
    alphaSize = s.save_alphaSize;
    nGroups = s.save_nGroups;
    nSelectors = s.save_nSelectors;
    EOB = s.save_EOB;
    groupNo = s.save_groupNo;
    groupPos = s.save_groupPos;
    nextSym = s.save_nextSym;
    nblockMAX = s.save_nblockMAX;
    nblock = s.save_nblock;
    es = s.save_es;
    N = s.save_N;
    curr = s.save_curr;
    let zt: i32 = s.save_zt;
    zn = s.save_zn;
    zvec = s.save_zvec;
    zj = s.save_zj;
    gSel = s.save_gSel;
    gMinlen = s.save_gMinlen;
    gLimit = s.save_gLimit;
    gBase = s.save_gBase;
    gPerm = s.save_gPerm;

    let ret_val: ReturnCode = 'save_state_and_return: {
        macro_rules! GET_UCHAR {
            ($strm:expr, $s:expr, $uuu:expr) => {
                GET_BITS!($strm, $s, $uuu, 8);
            };
        }

        macro_rules! GET_BIT {
            ($strm:expr, $s:expr, $uuu:expr) => {
                GET_BITS!($strm, $s, $uuu, 1);
            };
        }

        macro_rules! GET_BITS {
            ($strm:expr, $s:expr, $vvv:expr, $nnn:expr) => {
                loop {
                    if $s.bsLive >= $nnn {
                        let v: u32 = ($s.bsBuff >> ($s.bsLive - $nnn)) & ((1 << $nnn) - 1);
                        $s.bsLive -= $nnn;
                        $vvv = GetBitsConvert::convert(v);
                        break;
                    }

                    if let Some(next_byte) = strm.read_byte() {
                        $s.bsBuff = $s.bsBuff << 8 | next_byte as u32;
                        $s.bsLive += 8;
                    } else {
                        break 'save_state_and_return ReturnCode::BZ_OK;
                    }
                }
            };
        }

        macro_rules! update_group_pos {
            ($s:expr) => {
                if groupPos == 0 {
                    groupNo += 1;
                    if groupNo >= nSelectors {
                        error!(BZ_DATA_ERROR);
                    } else {
                        groupPos = 50;
                        gSel = $s.selector[groupNo as usize] as i32;
                        gMinlen = $s.minLens[gSel as usize];
                        gLimit = gSel;
                        gPerm = gSel;
                        gBase = gSel;
                    }
                }
                groupPos -= 1;
            };
        }

        macro_rules! error {
            ($code:ident) => {
                break 'save_state_and_return ReturnCode::$code;
            };
        }

        match s.state {
            State::BZ_X_MAGIC_1 => {
                s.state = State::BZ_X_MAGIC_1;

                GET_UCHAR!(strm, s, uc);

                if uc != b'B' {
                    error!(BZ_DATA_ERROR_MAGIC);
                }

                current_block = BZ_X_MAGIC_2;
            }
            State::BZ_X_MAGIC_2 => current_block = BZ_X_MAGIC_2,
            State::BZ_X_MAGIC_3 => current_block = BZ_X_MAGIC_3,
            State::BZ_X_MAGIC_4 => current_block = BZ_X_MAGIC_4,
            State::BZ_X_BLKHDR_1 => current_block = BZ_X_BLKHDR_1,
            State::BZ_X_BLKHDR_2 => current_block = BZ_X_BLKHDR_2,
            State::BZ_X_BLKHDR_3 => current_block = BZ_X_BLKHDR_3,
            State::BZ_X_BLKHDR_4 => current_block = BZ_X_BLKHDR_4,
            State::BZ_X_BLKHDR_5 => current_block = BZ_X_BLKHDR_5,
            State::BZ_X_BLKHDR_6 => current_block = BZ_X_BLKHDR_6,
            State::BZ_X_BCRC_1 => current_block = BZ_X_BCRC_1,
            State::BZ_X_BCRC_2 => current_block = BZ_X_BCRC_2,
            State::BZ_X_BCRC_3 => current_block = BZ_X_BCRC_3,
            State::BZ_X_BCRC_4 => current_block = BZ_X_BCRC_4,
            State::BZ_X_RANDBIT => current_block = BZ_X_RANDBIT,
            State::BZ_X_ORIGPTR_1 => current_block = BZ_X_ORIGPTR_1,
            State::BZ_X_ORIGPTR_2 => current_block = BZ_X_ORIGPTR_2,
            State::BZ_X_ORIGPTR_3 => current_block = BZ_X_ORIGPTR_3,
            State::BZ_X_MAPPING_1 => current_block = BZ_X_MAPPING_1,
            State::BZ_X_MAPPING_2 => current_block = BZ_X_MAPPING_2,
            State::BZ_X_SELECTOR_1 => current_block = BZ_X_SELECTOR_1,
            State::BZ_X_SELECTOR_2 => current_block = BZ_X_SELECTOR_2,
            State::BZ_X_SELECTOR_3 => current_block = BZ_X_SELECTOR_3,
            State::BZ_X_CODING_1 => current_block = BZ_X_CODING_1,
            State::BZ_X_CODING_2 => current_block = BZ_X_CODING_2,
            State::BZ_X_CODING_3 => current_block = BZ_X_CODING_3,
            State::BZ_X_MTF_1 => current_block = BZ_X_MTF_1,
            State::BZ_X_MTF_2 => current_block = BZ_X_MTF_2,
            State::BZ_X_MTF_3 => current_block = BZ_X_MTF_3,
            State::BZ_X_MTF_4 => current_block = BZ_X_MTF_4,
            State::BZ_X_MTF_5 => current_block = BZ_X_MTF_5,
            State::BZ_X_MTF_6 => current_block = BZ_X_MTF_6,
            State::BZ_X_ENDHDR_2 => current_block = BZ_X_ENDHDR_2,
            State::BZ_X_ENDHDR_3 => current_block = BZ_X_ENDHDR_3,
            State::BZ_X_ENDHDR_4 => current_block = BZ_X_ENDHDR_4,
            State::BZ_X_ENDHDR_5 => current_block = BZ_X_ENDHDR_5,
            State::BZ_X_ENDHDR_6 => current_block = BZ_X_ENDHDR_6,
            State::BZ_X_CCRC_1 => current_block = BZ_X_CCRC_1,
            State::BZ_X_CCRC_2 => current_block = BZ_X_CCRC_2,
            State::BZ_X_CCRC_3 => current_block = BZ_X_CCRC_3,
            State::BZ_X_CCRC_4 => current_block = BZ_X_CCRC_4,
            State::BZ_X_IDLE | State::BZ_X_OUTPUT => unreachable!(),
        }
        if current_block == BZ_X_MAGIC_2 {
            s.state = State::BZ_X_MAGIC_2;

            GET_UCHAR!(strm, s, uc);

            if uc != b'Z' {
                error!(BZ_DATA_ERROR_MAGIC);
            }

            current_block = BZ_X_MAGIC_3;
        }
        if current_block == BZ_X_MAGIC_3 {
            s.state = State::BZ_X_MAGIC_3;

            GET_UCHAR!(strm, s, uc);

            if uc != b'h' {
                error!(BZ_DATA_ERROR_MAGIC);
            }

            current_block = BZ_X_MAGIC_4;
        }
        if current_block == BZ_X_MAGIC_4 {
            s.state = State::BZ_X_MAGIC_4;

            GET_BITS!(strm, s, s.blockSize100k, 8);

            if !(b'1' as i32..=b'9' as i32).contains(&s.blockSize100k) {
                error!(BZ_DATA_ERROR_MAGIC);
            }

            s.blockSize100k -= b'0' as i32;

            match s.smallDecompress {
                DecompressMode::Small => {
                    // SAFETY: we assume allocation is safe
                    let ll16_len = s.blockSize100k as usize * 100000;
                    let Some(ll16) = DSlice::alloc(allocator, ll16_len) else {
                        error!(BZ_MEM_ERROR);
                    };

                    // SAFETY: we assume allocation is safe
                    let ll4_len = (1 + s.blockSize100k as usize * 100000) >> 1;
                    let Some(ll4) = DSlice::alloc(allocator, ll4_len) else {
                        error!(BZ_MEM_ERROR);
                    };

                    s.ll16 = ll16;
                    s.ll4 = ll4;
                }
                DecompressMode::Fast => {
                    // SAFETY: we assume allocation is safe
                    let tt_len = s.blockSize100k as usize * 100000;
                    let Some(tt) = DSlice::alloc(allocator, tt_len) else {
                        error!(BZ_MEM_ERROR);
                    };

                    s.tt = tt;
                }
            }

            current_block = BZ_X_BLKHDR_1;
        }
        if current_block == BZ_X_BLKHDR_1 {
            s.state = State::BZ_X_BLKHDR_1;

            GET_UCHAR!(strm, s, uc);

            if uc == 0x17 {
                // skips to `State::BZ_X_ENDHDR_2`
                current_block = BZ_X_ENDHDR_2;
            } else if uc != 0x31 {
                error!(BZ_DATA_ERROR);
            } else {
                current_block = BZ_X_BLKHDR_2;
            }
        }
        match current_block {
            BZ_X_ENDHDR_2 => {
                s.state = State::BZ_X_ENDHDR_2;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x72 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_ENDHDR_3;
            }
            BZ_X_BLKHDR_2 => {
                s.state = State::BZ_X_BLKHDR_2;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x41 {
                    error!(BZ_DATA_ERROR);
                }
                current_block = BZ_X_BLKHDR_3;
            }
            _ => {}
        }
        match current_block {
            BZ_X_ENDHDR_3 => {
                s.state = State::BZ_X_ENDHDR_3;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x45 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_ENDHDR_4;
            }
            BZ_X_BLKHDR_3 => {
                s.state = State::BZ_X_BLKHDR_3;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x59 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_BLKHDR_4;
            }
            _ => {}
        }
        match current_block {
            BZ_X_ENDHDR_4 => {
                s.state = State::BZ_X_ENDHDR_4;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x38 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_ENDHDR_5;
            }
            BZ_X_BLKHDR_4 => {
                s.state = State::BZ_X_BLKHDR_4;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x26 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_BLKHDR_5;
            }
            _ => {}
        }
        match current_block {
            BZ_X_ENDHDR_5 => {
                s.state = State::BZ_X_ENDHDR_5;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x50 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_ENDHDR_6;
            }
            BZ_X_BLKHDR_5 => {
                s.state = State::BZ_X_BLKHDR_5;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x53 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_BLKHDR_6;
            }
            _ => {}
        }
        match current_block {
            BZ_X_ENDHDR_6 => {
                s.state = State::BZ_X_ENDHDR_6;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x90 {
                    error!(BZ_DATA_ERROR);
                }

                s.storedCombinedCRC = 0_u32;
                current_block = BZ_X_CCRC_1;
            }
            BZ_X_BLKHDR_6 => {
                s.state = State::BZ_X_BLKHDR_6;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x59 {
                    error!(BZ_DATA_ERROR);
                }

                s.currBlockNo += 1;
                if s.verbosity >= 2 {
                    debug_log!("\n    [{}: huff+mtf ", s.currBlockNo);
                }
                s.storedBlockCRC = 0_u32;
                current_block = BZ_X_BCRC_1;
            }
            _ => {}
        }
        match current_block {
            BZ_X_CCRC_1 => {
                s.state = State::BZ_X_CCRC_1;

                GET_UCHAR!(strm, s, uc);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                current_block = BZ_X_CCRC_2;
            }
            BZ_X_BCRC_1 => {
                s.state = State::BZ_X_BCRC_1;

                GET_UCHAR!(strm, s, uc);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = BZ_X_BCRC_2;
            }
            _ => {}
        }
        match current_block {
            BZ_X_CCRC_2 => {
                s.state = State::BZ_X_CCRC_2;

                GET_UCHAR!(strm, s, uc);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                current_block = BZ_X_CCRC_3;
            }
            BZ_X_BCRC_2 => {
                s.state = State::BZ_X_BCRC_2;

                GET_UCHAR!(strm, s, uc);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = BZ_X_BCRC_3;
            }
            _ => {}
        }
        match current_block {
            BZ_X_CCRC_3 => {
                s.state = State::BZ_X_CCRC_3;

                GET_UCHAR!(strm, s, uc);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                current_block = BZ_X_CCRC_4;
            }
            BZ_X_BCRC_3 => {
                s.state = State::BZ_X_BCRC_3;

                GET_UCHAR!(strm, s, uc);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = BZ_X_BCRC_4;
            }
            _ => {}
        }
        match current_block {
            BZ_X_BCRC_4 => {
                s.state = State::BZ_X_BCRC_4;

                GET_UCHAR!(strm, s, uc);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = BZ_X_RANDBIT;
            }
            BZ_X_CCRC_4 => {
                s.state = State::BZ_X_CCRC_4;

                GET_UCHAR!(strm, s, uc);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                s.state = State::BZ_X_IDLE;
                error!(BZ_STREAM_END);
            }
            _ => {}
        }
        if current_block == BZ_X_RANDBIT {
            s.state = State::BZ_X_RANDBIT;

            GET_BITS!(strm, s, s.blockRandomised, 1);

            s.origPtr = 0;
            current_block = BZ_X_ORIGPTR_1;
        }
        if current_block == BZ_X_ORIGPTR_1 {
            s.state = State::BZ_X_ORIGPTR_1;

            GET_UCHAR!(strm, s, uc);

            s.origPtr = s.origPtr << 8 | uc as i32;
            current_block = BZ_X_ORIGPTR_2;
        }
        if current_block == BZ_X_ORIGPTR_2 {
            s.state = State::BZ_X_ORIGPTR_2;

            GET_UCHAR!(strm, s, uc);

            s.origPtr = s.origPtr << 8 | uc as i32;
            current_block = BZ_X_ORIGPTR_3;
        }
        if current_block == BZ_X_ORIGPTR_3 {
            s.state = State::BZ_X_ORIGPTR_3;

            GET_UCHAR!(strm, s, uc);

            s.origPtr = s.origPtr << 8 | uc as i32;
            if !(0..10 + 100000 * s.blockSize100k).contains(&s.origPtr) {
                error!(BZ_DATA_ERROR);
            }

            i = 0;
            current_block = Block43;
        }

        // mutable because they need to be reborrowed
        let mut tt = s.tt.as_mut_slice();
        let mut ll16 = s.ll16.as_mut_slice();
        let mut ll4 = s.ll4.as_mut_slice();

        'c_10064: loop {
            match current_block {
                BZ_X_MAPPING_1 => {
                    s.state = State::BZ_X_MAPPING_1;

                    GET_BIT!(strm, s, uc);

                    s.inUse16[i as usize] = uc == 1;
                    i += 1;
                    current_block = Block43;
                    continue;
                }
                Block43 => {
                    if i < 16 {
                        current_block = BZ_X_MAPPING_1;
                        continue;
                    }
                    i = 0;
                    while i < 256 {
                        s.inUse[i as usize] = false;
                        i += 1;
                    }
                    i = 0;
                    current_block = Block18;
                }
                BZ_X_MAPPING_2 => {
                    s.state = State::BZ_X_MAPPING_2;

                    GET_BIT!(strm, s, uc);

                    if uc == 1 {
                        s.inUse[(i * 16 + j) as usize] = true;
                    }
                    j += 1;
                    current_block = Block28;
                }
                BZ_X_SELECTOR_1 => {
                    s.state = State::BZ_X_SELECTOR_1;

                    GET_BITS!(strm, s, nGroups, 3);

                    if (2..=6).contains(&nGroups) {
                        current_block = BZ_X_SELECTOR_2;
                        continue;
                    }
                    error!(BZ_DATA_ERROR);
                }
                BZ_X_SELECTOR_2 => {
                    s.state = State::BZ_X_SELECTOR_2;

                    GET_BITS!(strm, s, nSelectors, 15);

                    if nSelectors < 1 {
                        error!(BZ_DATA_ERROR);
                    } else {
                        i = 0;
                    }
                    current_block = Block39;
                }
                BZ_X_SELECTOR_3 => {
                    s.state = State::BZ_X_SELECTOR_3;

                    GET_BIT!(strm, s, uc);

                    if uc == 0 {
                        current_block = Block1;
                    } else {
                        j += 1;
                        if j >= nGroups {
                            error!(BZ_DATA_ERROR);
                        } else {
                            current_block = Block25;
                        }
                    }
                }
                BZ_X_CODING_1 => {
                    s.state = State::BZ_X_CODING_1;

                    GET_BITS!(strm, s, curr, 5);

                    i = 0;
                    current_block = Block26;
                }
                BZ_X_CODING_2 => {
                    s.state = State::BZ_X_CODING_2;

                    GET_BIT!(strm, s, uc);

                    if uc != 0 {
                        current_block = BZ_X_CODING_3;
                        continue;
                    }
                    current_block = Block51;
                }
                BZ_X_CODING_3 => {
                    s.state = State::BZ_X_CODING_3;

                    GET_BIT!(strm, s, uc);

                    if uc == 0 {
                        curr += 1;
                    } else {
                        curr -= 1;
                    }

                    current_block = Block45;
                }
                BZ_X_MTF_1 => {
                    s.state = State::BZ_X_MTF_1;

                    GET_BITS!(strm, s, zvec, zn);

                    current_block = Block56;
                }
                BZ_X_MTF_2 => {
                    s.state = State::BZ_X_MTF_2;

                    GET_BIT!(strm, s, zj);

                    zvec = zvec << 1 | zj;
                    current_block = Block56;
                }
                BZ_X_MTF_3 => {
                    s.state = State::BZ_X_MTF_3;

                    GET_BITS!(strm, s, zvec, zn);

                    current_block = Block52;
                }
                BZ_X_MTF_4 => {
                    s.state = State::BZ_X_MTF_4;

                    GET_BIT!(strm, s, zj);

                    zvec = zvec << 1 | zj;
                    current_block = Block52;
                }
                BZ_X_MTF_5 => {
                    s.state = State::BZ_X_MTF_5;

                    GET_BITS!(strm, s, zvec, zn);

                    current_block = Block24;
                }
                _ => {
                    s.state = State::BZ_X_MTF_6;

                    GET_BIT!(strm, s, zj);

                    zvec = zvec << 1 | zj;
                    current_block = Block24;
                }
            }
            match current_block {
                Block24 => {
                    if zn > 20 {
                        error!(BZ_DATA_ERROR);
                    } else if zvec <= s.limit[gLimit as usize][zn as usize] {
                        if !(0..258).contains(&(zvec - s.base[gBase as usize][zn as usize])) {
                            error!(BZ_DATA_ERROR);
                        } else {
                            nextSym = s.perm[gPerm as usize]
                                [(zvec - s.base[gBase as usize][zn as usize]) as usize];
                        }
                    } else {
                        zn += 1;
                        current_block = BZ_X_MTF_6;
                        continue;
                    }
                    current_block = Block40;
                }
                Block52 => {
                    if zn > 20 {
                        error!(BZ_DATA_ERROR);
                    } else if zvec <= s.limit[gLimit as usize][zn as usize] {
                        if !(0..258).contains(&(zvec - s.base[gBase as usize][zn as usize])) {
                            error!(BZ_DATA_ERROR);
                        } else {
                            nextSym = s.perm[gPerm as usize]
                                [(zvec - s.base[gBase as usize][zn as usize]) as usize];
                            if nextSym == 0 || nextSym == 1 {
                                current_block = Block46;
                            } else {
                                es += 1;
                                uc = s.seqToUnseq[s.mtfa[s.mtfbase[0_usize] as usize] as usize];
                                s.unzftab[uc as usize] += es;
                                match s.smallDecompress {
                                    DecompressMode::Small => {
                                        while es > 0 {
                                            if nblock >= nblockMAX {
                                                error!(BZ_DATA_ERROR);
                                            } else {
                                                ll16[nblock as usize] = uc as u16;
                                                nblock += 1;
                                                es -= 1;
                                            }
                                        }
                                    }
                                    DecompressMode::Fast => {
                                        while es > 0 {
                                            if nblock >= nblockMAX {
                                                error!(BZ_DATA_ERROR);
                                            } else {
                                                tt[nblock as usize] = uc as u32;
                                                nblock += 1;
                                                es -= 1;
                                            }
                                        }
                                    }
                                }
                                current_block = Block40;
                            }
                        }
                    } else {
                        zn += 1;
                        current_block = BZ_X_MTF_4;
                        continue;
                    }
                }
                Block56 => {
                    if zn > 20 {
                        error!(BZ_DATA_ERROR);
                    } else if zvec <= s.limit[gLimit as usize][zn as usize] {
                        if !(0..258).contains(&(zvec - s.base[gBase as usize][zn as usize])) {
                            error!(BZ_DATA_ERROR);
                        } else {
                            nextSym = s.perm[gPerm as usize]
                                [(zvec - s.base[gBase as usize][zn as usize]) as usize];
                        }
                    } else {
                        zn += 1;
                        current_block = BZ_X_MTF_2;
                        continue;
                    }
                    current_block = Block40;
                }
                _ => {}
            }
            if current_block == Block40 {
                if nextSym == EOB {
                    current_block = Block41;
                } else {
                    if nextSym == 0 || nextSym == 1 {
                        es = -1;
                        N = 1;
                    } else if nblock >= nblockMAX {
                        error!(BZ_DATA_ERROR);
                    } else {
                        let mut ii_0: i32;
                        let mut jj_0: i32;
                        let mut kk_0: i32;
                        let mut pp: i32;
                        let mut lno: i32;
                        let off: i32;
                        let mut nn: u32;
                        nn = (nextSym - 1) as u32;
                        if nn < 16 {
                            pp = s.mtfbase[0_usize];
                            uc = s.mtfa[(pp as c_uint).wrapping_add(nn) as usize];
                            while nn > 3 {
                                let z: i32 = (pp as c_uint).wrapping_add(nn) as i32;
                                s.mtfa[z as usize] = s.mtfa[(z - 1) as usize];
                                s.mtfa[(z - 1) as usize] = s.mtfa[(z - 2) as usize];
                                s.mtfa[(z - 2) as usize] = s.mtfa[(z - 3) as usize];
                                s.mtfa[(z - 3) as usize] = s.mtfa[(z - 4) as usize];
                                nn = (nn).wrapping_sub(4);
                            }
                            while nn > 0 {
                                s.mtfa[(pp as c_uint).wrapping_add(nn) as usize] = s.mtfa
                                    [(pp as c_uint).wrapping_add(nn).wrapping_sub(1) as usize];
                                nn = nn.wrapping_sub(1);
                            }
                            s.mtfa[pp as usize] = uc;
                        } else {
                            lno = nn.wrapping_div(16) as i32;
                            off = nn.wrapping_rem(16) as i32;
                            pp = s.mtfbase[lno as usize] + off;
                            uc = s.mtfa[pp as usize];
                            while pp > s.mtfbase[lno as usize] {
                                s.mtfa[pp as usize] = s.mtfa[(pp - 1) as usize];
                                pp -= 1;
                            }
                            s.mtfbase[lno as usize] += 1;
                            while lno > 0 {
                                s.mtfbase[lno as usize] -= 1;
                                s.mtfa[s.mtfbase[lno as usize] as usize] =
                                    s.mtfa[(s.mtfbase[(lno - 1) as usize] + 16 - 1) as usize];
                                lno -= 1;
                            }
                            s.mtfbase[0_usize] -= 1;
                            s.mtfa[s.mtfbase[0_usize] as usize] = uc;
                            if s.mtfbase[0_usize] == 0 {
                                kk_0 = 4096 - 1;
                                ii_0 = 256 / 16 - 1;
                                while ii_0 >= 0 {
                                    jj_0 = 16 - 1;
                                    while jj_0 >= 0 {
                                        s.mtfa[kk_0 as usize] =
                                            s.mtfa[(s.mtfbase[ii_0 as usize] + jj_0) as usize];
                                        kk_0 -= 1;
                                        jj_0 -= 1;
                                    }
                                    s.mtfbase[ii_0 as usize] = kk_0 + 1;
                                    ii_0 -= 1;
                                }
                            }
                        }
                        s.unzftab[s.seqToUnseq[uc as usize] as usize] += 1;
                        match s.smallDecompress {
                            DecompressMode::Small => {
                                ll16[nblock as usize] = s.seqToUnseq[uc as usize] as u16
                            }
                            DecompressMode::Fast => {
                                tt[nblock as usize] = s.seqToUnseq[uc as usize] as u32
                            }
                        }
                        nblock += 1;
                        update_group_pos!(s);
                        zn = gMinlen;
                        current_block = BZ_X_MTF_5;
                        continue;
                    }
                    current_block = Block46;
                }
                match current_block {
                    Block46 => {}
                    _ => {
                        if s.origPtr < 0 || s.origPtr >= nblock {
                            error!(BZ_DATA_ERROR);
                        } else {
                            i = 0;
                            while i <= 255 {
                                if s.unzftab[i as usize] < 0 || s.unzftab[i as usize] > nblock {
                                    error!(BZ_DATA_ERROR);
                                } else {
                                    i += 1;
                                }
                            }
                            s.cftab[0_usize] = 0;
                            i = 1;
                            while i <= 256 {
                                s.cftab[i as usize] = s.unzftab[(i - 1) as usize];
                                i += 1;
                            }
                            i = 1;
                            while i <= 256 {
                                s.cftab[i as usize] += s.cftab[(i - 1) as usize];
                                i += 1;
                            }
                            i = 0;
                            while i <= 256 {
                                if s.cftab[i as usize] < 0 || s.cftab[i as usize] > nblock {
                                    error!(BZ_DATA_ERROR);
                                } else {
                                    i += 1;
                                }
                            }
                            i = 1;
                            while i <= 256 {
                                if s.cftab[(i - 1) as usize] > s.cftab[i as usize] {
                                    error!(BZ_DATA_ERROR);
                                } else {
                                    i += 1;
                                }
                            }
                            s.state_out_len = 0;
                            s.state_out_ch = 0_u8;
                            s.calculatedBlockCRC = 0xffffffffu32;
                            s.state = State::BZ_X_OUTPUT;
                            if s.verbosity >= 2 {
                                debug_log!("rt+rld");
                            }
                            match s.smallDecompress {
                                DecompressMode::Small => {
                                    i = 0;
                                    while i <= 256 {
                                        s.cftabCopy[i as usize] = s.cftab[i as usize];
                                        i += 1;
                                    }
                                    i = 0;
                                    while i < nblock {
                                        uc = ll16[i as usize] as u8;
                                        ll16[i as usize] =
                                            (s.cftabCopy[uc as usize] & 0xffff) as u16;
                                        if i & 0x1 == 0 {
                                            ll4[(i >> 1) as usize] =
                                                (ll4[(i >> 1) as usize] as c_int & 0xf0
                                                    | s.cftabCopy[uc as usize] >> 16)
                                                    as u8;
                                        } else {
                                            ll4[(i >> 1) as usize] =
                                                (ll4[(i >> 1) as usize] as c_int & 0xf
                                                    | (s.cftabCopy[uc as usize] >> 16) << 4)
                                                    as u8;
                                        }
                                        s.cftabCopy[uc as usize] += 1;
                                        i += 1;
                                    }
                                    i = s.origPtr;
                                    j = (ll16[i as usize] as u32
                                        | (ll4[(i >> 1) as usize] as u32 >> (i << 2 & 0x4) & 0xf)
                                            << 16) as i32;
                                    loop {
                                        let tmp_0: i32 = (ll16[j as usize] as u32
                                            | (ll4[(j >> 1) as usize] as u32 >> (j << 2 & 0x4)
                                                & 0xf)
                                                << 16)
                                            as i32;
                                        ll16[j as usize] = (i & 0xffff) as u16;
                                        if j & 0x1 == 0 {
                                            ll4[(j >> 1) as usize] =
                                                (ll4[(j >> 1) as usize] as c_int & 0xf0 | i >> 16)
                                                    as u8;
                                        } else {
                                            ll4[(j >> 1) as usize] =
                                                (ll4[(j >> 1) as usize] as c_int & 0xf
                                                    | (i >> 16) << 4)
                                                    as u8;
                                        }
                                        i = j;
                                        j = tmp_0;
                                        if i == s.origPtr {
                                            break;
                                        }
                                    }
                                    s.tPos = s.origPtr as u32;
                                    s.nblock_used = 0;
                                    if s.blockRandomised {
                                        s.rNToGo = 0;
                                        s.rTPos = 0;
                                        if s.tPos >= 100000_u32.wrapping_mul(s.blockSize100k as u32)
                                        {
                                            // NOTE: this originates in the BZ_GET_FAST macro, and the
                                            // `return true` is probably uninitentional?!
                                            return ReturnCode::BZ_RUN_OK;
                                        }
                                        s.k0 = index_into_f(s.tPos as i32, &s.cftab);
                                        s.tPos = ll16[s.tPos as usize] as u32
                                            | (ll4[(s.tPos >> 1) as usize] as u32
                                                >> (s.tPos << 2 & 0x4)
                                                & 0xf)
                                                << 16;
                                        s.nblock_used += 1;
                                        if s.rNToGo == 0 {
                                            s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                                            s.rTPos += 1;
                                            if s.rTPos == 512 {
                                                s.rTPos = 0;
                                            }
                                        }
                                        s.rNToGo -= 1;
                                        s.k0 ^= if s.rNToGo == 1 { 1 } else { 0 };
                                    } else {
                                        if s.tPos >= 100000_u32.wrapping_mul(s.blockSize100k as u32)
                                        {
                                            // NOTE: this originates in the BZ_GET_FAST macro, and the
                                            // `return true` is probably uninitentional?!
                                            return ReturnCode::BZ_RUN_OK;
                                        }
                                        s.k0 = index_into_f(s.tPos as i32, &s.cftab);
                                        s.tPos = ll16[s.tPos as usize] as u32
                                            | (ll4[(s.tPos >> 1) as usize] as u32
                                                >> (s.tPos << 2 & 0x4)
                                                & 0xf)
                                                << 16;
                                        s.nblock_used += 1;
                                    }
                                }
                                DecompressMode::Fast => {
                                    i = 0;
                                    while i < nblock {
                                        uc = (tt[i as usize] & 0xff) as u8;
                                        tt[s.cftab[uc as usize] as usize] |= (i << 8) as c_uint;
                                        s.cftab[uc as usize] += 1;
                                        i += 1;
                                    }
                                    s.tPos = tt[s.origPtr as usize] >> 8;
                                    s.nblock_used = 0;
                                    if s.blockRandomised {
                                        s.rNToGo = 0;
                                        s.rTPos = 0;
                                        if s.tPos >= 100000_u32.wrapping_mul(s.blockSize100k as u32)
                                        {
                                            // NOTE: this originates in the BZ_GET_FAST macro, and the
                                            // `return true` is probably uninitentional?!
                                            return ReturnCode::BZ_RUN_OK;
                                        }
                                        s.tPos = tt[s.tPos as usize];
                                        s.k0 = (s.tPos & 0xff) as u8;
                                        s.tPos >>= 8;
                                        s.nblock_used += 1;
                                        if s.rNToGo == 0 {
                                            s.rNToGo = BZ2_RNUMS[s.rTPos as usize];
                                            s.rTPos += 1;
                                            if s.rTPos == 512 {
                                                s.rTPos = 0;
                                            }
                                        }
                                        s.rNToGo -= 1;
                                        s.k0 ^= if s.rNToGo == 1 { 1 } else { 0 };
                                    } else {
                                        if s.tPos >= 100000_u32.wrapping_mul(s.blockSize100k as u32)
                                        {
                                            // NOTE: this originates in the BZ_GET_FAST macro, and the
                                            // `return true` is probably uninitentional?!
                                            return ReturnCode::BZ_RUN_OK;
                                        }
                                        s.tPos = tt[s.tPos as usize];
                                        s.k0 = (s.tPos & 0xff) as u8;
                                        s.tPos >>= 8;
                                        s.nblock_used += 1;
                                    }
                                }
                            }

                            break 'save_state_and_return ReturnCode::BZ_OK;
                        }
                    }
                }
            }
            if current_block == Block46 {
                if N >= 2 * 1024 * 1024 {
                    error!(BZ_DATA_ERROR);
                } else {
                    if nextSym == 0 {
                        es += N;
                    } else if nextSym == 1 {
                        es += (1 + 1) * N;
                    }
                    N *= 2;
                    update_group_pos!(s);
                    zn = gMinlen;
                    current_block = BZ_X_MTF_3;
                    continue;
                }
            }
            loop {
                match current_block {
                    Block28 => {
                        if j < 16 {
                            current_block = BZ_X_MAPPING_2;
                            continue 'c_10064;
                        }
                    }
                    Block39 => {
                        if i < nSelectors {
                            j = 0;
                            current_block = Block25;
                            continue;
                        } else {
                            if nSelectors > 2 + 900000 / 50 {
                                nSelectors = 2 + 900000 / 50;
                            }
                            let mut pos: [u8; 6] = [0; 6];
                            let mut tmp: u8;
                            let mut v_22: u8;
                            v_22 = 0_u8;
                            while (v_22 as c_int) < nGroups {
                                pos[v_22 as usize] = v_22;
                                v_22 = v_22.wrapping_add(1);
                            }
                            i = 0;
                            while i < nSelectors {
                                v_22 = s.selectorMtf[i as usize];
                                tmp = pos[v_22 as usize];
                                while v_22 > 0 {
                                    pos[v_22 as usize] = pos[(v_22 - 1) as usize];
                                    v_22 = v_22.wrapping_sub(1);
                                }
                                pos[0_usize] = tmp;
                                s.selector[i as usize] = tmp;
                                i += 1;
                            }
                            t = 0;
                            current_block = Block35;
                            break;
                        }
                    }
                    Block18 => {
                        if i < 16 {
                            if s.inUse16[i as usize] {
                                j = 0;
                                current_block = Block28;
                                continue;
                            }
                        } else {
                            make_maps_d(s);

                            // reborrow
                            tt = s.tt.as_mut_slice();
                            ll16 = s.ll16.as_mut_slice();
                            ll4 = s.ll4.as_mut_slice();

                            if s.nInUse == 0 {
                                current_block = Block11;
                                break;
                            } else {
                                current_block = Block58;
                                break;
                            }
                        }
                    }
                    Block51 => {
                        s.len[t as usize][i as usize] = curr as u8;
                        i += 1;
                        current_block = Block26;
                        continue;
                    }
                    Block26 => {
                        if i < alphaSize {
                            current_block = Block45;
                            continue;
                        }
                        t += 1;
                        current_block = Block35;
                        break;
                    }
                    Block1 => {
                        if i < 2 + 900000 / 50 {
                            s.selectorMtf[i as usize] = j as u8;
                        }
                        i += 1;
                        current_block = Block39;
                        continue;
                    }
                    Block25 => {
                        current_block = BZ_X_SELECTOR_3;
                        continue 'c_10064;
                    }
                    _ => {
                        if false {
                            current_block = Block51;
                            continue;
                        }
                        if (1..=20).contains(&curr) {
                            current_block = BZ_X_CODING_2;
                            continue 'c_10064;
                        }
                        error!(BZ_DATA_ERROR);
                    }
                }
                i += 1;
                current_block = Block18;
            }
            match current_block {
                Block58 => {
                    alphaSize = s.nInUse + 2;
                    current_block = BZ_X_SELECTOR_1;
                }
                Block11 => {
                    error!(BZ_DATA_ERROR);
                }
                _ => {
                    if t < nGroups {
                        current_block = BZ_X_CODING_1;
                        continue;
                    }

                    /*--- Create the Huffman decoding tables ---*/
                    for t in 0..nGroups {
                        minLen = 32;
                        maxLen = 0;
                        for current in &s.len[t as usize][..alphaSize as usize] {
                            maxLen = Ord::max(maxLen, *current as i32);
                            minLen = Ord::min(minLen, *current as i32);
                        }
                        huffman::create_decode_tables(
                            &mut s.limit[t as usize],
                            &mut s.base[t as usize],
                            &mut s.perm[t as usize],
                            &mut s.len[t as usize],
                            minLen,
                            maxLen,
                            alphaSize,
                        );
                        s.minLens[t as usize] = minLen;
                    }

                    /*--- Now the MTF values ---*/

                    EOB = s.nInUse + 1;
                    nblockMAX = 100000 * s.blockSize100k;
                    groupNo = -1;
                    groupPos = 0;
                    s.unzftab.fill(0);

                    /*-- MTF init --*/
                    let mut kk: i32;
                    kk = MTFA_SIZE - 1;
                    for ii in (0..256 / MTFL_SIZE).rev() {
                        for jj in (0..MTFL_SIZE).rev() {
                            s.mtfa[kk as usize] = (ii * MTFL_SIZE + jj) as u8;
                            kk -= 1;
                        }
                        s.mtfbase[ii as usize] = kk + 1;
                    }
                    /*-- end MTF init --*/

                    nblock = 0;
                    update_group_pos!(s);

                    zn = gMinlen;
                    current_block = BZ_X_MTF_1;
                }
            }
        }
    };

    s.save_i = i;
    s.save_j = j;
    s.save_t = t;
    s.save_alphaSize = alphaSize;
    s.save_nGroups = nGroups;
    s.save_nSelectors = nSelectors;
    s.save_EOB = EOB;
    s.save_groupNo = groupNo;
    s.save_groupPos = groupPos;
    s.save_nextSym = nextSym;
    s.save_nblockMAX = nblockMAX;
    s.save_nblock = nblock;
    s.save_es = es;
    s.save_N = N;
    s.save_curr = curr;
    s.save_zt = zt;
    s.save_zn = zn;
    s.save_zvec = zvec;
    s.save_zj = zj;
    s.save_gSel = gSel;
    s.save_gMinlen = gMinlen;
    s.save_gLimit = gLimit;
    s.save_gBase = gBase;
    s.save_gPerm = gPerm;

    ret_val
}

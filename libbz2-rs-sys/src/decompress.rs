#![forbid(unsafe_code)]

use core::ffi::{c_int, c_uint};

use crate::allocator::Allocator;
use crate::bzlib::{
    index_into_f, BzStream, DSlice, DState, DecompressMode, ReturnCode, SaveArea, BZ_MAX_SELECTORS,
    BZ_RUNA, BZ_RUNB,
};
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

pub(crate) fn decompress(
    strm: &mut BzStream<DState>,
    s: &mut DState,
    allocator: &Allocator,
) -> ReturnCode {
    let mut current_block: Block;
    let mut uc: u8;

    if let State::BZ_X_MAGIC_1 = s.state {
        /*zero out the save area*/
        s.save = SaveArea::default();
    }

    /*restore from the save area*/
    let SaveArea {
        mut i,
        mut j,
        mut t,
        mut alphaSize,
        mut nGroups,
        mut nSelectors,
        mut EOB,
        mut groupNo,
        mut groupPos,
        mut nextSym,
        mut nblockMAX100k,
        mut nblock,
        mut es,
        mut logN,
        mut curr,
        mut zn,
        mut zvec,
        mut zj,
        mut gSel,
        mut gMinlen,
        mut gLimit,
        mut gBase,
        mut gPerm,
    } = s.save;

    let ret_val: ReturnCode = 'save_state_and_return: {
        macro_rules! GET_BYTE {
            ($strm:expr, $s:expr) => {
                (GET_BITS!($strm, $s, 8) & 0xFF) as u8
            };
        }

        macro_rules! GET_BIT {
            ($strm:expr, $s:expr) => {
                GET_BITS!($strm, $s, 1) != 0
            };
        }

        macro_rules! GET_BITS {
            ($strm:expr, $s:expr, $nnn:expr) => {
                loop {
                    if $s.bsLive >= $nnn {
                        let v: u32 = ($s.bsBuff >> ($s.bsLive - $nnn)) & ((1 << $nnn) - 1);
                        $s.bsLive -= $nnn;
                        break v;
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
                    gSel = match $s.selector[..usize::from(nSelectors)].get(groupNo as usize) {
                        Some(&gSel) => gSel,
                        None => error!(BZ_DATA_ERROR),
                    };
                    gLimit = gSel;
                    gPerm = gSel;
                    gBase = gSel;
                    gMinlen = $s.minLens[gSel as usize];
                    groupPos = 50;
                }
                groupPos -= 1;
            };
        }

        macro_rules! error {
            ($code:ident) => {{
                break 'save_state_and_return ReturnCode::$code;
            }};
        }

        match s.state {
            State::BZ_X_MAGIC_1 => {
                s.state = State::BZ_X_MAGIC_1;

                uc = GET_BYTE!(strm, s);

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

            uc = GET_BYTE!(strm, s);

            if uc != b'Z' {
                error!(BZ_DATA_ERROR_MAGIC);
            }

            current_block = BZ_X_MAGIC_3;
        }
        if current_block == BZ_X_MAGIC_3 {
            s.state = State::BZ_X_MAGIC_3;

            uc = GET_BYTE!(strm, s);

            if uc != b'h' {
                error!(BZ_DATA_ERROR_MAGIC);
            }

            current_block = BZ_X_MAGIC_4;
        }
        if current_block == BZ_X_MAGIC_4 {
            s.state = State::BZ_X_MAGIC_4;

            s.blockSize100k = GET_BITS!(strm, s, 8) as i32;

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

            uc = GET_BYTE!(strm, s);

            match uc {
                0x17 => current_block = BZ_X_ENDHDR_2,
                0x31 => current_block = BZ_X_BLKHDR_2,
                _ => error!(BZ_DATA_ERROR),
            };
        }
        match current_block {
            BZ_X_ENDHDR_2 => {
                s.state = State::BZ_X_ENDHDR_2;

                uc = GET_BYTE!(strm, s);

                if uc != 0x72 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_ENDHDR_3;
            }
            BZ_X_BLKHDR_2 => {
                s.state = State::BZ_X_BLKHDR_2;

                uc = GET_BYTE!(strm, s);

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

                uc = GET_BYTE!(strm, s);

                if uc != 0x45 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_ENDHDR_4;
            }
            BZ_X_BLKHDR_3 => {
                s.state = State::BZ_X_BLKHDR_3;

                uc = GET_BYTE!(strm, s);

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

                uc = GET_BYTE!(strm, s);

                if uc != 0x38 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_ENDHDR_5;
            }
            BZ_X_BLKHDR_4 => {
                s.state = State::BZ_X_BLKHDR_4;

                uc = GET_BYTE!(strm, s);

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

                uc = GET_BYTE!(strm, s);

                if uc != 0x50 {
                    error!(BZ_DATA_ERROR);
                }

                current_block = BZ_X_ENDHDR_6;
            }
            BZ_X_BLKHDR_5 => {
                s.state = State::BZ_X_BLKHDR_5;

                uc = GET_BYTE!(strm, s);

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

                uc = GET_BYTE!(strm, s);

                if uc != 0x90 {
                    error!(BZ_DATA_ERROR);
                }

                s.storedCombinedCRC = 0_u32;
                current_block = BZ_X_CCRC_1;
            }
            BZ_X_BLKHDR_6 => {
                s.state = State::BZ_X_BLKHDR_6;

                uc = GET_BYTE!(strm, s);

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

                uc = GET_BYTE!(strm, s);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                current_block = BZ_X_CCRC_2;
            }
            BZ_X_BCRC_1 => {
                s.state = State::BZ_X_BCRC_1;

                uc = GET_BYTE!(strm, s);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = BZ_X_BCRC_2;
            }
            _ => {}
        }
        match current_block {
            BZ_X_CCRC_2 => {
                s.state = State::BZ_X_CCRC_2;

                uc = GET_BYTE!(strm, s);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                current_block = BZ_X_CCRC_3;
            }
            BZ_X_BCRC_2 => {
                s.state = State::BZ_X_BCRC_2;

                uc = GET_BYTE!(strm, s);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = BZ_X_BCRC_3;
            }
            _ => {}
        }
        match current_block {
            BZ_X_CCRC_3 => {
                s.state = State::BZ_X_CCRC_3;

                uc = GET_BYTE!(strm, s);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                current_block = BZ_X_CCRC_4;
            }
            BZ_X_BCRC_3 => {
                s.state = State::BZ_X_BCRC_3;

                uc = GET_BYTE!(strm, s);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = BZ_X_BCRC_4;
            }
            _ => {}
        }
        match current_block {
            BZ_X_BCRC_4 => {
                s.state = State::BZ_X_BCRC_4;

                uc = GET_BYTE!(strm, s);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = BZ_X_RANDBIT;
            }
            BZ_X_CCRC_4 => {
                s.state = State::BZ_X_CCRC_4;

                uc = GET_BYTE!(strm, s);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                s.state = State::BZ_X_IDLE;
                error!(BZ_STREAM_END);
            }
            _ => {}
        }
        if current_block == BZ_X_RANDBIT {
            s.state = State::BZ_X_RANDBIT;

            s.blockRandomised = GET_BITS!(strm, s, 1) != 0;

            s.origPtr = 0;
            current_block = BZ_X_ORIGPTR_1;
        }
        if current_block == BZ_X_ORIGPTR_1 {
            s.state = State::BZ_X_ORIGPTR_1;

            uc = GET_BYTE!(strm, s);

            s.origPtr = s.origPtr << 8 | uc as i32;
            current_block = BZ_X_ORIGPTR_2;
        }
        if current_block == BZ_X_ORIGPTR_2 {
            s.state = State::BZ_X_ORIGPTR_2;

            uc = GET_BYTE!(strm, s);

            s.origPtr = s.origPtr << 8 | uc as i32;
            current_block = BZ_X_ORIGPTR_3;
        }
        if current_block == BZ_X_ORIGPTR_3 {
            s.state = State::BZ_X_ORIGPTR_3;

            uc = GET_BYTE!(strm, s);

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

                    uc = GET_BIT!(strm, s) as u8;

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
                    s.inUse.fill(false);
                    i = 0;
                    current_block = Block18;
                }
                BZ_X_MAPPING_2 => {
                    s.state = State::BZ_X_MAPPING_2;

                    uc = GET_BIT!(strm, s) as u8;

                    if uc == 1 {
                        s.inUse[(i * 16 + j) as usize] = true;
                    }
                    j += 1;
                    current_block = Block28;
                }
                BZ_X_SELECTOR_1 => {
                    s.state = State::BZ_X_SELECTOR_1;

                    nGroups = GET_BITS!(strm, s, 3) as u8;

                    if (2..=6).contains(&nGroups) {
                        current_block = BZ_X_SELECTOR_2;
                        continue;
                    }
                    error!(BZ_DATA_ERROR);
                }
                BZ_X_SELECTOR_2 => {
                    s.state = State::BZ_X_SELECTOR_2;

                    nSelectors = GET_BITS!(strm, s, 15) as u16;

                    if nSelectors < 1 {
                        error!(BZ_DATA_ERROR);
                    } else {
                        i = 0;
                    }
                    current_block = Block39;
                }
                BZ_X_SELECTOR_3 => {
                    s.state = State::BZ_X_SELECTOR_3;

                    uc = GET_BIT!(strm, s) as u8;

                    if uc == 0 {
                        current_block = Block1;
                    } else {
                        j += 1;
                        if j >= nGroups as i32 {
                            error!(BZ_DATA_ERROR);
                        } else {
                            current_block = Block25;
                        }
                    }
                }
                BZ_X_CODING_1 => {
                    s.state = State::BZ_X_CODING_1;

                    curr = GET_BITS!(strm, s, 5) as u8;

                    i = 0;
                    current_block = Block26;
                }
                BZ_X_CODING_2 => {
                    s.state = State::BZ_X_CODING_2;

                    uc = GET_BIT!(strm, s) as u8;

                    if uc != 0 {
                        current_block = BZ_X_CODING_3;
                        continue;
                    }
                    current_block = Block51;
                }
                BZ_X_CODING_3 => {
                    s.state = State::BZ_X_CODING_3;

                    uc = GET_BIT!(strm, s) as u8;

                    match uc {
                        0 => curr += 1,
                        _ => curr -= 1,
                    }

                    current_block = Block45;
                }
                BZ_X_MTF_1 => {
                    s.state = State::BZ_X_MTF_1;

                    zvec = GET_BITS!(strm, s, zn as i32) as i32;

                    current_block = Block56;
                }
                BZ_X_MTF_2 => {
                    s.state = State::BZ_X_MTF_2;

                    zj = GET_BIT!(strm, s);

                    zvec = zvec << 1 | zj as i32;
                    current_block = Block56;
                }
                BZ_X_MTF_3 => {
                    s.state = State::BZ_X_MTF_3;

                    zvec = GET_BITS!(strm, s, zn as i32) as i32;

                    current_block = Block52;
                }
                BZ_X_MTF_4 => {
                    s.state = State::BZ_X_MTF_4;

                    zj = GET_BIT!(strm, s);

                    zvec = zvec << 1 | zj as i32;
                    current_block = Block52;
                }
                BZ_X_MTF_5 => {
                    s.state = State::BZ_X_MTF_5;

                    zvec = GET_BITS!(strm, s, zn as i32) as i32;

                    current_block = Block24;
                }
                _ => {
                    s.state = State::BZ_X_MTF_6;

                    zj = GET_BIT!(strm, s);

                    zvec = zvec << 1 | zj as i32;
                    current_block = Block24;
                }
            }
            match current_block {
                Block24 => {
                    if zn > 20 {
                        error!(BZ_DATA_ERROR);
                    } else if zvec <= s.limit[gLimit as usize][zn as usize] {
                        let index = zvec - s.base[gBase as usize][zn as usize];
                        nextSym = match s.perm[gPerm as usize].get(index as usize) {
                            Some(&nextSym) => nextSym,
                            None => error!(BZ_DATA_ERROR),
                        };
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
                        let index = zvec - s.base[gBase as usize][zn as usize];
                        nextSym = match s.perm[gPerm as usize].get(index as usize) {
                            Some(&nextSym) => nextSym,
                            None => error!(BZ_DATA_ERROR),
                        };
                        if nextSym == BZ_RUNA as i32 || nextSym == BZ_RUNB as i32 {
                            current_block = Block46;
                        } else {
                            es += 1;
                            uc = s.seqToUnseq[s.mtfa[s.mtfbase[0_usize] as usize] as usize];
                            s.unzftab[uc as usize] += es;
                            match s.smallDecompress {
                                DecompressMode::Small => {
                                    while es > 0 {
                                        if nblock >= 100000 * nblockMAX100k as u32 {
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
                                        if nblock >= 100000 * nblockMAX100k as u32 {
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
                        let index = zvec - s.base[gBase as usize][zn as usize];
                        nextSym = match s.perm[gPerm as usize].get(index as usize) {
                            Some(&nextSym) => nextSym,
                            None => error!(BZ_DATA_ERROR),
                        };
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
                if nextSym == EOB as i32 {
                    current_block = Block41;
                } else {
                    if nextSym == 0 || nextSym == 1 {
                        es = -1;
                        logN = 0;
                    } else if nblock >= 100000 * nblockMAX100k as u32 {
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
                        if s.origPtr < 0 || s.origPtr >= nblock as i32 {
                            error!(BZ_DATA_ERROR);
                        } else {
                            if s.unzftab.iter().any(|e| !(0..=nblock as i32).contains(e)) {
                                error!(BZ_DATA_ERROR);
                            }
                            s.cftab[0] = 0;
                            s.cftab[1..].copy_from_slice(&s.unzftab);
                            for i in 1..s.cftab.len() {
                                s.cftab[i] += s.cftab[i - 1];
                            }
                            if s.cftab.iter().any(|e| !(0..=nblock as i32).contains(e)) {
                                error!(BZ_DATA_ERROR);
                            }
                            // FIXME: use https://doc.rust-lang.org/std/primitive.slice.html#method.is_sorted
                            // when available in our MSRV (requires >= 1.82.0)
                            if s.cftab.windows(2).any(|w| w[0] > w[1]) {
                                error!(BZ_DATA_ERROR);
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
                                    while i < nblock as i32 {
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
                                    while i < nblock as i32 {
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
                // Check that N doesn't get too big, so that es doesn't
                // go negative.  The maximum value that can be
                // RUNA/RUNB encoded is equal to the block size (post
                // the initial RLE), viz, 900k, so bounding N at 2
                // million should guard against overflow without
                // rejecting any legitimate inputs.
                const LOG_2MB: u8 = 21; // 2 * 1024 * 1024

                if logN >= LOG_2MB {
                    error!(BZ_DATA_ERROR);
                } else {
                    let mul = match nextSym as u16 {
                        BZ_RUNA => 1,
                        BZ_RUNB => 2,
                        _ => 0,
                    };
                    es += mul * (1 << logN);
                    logN += 1;
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
                        if i < nSelectors as i32 {
                            j = 0;
                            current_block = Block25;
                            continue;
                        } else {
                            // make sure that the constant fits in a u16
                            const _: () = assert!((BZ_MAX_SELECTORS >> 16) == 0);
                            nSelectors = Ord::min(nSelectors, BZ_MAX_SELECTORS as u16);

                            let mut pos: [u8; 6] = [0; 6];
                            let mut tmp: u8;
                            let mut v_22: u8;
                            v_22 = 0_u8;
                            while v_22 < nGroups {
                                pos[v_22 as usize] = v_22;
                                v_22 = v_22.wrapping_add(1);
                            }
                            i = 0;
                            while i < nSelectors as i32 {
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
                        s.len[t as usize][i as usize] = curr;
                        i += 1;
                        current_block = Block26;
                        continue;
                    }
                    Block26 => {
                        if i < alphaSize as i32 {
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
                    if t < nGroups as i32 {
                        current_block = BZ_X_CODING_1;
                        continue;
                    }

                    /*--- Create the Huffman decoding tables ---*/
                    for t in 0..nGroups as usize {
                        // NOTE: s.nInUse <= 256, alphaSize <= 258
                        let len = &s.len[t][..alphaSize as usize];

                        let mut minLen = 32u8;
                        let mut maxLen = 0u8;
                        for &current in len {
                            maxLen = Ord::max(maxLen, current);
                            minLen = Ord::min(minLen, current);
                        }
                        s.minLens[t] = minLen;

                        huffman::create_decode_tables(
                            &mut s.limit[t],
                            &mut s.base[t],
                            &mut s.perm[t],
                            len,
                            minLen,
                            maxLen,
                        );
                    }

                    /*--- Now the MTF values ---*/

                    EOB = s.nInUse + 1;
                    nblockMAX100k = s.blockSize100k as u8;
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
                    groupNo = -1;
                    groupPos = 0;
                    update_group_pos!(s);

                    zn = gMinlen;
                    current_block = BZ_X_MTF_1;
                }
            }
        }
    };

    s.save = SaveArea {
        i,
        j,
        t,
        alphaSize,
        nGroups,
        nSelectors,
        EOB,
        groupNo,
        groupPos,
        nextSym,
        nblockMAX100k,
        nblock,
        es,
        logN,
        curr,
        zn,
        zvec,
        zj,
        gSel,
        gMinlen,
        gLimit,
        gBase,
        gPerm,
    };

    ret_val
}

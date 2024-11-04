use crate::bzlib::{bz_stream, BZ2_indexIntoF, DSlice, DState, ReturnCode};
use crate::huffman::BZ2_hbCreateDecodeTables;
use crate::randtable::BZ2_RNUMS;

/*-- Constants for the fast MTF decoder. --*/

const MTFA_SIZE: i32 = 4096;
const MTFL_SIZE: i32 = 16;

#[repr(i32)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum State {
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

fn makeMaps_d(s: &mut DState) {
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

pub unsafe fn BZ2_decompress(strm: &mut bz_stream, s: &mut DState) -> ReturnCode {
    let mut current_block: u64;
    let mut uc: u8;
    let mut retVal: ReturnCode;
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

    retVal = ReturnCode::BZ_OK;
    const SAVE_STATE_AND_RETURN: u64 = 3350591128142761507;

    'save_state_and_return: {
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

                    if $strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        break 'save_state_and_return;
                    }

                    $s.bsBuff = $s.bsBuff << 8 | *($strm.next_in as *mut u8) as u32;
                    $s.bsLive += 8;
                    $strm.next_in = $strm.next_in.wrapping_add(1);
                    $strm.avail_in -= 1;
                    $strm.total_in_lo32 += 1;
                    if $strm.total_in_lo32 == 0 {
                        $strm.total_in_hi32 += 1;
                    }
                }
            };
        }

        macro_rules! update_group_pos {
            ($s:expr) => {
                if groupPos == 0 {
                    groupNo += 1;
                    if groupNo >= nSelectors {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        break 'save_state_and_return;
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

        match s.state {
            State::BZ_X_MAGIC_1 => {
                s.state = State::BZ_X_MAGIC_1;

                GET_UCHAR!(strm, s, uc);

                if uc != b'B' {
                    retVal = ReturnCode::BZ_DATA_ERROR_MAGIC;
                    break 'save_state_and_return;
                }

                current_block = 15360092558900836893;
            }
            State::BZ_X_MAGIC_2 => {
                current_block = 15360092558900836893;
            }
            State::BZ_X_MAGIC_3 => {
                current_block = 15953825877604003206;
            }
            State::BZ_X_MAGIC_4 => {
                current_block = 1137006006685247392;
            }
            State::BZ_X_BLKHDR_1 => {
                current_block = 16838365919992687769;
            }
            State::BZ_X_BLKHDR_2 => {
                current_block = 5889181040567946013;
            }
            State::BZ_X_BLKHDR_3 => {
                current_block = 887841530443712878;
            }
            State::BZ_X_BLKHDR_4 => {
                current_block = 17767742176799939193;
            }
            State::BZ_X_BLKHDR_5 => {
                current_block = 16325921850189496668;
            }
            State::BZ_X_BLKHDR_6 => {
                current_block = 3202472413399101603;
            }
            State::BZ_X_BCRC_1 => {
                current_block = 5821827988509819404;
            }
            State::BZ_X_BCRC_2 => {
                current_block = 5023088878038355716;
            }
            State::BZ_X_BCRC_3 => {
                current_block = 8515868523999336537;
            }
            State::BZ_X_BCRC_4 => {
                current_block = 18234918597811156654;
            }
            State::BZ_X_RANDBIT => {
                current_block = 12310871532727186508;
            }
            State::BZ_X_ORIGPTR_1 => {
                current_block = 3338455798814466984;
            }
            State::BZ_X_ORIGPTR_2 => {
                current_block = 10262367570716242252;
            }
            State::BZ_X_ORIGPTR_3 => {
                current_block = 17024493544560437554;
            }
            State::BZ_X_MAPPING_1 => {
                current_block = 1154520408629132897;
            }
            State::BZ_X_MAPPING_2 => {
                current_block = 15451013008180677144;
            }
            State::BZ_X_SELECTOR_1 => {
                current_block = 9434444550647791986;
            }
            State::BZ_X_SELECTOR_2 => {
                current_block = 14590825336193814119;
            }
            State::BZ_X_SELECTOR_3 => {
                current_block = 15957329598978927534;
            }
            State::BZ_X_CODING_1 => {
                current_block = 11569294379105328467;
            }
            State::BZ_X_CODING_2 => {
                current_block = 17216244326479313607;
            }
            State::BZ_X_CODING_3 => {
                current_block = 7191958063352112897;
            }
            State::BZ_X_MTF_1 => {
                current_block = 13155828021133314705;
            }
            State::BZ_X_MTF_2 => {
                current_block = 1010107409739284736;
            }
            State::BZ_X_MTF_3 => {
                current_block = 9335356017384149594;
            }
            State::BZ_X_MTF_4 => {
                current_block = 12127014564286193091;
            }
            State::BZ_X_MTF_5 => {
                current_block = 9050093969003559074;
            }
            State::BZ_X_MTF_6 => {
                current_block = 10797958389266113496;
            }
            State::BZ_X_ENDHDR_2 => {
                current_block = 14366592556287126287;
            }
            State::BZ_X_ENDHDR_3 => {
                current_block = 7651522734817633728;
            }
            State::BZ_X_ENDHDR_4 => {
                current_block = 15818849443713787272;
            }
            State::BZ_X_ENDHDR_5 => {
                current_block = 15153555825877660840;
            }
            State::BZ_X_ENDHDR_6 => {
                current_block = 1857046018890652364;
            }
            State::BZ_X_CCRC_1 => {
                current_block = 10292318171587122742;
            }
            State::BZ_X_CCRC_2 => {
                current_block = 14748314904637597825;
            }
            State::BZ_X_CCRC_3 => {
                current_block = 4092966239614665407;
            }
            State::BZ_X_CCRC_4 => {
                current_block = 18389040574536762539;
            }
            State::BZ_X_IDLE | State::BZ_X_OUTPUT => unreachable!(),
        }
        if current_block == 15360092558900836893 {
            s.state = State::BZ_X_MAGIC_2;

            GET_UCHAR!(strm, s, uc);

            if uc != b'Z' {
                retVal = ReturnCode::BZ_DATA_ERROR_MAGIC;
                break 'save_state_and_return;
            }

            current_block = 15953825877604003206;
        }
        if current_block == 15953825877604003206 {
            s.state = State::BZ_X_MAGIC_3;

            GET_UCHAR!(strm, s, uc);

            if uc != b'h' {
                retVal = ReturnCode::BZ_DATA_ERROR_MAGIC;
                break 'save_state_and_return;
            }

            current_block = 1137006006685247392;
        }
        if current_block == 1137006006685247392 {
            s.state = State::BZ_X_MAGIC_4;

            GET_BITS!(strm, s, s.blockSize100k, 8);

            if !(b'1' as i32..=b'9' as i32).contains(&s.blockSize100k) {
                retVal = ReturnCode::BZ_DATA_ERROR_MAGIC;
                break 'save_state_and_return;
            }

            s.blockSize100k -= b'0' as i32;

            let Some(bzalloc) = strm.bzalloc else {
                retVal = ReturnCode::BZ_PARAM_ERROR;
                break 'save_state_and_return;
            };

            if s.smallDecompress {
                let ll16_len = s.blockSize100k as usize * 100000;
                let Some(ll16) = (unsafe { DSlice::alloc(bzalloc, strm.opaque, ll16_len) }) else {
                    retVal = ReturnCode::BZ_MEM_ERROR;
                    break 'save_state_and_return;
                };

                let ll4_len = (1 + s.blockSize100k as usize * 100000) >> 1;
                let Some(ll4) = (unsafe { DSlice::alloc(bzalloc, strm.opaque, ll4_len) }) else {
                    retVal = ReturnCode::BZ_MEM_ERROR;
                    break 'save_state_and_return;
                };

                s.ll16 = ll16;
                s.ll4 = ll4;
            } else {
                let tt_len = s.blockSize100k as usize * 100000;
                let Some(tt) = (unsafe { DSlice::alloc(bzalloc, strm.opaque, tt_len) }) else {
                    retVal = ReturnCode::BZ_MEM_ERROR;
                    break 'save_state_and_return;
                };

                s.tt = tt;
            }

            current_block = 16838365919992687769;
        }
        if current_block == 16838365919992687769 {
            s.state = State::BZ_X_BLKHDR_1;

            GET_UCHAR!(strm, s, uc);

            if uc == 0x17 {
                // skips to `State::BZ_X_ENDHDR_2`
                current_block = 14366592556287126287;
            } else if uc != 0x31 {
                retVal = ReturnCode::BZ_DATA_ERROR;
                break 'save_state_and_return;
            } else {
                current_block = 5889181040567946013;
            }
        }
        match current_block {
            14366592556287126287 => {
                s.state = State::BZ_X_ENDHDR_2;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x72 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }

                current_block = 7651522734817633728;
            }
            5889181040567946013 => {
                s.state = State::BZ_X_BLKHDR_2;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x41 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }
                current_block = 887841530443712878;
            }
            _ => {}
        }
        match current_block {
            7651522734817633728 => {
                s.state = State::BZ_X_ENDHDR_3;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x45 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }

                current_block = 15818849443713787272;
            }
            887841530443712878 => {
                s.state = State::BZ_X_BLKHDR_3;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x59 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }

                current_block = 17767742176799939193;
            }
            _ => {}
        }
        match current_block {
            15818849443713787272 => {
                s.state = State::BZ_X_ENDHDR_4;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x38 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }

                current_block = 15153555825877660840;
            }
            17767742176799939193 => {
                s.state = State::BZ_X_BLKHDR_4;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x26 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }

                current_block = 16325921850189496668;
            }
            _ => {}
        }
        match current_block {
            15153555825877660840 => {
                s.state = State::BZ_X_ENDHDR_5;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x50 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }

                current_block = 1857046018890652364;
            }
            16325921850189496668 => {
                s.state = State::BZ_X_BLKHDR_5;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x53 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }

                current_block = 3202472413399101603;
            }
            _ => {}
        }
        match current_block {
            1857046018890652364 => {
                s.state = State::BZ_X_ENDHDR_6;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x90 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }

                s.storedCombinedCRC = 0_u32;
                current_block = 10292318171587122742;
            }
            3202472413399101603 => {
                s.state = State::BZ_X_BLKHDR_6;

                GET_UCHAR!(strm, s, uc);

                if uc != 0x59 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }

                s.currBlockNo += 1;
                if s.verbosity >= 2 {
                    eprint!("\n    [{}: huff+mtf ", s.currBlockNo);
                }
                s.storedBlockCRC = 0_u32;
                current_block = 5821827988509819404;
            }
            _ => {}
        }
        match current_block {
            10292318171587122742 => {
                s.state = State::BZ_X_CCRC_1;

                GET_UCHAR!(strm, s, uc);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                current_block = 14748314904637597825;
            }
            5821827988509819404 => {
                s.state = State::BZ_X_BCRC_1;

                GET_UCHAR!(strm, s, uc);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = 5023088878038355716;
            }
            _ => {}
        }
        match current_block {
            14748314904637597825 => {
                s.state = State::BZ_X_CCRC_2;

                GET_UCHAR!(strm, s, uc);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                current_block = 4092966239614665407;
            }
            5023088878038355716 => {
                s.state = State::BZ_X_BCRC_2;

                GET_UCHAR!(strm, s, uc);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = 8515868523999336537;
            }
            _ => {}
        }
        match current_block {
            4092966239614665407 => {
                s.state = State::BZ_X_CCRC_3;

                GET_UCHAR!(strm, s, uc);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                current_block = 18389040574536762539;
            }
            8515868523999336537 => {
                s.state = State::BZ_X_BCRC_3;

                GET_UCHAR!(strm, s, uc);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = 18234918597811156654;
            }
            _ => {}
        }
        match current_block {
            18234918597811156654 => {
                s.state = State::BZ_X_BCRC_4;

                GET_UCHAR!(strm, s, uc);

                s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                current_block = 12310871532727186508;
            }
            18389040574536762539 => {
                s.state = State::BZ_X_CCRC_4;

                GET_UCHAR!(strm, s, uc);

                s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                s.state = State::BZ_X_IDLE;
                retVal = ReturnCode::BZ_STREAM_END;
                current_block = SAVE_STATE_AND_RETURN;
            }
            _ => {}
        }
        if current_block == 12310871532727186508 {
            s.state = State::BZ_X_RANDBIT;

            GET_BITS!(strm, s, s.blockRandomised, 1);

            s.origPtr = 0;
            current_block = 3338455798814466984;
        }
        if current_block == 3338455798814466984 {
            s.state = State::BZ_X_ORIGPTR_1;

            GET_UCHAR!(strm, s, uc);

            s.origPtr = s.origPtr << 8 | uc as i32;
            current_block = 10262367570716242252;
        }
        if current_block == 10262367570716242252 {
            s.state = State::BZ_X_ORIGPTR_2;

            GET_UCHAR!(strm, s, uc);

            s.origPtr = s.origPtr << 8 | uc as i32;
            current_block = 17024493544560437554;
        }
        if current_block == 17024493544560437554 {
            s.state = State::BZ_X_ORIGPTR_3;

            GET_UCHAR!(strm, s, uc);

            s.origPtr = s.origPtr << 8 | uc as i32;
            if s.origPtr < 0 {
                retVal = ReturnCode::BZ_DATA_ERROR;
                break 'save_state_and_return;
            } else if s.origPtr > 10 + 100000 * s.blockSize100k {
                retVal = ReturnCode::BZ_DATA_ERROR;
                break 'save_state_and_return;
            } else {
                i = 0;
                current_block = 454873545234741267;
            }
        }

        // mutable because they need to be reborrowed
        let mut tt = s.tt.as_mut_slice();
        let mut ll16 = s.ll16.as_mut_slice();
        let mut ll4 = s.ll4.as_mut_slice();

        'c_10064: loop {
            match current_block {
                SAVE_STATE_AND_RETURN => {
                    s.save_i = i;
                    break;
                }
                9050093969003559074 => {
                    s.state = State::BZ_X_MTF_5;

                    GET_BITS!(strm, s, zvec, zn);

                    current_block = 16348713635569416413;
                }
                12127014564286193091 => {
                    s.state = State::BZ_X_MTF_4;

                    GET_BIT!(strm, s, zj);

                    zvec = zvec << 1 | zj;
                    current_block = 7923635230025172457;
                }
                9335356017384149594 => {
                    s.state = State::BZ_X_MTF_3;

                    GET_BITS!(strm, s, zvec, zn);

                    current_block = 7923635230025172457;
                }
                1010107409739284736 => {
                    s.state = State::BZ_X_MTF_2;

                    GET_BIT!(strm, s, zj);

                    zvec = zvec << 1 | zj;
                    current_block = 9186389159759284570;
                }
                13155828021133314705 => {
                    s.state = State::BZ_X_MTF_1;

                    GET_BITS!(strm, s, zvec, zn);

                    current_block = 9186389159759284570;
                }
                7191958063352112897 => {
                    s.state = State::BZ_X_CODING_3;

                    GET_BIT!(strm, s, uc);

                    if uc == 0 {
                        curr += 1;
                    } else {
                        curr -= 1;
                    }

                    current_block = 5533056661327372531;
                }
                17216244326479313607 => {
                    s.state = State::BZ_X_CODING_2;

                    GET_BIT!(strm, s, uc);

                    if uc != 0 {
                        current_block = 7191958063352112897;
                        continue;
                    }
                    current_block = 7746242308555130918;
                }
                11569294379105328467 => {
                    s.state = State::BZ_X_CODING_1;

                    GET_BITS!(strm, s, curr, 5);

                    i = 0;
                    current_block = 16642413284942005565;
                }
                15957329598978927534 => {
                    s.state = State::BZ_X_SELECTOR_3;

                    GET_BIT!(strm, s, uc);

                    if uc == 0 {
                        current_block = 10081471997089450706;
                    } else {
                        j += 1;
                        if j >= nGroups {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            break 'save_state_and_return;
                        } else {
                            current_block = 16531797892856733396;
                        }
                    }
                }
                14590825336193814119 => {
                    s.state = State::BZ_X_SELECTOR_2;

                    GET_BITS!(strm, s, nSelectors, 15);

                    if nSelectors < 1 {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        break 'save_state_and_return;
                    } else {
                        i = 0;
                    }
                    current_block = 3503188808869013853;
                }
                9434444550647791986 => {
                    s.state = State::BZ_X_SELECTOR_1;

                    GET_BITS!(strm, s, nGroups, 3);

                    if !!(2..=6).contains(&nGroups) {
                        current_block = 14590825336193814119;
                        continue;
                    }
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }
                15451013008180677144 => {
                    s.state = State::BZ_X_MAPPING_2;

                    GET_BIT!(strm, s, uc);

                    if uc == 1 {
                        s.inUse[(i * 16 + j) as usize] = true;
                    }
                    j += 1;
                    current_block = 16953886395775657100;
                }
                454873545234741267 => {
                    if i < 16 {
                        current_block = 1154520408629132897;
                        continue;
                    }
                    i = 0;
                    while i < 256 {
                        s.inUse[i as usize] = false;
                        i += 1;
                    }
                    i = 0;
                    current_block = 15415362524153386998;
                }
                1154520408629132897 => {
                    s.state = State::BZ_X_MAPPING_1;

                    GET_BIT!(strm, s, uc);

                    s.inUse16[i as usize] = uc == 1;
                    i += 1;
                    current_block = 454873545234741267;
                    continue;
                }
                _ => {
                    s.state = State::BZ_X_MTF_6;

                    GET_BIT!(strm, s, zj);

                    zvec = zvec << 1 | zj;
                    current_block = 16348713635569416413;
                }
            }
            match current_block {
                16348713635569416413 => {
                    if zn > 20 {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        break 'save_state_and_return;
                    } else if zvec <= s.limit[gLimit as usize][zn as usize] {
                        if !(0..258).contains(&(zvec - s.base[gBase as usize][zn as usize])) {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            break 'save_state_and_return;
                        } else {
                            nextSym = s.perm[gPerm as usize]
                                [(zvec - s.base[gBase as usize][zn as usize]) as usize];
                        }
                    } else {
                        zn += 1;
                        current_block = 10797958389266113496;
                        continue;
                    }
                    current_block = 3575340618357869479;
                }
                7923635230025172457 => {
                    if zn > 20 {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        break 'save_state_and_return;
                    } else if zvec <= s.limit[gLimit as usize][zn as usize] {
                        if !(0..258).contains(&(zvec - s.base[gBase as usize][zn as usize])) {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            break 'save_state_and_return;
                        } else {
                            nextSym = s.perm[gPerm as usize]
                                [(zvec - s.base[gBase as usize][zn as usize]) as usize];
                            if nextSym == 0 || nextSym == 1 {
                                current_block = 5649595406143318745;
                            } else {
                                es += 1;
                                uc = s.seqToUnseq[s.mtfa[s.mtfbase[0_usize] as usize] as usize];
                                s.unzftab[uc as usize] += es;
                                if s.smallDecompress {
                                    while es > 0 {
                                        if nblock >= nblockMAX {
                                            retVal = ReturnCode::BZ_DATA_ERROR;
                                            break 'save_state_and_return;
                                        } else {
                                            ll16[nblock as usize] = uc as u16;
                                            nblock += 1;
                                            es -= 1;
                                        }
                                    }
                                } else {
                                    while es > 0 {
                                        if nblock >= nblockMAX {
                                            retVal = ReturnCode::BZ_DATA_ERROR;
                                            break 'save_state_and_return;
                                        } else {
                                            tt[nblock as usize] = uc as u32;
                                            nblock += 1;
                                            es -= 1;
                                        }
                                    }
                                }
                                current_block = 3575340618357869479;
                            }
                        }
                    } else {
                        zn += 1;
                        current_block = 12127014564286193091;
                        continue;
                    }
                }
                9186389159759284570 => {
                    if zn > 20 {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        break 'save_state_and_return;
                    } else if zvec <= s.limit[gLimit as usize][zn as usize] {
                        if !(0..258).contains(&(zvec - s.base[gBase as usize][zn as usize])) {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            break 'save_state_and_return;
                        } else {
                            nextSym = s.perm[gPerm as usize]
                                [(zvec - s.base[gBase as usize][zn as usize]) as usize];
                        }
                    } else {
                        zn += 1;
                        current_block = 1010107409739284736;
                        continue;
                    }
                    current_block = 3575340618357869479;
                }
                _ => {}
            }
            if current_block == 3575340618357869479 {
                if 1 != 0 {
                    if nextSym == EOB {
                        current_block = 4069074773319880902;
                    } else {
                        if nextSym == 0 || nextSym == 1 {
                            es = -1;
                            N = 1;
                        } else if nblock >= nblockMAX {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            break 'save_state_and_return;
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
                                uc = s.mtfa[(pp as libc::c_uint).wrapping_add(nn) as usize];
                                while nn > 3 {
                                    let z: i32 = (pp as libc::c_uint).wrapping_add(nn) as i32;
                                    s.mtfa[z as usize] = s.mtfa[(z - 1) as usize];
                                    s.mtfa[(z - 1) as usize] = s.mtfa[(z - 2) as usize];
                                    s.mtfa[(z - 2) as usize] = s.mtfa[(z - 3) as usize];
                                    s.mtfa[(z - 3) as usize] = s.mtfa[(z - 4) as usize];
                                    nn = (nn).wrapping_sub(4);
                                }
                                while nn > 0 {
                                    s.mtfa[(pp as libc::c_uint).wrapping_add(nn) as usize] = s.mtfa
                                        [(pp as libc::c_uint).wrapping_add(nn).wrapping_sub(1)
                                            as usize];
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
                                s.mtfbase[lno as usize];
                                while lno > 0 {
                                    s.mtfbase[lno as usize] -= 1;
                                    s.mtfbase[lno as usize];
                                    s.mtfa[s.mtfbase[lno as usize] as usize] =
                                        s.mtfa[(s.mtfbase[(lno - 1) as usize] + 16 - 1) as usize];
                                    lno -= 1;
                                }
                                s.mtfbase[0_usize] -= 1;
                                s.mtfbase[0_usize];
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
                            s.unzftab[s.seqToUnseq[uc as usize] as usize];
                            if s.smallDecompress {
                                ll16[nblock as usize] = s.seqToUnseq[uc as usize] as u16;
                            } else {
                                tt[nblock as usize] = s.seqToUnseq[uc as usize] as u32;
                            }
                            nblock += 1;
                            update_group_pos!(s);
                            zn = gMinlen;
                            current_block = 9050093969003559074;
                            continue;
                        }
                        current_block = 5649595406143318745;
                    }
                } else {
                    current_block = 4069074773319880902;
                }
                match current_block {
                    5649595406143318745 => {}
                    _ => {
                        if s.origPtr < 0 || s.origPtr >= nblock {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            break 'save_state_and_return;
                        } else {
                            i = 0;
                            while i <= 255 {
                                if s.unzftab[i as usize] < 0 || s.unzftab[i as usize] > nblock {
                                    retVal = ReturnCode::BZ_DATA_ERROR;
                                    break 'save_state_and_return;
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
                                    retVal = ReturnCode::BZ_DATA_ERROR;
                                    break 'save_state_and_return;
                                } else {
                                    i += 1;
                                }
                            }
                            i = 1;
                            while i <= 256 {
                                if s.cftab[(i - 1) as usize] > s.cftab[i as usize] {
                                    retVal = ReturnCode::BZ_DATA_ERROR;
                                    break 'save_state_and_return;
                                } else {
                                    i += 1;
                                }
                            }
                            s.state_out_len = 0;
                            s.state_out_ch = 0_u8;
                            s.calculatedBlockCRC = 0xffffffff as libc::c_long as u32;
                            s.state = State::BZ_X_OUTPUT;
                            if s.verbosity >= 2 {
                                eprint!("rt+rld");
                            }
                            if s.smallDecompress {
                                i = 0;
                                while i <= 256 {
                                    s.cftabCopy[i as usize] = s.cftab[i as usize];
                                    i += 1;
                                }
                                i = 0;
                                while i < nblock {
                                    uc = ll16[i as usize] as u8;
                                    ll16[i as usize] = (s.cftabCopy[uc as usize] & 0xffff) as u16;
                                    if i & 0x1 == 0 {
                                        ll4[(i >> 1) as usize] =
                                            (ll4[(i >> 1) as usize] as libc::c_int & 0xf0
                                                | s.cftabCopy[uc as usize] >> 16)
                                                as u8;
                                    } else {
                                        ll4[(i >> 1) as usize] =
                                            (ll4[(i >> 1) as usize] as libc::c_int & 0xf
                                                | (s.cftabCopy[uc as usize] >> 16) << 4)
                                                as u8;
                                    }
                                    s.cftabCopy[uc as usize] += 1;
                                    s.cftabCopy[uc as usize];
                                    i += 1;
                                }
                                i = s.origPtr;
                                j = (ll16[i as usize] as u32
                                    | (ll4[(i >> 1) as usize] as u32 >> (i << 2 & 0x4) & 0xf) << 16)
                                    as i32;
                                loop {
                                    let tmp_0: i32 = (ll16[j as usize] as u32
                                        | (ll4[(j >> 1) as usize] as u32 >> (j << 2 & 0x4) & 0xf)
                                            << 16)
                                        as i32;
                                    ll16[j as usize] = (i & 0xffff) as u16;
                                    if j & 0x1 == 0 {
                                        ll4[(j >> 1) as usize] = (ll4[(j >> 1) as usize]
                                            as libc::c_int
                                            & 0xf0
                                            | i >> 16)
                                            as u8;
                                    } else {
                                        ll4[(j >> 1) as usize] =
                                            (ll4[(j >> 1) as usize] as libc::c_int & 0xf
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
                                    if s.tPos >= 100000_u32.wrapping_mul(s.blockSize100k as u32) {
                                        // NOTE: this originates in the BZ_GET_FAST macro, and the
                                        // `return true` is probably uninitentional?!
                                        return ReturnCode::BZ_RUN_OK;
                                    }
                                    s.k0 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab);
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
                                    if s.tPos >= 100000_u32.wrapping_mul(s.blockSize100k as u32) {
                                        // NOTE: this originates in the BZ_GET_FAST macro, and the
                                        // `return true` is probably uninitentional?!
                                        return ReturnCode::BZ_RUN_OK;
                                    }
                                    s.k0 = BZ2_indexIntoF(s.tPos as i32, &mut s.cftab);
                                    s.tPos = ll16[s.tPos as usize] as u32
                                        | (ll4[(s.tPos >> 1) as usize] as u32
                                            >> (s.tPos << 2 & 0x4)
                                            & 0xf)
                                            << 16;
                                    s.nblock_used += 1;
                                }
                            } else {
                                i = 0;
                                while i < nblock {
                                    uc = (tt[i as usize] & 0xff) as u8;
                                    let fresh0 = &mut (tt[s.cftab[uc as usize] as usize]);
                                    *fresh0 |= (i << 8) as libc::c_uint;
                                    s.cftab[uc as usize] += 1;
                                    s.cftab[uc as usize];
                                    i += 1;
                                }
                                s.tPos = tt[s.origPtr as usize] >> 8;
                                s.nblock_used = 0;
                                if s.blockRandomised {
                                    s.rNToGo = 0;
                                    s.rTPos = 0;
                                    if s.tPos >= 100000_u32.wrapping_mul(s.blockSize100k as u32) {
                                        // NOTE: this originates in the BZ_GET_FAST macro, and the
                                        // `return true` is probably uninitentional?!
                                        return ReturnCode::BZ_RUN_OK;
                                    }
                                    s.tPos = tt[s.tPos as usize];
                                    s.k0 = (s.tPos & 0xff) as u8 as i32;
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
                                    if s.tPos >= 100000_u32.wrapping_mul(s.blockSize100k as u32) {
                                        // NOTE: this originates in the BZ_GET_FAST macro, and the
                                        // `return true` is probably uninitentional?!
                                        return ReturnCode::BZ_RUN_OK;
                                    }
                                    s.tPos = tt[s.tPos as usize];
                                    s.k0 = (s.tPos & 0xff) as u8 as i32;
                                    s.tPos >>= 8;
                                    s.nblock_used += 1;
                                }
                            }
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue;
                        }
                    }
                }
            }
            if current_block == 5649595406143318745 {
                if N >= 2 * 1024 * 1024 {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                } else {
                    if nextSym == 0 {
                        es += N;
                    } else if nextSym == 1 {
                        es += (1 + 1) * N;
                    }
                    N *= 2;
                    update_group_pos!(s);
                    zn = gMinlen;
                    current_block = 9335356017384149594;
                    continue;
                }
            }
            loop {
                match current_block {
                    16953886395775657100 => {
                        if j < 16 {
                            current_block = 15451013008180677144;
                            continue 'c_10064;
                        }
                    }
                    3503188808869013853 => {
                        if i < nSelectors {
                            j = 0;
                            current_block = 16531797892856733396;
                            continue;
                        } else {
                            if nSelectors > 2 + 900000 / 50 {
                                nSelectors = 2 + 900000 / 50;
                            }
                            let mut pos: [u8; 6] = [0; 6];
                            let mut tmp: u8;
                            let mut v_22: u8;
                            v_22 = 0_u8;
                            while (v_22 as libc::c_int) < nGroups {
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
                            current_block = 2488856075421756534;
                            break;
                        }
                    }
                    15415362524153386998 => {
                        if i < 16 {
                            if s.inUse16[i as usize] {
                                j = 0;
                                current_block = 16953886395775657100;
                                continue;
                            }
                        } else {
                            makeMaps_d(s);

                            // reborrow
                            tt = s.tt.as_mut_slice();
                            ll16 = s.ll16.as_mut_slice();
                            ll4 = s.ll4.as_mut_slice();

                            if s.nInUse == 0 {
                                current_block = 12571193857528100212;
                                break;
                            } else {
                                current_block = 9416928054198617439;
                                break;
                            }
                        }
                    }
                    7746242308555130918 => {
                        s.len[t as usize][i as usize] = curr as u8;
                        i += 1;
                        current_block = 16642413284942005565;
                        continue;
                    }
                    16642413284942005565 => {
                        if i < alphaSize {
                            current_block = 5533056661327372531;
                            continue;
                        }
                        t += 1;
                        current_block = 2488856075421756534;
                        break;
                    }
                    10081471997089450706 => {
                        if i < 2 + 900000 / 50 {
                            s.selectorMtf[i as usize] = j as u8;
                        }
                        i += 1;
                        current_block = 3503188808869013853;
                        continue;
                    }
                    16531797892856733396 => {
                        if 1 != 0 {
                            current_block = 15957329598978927534;
                            continue 'c_10064;
                        } else {
                            current_block = 10081471997089450706;
                            continue;
                        }
                    }
                    _ => {
                        if false {
                            current_block = 7746242308555130918;
                            continue;
                        }
                        if !!(1..=20).contains(&curr) {
                            current_block = 17216244326479313607;
                            continue 'c_10064;
                        }
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        break 'save_state_and_return;
                    }
                }
                i += 1;
                current_block = 15415362524153386998;
            }
            match current_block {
                9416928054198617439 => {
                    alphaSize = s.nInUse + 2;
                    current_block = 9434444550647791986;
                }
                12571193857528100212 => {
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    break 'save_state_and_return;
                }
                _ => {
                    if t < nGroups {
                        current_block = 11569294379105328467;
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
                        BZ2_hbCreateDecodeTables(
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
                    current_block = 13155828021133314705;
                }
            }
        }
    }

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

    retVal
}

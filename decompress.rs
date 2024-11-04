use crate::bzlib::{bz_stream, BZ2_bz__AssertH__fail, BZ2_indexIntoF, DState, ReturnCode};
use crate::huffman::BZ2_hbCreateDecodeTables;
use crate::randtable::BZ2_RNUMS;

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
    let mut uc: u8 = 0;
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
    let mut gLimit: *mut i32;
    let mut gBase: *mut i32;
    let mut gPerm: *mut i32;

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
        s.save_gLimit = std::ptr::null_mut::<i32>();
        s.save_gBase = std::ptr::null_mut::<i32>();
        s.save_gPerm = std::ptr::null_mut::<i32>();
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

        match s.state {
            State::BZ_X_MAGIC_1 => {
                s.state = State::BZ_X_MAGIC_1;

                GET_UCHAR!(strm, s, uc);

                if uc != b'B' {
                    retVal = ReturnCode::BZ_DATA_ERROR_MAGIC;
                    break 'save_state_and_return;
                } else {
                    current_block = 15360092558900836893;
                }
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
            _ => {
                if 0 == 0 {
                    BZ2_bz__AssertH__fail(4001);
                }
                if 0 == 0 {
                    BZ2_bz__AssertH__fail(4002);
                }
                current_block = SAVE_STATE_AND_RETURN;
            }
        }
        if current_block == 15360092558900836893 {
            s.state = State::BZ_X_MAGIC_2;
            loop {
                if s.bsLive >= 8 {
                    let v_0: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                    s.bsLive -= 8;
                    uc = v_0 as u8;
                    current_block = 2168227384378665163;
                    break;
                } else if strm.avail_in == 0 {
                    retVal = ReturnCode::BZ_OK;
                    current_block = SAVE_STATE_AND_RETURN;
                    break;
                } else {
                    s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                    s.bsLive += 8;
                    strm.next_in = (strm.next_in).offset(1);
                    strm.avail_in = (strm.avail_in).wrapping_sub(1);
                    strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                    if strm.total_in_lo32 == 0 {
                        strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                    }
                }
            }
            match current_block {
                SAVE_STATE_AND_RETURN => {}
                _ => {
                    if uc != 0x5a {
                        retVal = ReturnCode::BZ_DATA_ERROR_MAGIC;
                        current_block = SAVE_STATE_AND_RETURN;
                    } else {
                        current_block = 15953825877604003206;
                    }
                }
            }
        }
        if current_block == 15953825877604003206 {
            s.state = State::BZ_X_MAGIC_3;
            loop {
                if s.bsLive >= 8 {
                    let v_1: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                    s.bsLive -= 8;
                    uc = v_1 as u8;
                    current_block = 178030534879405462;
                    break;
                } else if strm.avail_in == 0 {
                    retVal = ReturnCode::BZ_OK;
                    current_block = SAVE_STATE_AND_RETURN;
                    break;
                } else {
                    s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                    s.bsLive += 8;
                    strm.next_in = (strm.next_in).offset(1);
                    strm.avail_in = (strm.avail_in).wrapping_sub(1);
                    strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                    if strm.total_in_lo32 == 0 {
                        strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                    }
                }
            }
            match current_block {
                SAVE_STATE_AND_RETURN => {}
                _ => {
                    if uc != 0x68 {
                        retVal = ReturnCode::BZ_DATA_ERROR_MAGIC;
                        current_block = SAVE_STATE_AND_RETURN;
                    } else {
                        current_block = 1137006006685247392;
                    }
                }
            }
        }
        if current_block == 1137006006685247392 {
            s.state = State::BZ_X_MAGIC_4;
            loop {
                if s.bsLive >= 8 {
                    let v_2: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                    s.bsLive -= 8;
                    s.blockSize100k = v_2 as i32;
                    current_block = 7639320476250304355;
                    break;
                } else if strm.avail_in == 0 {
                    retVal = ReturnCode::BZ_OK;
                    current_block = SAVE_STATE_AND_RETURN;
                    break;
                } else {
                    s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                    s.bsLive += 8;
                    strm.next_in = (strm.next_in).offset(1);
                    strm.avail_in = (strm.avail_in).wrapping_sub(1);
                    strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                    if strm.total_in_lo32 == 0 {
                        strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                    }
                }
            }
            match current_block {
                SAVE_STATE_AND_RETURN => {}
                _ => {
                    if s.blockSize100k < 0x30 + 1 || s.blockSize100k > 0x30 + 9 {
                        retVal = ReturnCode::BZ_DATA_ERROR_MAGIC;
                        current_block = SAVE_STATE_AND_RETURN;
                    } else {
                        s.blockSize100k -= 0x30;
                        if s.smallDecompress {
                            let ll16_len = s.blockSize100k as usize * 100000;
                            s.ll16 = (strm.bzalloc).expect("non-null function pointer")(
                                strm.opaque,
                                (ll16_len as libc::c_ulong)
                                    .wrapping_mul(::core::mem::size_of::<u16>() as libc::c_ulong)
                                    as libc::c_int,
                                1,
                            ) as *mut u16;

                            let ll4_len = (1 + s.blockSize100k as usize * 100000) >> 1;
                            s.ll4 = (strm.bzalloc).expect("non-null function pointer")(
                                strm.opaque,
                                (ll4_len as libc::c_ulong)
                                    .wrapping_mul(::core::mem::size_of::<u8>() as libc::c_ulong)
                                    as libc::c_int,
                                1,
                            ) as *mut u8;

                            if (s.ll16).is_null() || (s.ll4).is_null() {
                                retVal = ReturnCode::BZ_MEM_ERROR;
                                current_block = SAVE_STATE_AND_RETURN;
                            } else {
                                // NOTE: bzip2 does not initialize this memory
                                core::ptr::write_bytes(s.ll16, 0, ll16_len);
                                core::ptr::write_bytes(s.ll4, 0, ll4_len);

                                current_block = 16838365919992687769;
                            }
                        } else {
                            s.tt = (strm.bzalloc).expect("non-null function pointer")(
                                strm.opaque,
                                ((s.blockSize100k * 100000) as libc::c_ulong)
                                    .wrapping_mul(::core::mem::size_of::<i32>() as libc::c_ulong)
                                    as libc::c_int,
                                1,
                            ) as *mut u32;
                            if (s.tt).is_null() {
                                retVal = ReturnCode::BZ_MEM_ERROR;
                                current_block = SAVE_STATE_AND_RETURN;
                            } else {
                                current_block = 16838365919992687769;
                            }
                        }
                    }
                }
            }
        }
        if current_block == 16838365919992687769 {
            s.state = State::BZ_X_BLKHDR_1;
            loop {
                if s.bsLive >= 8 {
                    let v_3: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                    s.bsLive -= 8;
                    uc = v_3 as u8;
                    current_block = 16937825661756021828;
                    break;
                } else if strm.avail_in == 0 {
                    retVal = ReturnCode::BZ_OK;
                    current_block = SAVE_STATE_AND_RETURN;
                    break;
                } else {
                    s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                    s.bsLive += 8;
                    strm.next_in = (strm.next_in).offset(1);
                    strm.avail_in = (strm.avail_in).wrapping_sub(1);
                    strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                    if strm.total_in_lo32 == 0 {
                        strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                    }
                }
            }
            match current_block {
                SAVE_STATE_AND_RETURN => {}
                _ => {
                    if uc == 0x17 {
                        current_block = 14366592556287126287;
                    } else if uc != 0x31 {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        current_block = SAVE_STATE_AND_RETURN;
                    } else {
                        current_block = 5889181040567946013;
                    }
                }
            }
        }
        match current_block {
            14366592556287126287 => {
                s.state = State::BZ_X_ENDHDR_2;
                loop {
                    if s.bsLive >= 8 {
                        let v_32: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_32 as u8;
                        current_block = 13733404100380861831;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x72 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            current_block = 7651522734817633728;
                        }
                    }
                }
            }
            5889181040567946013 => {
                s.state = State::BZ_X_BLKHDR_2;
                loop {
                    if s.bsLive >= 8 {
                        let v_4: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_4 as u8;
                        current_block = 1228639923084383292;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x41 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            current_block = 887841530443712878;
                        }
                    }
                }
            }
            _ => {}
        }
        match current_block {
            7651522734817633728 => {
                s.state = State::BZ_X_ENDHDR_3;
                loop {
                    if s.bsLive >= 8 {
                        let v_33: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_33 as u8;
                        current_block = 12721425419429475574;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x45 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            current_block = 15818849443713787272;
                        }
                    }
                }
            }
            887841530443712878 => {
                s.state = State::BZ_X_BLKHDR_3;
                loop {
                    if s.bsLive >= 8 {
                        let v_5: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_5 as u8;
                        current_block = 9235179519944561532;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x59 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            current_block = 17767742176799939193;
                        }
                    }
                }
            }
            _ => {}
        }
        match current_block {
            15818849443713787272 => {
                s.state = State::BZ_X_ENDHDR_4;
                loop {
                    if s.bsLive >= 8 {
                        let v_34: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_34 as u8;
                        current_block = 13813414375753095368;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x38 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            current_block = 15153555825877660840;
                        }
                    }
                }
            }
            17767742176799939193 => {
                s.state = State::BZ_X_BLKHDR_4;
                loop {
                    if s.bsLive >= 8 {
                        let v_6: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_6 as u8;
                        current_block = 12467039471581323981;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x26 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            current_block = 16325921850189496668;
                        }
                    }
                }
            }
            _ => {}
        }
        match current_block {
            15153555825877660840 => {
                s.state = State::BZ_X_ENDHDR_5;
                loop {
                    if s.bsLive >= 8 {
                        let v_35: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_35 as u8;
                        current_block = 1472103348880861285;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x50 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            current_block = 1857046018890652364;
                        }
                    }
                }
            }
            16325921850189496668 => {
                s.state = State::BZ_X_BLKHDR_5;
                loop {
                    if s.bsLive >= 8 {
                        let v_7: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_7 as u8;
                        current_block = 13164310931121142693;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x53 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            current_block = 3202472413399101603;
                        }
                    }
                }
            }
            _ => {}
        }
        match current_block {
            1857046018890652364 => {
                s.state = State::BZ_X_ENDHDR_6;
                loop {
                    if s.bsLive >= 8 {
                        let v_36: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_36 as u8;
                        current_block = 8232347840743503282;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x90 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            s.storedCombinedCRC = 0_u32;
                            current_block = 10292318171587122742;
                        }
                    }
                }
            }
            3202472413399101603 => {
                s.state = State::BZ_X_BLKHDR_6;
                loop {
                    if s.bsLive >= 8 {
                        let v_8: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_8 as u8;
                        current_block = 14723615986260991866;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        if uc != 0x59 {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                        } else {
                            s.currBlockNo += 1;
                            if s.verbosity >= 2 {
                                eprint!("\n    [{}: huff+mtf ", s.currBlockNo);
                            }
                            s.storedBlockCRC = 0_u32;
                            current_block = 5821827988509819404;
                        }
                    }
                }
            }
            _ => {}
        }
        match current_block {
            10292318171587122742 => {
                s.state = State::BZ_X_CCRC_1;
                loop {
                    if s.bsLive >= 8 {
                        let v_37: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_37 as u8;
                        current_block = 5465979950226085365;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                        current_block = 14748314904637597825;
                    }
                }
            }
            5821827988509819404 => {
                s.state = State::BZ_X_BCRC_1;
                loop {
                    if s.bsLive >= 8 {
                        let v_9: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_9 as u8;
                        current_block = 15627786036016112248;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                        current_block = 5023088878038355716;
                    }
                }
            }
            _ => {}
        }
        match current_block {
            14748314904637597825 => {
                s.state = State::BZ_X_CCRC_2;
                loop {
                    if s.bsLive >= 8 {
                        let v_38: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_38 as u8;
                        current_block = 3854366583354019639;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                        current_block = 4092966239614665407;
                    }
                }
            }
            5023088878038355716 => {
                s.state = State::BZ_X_BCRC_2;
                loop {
                    if s.bsLive >= 8 {
                        let v_10: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_10 as u8;
                        current_block = 13493279574219925475;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                        current_block = 8515868523999336537;
                    }
                }
            }
            _ => {}
        }
        match current_block {
            4092966239614665407 => {
                s.state = State::BZ_X_CCRC_3;
                loop {
                    if s.bsLive >= 8 {
                        let v_39: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_39 as u8;
                        current_block = 12082794684616777938;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                        current_block = 18389040574536762539;
                    }
                }
            }
            8515868523999336537 => {
                s.state = State::BZ_X_BCRC_3;
                loop {
                    if s.bsLive >= 8 {
                        let v_11: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_11 as u8;
                        current_block = 4839309778395429725;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                        current_block = 18234918597811156654;
                    }
                }
            }
            _ => {}
        }
        match current_block {
            18234918597811156654 => {
                s.state = State::BZ_X_BCRC_4;
                loop {
                    if s.bsLive >= 8 {
                        let v_12: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_12 as u8;
                        current_block = 17937968408868551711;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        s.storedBlockCRC = s.storedBlockCRC << 8 | uc as u32;
                        current_block = 12310871532727186508;
                    }
                }
            }
            18389040574536762539 => {
                s.state = State::BZ_X_CCRC_4;
                loop {
                    if s.bsLive >= 8 {
                        let v_40: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                        s.bsLive -= 8;
                        uc = v_40 as u8;
                        current_block = 6276941480907995842;
                        break;
                    } else if strm.avail_in == 0 {
                        retVal = ReturnCode::BZ_OK;
                        current_block = SAVE_STATE_AND_RETURN;
                        break;
                    } else {
                        s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                        s.bsLive += 8;
                        strm.next_in = (strm.next_in).offset(1);
                        strm.avail_in = (strm.avail_in).wrapping_sub(1);
                        strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                        if strm.total_in_lo32 == 0 {
                            strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                        }
                    }
                }
                match current_block {
                    SAVE_STATE_AND_RETURN => {}
                    _ => {
                        s.storedCombinedCRC = s.storedCombinedCRC << 8 | uc as u32;
                        s.state = State::BZ_X_IDLE;
                        retVal = ReturnCode::BZ_STREAM_END;
                        current_block = SAVE_STATE_AND_RETURN;
                    }
                }
            }
            _ => {}
        }
        if current_block == 12310871532727186508 {
            s.state = State::BZ_X_RANDBIT;
            loop {
                if s.bsLive >= 1 {
                    let v_13: u32 = s.bsBuff >> (s.bsLive - 1) & (((1) << 1) - 1);
                    s.bsLive -= 1;
                    s.blockRandomised = v_13 != 0;
                    current_block = 7926734633677835471;
                    break;
                } else if strm.avail_in == 0 {
                    retVal = ReturnCode::BZ_OK;
                    current_block = SAVE_STATE_AND_RETURN;
                    break;
                } else {
                    s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                    s.bsLive += 8;
                    strm.next_in = (strm.next_in).offset(1);
                    strm.avail_in = (strm.avail_in).wrapping_sub(1);
                    strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                    if strm.total_in_lo32 == 0 {
                        strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                    }
                }
            }
            match current_block {
                SAVE_STATE_AND_RETURN => {}
                _ => {
                    s.origPtr = 0;
                    current_block = 3338455798814466984;
                }
            }
        }
        if current_block == 3338455798814466984 {
            s.state = State::BZ_X_ORIGPTR_1;
            loop {
                if s.bsLive >= 8 {
                    let v_14: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                    s.bsLive -= 8;
                    uc = v_14 as u8;
                    current_block = 5948065351908552372;
                    break;
                } else if strm.avail_in == 0 {
                    retVal = ReturnCode::BZ_OK;
                    current_block = SAVE_STATE_AND_RETURN;
                    break;
                } else {
                    s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                    s.bsLive += 8;
                    strm.next_in = (strm.next_in).offset(1);
                    strm.avail_in = (strm.avail_in).wrapping_sub(1);
                    strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                    if strm.total_in_lo32 == 0 {
                        strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                    }
                }
            }
            match current_block {
                SAVE_STATE_AND_RETURN => {}
                _ => {
                    s.origPtr = s.origPtr << 8 | uc as i32;
                    current_block = 10262367570716242252;
                }
            }
        }
        if current_block == 10262367570716242252 {
            s.state = State::BZ_X_ORIGPTR_2;
            loop {
                if s.bsLive >= 8 {
                    let v_15: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                    s.bsLive -= 8;
                    uc = v_15 as u8;
                    current_block = 8940662058537996670;
                    break;
                } else if strm.avail_in == 0 {
                    retVal = ReturnCode::BZ_OK;
                    current_block = SAVE_STATE_AND_RETURN;
                    break;
                } else {
                    s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                    s.bsLive += 8;
                    strm.next_in = (strm.next_in).offset(1);
                    strm.avail_in = (strm.avail_in).wrapping_sub(1);
                    strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                    if strm.total_in_lo32 == 0 {
                        strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                    }
                }
            }
            match current_block {
                SAVE_STATE_AND_RETURN => {}
                _ => {
                    s.origPtr = s.origPtr << 8 | uc as i32;
                    current_block = 17024493544560437554;
                }
            }
        }
        if current_block == 17024493544560437554 {
            s.state = State::BZ_X_ORIGPTR_3;
            loop {
                if s.bsLive >= 8 {
                    let v_16: u32 = s.bsBuff >> (s.bsLive - 8) & (((1) << 8) - 1);
                    s.bsLive -= 8;
                    uc = v_16 as u8;
                    current_block = 13366002463409402866;
                    break;
                } else if strm.avail_in == 0 {
                    retVal = ReturnCode::BZ_OK;
                    current_block = SAVE_STATE_AND_RETURN;
                    break;
                } else {
                    s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                    s.bsLive += 8;
                    strm.next_in = (strm.next_in).offset(1);
                    strm.avail_in = (strm.avail_in).wrapping_sub(1);
                    strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                    if strm.total_in_lo32 == 0 {
                        strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                    }
                }
            }
            match current_block {
                SAVE_STATE_AND_RETURN => {}
                _ => {
                    s.origPtr = s.origPtr << 8 | uc as i32;
                    if s.origPtr < 0 {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        current_block = SAVE_STATE_AND_RETURN;
                    } else if s.origPtr > 10 + 100000 * s.blockSize100k {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        current_block = SAVE_STATE_AND_RETURN;
                    } else {
                        i = 0;
                        current_block = 454873545234741267;
                    }
                }
            }
        }
        'c_10064: loop {
            match current_block {
                SAVE_STATE_AND_RETURN => {
                    s.save_i = i;
                    break;
                }
                9050093969003559074 => {
                    s.state = State::BZ_X_MTF_5;
                    loop {
                        if s.bsLive >= zn {
                            let v_30: u32 = s.bsBuff >> (s.bsLive - zn) & (((1) << zn) - 1);
                            s.bsLive -= zn;
                            zvec = v_30 as i32;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    current_block = 16348713635569416413;
                }
                12127014564286193091 => {
                    s.state = State::BZ_X_MTF_4;
                    loop {
                        if s.bsLive >= 1 {
                            let v_29: u32 = s.bsBuff >> (s.bsLive - 1) & (((1) << 1) - 1);
                            s.bsLive -= 1;
                            zj = v_29 as i32;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    zvec = zvec << 1 | zj;
                    current_block = 7923635230025172457;
                }
                9335356017384149594 => {
                    s.state = State::BZ_X_MTF_3;
                    loop {
                        if s.bsLive >= zn {
                            let v_28: u32 = s.bsBuff >> (s.bsLive - zn) & (((1) << zn) - 1);
                            s.bsLive -= zn;
                            zvec = v_28 as i32;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    current_block = 7923635230025172457;
                }
                1010107409739284736 => {
                    s.state = State::BZ_X_MTF_2;
                    loop {
                        if s.bsLive >= 1 {
                            let v_27: u32 = s.bsBuff >> (s.bsLive - 1) & (((1) << 1) - 1);
                            s.bsLive -= 1;
                            zj = v_27 as i32;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    zvec = zvec << 1 | zj;
                    current_block = 9186389159759284570;
                }
                13155828021133314705 => {
                    s.state = State::BZ_X_MTF_1;
                    loop {
                        if s.bsLive >= zn {
                            let v_26: u32 = s.bsBuff >> (s.bsLive - zn) & (((1) << zn) - 1);
                            s.bsLive -= zn;
                            zvec = v_26 as i32;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    current_block = 9186389159759284570;
                }
                7191958063352112897 => {
                    s.state = State::BZ_X_CODING_3;
                    loop {
                        if s.bsLive >= 1 {
                            let v_25: u32 = s.bsBuff >> (s.bsLive - 1) & (((1) << 1) - 1);
                            s.bsLive -= 1;
                            uc = v_25 as u8;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    if uc == 0 {
                        curr += 1;
                    } else {
                        curr -= 1;
                    }
                    current_block = 5533056661327372531;
                }
                17216244326479313607 => {
                    s.state = State::BZ_X_CODING_2;
                    loop {
                        if s.bsLive >= 1 {
                            let v_24: u32 = s.bsBuff >> (s.bsLive - 1) & (((1) << 1) - 1);
                            s.bsLive -= 1;
                            uc = v_24 as u8;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    if uc != 0 {
                        current_block = 7191958063352112897;
                        continue;
                    }
                    current_block = 7746242308555130918;
                }
                11569294379105328467 => {
                    s.state = State::BZ_X_CODING_1;
                    loop {
                        if s.bsLive >= 5 {
                            let v_23: u32 = s.bsBuff >> (s.bsLive - 5) & (((1) << 5) - 1);
                            s.bsLive -= 5;
                            curr = v_23 as i32;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    i = 0;
                    current_block = 16642413284942005565;
                }
                15957329598978927534 => {
                    s.state = State::BZ_X_SELECTOR_3;
                    loop {
                        if s.bsLive >= 1 {
                            let v_21: u32 = s.bsBuff >> (s.bsLive - 1) & (((1) << 1) - 1);
                            s.bsLive -= 1;
                            uc = v_21 as u8;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    if uc == 0 {
                        current_block = 10081471997089450706;
                    } else {
                        j += 1;
                        if j >= nGroups {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue;
                        } else {
                            current_block = 16531797892856733396;
                        }
                    }
                }
                14590825336193814119 => {
                    s.state = State::BZ_X_SELECTOR_2;
                    loop {
                        if s.bsLive >= 15 {
                            let v_20: u32 = s.bsBuff >> (s.bsLive - 15) & (((1) << 15) - 1);
                            s.bsLive -= 15;
                            nSelectors = v_20 as i32;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    if nSelectors < 1 {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        current_block = SAVE_STATE_AND_RETURN;
                        continue;
                    } else {
                        i = 0;
                    }
                    current_block = 3503188808869013853;
                }
                9434444550647791986 => {
                    s.state = State::BZ_X_SELECTOR_1;
                    loop {
                        if s.bsLive >= 3 {
                            let v_19: u32 = s.bsBuff >> (s.bsLive - 3) & (((1) << 3) - 1);
                            s.bsLive -= 3;
                            nGroups = v_19 as i32;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    if !!(2..=6).contains(&nGroups) {
                        current_block = 14590825336193814119;
                        continue;
                    }
                    retVal = ReturnCode::BZ_DATA_ERROR;
                    current_block = SAVE_STATE_AND_RETURN;
                    continue;
                }
                15451013008180677144 => {
                    s.state = State::BZ_X_MAPPING_2;
                    loop {
                        if s.bsLive >= 1 {
                            let v_18: u32 = s.bsBuff >> (s.bsLive - 1) & (((1) << 1) - 1);
                            s.bsLive -= 1;
                            uc = v_18 as u8;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
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
                    loop {
                        if s.bsLive >= 1 {
                            let v_17: u32 = s.bsBuff >> (s.bsLive - 1) & (((1) << 1) - 1);
                            s.bsLive -= 1;
                            uc = v_17 as u8;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    s.inUse16[i as usize] = uc == 1;
                    i += 1;
                    current_block = 454873545234741267;
                    continue;
                }
                _ => {
                    s.state = State::BZ_X_MTF_6;
                    loop {
                        if s.bsLive >= 1 {
                            let v_31: u32 = s.bsBuff >> (s.bsLive - 1) & (((1) << 1) - 1);
                            s.bsLive -= 1;
                            zj = v_31 as i32;
                            break;
                        } else if strm.avail_in == 0 {
                            retVal = ReturnCode::BZ_OK;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue 'c_10064;
                        } else {
                            s.bsBuff = s.bsBuff << 8 | *(strm.next_in as *mut u8) as u32;
                            s.bsLive += 8;
                            strm.next_in = (strm.next_in).offset(1);
                            strm.avail_in = (strm.avail_in).wrapping_sub(1);
                            strm.total_in_lo32 = (strm.total_in_lo32).wrapping_add(1);
                            if strm.total_in_lo32 == 0 {
                                strm.total_in_hi32 = (strm.total_in_hi32).wrapping_add(1);
                            }
                        }
                    }
                    zvec = zvec << 1 | zj;
                    current_block = 16348713635569416413;
                }
            }
            match current_block {
                16348713635569416413 => {
                    if zn > 20 {
                        retVal = ReturnCode::BZ_DATA_ERROR;
                        current_block = SAVE_STATE_AND_RETURN;
                        continue;
                    } else if zvec <= *gLimit.offset(zn as isize) {
                        if zvec - *gBase.offset(zn as isize) < 0
                            || zvec - *gBase.offset(zn as isize) >= 258
                        {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue;
                        } else {
                            nextSym = *gPerm.offset((zvec - *gBase.offset(zn as isize)) as isize);
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
                        current_block = SAVE_STATE_AND_RETURN;
                        continue;
                    } else if zvec <= *gLimit.offset(zn as isize) {
                        if zvec - *gBase.offset(zn as isize) < 0
                            || zvec - *gBase.offset(zn as isize) >= 258
                        {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue;
                        } else {
                            nextSym = *gPerm.offset((zvec - *gBase.offset(zn as isize)) as isize);
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
                                            current_block = SAVE_STATE_AND_RETURN;
                                            continue 'c_10064;
                                        } else {
                                            *(s.ll16).offset(nblock as isize) = uc as u16;
                                            nblock += 1;
                                            es -= 1;
                                        }
                                    }
                                } else {
                                    while es > 0 {
                                        if nblock >= nblockMAX {
                                            retVal = ReturnCode::BZ_DATA_ERROR;
                                            current_block = SAVE_STATE_AND_RETURN;
                                            continue 'c_10064;
                                        } else {
                                            *(s.tt).offset(nblock as isize) = uc as u32;
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
                        current_block = SAVE_STATE_AND_RETURN;
                        continue;
                    } else if zvec <= *gLimit.offset(zn as isize) {
                        if zvec - *gBase.offset(zn as isize) < 0
                            || zvec - *gBase.offset(zn as isize) >= 258
                        {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue;
                        } else {
                            nextSym = *gPerm.offset((zvec - *gBase.offset(zn as isize)) as isize);
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
                            current_block = SAVE_STATE_AND_RETURN;
                            continue;
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
                                *(s.ll16).offset(nblock as isize) =
                                    s.seqToUnseq[uc as usize] as u16;
                            } else {
                                *(s.tt).offset(nblock as isize) = s.seqToUnseq[uc as usize] as u32;
                            }
                            nblock += 1;
                            if groupPos == 0 {
                                groupNo += 1;
                                if groupNo >= nSelectors {
                                    retVal = ReturnCode::BZ_DATA_ERROR;
                                    current_block = SAVE_STATE_AND_RETURN;
                                    continue;
                                } else {
                                    groupPos = 50;
                                    gSel = s.selector[groupNo as usize] as i32;
                                    gMinlen = s.minLens[gSel as usize];
                                    gLimit = (*(s.limit).as_mut_ptr().offset(gSel as isize))
                                        .as_mut_ptr()
                                        .offset(0_isize)
                                        as *mut i32;
                                    gPerm = (*(s.perm).as_mut_ptr().offset(gSel as isize))
                                        .as_mut_ptr()
                                        .offset(0_isize)
                                        as *mut i32;
                                    gBase = (*(s.base).as_mut_ptr().offset(gSel as isize))
                                        .as_mut_ptr()
                                        .offset(0_isize)
                                        as *mut i32;
                                }
                            }
                            groupPos -= 1;
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
                            current_block = SAVE_STATE_AND_RETURN;
                            continue;
                        } else {
                            i = 0;
                            while i <= 255 {
                                if s.unzftab[i as usize] < 0 || s.unzftab[i as usize] > nblock {
                                    retVal = ReturnCode::BZ_DATA_ERROR;
                                    current_block = SAVE_STATE_AND_RETURN;
                                    continue 'c_10064;
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
                                    current_block = SAVE_STATE_AND_RETURN;
                                    continue 'c_10064;
                                } else {
                                    i += 1;
                                }
                            }
                            i = 1;
                            while i <= 256 {
                                if s.cftab[(i - 1) as usize] > s.cftab[i as usize] {
                                    retVal = ReturnCode::BZ_DATA_ERROR;
                                    current_block = SAVE_STATE_AND_RETURN;
                                    continue 'c_10064;
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
                                    uc = *(s.ll16).offset(i as isize) as u8;
                                    *(s.ll16).offset(i as isize) =
                                        (s.cftabCopy[uc as usize] & 0xffff) as u16;
                                    if i & 0x1 == 0 {
                                        *(s.ll4).offset((i >> 1) as isize) =
                                            (*(s.ll4).offset((i >> 1) as isize) as libc::c_int
                                                & 0xf0
                                                | s.cftabCopy[uc as usize] >> 16)
                                                as u8;
                                    } else {
                                        *(s.ll4).offset((i >> 1) as isize) =
                                            (*(s.ll4).offset((i >> 1) as isize) as libc::c_int
                                                & 0xf
                                                | (s.cftabCopy[uc as usize] >> 16) << 4)
                                                as u8;
                                    }
                                    s.cftabCopy[uc as usize] += 1;
                                    s.cftabCopy[uc as usize];
                                    i += 1;
                                }
                                i = s.origPtr;
                                j = (*(s.ll16).offset(i as isize) as u32
                                    | (*(s.ll4).offset((i >> 1) as isize) as u32 >> (i << 2 & 0x4)
                                        & 0xf)
                                        << 16) as i32;
                                loop {
                                    let tmp_0: i32 = (*(s.ll16).offset(j as isize) as u32
                                        | (*(s.ll4).offset((j >> 1) as isize) as u32
                                            >> (j << 2 & 0x4)
                                            & 0xf)
                                            << 16)
                                        as i32;
                                    *(s.ll16).offset(j as isize) = (i & 0xffff) as u16;
                                    if j & 0x1 == 0 {
                                        *(s.ll4).offset((j >> 1) as isize) =
                                            (*(s.ll4).offset((j >> 1) as isize) as libc::c_int
                                                & 0xf0
                                                | i >> 16)
                                                as u8;
                                    } else {
                                        *(s.ll4).offset((j >> 1) as isize) =
                                            (*(s.ll4).offset((j >> 1) as isize) as libc::c_int
                                                & 0xf
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
                                    s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                                        | (*(s.ll4).offset((s.tPos >> 1) as isize) as u32
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
                                    s.tPos = *(s.ll16).offset(s.tPos as isize) as u32
                                        | (*(s.ll4).offset((s.tPos >> 1) as isize) as u32
                                            >> (s.tPos << 2 & 0x4)
                                            & 0xf)
                                            << 16;
                                    s.nblock_used += 1;
                                }
                            } else {
                                i = 0;
                                while i < nblock {
                                    uc = (*(s.tt).offset(i as isize) & 0xff) as u8;
                                    let fresh0 =
                                        &mut (*(s.tt).offset(s.cftab[uc as usize] as isize));
                                    *fresh0 |= (i << 8) as libc::c_uint;
                                    s.cftab[uc as usize] += 1;
                                    s.cftab[uc as usize];
                                    i += 1;
                                }
                                s.tPos = *(s.tt).offset(s.origPtr as isize) >> 8;
                                s.nblock_used = 0;
                                if s.blockRandomised {
                                    s.rNToGo = 0;
                                    s.rTPos = 0;
                                    if s.tPos >= 100000_u32.wrapping_mul(s.blockSize100k as u32) {
                                        // NOTE: this originates in the BZ_GET_FAST macro, and the
                                        // `return true` is probably uninitentional?!
                                        return ReturnCode::BZ_RUN_OK;
                                    }
                                    s.tPos = *(s.tt).offset(s.tPos as isize);
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
                                    s.tPos = *(s.tt).offset(s.tPos as isize);
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
                    current_block = SAVE_STATE_AND_RETURN;
                    continue;
                } else {
                    if nextSym == 0 {
                        es += N;
                    } else if nextSym == 1 {
                        es += (1 + 1) * N;
                    }
                    N *= 2;
                    if groupPos == 0 {
                        groupNo += 1;
                        if groupNo >= nSelectors {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue;
                        } else {
                            groupPos = 50;
                            gSel = s.selector[groupNo as usize] as i32;
                            gMinlen = s.minLens[gSel as usize];
                            gLimit = (*(s.limit).as_mut_ptr().offset(gSel as isize))
                                .as_mut_ptr()
                                .offset(0_isize) as *mut i32;
                            gPerm = (*(s.perm).as_mut_ptr().offset(gSel as isize))
                                .as_mut_ptr()
                                .offset(0_isize) as *mut i32;
                            gBase = (*(s.base).as_mut_ptr().offset(gSel as isize))
                                .as_mut_ptr()
                                .offset(0_isize) as *mut i32;
                        }
                    }
                    groupPos -= 1;
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
                        current_block = SAVE_STATE_AND_RETURN;
                        continue 'c_10064;
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
                    current_block = SAVE_STATE_AND_RETURN;
                }
                _ => {
                    if t < nGroups {
                        current_block = 11569294379105328467;
                        continue;
                    }
                    t = 0;
                    while t < nGroups {
                        minLen = 32;
                        maxLen = 0;
                        i = 0;
                        while i < alphaSize {
                            if s.len[t as usize][i as usize] as libc::c_int > maxLen {
                                maxLen = s.len[t as usize][i as usize] as i32;
                            }
                            if (s.len[t as usize][i as usize] as libc::c_int) < minLen {
                                minLen = s.len[t as usize][i as usize] as i32;
                            }
                            i += 1;
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
                        t += 1;
                    }
                    EOB = s.nInUse + 1;
                    nblockMAX = 100000 * s.blockSize100k;
                    groupNo = -1;
                    groupPos = 0;
                    i = 0;
                    while i <= 255 {
                        s.unzftab[i as usize] = 0;
                        i += 1;
                    }
                    let mut ii: i32;
                    let mut jj: i32;
                    let mut kk: i32;
                    kk = 4096 - 1;
                    ii = 256 / 16 - 1;
                    while ii >= 0 {
                        jj = 16 - 1;
                        while jj >= 0 {
                            s.mtfa[kk as usize] = (ii * 16 + jj) as u8;
                            kk -= 1;
                            jj -= 1;
                        }
                        s.mtfbase[ii as usize] = kk + 1;
                        ii -= 1;
                    }
                    nblock = 0;
                    if groupPos == 0 {
                        groupNo += 1;
                        if groupNo >= nSelectors {
                            retVal = ReturnCode::BZ_DATA_ERROR;
                            current_block = SAVE_STATE_AND_RETURN;
                            continue;
                        } else {
                            groupPos = 50;
                            gSel = s.selector[groupNo as usize] as i32;
                            gMinlen = s.minLens[gSel as usize];
                            gLimit = (*(s.limit).as_mut_ptr().offset(gSel as isize))
                                .as_mut_ptr()
                                .offset(0_isize) as *mut i32;
                            gPerm = (*(s.perm).as_mut_ptr().offset(gSel as isize))
                                .as_mut_ptr()
                                .offset(0_isize) as *mut i32;
                            gBase = (*(s.base).as_mut_ptr().offset(gSel as isize))
                                .as_mut_ptr()
                                .offset(0_isize) as *mut i32;
                        }
                    }
                    groupPos -= 1;
                    zn = gMinlen;
                    current_block = 13155828021133314705;
                }
            }
        }
    }

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

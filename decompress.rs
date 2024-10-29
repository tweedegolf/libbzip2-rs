use ::libc;
extern "C" {
    pub type _IO_wide_data;
    pub type _IO_codecvt;
    pub type _IO_marker;
    static mut stderr: *mut FILE;
    fn fprintf(_: *mut FILE, _: *const libc::c_char, _: ...) -> libc::c_int;
    fn BZ2_bz__AssertH__fail(errcode: libc::c_int);
    static mut BZ2_rNums: [Int32; 512];
    fn BZ2_indexIntoF(_: Int32, _: *mut Int32) -> Int32;
    fn BZ2_hbCreateDecodeTables(
        _: *mut Int32,
        _: *mut Int32,
        _: *mut Int32,
        _: *mut UChar,
        _: Int32,
        _: Int32,
        _: Int32,
    );
}
pub type size_t = libc::c_ulong;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _IO_FILE {
    pub _flags: libc::c_int,
    pub _IO_read_ptr: *mut libc::c_char,
    pub _IO_read_end: *mut libc::c_char,
    pub _IO_read_base: *mut libc::c_char,
    pub _IO_write_base: *mut libc::c_char,
    pub _IO_write_ptr: *mut libc::c_char,
    pub _IO_write_end: *mut libc::c_char,
    pub _IO_buf_base: *mut libc::c_char,
    pub _IO_buf_end: *mut libc::c_char,
    pub _IO_save_base: *mut libc::c_char,
    pub _IO_backup_base: *mut libc::c_char,
    pub _IO_save_end: *mut libc::c_char,
    pub _markers: *mut _IO_marker,
    pub _chain: *mut _IO_FILE,
    pub _fileno: libc::c_int,
    pub _flags2: libc::c_int,
    pub _old_offset: __off_t,
    pub _cur_column: libc::c_ushort,
    pub _vtable_offset: libc::c_schar,
    pub _shortbuf: [libc::c_char; 1],
    pub _lock: *mut libc::c_void,
    pub _offset: __off64_t,
    pub _codecvt: *mut _IO_codecvt,
    pub _wide_data: *mut _IO_wide_data,
    pub _freeres_list: *mut _IO_FILE,
    pub _freeres_buf: *mut libc::c_void,
    pub __pad5: size_t,
    pub _mode: libc::c_int,
    pub _unused2: [libc::c_char; 20],
}
pub type _IO_lock_t = ();
pub type FILE = _IO_FILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct bz_stream {
    pub next_in: *mut libc::c_char,
    pub avail_in: libc::c_uint,
    pub total_in_lo32: libc::c_uint,
    pub total_in_hi32: libc::c_uint,
    pub next_out: *mut libc::c_char,
    pub avail_out: libc::c_uint,
    pub total_out_lo32: libc::c_uint,
    pub total_out_hi32: libc::c_uint,
    pub state: *mut libc::c_void,
    pub bzalloc: Option<
        unsafe extern "C" fn(*mut libc::c_void, libc::c_int, libc::c_int) -> *mut libc::c_void,
    >,
    pub bzfree: Option<unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void) -> ()>,
    pub opaque: *mut libc::c_void,
}
pub type Bool = libc::c_uchar;
pub type UChar = libc::c_uchar;
pub type Int32 = libc::c_int;
pub type UInt32 = libc::c_uint;
pub type UInt16 = libc::c_ushort;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DState {
    pub strm: *mut bz_stream,
    pub state: Int32,
    pub state_out_ch: UChar,
    pub state_out_len: Int32,
    pub blockRandomised: Bool,
    pub rNToGo: Int32,
    pub rTPos: Int32,
    pub bsBuff: UInt32,
    pub bsLive: Int32,
    pub blockSize100k: Int32,
    pub smallDecompress: Bool,
    pub currBlockNo: Int32,
    pub verbosity: Int32,
    pub origPtr: Int32,
    pub tPos: UInt32,
    pub k0: Int32,
    pub unzftab: [Int32; 256],
    pub nblock_used: Int32,
    pub cftab: [Int32; 257],
    pub cftabCopy: [Int32; 257],
    pub tt: *mut UInt32,
    pub ll16: *mut UInt16,
    pub ll4: *mut UChar,
    pub storedBlockCRC: UInt32,
    pub storedCombinedCRC: UInt32,
    pub calculatedBlockCRC: UInt32,
    pub calculatedCombinedCRC: UInt32,
    pub nInUse: Int32,
    pub inUse: [Bool; 256],
    pub inUse16: [Bool; 16],
    pub seqToUnseq: [UChar; 256],
    pub mtfa: [UChar; 4096],
    pub mtfbase: [Int32; 16],
    pub selector: [UChar; 18002],
    pub selectorMtf: [UChar; 18002],
    pub len: [[UChar; 258]; 6],
    pub limit: [[Int32; 258]; 6],
    pub base: [[Int32; 258]; 6],
    pub perm: [[Int32; 258]; 6],
    pub minLens: [Int32; 6],
    pub save_i: Int32,
    pub save_j: Int32,
    pub save_t: Int32,
    pub save_alphaSize: Int32,
    pub save_nGroups: Int32,
    pub save_nSelectors: Int32,
    pub save_EOB: Int32,
    pub save_groupNo: Int32,
    pub save_groupPos: Int32,
    pub save_nextSym: Int32,
    pub save_nblockMAX: Int32,
    pub save_nblock: Int32,
    pub save_es: Int32,
    pub save_N: Int32,
    pub save_curr: Int32,
    pub save_zt: Int32,
    pub save_zn: Int32,
    pub save_zvec: Int32,
    pub save_zj: Int32,
    pub save_gSel: Int32,
    pub save_gMinlen: Int32,
    pub save_gLimit: *mut Int32,
    pub save_gBase: *mut Int32,
    pub save_gPerm: *mut Int32,
}
unsafe extern "C" fn makeMaps_d(mut s: *mut DState) {
    let mut i: Int32 = 0;
    (*s).nInUse = 0 as libc::c_int;
    i = 0 as libc::c_int;
    while i < 256 as libc::c_int {
        if (*s).inUse[i as usize] != 0 {
            (*s).seqToUnseq[(*s).nInUse as usize] = i as UChar;
            (*s).nInUse += 1;
            (*s).nInUse;
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn BZ2_decompress(mut s: *mut DState) -> Int32 {
    let mut current_block: u64;
    let mut uc: UChar = 0;
    let mut retVal: Int32 = 0;
    let mut minLen: Int32 = 0;
    let mut maxLen: Int32 = 0;
    let mut strm: *mut bz_stream = (*s).strm;
    let mut i: Int32 = 0;
    let mut j: Int32 = 0;
    let mut t: Int32 = 0;
    let mut alphaSize: Int32 = 0;
    let mut nGroups: Int32 = 0;
    let mut nSelectors: Int32 = 0;
    let mut EOB: Int32 = 0;
    let mut groupNo: Int32 = 0;
    let mut groupPos: Int32 = 0;
    let mut nextSym: Int32 = 0;
    let mut nblockMAX: Int32 = 0;
    let mut nblock: Int32 = 0;
    let mut es: Int32 = 0;
    let mut N: Int32 = 0;
    let mut curr: Int32 = 0;
    let mut zt: Int32 = 0;
    let mut zn: Int32 = 0;
    let mut zvec: Int32 = 0;
    let mut zj: Int32 = 0;
    let mut gSel: Int32 = 0;
    let mut gMinlen: Int32 = 0;
    let mut gLimit: *mut Int32 = std::ptr::null_mut::<Int32>();
    let mut gBase: *mut Int32 = std::ptr::null_mut::<Int32>();
    let mut gPerm: *mut Int32 = std::ptr::null_mut::<Int32>();
    if (*s).state == 10 as libc::c_int {
        (*s).save_i = 0 as libc::c_int;
        (*s).save_j = 0 as libc::c_int;
        (*s).save_t = 0 as libc::c_int;
        (*s).save_alphaSize = 0 as libc::c_int;
        (*s).save_nGroups = 0 as libc::c_int;
        (*s).save_nSelectors = 0 as libc::c_int;
        (*s).save_EOB = 0 as libc::c_int;
        (*s).save_groupNo = 0 as libc::c_int;
        (*s).save_groupPos = 0 as libc::c_int;
        (*s).save_nextSym = 0 as libc::c_int;
        (*s).save_nblockMAX = 0 as libc::c_int;
        (*s).save_nblock = 0 as libc::c_int;
        (*s).save_es = 0 as libc::c_int;
        (*s).save_N = 0 as libc::c_int;
        (*s).save_curr = 0 as libc::c_int;
        (*s).save_zt = 0 as libc::c_int;
        (*s).save_zn = 0 as libc::c_int;
        (*s).save_zvec = 0 as libc::c_int;
        (*s).save_zj = 0 as libc::c_int;
        (*s).save_gSel = 0 as libc::c_int;
        (*s).save_gMinlen = 0 as libc::c_int;
        (*s).save_gLimit = std::ptr::null_mut::<Int32>();
        (*s).save_gBase = std::ptr::null_mut::<Int32>();
        (*s).save_gPerm = std::ptr::null_mut::<Int32>();
    }
    i = (*s).save_i;
    j = (*s).save_j;
    t = (*s).save_t;
    alphaSize = (*s).save_alphaSize;
    nGroups = (*s).save_nGroups;
    nSelectors = (*s).save_nSelectors;
    EOB = (*s).save_EOB;
    groupNo = (*s).save_groupNo;
    groupPos = (*s).save_groupPos;
    nextSym = (*s).save_nextSym;
    nblockMAX = (*s).save_nblockMAX;
    nblock = (*s).save_nblock;
    es = (*s).save_es;
    N = (*s).save_N;
    curr = (*s).save_curr;
    zt = (*s).save_zt;
    zn = (*s).save_zn;
    zvec = (*s).save_zvec;
    zj = (*s).save_zj;
    gSel = (*s).save_gSel;
    gMinlen = (*s).save_gMinlen;
    gLimit = (*s).save_gLimit;
    gBase = (*s).save_gBase;
    gPerm = (*s).save_gPerm;
    retVal = 0 as libc::c_int;
    match (*s).state {
        10 => {
            (*s).state = 10 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 5235537862154438448;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v: UInt32 = 0;
                    v = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v as UChar;
                    current_block = 5235537862154438448;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x42 as libc::c_int {
                        retVal = -(5 as libc::c_int);
                        current_block = 3350591128142761507;
                    } else {
                        current_block = 15360092558900836893;
                    }
                }
            }
        }
        11 => {
            current_block = 15360092558900836893;
        }
        12 => {
            current_block = 15953825877604003206;
        }
        13 => {
            current_block = 1137006006685247392;
        }
        14 => {
            current_block = 16838365919992687769;
        }
        15 => {
            current_block = 5889181040567946013;
        }
        16 => {
            current_block = 887841530443712878;
        }
        17 => {
            current_block = 17767742176799939193;
        }
        18 => {
            current_block = 16325921850189496668;
        }
        19 => {
            current_block = 3202472413399101603;
        }
        20 => {
            current_block = 5821827988509819404;
        }
        21 => {
            current_block = 5023088878038355716;
        }
        22 => {
            current_block = 8515868523999336537;
        }
        23 => {
            current_block = 18234918597811156654;
        }
        24 => {
            current_block = 12310871532727186508;
        }
        25 => {
            current_block = 3338455798814466984;
        }
        26 => {
            current_block = 10262367570716242252;
        }
        27 => {
            current_block = 17024493544560437554;
        }
        28 => {
            current_block = 1154520408629132897;
        }
        29 => {
            current_block = 15451013008180677144;
        }
        30 => {
            current_block = 9434444550647791986;
        }
        31 => {
            current_block = 14590825336193814119;
        }
        32 => {
            current_block = 15957329598978927534;
        }
        33 => {
            current_block = 11569294379105328467;
        }
        34 => {
            current_block = 17216244326479313607;
        }
        35 => {
            current_block = 7191958063352112897;
        }
        36 => {
            current_block = 13155828021133314705;
        }
        37 => {
            current_block = 1010107409739284736;
        }
        38 => {
            current_block = 9335356017384149594;
        }
        39 => {
            current_block = 12127014564286193091;
        }
        40 => {
            current_block = 9050093969003559074;
        }
        41 => {
            current_block = 10797958389266113496;
        }
        42 => {
            current_block = 14366592556287126287;
        }
        43 => {
            current_block = 7651522734817633728;
        }
        44 => {
            current_block = 15818849443713787272;
        }
        45 => {
            current_block = 15153555825877660840;
        }
        46 => {
            current_block = 1857046018890652364;
        }
        47 => {
            current_block = 10292318171587122742;
        }
        48 => {
            current_block = 14748314904637597825;
        }
        49 => {
            current_block = 4092966239614665407;
        }
        50 => {
            current_block = 18389040574536762539;
        }
        _ => {
            if 0 as libc::c_int as Bool == 0 {
                BZ2_bz__AssertH__fail(4001 as libc::c_int);
            }
            if 0 as libc::c_int as Bool == 0 {
                BZ2_bz__AssertH__fail(4002 as libc::c_int);
            }
            current_block = 3350591128142761507;
        }
    }
    if current_block == 15360092558900836893 {
        (*s).state = 11 as libc::c_int;
        loop {
            if 1 as libc::c_int as Bool == 0 {
                current_block = 2168227384378665163;
                break;
            }
            if (*s).bsLive >= 8 as libc::c_int {
                let mut v_0: UInt32 = 0;
                v_0 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                    & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int) as libc::c_uint;
                (*s).bsLive -= 8 as libc::c_int;
                uc = v_0 as UChar;
                current_block = 2168227384378665163;
                break;
            } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                retVal = 0 as libc::c_int;
                current_block = 3350591128142761507;
                break;
            } else {
                (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                    | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                (*s).bsLive += 8 as libc::c_int;
                (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                (*(*s).strm).next_in;
                (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                (*(*s).strm).avail_in;
                (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                (*(*s).strm).total_in_lo32;
                if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                    (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                    (*(*s).strm).total_in_hi32;
                }
            }
        }
        match current_block {
            3350591128142761507 => {}
            _ => {
                if uc as libc::c_int != 0x5a as libc::c_int {
                    retVal = -(5 as libc::c_int);
                    current_block = 3350591128142761507;
                } else {
                    current_block = 15953825877604003206;
                }
            }
        }
    }
    if current_block == 15953825877604003206 {
        (*s).state = 12 as libc::c_int;
        loop {
            if 1 as libc::c_int as Bool == 0 {
                current_block = 178030534879405462;
                break;
            }
            if (*s).bsLive >= 8 as libc::c_int {
                let mut v_1: UInt32 = 0;
                v_1 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                    & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int) as libc::c_uint;
                (*s).bsLive -= 8 as libc::c_int;
                uc = v_1 as UChar;
                current_block = 178030534879405462;
                break;
            } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                retVal = 0 as libc::c_int;
                current_block = 3350591128142761507;
                break;
            } else {
                (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                    | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                (*s).bsLive += 8 as libc::c_int;
                (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                (*(*s).strm).next_in;
                (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                (*(*s).strm).avail_in;
                (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                (*(*s).strm).total_in_lo32;
                if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                    (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                    (*(*s).strm).total_in_hi32;
                }
            }
        }
        match current_block {
            3350591128142761507 => {}
            _ => {
                if uc as libc::c_int != 0x68 as libc::c_int {
                    retVal = -(5 as libc::c_int);
                    current_block = 3350591128142761507;
                } else {
                    current_block = 1137006006685247392;
                }
            }
        }
    }
    if current_block == 1137006006685247392 {
        (*s).state = 13 as libc::c_int;
        loop {
            if 1 as libc::c_int as Bool == 0 {
                current_block = 7639320476250304355;
                break;
            }
            if (*s).bsLive >= 8 as libc::c_int {
                let mut v_2: UInt32 = 0;
                v_2 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                    & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int) as libc::c_uint;
                (*s).bsLive -= 8 as libc::c_int;
                (*s).blockSize100k = v_2 as Int32;
                current_block = 7639320476250304355;
                break;
            } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                retVal = 0 as libc::c_int;
                current_block = 3350591128142761507;
                break;
            } else {
                (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                    | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                (*s).bsLive += 8 as libc::c_int;
                (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                (*(*s).strm).next_in;
                (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                (*(*s).strm).avail_in;
                (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                (*(*s).strm).total_in_lo32;
                if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                    (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                    (*(*s).strm).total_in_hi32;
                }
            }
        }
        match current_block {
            3350591128142761507 => {}
            _ => {
                if (*s).blockSize100k < 0x30 as libc::c_int + 1 as libc::c_int
                    || (*s).blockSize100k > 0x30 as libc::c_int + 9 as libc::c_int
                {
                    retVal = -(5 as libc::c_int);
                    current_block = 3350591128142761507;
                } else {
                    (*s).blockSize100k -= 0x30 as libc::c_int;
                    if (*s).smallDecompress != 0 {
                        (*s).ll16 = ((*strm).bzalloc).expect("non-null function pointer")(
                            (*strm).opaque,
                            (((*s).blockSize100k * 100000 as libc::c_int) as libc::c_ulong)
                                .wrapping_mul(::core::mem::size_of::<UInt16>() as libc::c_ulong)
                                as libc::c_int,
                            1 as libc::c_int,
                        ) as *mut UInt16;
                        (*s).ll4 = ((*strm).bzalloc).expect("non-null function pointer")(
                            (*strm).opaque,
                            (((1 as libc::c_int + (*s).blockSize100k * 100000 as libc::c_int)
                                >> 1 as libc::c_int) as libc::c_ulong)
                                .wrapping_mul(::core::mem::size_of::<UChar>() as libc::c_ulong)
                                as libc::c_int,
                            1 as libc::c_int,
                        ) as *mut UChar;
                        if ((*s).ll16).is_null() || ((*s).ll4).is_null() {
                            retVal = -(3 as libc::c_int);
                            current_block = 3350591128142761507;
                        } else {
                            current_block = 16838365919992687769;
                        }
                    } else {
                        (*s).tt = ((*strm).bzalloc).expect("non-null function pointer")(
                            (*strm).opaque,
                            (((*s).blockSize100k * 100000 as libc::c_int) as libc::c_ulong)
                                .wrapping_mul(::core::mem::size_of::<Int32>() as libc::c_ulong)
                                as libc::c_int,
                            1 as libc::c_int,
                        ) as *mut UInt32;
                        if ((*s).tt).is_null() {
                            retVal = -(3 as libc::c_int);
                            current_block = 3350591128142761507;
                        } else {
                            current_block = 16838365919992687769;
                        }
                    }
                }
            }
        }
    }
    if current_block == 16838365919992687769 {
        (*s).state = 14 as libc::c_int;
        loop {
            if 1 as libc::c_int as Bool == 0 {
                current_block = 16937825661756021828;
                break;
            }
            if (*s).bsLive >= 8 as libc::c_int {
                let mut v_3: UInt32 = 0;
                v_3 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                    & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int) as libc::c_uint;
                (*s).bsLive -= 8 as libc::c_int;
                uc = v_3 as UChar;
                current_block = 16937825661756021828;
                break;
            } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                retVal = 0 as libc::c_int;
                current_block = 3350591128142761507;
                break;
            } else {
                (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                    | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                (*s).bsLive += 8 as libc::c_int;
                (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                (*(*s).strm).next_in;
                (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                (*(*s).strm).avail_in;
                (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                (*(*s).strm).total_in_lo32;
                if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                    (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                    (*(*s).strm).total_in_hi32;
                }
            }
        }
        match current_block {
            3350591128142761507 => {}
            _ => {
                if uc as libc::c_int == 0x17 as libc::c_int {
                    current_block = 14366592556287126287;
                } else if uc as libc::c_int != 0x31 as libc::c_int {
                    retVal = -(4 as libc::c_int);
                    current_block = 3350591128142761507;
                } else {
                    current_block = 5889181040567946013;
                }
            }
        }
    }
    match current_block {
        14366592556287126287 => {
            (*s).state = 42 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 13733404100380861831;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_32: UInt32 = 0;
                    v_32 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_32 as UChar;
                    current_block = 13733404100380861831;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x72 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                    } else {
                        current_block = 7651522734817633728;
                    }
                }
            }
        }
        5889181040567946013 => {
            (*s).state = 15 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 1228639923084383292;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_4: UInt32 = 0;
                    v_4 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_4 as UChar;
                    current_block = 1228639923084383292;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x41 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
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
            (*s).state = 43 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 12721425419429475574;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_33: UInt32 = 0;
                    v_33 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_33 as UChar;
                    current_block = 12721425419429475574;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x45 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                    } else {
                        current_block = 15818849443713787272;
                    }
                }
            }
        }
        887841530443712878 => {
            (*s).state = 16 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 9235179519944561532;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_5: UInt32 = 0;
                    v_5 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_5 as UChar;
                    current_block = 9235179519944561532;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x59 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
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
            (*s).state = 44 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 13813414375753095368;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_34: UInt32 = 0;
                    v_34 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_34 as UChar;
                    current_block = 13813414375753095368;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x38 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                    } else {
                        current_block = 15153555825877660840;
                    }
                }
            }
        }
        17767742176799939193 => {
            (*s).state = 17 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 12467039471581323981;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_6: UInt32 = 0;
                    v_6 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_6 as UChar;
                    current_block = 12467039471581323981;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x26 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
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
            (*s).state = 45 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 1472103348880861285;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_35: UInt32 = 0;
                    v_35 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_35 as UChar;
                    current_block = 1472103348880861285;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x50 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                    } else {
                        current_block = 1857046018890652364;
                    }
                }
            }
        }
        16325921850189496668 => {
            (*s).state = 18 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 13164310931121142693;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_7: UInt32 = 0;
                    v_7 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_7 as UChar;
                    current_block = 13164310931121142693;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x53 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
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
            (*s).state = 46 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 8232347840743503282;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_36: UInt32 = 0;
                    v_36 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_36 as UChar;
                    current_block = 8232347840743503282;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x90 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                    } else {
                        (*s).storedCombinedCRC = 0 as libc::c_int as UInt32;
                        current_block = 10292318171587122742;
                    }
                }
            }
        }
        3202472413399101603 => {
            (*s).state = 19 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 14723615986260991866;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_8: UInt32 = 0;
                    v_8 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_8 as UChar;
                    current_block = 14723615986260991866;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    if uc as libc::c_int != 0x59 as libc::c_int {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                    } else {
                        (*s).currBlockNo += 1;
                        (*s).currBlockNo;
                        if (*s).verbosity >= 2 as libc::c_int {
                            fprintf(
                                stderr,
                                b"\n    [%d: huff+mtf \0" as *const u8 as *const libc::c_char,
                                (*s).currBlockNo,
                            );
                        }
                        (*s).storedBlockCRC = 0 as libc::c_int as UInt32;
                        current_block = 5821827988509819404;
                    }
                }
            }
        }
        _ => {}
    }
    match current_block {
        10292318171587122742 => {
            (*s).state = 47 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 5465979950226085365;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_37: UInt32 = 0;
                    v_37 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_37 as UChar;
                    current_block = 5465979950226085365;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    (*s).storedCombinedCRC =
                        (*s).storedCombinedCRC << 8 as libc::c_int | uc as UInt32;
                    current_block = 14748314904637597825;
                }
            }
        }
        5821827988509819404 => {
            (*s).state = 20 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 15627786036016112248;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_9: UInt32 = 0;
                    v_9 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_9 as UChar;
                    current_block = 15627786036016112248;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    (*s).storedBlockCRC = (*s).storedBlockCRC << 8 as libc::c_int | uc as UInt32;
                    current_block = 5023088878038355716;
                }
            }
        }
        _ => {}
    }
    match current_block {
        14748314904637597825 => {
            (*s).state = 48 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 3854366583354019639;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_38: UInt32 = 0;
                    v_38 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_38 as UChar;
                    current_block = 3854366583354019639;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    (*s).storedCombinedCRC =
                        (*s).storedCombinedCRC << 8 as libc::c_int | uc as UInt32;
                    current_block = 4092966239614665407;
                }
            }
        }
        5023088878038355716 => {
            (*s).state = 21 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 13493279574219925475;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_10: UInt32 = 0;
                    v_10 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_10 as UChar;
                    current_block = 13493279574219925475;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    (*s).storedBlockCRC = (*s).storedBlockCRC << 8 as libc::c_int | uc as UInt32;
                    current_block = 8515868523999336537;
                }
            }
        }
        _ => {}
    }
    match current_block {
        4092966239614665407 => {
            (*s).state = 49 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 12082794684616777938;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_39: UInt32 = 0;
                    v_39 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_39 as UChar;
                    current_block = 12082794684616777938;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    (*s).storedCombinedCRC =
                        (*s).storedCombinedCRC << 8 as libc::c_int | uc as UInt32;
                    current_block = 18389040574536762539;
                }
            }
        }
        8515868523999336537 => {
            (*s).state = 22 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 4839309778395429725;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_11: UInt32 = 0;
                    v_11 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_11 as UChar;
                    current_block = 4839309778395429725;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    (*s).storedBlockCRC = (*s).storedBlockCRC << 8 as libc::c_int | uc as UInt32;
                    current_block = 18234918597811156654;
                }
            }
        }
        _ => {}
    }
    match current_block {
        18234918597811156654 => {
            (*s).state = 23 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 17937968408868551711;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_12: UInt32 = 0;
                    v_12 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_12 as UChar;
                    current_block = 17937968408868551711;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    (*s).storedBlockCRC = (*s).storedBlockCRC << 8 as libc::c_int | uc as UInt32;
                    current_block = 12310871532727186508;
                }
            }
        }
        18389040574536762539 => {
            (*s).state = 50 as libc::c_int;
            loop {
                if 1 as libc::c_int as Bool == 0 {
                    current_block = 6276941480907995842;
                    break;
                }
                if (*s).bsLive >= 8 as libc::c_int {
                    let mut v_40: UInt32 = 0;
                    v_40 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                        & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int)
                            as libc::c_uint;
                    (*s).bsLive -= 8 as libc::c_int;
                    uc = v_40 as UChar;
                    current_block = 6276941480907995842;
                    break;
                } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                    retVal = 0 as libc::c_int;
                    current_block = 3350591128142761507;
                    break;
                } else {
                    (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                        | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                    (*s).bsLive += 8 as libc::c_int;
                    (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                    (*(*s).strm).next_in;
                    (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                    (*(*s).strm).avail_in;
                    (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                    (*(*s).strm).total_in_lo32;
                    if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                        (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                        (*(*s).strm).total_in_hi32;
                    }
                }
            }
            match current_block {
                3350591128142761507 => {}
                _ => {
                    (*s).storedCombinedCRC =
                        (*s).storedCombinedCRC << 8 as libc::c_int | uc as UInt32;
                    (*s).state = 1 as libc::c_int;
                    retVal = 4 as libc::c_int;
                    current_block = 3350591128142761507;
                }
            }
        }
        _ => {}
    }
    if current_block == 12310871532727186508 {
        (*s).state = 24 as libc::c_int;
        loop {
            if 1 as libc::c_int as Bool == 0 {
                current_block = 7926734633677835471;
                break;
            }
            if (*s).bsLive >= 1 as libc::c_int {
                let mut v_13: UInt32 = 0;
                v_13 = (*s).bsBuff >> ((*s).bsLive - 1 as libc::c_int)
                    & (((1 as libc::c_int) << 1 as libc::c_int) - 1 as libc::c_int) as libc::c_uint;
                (*s).bsLive -= 1 as libc::c_int;
                (*s).blockRandomised = v_13 as Bool;
                current_block = 7926734633677835471;
                break;
            } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                retVal = 0 as libc::c_int;
                current_block = 3350591128142761507;
                break;
            } else {
                (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                    | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                (*s).bsLive += 8 as libc::c_int;
                (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                (*(*s).strm).next_in;
                (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                (*(*s).strm).avail_in;
                (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                (*(*s).strm).total_in_lo32;
                if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                    (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                    (*(*s).strm).total_in_hi32;
                }
            }
        }
        match current_block {
            3350591128142761507 => {}
            _ => {
                (*s).origPtr = 0 as libc::c_int;
                current_block = 3338455798814466984;
            }
        }
    }
    if current_block == 3338455798814466984 {
        (*s).state = 25 as libc::c_int;
        loop {
            if 1 as libc::c_int as Bool == 0 {
                current_block = 5948065351908552372;
                break;
            }
            if (*s).bsLive >= 8 as libc::c_int {
                let mut v_14: UInt32 = 0;
                v_14 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                    & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int) as libc::c_uint;
                (*s).bsLive -= 8 as libc::c_int;
                uc = v_14 as UChar;
                current_block = 5948065351908552372;
                break;
            } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                retVal = 0 as libc::c_int;
                current_block = 3350591128142761507;
                break;
            } else {
                (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                    | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                (*s).bsLive += 8 as libc::c_int;
                (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                (*(*s).strm).next_in;
                (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                (*(*s).strm).avail_in;
                (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                (*(*s).strm).total_in_lo32;
                if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                    (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                    (*(*s).strm).total_in_hi32;
                }
            }
        }
        match current_block {
            3350591128142761507 => {}
            _ => {
                (*s).origPtr = (*s).origPtr << 8 as libc::c_int | uc as Int32;
                current_block = 10262367570716242252;
            }
        }
    }
    if current_block == 10262367570716242252 {
        (*s).state = 26 as libc::c_int;
        loop {
            if 1 as libc::c_int as Bool == 0 {
                current_block = 8940662058537996670;
                break;
            }
            if (*s).bsLive >= 8 as libc::c_int {
                let mut v_15: UInt32 = 0;
                v_15 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                    & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int) as libc::c_uint;
                (*s).bsLive -= 8 as libc::c_int;
                uc = v_15 as UChar;
                current_block = 8940662058537996670;
                break;
            } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                retVal = 0 as libc::c_int;
                current_block = 3350591128142761507;
                break;
            } else {
                (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                    | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                (*s).bsLive += 8 as libc::c_int;
                (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                (*(*s).strm).next_in;
                (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                (*(*s).strm).avail_in;
                (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                (*(*s).strm).total_in_lo32;
                if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                    (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                    (*(*s).strm).total_in_hi32;
                }
            }
        }
        match current_block {
            3350591128142761507 => {}
            _ => {
                (*s).origPtr = (*s).origPtr << 8 as libc::c_int | uc as Int32;
                current_block = 17024493544560437554;
            }
        }
    }
    if current_block == 17024493544560437554 {
        (*s).state = 27 as libc::c_int;
        loop {
            if 1 as libc::c_int as Bool == 0 {
                current_block = 13366002463409402866;
                break;
            }
            if (*s).bsLive >= 8 as libc::c_int {
                let mut v_16: UInt32 = 0;
                v_16 = (*s).bsBuff >> ((*s).bsLive - 8 as libc::c_int)
                    & (((1 as libc::c_int) << 8 as libc::c_int) - 1 as libc::c_int) as libc::c_uint;
                (*s).bsLive -= 8 as libc::c_int;
                uc = v_16 as UChar;
                current_block = 13366002463409402866;
                break;
            } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                retVal = 0 as libc::c_int;
                current_block = 3350591128142761507;
                break;
            } else {
                (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                    | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                (*s).bsLive += 8 as libc::c_int;
                (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                (*(*s).strm).next_in;
                (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                (*(*s).strm).avail_in;
                (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                (*(*s).strm).total_in_lo32;
                if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                    (*(*s).strm).total_in_hi32 = ((*(*s).strm).total_in_hi32).wrapping_add(1);
                    (*(*s).strm).total_in_hi32;
                }
            }
        }
        match current_block {
            3350591128142761507 => {}
            _ => {
                (*s).origPtr = (*s).origPtr << 8 as libc::c_int | uc as Int32;
                if (*s).origPtr < 0 as libc::c_int {
                    retVal = -(4 as libc::c_int);
                    current_block = 3350591128142761507;
                } else if (*s).origPtr
                    > 10 as libc::c_int + 100000 as libc::c_int * (*s).blockSize100k
                {
                    retVal = -(4 as libc::c_int);
                    current_block = 3350591128142761507;
                } else {
                    i = 0 as libc::c_int;
                    current_block = 454873545234741267;
                }
            }
        }
    }
    'c_10064: loop {
        match current_block {
            3350591128142761507 => {
                (*s).save_i = i;
                break;
            }
            9050093969003559074 => {
                (*s).state = 40 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= zn {
                        let mut v_30: UInt32 = 0;
                        v_30 = (*s).bsBuff >> ((*s).bsLive - zn)
                            & (((1 as libc::c_int) << zn) - 1 as libc::c_int) as libc::c_uint;
                        (*s).bsLive -= zn;
                        zvec = v_30 as Int32;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                current_block = 16348713635569416413;
            }
            12127014564286193091 => {
                (*s).state = 39 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 1 as libc::c_int {
                        let mut v_29: UInt32 = 0;
                        v_29 = (*s).bsBuff >> ((*s).bsLive - 1 as libc::c_int)
                            & (((1 as libc::c_int) << 1 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 1 as libc::c_int;
                        zj = v_29 as Int32;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                zvec = zvec << 1 as libc::c_int | zj;
                current_block = 7923635230025172457;
            }
            9335356017384149594 => {
                (*s).state = 38 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= zn {
                        let mut v_28: UInt32 = 0;
                        v_28 = (*s).bsBuff >> ((*s).bsLive - zn)
                            & (((1 as libc::c_int) << zn) - 1 as libc::c_int) as libc::c_uint;
                        (*s).bsLive -= zn;
                        zvec = v_28 as Int32;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                current_block = 7923635230025172457;
            }
            1010107409739284736 => {
                (*s).state = 37 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 1 as libc::c_int {
                        let mut v_27: UInt32 = 0;
                        v_27 = (*s).bsBuff >> ((*s).bsLive - 1 as libc::c_int)
                            & (((1 as libc::c_int) << 1 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 1 as libc::c_int;
                        zj = v_27 as Int32;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                zvec = zvec << 1 as libc::c_int | zj;
                current_block = 9186389159759284570;
            }
            13155828021133314705 => {
                (*s).state = 36 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= zn {
                        let mut v_26: UInt32 = 0;
                        v_26 = (*s).bsBuff >> ((*s).bsLive - zn)
                            & (((1 as libc::c_int) << zn) - 1 as libc::c_int) as libc::c_uint;
                        (*s).bsLive -= zn;
                        zvec = v_26 as Int32;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                current_block = 9186389159759284570;
            }
            7191958063352112897 => {
                (*s).state = 35 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 1 as libc::c_int {
                        let mut v_25: UInt32 = 0;
                        v_25 = (*s).bsBuff >> ((*s).bsLive - 1 as libc::c_int)
                            & (((1 as libc::c_int) << 1 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 1 as libc::c_int;
                        uc = v_25 as UChar;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                if uc as libc::c_int == 0 as libc::c_int {
                    curr += 1;
                } else {
                    curr -= 1;
                }
                current_block = 5533056661327372531;
            }
            17216244326479313607 => {
                (*s).state = 34 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 1 as libc::c_int {
                        let mut v_24: UInt32 = 0;
                        v_24 = (*s).bsBuff >> ((*s).bsLive - 1 as libc::c_int)
                            & (((1 as libc::c_int) << 1 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 1 as libc::c_int;
                        uc = v_24 as UChar;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                if uc as libc::c_int != 0 as libc::c_int {
                    current_block = 7191958063352112897;
                    continue;
                }
                current_block = 7746242308555130918;
            }
            11569294379105328467 => {
                (*s).state = 33 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 5 as libc::c_int {
                        let mut v_23: UInt32 = 0;
                        v_23 = (*s).bsBuff >> ((*s).bsLive - 5 as libc::c_int)
                            & (((1 as libc::c_int) << 5 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 5 as libc::c_int;
                        curr = v_23 as Int32;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                i = 0 as libc::c_int;
                current_block = 16642413284942005565;
            }
            15957329598978927534 => {
                (*s).state = 32 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 1 as libc::c_int {
                        let mut v_21: UInt32 = 0;
                        v_21 = (*s).bsBuff >> ((*s).bsLive - 1 as libc::c_int)
                            & (((1 as libc::c_int) << 1 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 1 as libc::c_int;
                        uc = v_21 as UChar;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                if uc as libc::c_int == 0 as libc::c_int {
                    current_block = 10081471997089450706;
                } else {
                    j += 1;
                    if j >= nGroups {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                        continue;
                    } else {
                        current_block = 16531797892856733396;
                    }
                }
            }
            14590825336193814119 => {
                (*s).state = 31 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 15 as libc::c_int {
                        let mut v_20: UInt32 = 0;
                        v_20 = (*s).bsBuff >> ((*s).bsLive - 15 as libc::c_int)
                            & (((1 as libc::c_int) << 15 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 15 as libc::c_int;
                        nSelectors = v_20 as Int32;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                if nSelectors < 1 as libc::c_int {
                    retVal = -(4 as libc::c_int);
                    current_block = 3350591128142761507;
                    continue;
                } else {
                    i = 0 as libc::c_int;
                }
                current_block = 3503188808869013853;
            }
            9434444550647791986 => {
                (*s).state = 30 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 3 as libc::c_int {
                        let mut v_19: UInt32 = 0;
                        v_19 = (*s).bsBuff >> ((*s).bsLive - 3 as libc::c_int)
                            & (((1 as libc::c_int) << 3 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 3 as libc::c_int;
                        nGroups = v_19 as Int32;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                if !(nGroups < 2 as libc::c_int || nGroups > 6 as libc::c_int) {
                    current_block = 14590825336193814119;
                    continue;
                }
                retVal = -(4 as libc::c_int);
                current_block = 3350591128142761507;
                continue;
            }
            15451013008180677144 => {
                (*s).state = 29 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 1 as libc::c_int {
                        let mut v_18: UInt32 = 0;
                        v_18 = (*s).bsBuff >> ((*s).bsLive - 1 as libc::c_int)
                            & (((1 as libc::c_int) << 1 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 1 as libc::c_int;
                        uc = v_18 as UChar;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                if uc as libc::c_int == 1 as libc::c_int {
                    (*s).inUse[(i * 16 as libc::c_int + j) as usize] = 1 as libc::c_int as Bool;
                }
                j += 1;
                current_block = 16953886395775657100;
            }
            454873545234741267 => {
                if i < 16 as libc::c_int {
                    current_block = 1154520408629132897;
                    continue;
                }
                i = 0 as libc::c_int;
                while i < 256 as libc::c_int {
                    (*s).inUse[i as usize] = 0 as libc::c_int as Bool;
                    i += 1;
                }
                i = 0 as libc::c_int;
                current_block = 15415362524153386998;
            }
            1154520408629132897 => {
                (*s).state = 28 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 1 as libc::c_int {
                        let mut v_17: UInt32 = 0;
                        v_17 = (*s).bsBuff >> ((*s).bsLive - 1 as libc::c_int)
                            & (((1 as libc::c_int) << 1 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 1 as libc::c_int;
                        uc = v_17 as UChar;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                if uc as libc::c_int == 1 as libc::c_int {
                    (*s).inUse16[i as usize] = 1 as libc::c_int as Bool;
                } else {
                    (*s).inUse16[i as usize] = 0 as libc::c_int as Bool;
                }
                i += 1;
                current_block = 454873545234741267;
                continue;
            }
            _ => {
                (*s).state = 41 as libc::c_int;
                while 1 as libc::c_int as Bool != 0 {
                    if (*s).bsLive >= 1 as libc::c_int {
                        let mut v_31: UInt32 = 0;
                        v_31 = (*s).bsBuff >> ((*s).bsLive - 1 as libc::c_int)
                            & (((1 as libc::c_int) << 1 as libc::c_int) - 1 as libc::c_int)
                                as libc::c_uint;
                        (*s).bsLive -= 1 as libc::c_int;
                        zj = v_31 as Int32;
                        break;
                    } else if (*(*s).strm).avail_in == 0 as libc::c_int as libc::c_uint {
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue 'c_10064;
                    } else {
                        (*s).bsBuff = (*s).bsBuff << 8 as libc::c_int
                            | *((*(*s).strm).next_in as *mut UChar) as UInt32;
                        (*s).bsLive += 8 as libc::c_int;
                        (*(*s).strm).next_in = ((*(*s).strm).next_in).offset(1);
                        (*(*s).strm).next_in;
                        (*(*s).strm).avail_in = ((*(*s).strm).avail_in).wrapping_sub(1);
                        (*(*s).strm).avail_in;
                        (*(*s).strm).total_in_lo32 = ((*(*s).strm).total_in_lo32).wrapping_add(1);
                        (*(*s).strm).total_in_lo32;
                        if (*(*s).strm).total_in_lo32 == 0 as libc::c_int as libc::c_uint {
                            (*(*s).strm).total_in_hi32 =
                                ((*(*s).strm).total_in_hi32).wrapping_add(1);
                            (*(*s).strm).total_in_hi32;
                        }
                    }
                }
                zvec = zvec << 1 as libc::c_int | zj;
                current_block = 16348713635569416413;
            }
        }
        match current_block {
            16348713635569416413 => {
                if zn > 20 as libc::c_int {
                    retVal = -(4 as libc::c_int);
                    current_block = 3350591128142761507;
                    continue;
                } else if zvec <= *gLimit.offset(zn as isize) {
                    if zvec - *gBase.offset(zn as isize) < 0 as libc::c_int
                        || zvec - *gBase.offset(zn as isize) >= 258 as libc::c_int
                    {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
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
                if zn > 20 as libc::c_int {
                    retVal = -(4 as libc::c_int);
                    current_block = 3350591128142761507;
                    continue;
                } else if zvec <= *gLimit.offset(zn as isize) {
                    if zvec - *gBase.offset(zn as isize) < 0 as libc::c_int
                        || zvec - *gBase.offset(zn as isize) >= 258 as libc::c_int
                    {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                        continue;
                    } else {
                        nextSym = *gPerm.offset((zvec - *gBase.offset(zn as isize)) as isize);
                        if nextSym == 0 as libc::c_int || nextSym == 1 as libc::c_int {
                            current_block = 5649595406143318745;
                        } else {
                            es += 1;
                            uc = (*s).seqToUnseq[(*s).mtfa
                                [(*s).mtfbase[0 as libc::c_int as usize] as usize]
                                as usize];
                            (*s).unzftab[uc as usize] += es;
                            if (*s).smallDecompress != 0 {
                                while es > 0 as libc::c_int {
                                    if nblock >= nblockMAX {
                                        retVal = -(4 as libc::c_int);
                                        current_block = 3350591128142761507;
                                        continue 'c_10064;
                                    } else {
                                        *((*s).ll16).offset(nblock as isize) = uc as UInt16;
                                        nblock += 1;
                                        es -= 1;
                                    }
                                }
                            } else {
                                while es > 0 as libc::c_int {
                                    if nblock >= nblockMAX {
                                        retVal = -(4 as libc::c_int);
                                        current_block = 3350591128142761507;
                                        continue 'c_10064;
                                    } else {
                                        *((*s).tt).offset(nblock as isize) = uc as UInt32;
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
                if zn > 20 as libc::c_int {
                    retVal = -(4 as libc::c_int);
                    current_block = 3350591128142761507;
                    continue;
                } else if zvec <= *gLimit.offset(zn as isize) {
                    if zvec - *gBase.offset(zn as isize) < 0 as libc::c_int
                        || zvec - *gBase.offset(zn as isize) >= 258 as libc::c_int
                    {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
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
            if 1 as libc::c_int as Bool != 0 {
                if nextSym == EOB {
                    current_block = 4069074773319880902;
                } else {
                    if nextSym == 0 as libc::c_int || nextSym == 1 as libc::c_int {
                        es = -(1 as libc::c_int);
                        N = 1 as libc::c_int;
                    } else if nblock >= nblockMAX {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                        continue;
                    } else {
                        let mut ii_0: Int32 = 0;
                        let mut jj_0: Int32 = 0;
                        let mut kk_0: Int32 = 0;
                        let mut pp: Int32 = 0;
                        let mut lno: Int32 = 0;
                        let mut off: Int32 = 0;
                        let mut nn: UInt32 = 0;
                        nn = (nextSym - 1 as libc::c_int) as UInt32;
                        if nn < 16 as libc::c_int as libc::c_uint {
                            pp = (*s).mtfbase[0 as libc::c_int as usize];
                            uc = (*s).mtfa[(pp as libc::c_uint).wrapping_add(nn) as usize];
                            while nn > 3 as libc::c_int as libc::c_uint {
                                let mut z: Int32 = (pp as libc::c_uint).wrapping_add(nn) as Int32;
                                (*s).mtfa[z as usize] = (*s).mtfa[(z - 1 as libc::c_int) as usize];
                                (*s).mtfa[(z - 1 as libc::c_int) as usize] =
                                    (*s).mtfa[(z - 2 as libc::c_int) as usize];
                                (*s).mtfa[(z - 2 as libc::c_int) as usize] =
                                    (*s).mtfa[(z - 3 as libc::c_int) as usize];
                                (*s).mtfa[(z - 3 as libc::c_int) as usize] =
                                    (*s).mtfa[(z - 4 as libc::c_int) as usize];
                                nn = (nn as libc::c_uint)
                                    .wrapping_sub(4 as libc::c_int as libc::c_uint)
                                    as UInt32 as UInt32;
                            }
                            while nn > 0 as libc::c_int as libc::c_uint {
                                (*s).mtfa[(pp as libc::c_uint).wrapping_add(nn) as usize] = (*s)
                                    .mtfa[(pp as libc::c_uint)
                                    .wrapping_add(nn)
                                    .wrapping_sub(1 as libc::c_int as libc::c_uint)
                                    as usize];
                                nn = nn.wrapping_sub(1);
                            }
                            (*s).mtfa[pp as usize] = uc;
                        } else {
                            lno = nn.wrapping_div(16 as libc::c_int as libc::c_uint) as Int32;
                            off = nn.wrapping_rem(16 as libc::c_int as libc::c_uint) as Int32;
                            pp = (*s).mtfbase[lno as usize] + off;
                            uc = (*s).mtfa[pp as usize];
                            while pp > (*s).mtfbase[lno as usize] {
                                (*s).mtfa[pp as usize] =
                                    (*s).mtfa[(pp - 1 as libc::c_int) as usize];
                                pp -= 1;
                            }
                            (*s).mtfbase[lno as usize] += 1;
                            (*s).mtfbase[lno as usize];
                            while lno > 0 as libc::c_int {
                                (*s).mtfbase[lno as usize] -= 1;
                                (*s).mtfbase[lno as usize];
                                (*s).mtfa[(*s).mtfbase[lno as usize] as usize] =
                                    (*s).mtfa[((*s).mtfbase[(lno - 1 as libc::c_int) as usize]
                                        + 16 as libc::c_int
                                        - 1 as libc::c_int)
                                        as usize];
                                lno -= 1;
                            }
                            (*s).mtfbase[0 as libc::c_int as usize] -= 1;
                            (*s).mtfbase[0 as libc::c_int as usize];
                            (*s).mtfa[(*s).mtfbase[0 as libc::c_int as usize] as usize] = uc;
                            if (*s).mtfbase[0 as libc::c_int as usize] == 0 as libc::c_int {
                                kk_0 = 4096 as libc::c_int - 1 as libc::c_int;
                                ii_0 = 256 as libc::c_int / 16 as libc::c_int - 1 as libc::c_int;
                                while ii_0 >= 0 as libc::c_int {
                                    jj_0 = 16 as libc::c_int - 1 as libc::c_int;
                                    while jj_0 >= 0 as libc::c_int {
                                        (*s).mtfa[kk_0 as usize] = (*s).mtfa
                                            [((*s).mtfbase[ii_0 as usize] + jj_0) as usize];
                                        kk_0 -= 1;
                                        jj_0 -= 1;
                                    }
                                    (*s).mtfbase[ii_0 as usize] = kk_0 + 1 as libc::c_int;
                                    ii_0 -= 1;
                                }
                            }
                        }
                        (*s).unzftab[(*s).seqToUnseq[uc as usize] as usize] += 1;
                        (*s).unzftab[(*s).seqToUnseq[uc as usize] as usize];
                        if (*s).smallDecompress != 0 {
                            *((*s).ll16).offset(nblock as isize) =
                                (*s).seqToUnseq[uc as usize] as UInt16;
                        } else {
                            *((*s).tt).offset(nblock as isize) =
                                (*s).seqToUnseq[uc as usize] as UInt32;
                        }
                        nblock += 1;
                        if groupPos == 0 as libc::c_int {
                            groupNo += 1;
                            if groupNo >= nSelectors {
                                retVal = -(4 as libc::c_int);
                                current_block = 3350591128142761507;
                                continue;
                            } else {
                                groupPos = 50 as libc::c_int;
                                gSel = (*s).selector[groupNo as usize] as Int32;
                                gMinlen = (*s).minLens[gSel as usize];
                                gLimit = &mut *(*((*s).limit).as_mut_ptr().offset(gSel as isize))
                                    .as_mut_ptr()
                                    .offset(0 as libc::c_int as isize)
                                    as *mut Int32;
                                gPerm = &mut *(*((*s).perm).as_mut_ptr().offset(gSel as isize))
                                    .as_mut_ptr()
                                    .offset(0 as libc::c_int as isize)
                                    as *mut Int32;
                                gBase = &mut *(*((*s).base).as_mut_ptr().offset(gSel as isize))
                                    .as_mut_ptr()
                                    .offset(0 as libc::c_int as isize)
                                    as *mut Int32;
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
                    if (*s).origPtr < 0 as libc::c_int || (*s).origPtr >= nblock {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                        continue;
                    } else {
                        i = 0 as libc::c_int;
                        while i <= 255 as libc::c_int {
                            if (*s).unzftab[i as usize] < 0 as libc::c_int
                                || (*s).unzftab[i as usize] > nblock
                            {
                                retVal = -(4 as libc::c_int);
                                current_block = 3350591128142761507;
                                continue 'c_10064;
                            } else {
                                i += 1;
                            }
                        }
                        (*s).cftab[0 as libc::c_int as usize] = 0 as libc::c_int;
                        i = 1 as libc::c_int;
                        while i <= 256 as libc::c_int {
                            (*s).cftab[i as usize] = (*s).unzftab[(i - 1 as libc::c_int) as usize];
                            i += 1;
                        }
                        i = 1 as libc::c_int;
                        while i <= 256 as libc::c_int {
                            (*s).cftab[i as usize] += (*s).cftab[(i - 1 as libc::c_int) as usize];
                            i += 1;
                        }
                        i = 0 as libc::c_int;
                        while i <= 256 as libc::c_int {
                            if (*s).cftab[i as usize] < 0 as libc::c_int
                                || (*s).cftab[i as usize] > nblock
                            {
                                retVal = -(4 as libc::c_int);
                                current_block = 3350591128142761507;
                                continue 'c_10064;
                            } else {
                                i += 1;
                            }
                        }
                        i = 1 as libc::c_int;
                        while i <= 256 as libc::c_int {
                            if (*s).cftab[(i - 1 as libc::c_int) as usize] > (*s).cftab[i as usize]
                            {
                                retVal = -(4 as libc::c_int);
                                current_block = 3350591128142761507;
                                continue 'c_10064;
                            } else {
                                i += 1;
                            }
                        }
                        (*s).state_out_len = 0 as libc::c_int;
                        (*s).state_out_ch = 0 as libc::c_int as UChar;
                        (*s).calculatedBlockCRC = 0xffffffff as libc::c_long as UInt32;
                        (*s).state = 2 as libc::c_int;
                        if (*s).verbosity >= 2 as libc::c_int {
                            fprintf(stderr, b"rt+rld\0" as *const u8 as *const libc::c_char);
                        }
                        if (*s).smallDecompress != 0 {
                            i = 0 as libc::c_int;
                            while i <= 256 as libc::c_int {
                                (*s).cftabCopy[i as usize] = (*s).cftab[i as usize];
                                i += 1;
                            }
                            i = 0 as libc::c_int;
                            while i < nblock {
                                uc = *((*s).ll16).offset(i as isize) as UChar;
                                *((*s).ll16).offset(i as isize) =
                                    ((*s).cftabCopy[uc as usize] & 0xffff as libc::c_int) as UInt16;
                                if i & 0x1 as libc::c_int == 0 as libc::c_int {
                                    *((*s).ll4).offset((i >> 1 as libc::c_int) as isize) =
                                        (*((*s).ll4).offset((i >> 1 as libc::c_int) as isize)
                                            as libc::c_int
                                            & 0xf0 as libc::c_int
                                            | (*s).cftabCopy[uc as usize] >> 16 as libc::c_int)
                                            as UChar;
                                } else {
                                    *((*s).ll4).offset((i >> 1 as libc::c_int) as isize) =
                                        (*((*s).ll4).offset((i >> 1 as libc::c_int) as isize)
                                            as libc::c_int
                                            & 0xf as libc::c_int
                                            | ((*s).cftabCopy[uc as usize] >> 16 as libc::c_int)
                                                << 4 as libc::c_int)
                                            as UChar;
                                }
                                (*s).cftabCopy[uc as usize] += 1;
                                (*s).cftabCopy[uc as usize];
                                i += 1;
                            }
                            i = (*s).origPtr;
                            j = (*((*s).ll16).offset(i as isize) as UInt32
                                | (*((*s).ll4).offset((i >> 1 as libc::c_int) as isize) as UInt32
                                    >> (i << 2 as libc::c_int & 0x4 as libc::c_int)
                                    & 0xf as libc::c_int as libc::c_uint)
                                    << 16 as libc::c_int) as Int32;
                            loop {
                                let mut tmp_0: Int32 = (*((*s).ll16).offset(j as isize) as UInt32
                                    | (*((*s).ll4).offset((j >> 1 as libc::c_int) as isize)
                                        as UInt32
                                        >> (j << 2 as libc::c_int & 0x4 as libc::c_int)
                                        & 0xf as libc::c_int as libc::c_uint)
                                        << 16 as libc::c_int)
                                    as Int32;
                                *((*s).ll16).offset(j as isize) =
                                    (i & 0xffff as libc::c_int) as UInt16;
                                if j & 0x1 as libc::c_int == 0 as libc::c_int {
                                    *((*s).ll4).offset((j >> 1 as libc::c_int) as isize) =
                                        (*((*s).ll4).offset((j >> 1 as libc::c_int) as isize)
                                            as libc::c_int
                                            & 0xf0 as libc::c_int
                                            | i >> 16 as libc::c_int)
                                            as UChar;
                                } else {
                                    *((*s).ll4).offset((j >> 1 as libc::c_int) as isize) =
                                        (*((*s).ll4).offset((j >> 1 as libc::c_int) as isize)
                                            as libc::c_int
                                            & 0xf as libc::c_int
                                            | (i >> 16 as libc::c_int) << 4 as libc::c_int)
                                            as UChar;
                                }
                                i = j;
                                j = tmp_0;
                                if i == (*s).origPtr {
                                    break;
                                }
                            }
                            (*s).tPos = (*s).origPtr as UInt32;
                            (*s).nblock_used = 0 as libc::c_int;
                            if (*s).blockRandomised != 0 {
                                (*s).rNToGo = 0 as libc::c_int;
                                (*s).rTPos = 0 as libc::c_int;
                                if (*s).tPos
                                    >= (100000 as libc::c_int as UInt32)
                                        .wrapping_mul((*s).blockSize100k as UInt32)
                                {
                                    return 1 as libc::c_int as Bool as Int32;
                                }
                                (*s).k0 =
                                    BZ2_indexIntoF((*s).tPos as Int32, ((*s).cftab).as_mut_ptr());
                                (*s).tPos = *((*s).ll16).offset((*s).tPos as isize) as UInt32
                                    | (*((*s).ll4).offset(((*s).tPos >> 1 as libc::c_int) as isize)
                                        as UInt32
                                        >> ((*s).tPos << 2 as libc::c_int
                                            & 0x4 as libc::c_int as libc::c_uint)
                                        & 0xf as libc::c_int as libc::c_uint)
                                        << 16 as libc::c_int;
                                (*s).nblock_used += 1;
                                (*s).nblock_used;
                                if (*s).rNToGo == 0 as libc::c_int {
                                    (*s).rNToGo = BZ2_rNums[(*s).rTPos as usize];
                                    (*s).rTPos += 1;
                                    (*s).rTPos;
                                    if (*s).rTPos == 512 as libc::c_int {
                                        (*s).rTPos = 0 as libc::c_int;
                                    }
                                }
                                (*s).rNToGo -= 1;
                                (*s).rNToGo;
                                (*s).k0 ^= if (*s).rNToGo == 1 as libc::c_int {
                                    1 as libc::c_int
                                } else {
                                    0 as libc::c_int
                                };
                            } else {
                                if (*s).tPos
                                    >= (100000 as libc::c_int as UInt32)
                                        .wrapping_mul((*s).blockSize100k as UInt32)
                                {
                                    return 1 as libc::c_int as Bool as Int32;
                                }
                                (*s).k0 =
                                    BZ2_indexIntoF((*s).tPos as Int32, ((*s).cftab).as_mut_ptr());
                                (*s).tPos = *((*s).ll16).offset((*s).tPos as isize) as UInt32
                                    | (*((*s).ll4).offset(((*s).tPos >> 1 as libc::c_int) as isize)
                                        as UInt32
                                        >> ((*s).tPos << 2 as libc::c_int
                                            & 0x4 as libc::c_int as libc::c_uint)
                                        & 0xf as libc::c_int as libc::c_uint)
                                        << 16 as libc::c_int;
                                (*s).nblock_used += 1;
                                (*s).nblock_used;
                            }
                        } else {
                            i = 0 as libc::c_int;
                            while i < nblock {
                                uc = (*((*s).tt).offset(i as isize)
                                    & 0xff as libc::c_int as libc::c_uint)
                                    as UChar;
                                let fresh0 =
                                    &mut (*((*s).tt).offset((*s).cftab[uc as usize] as isize));
                                *fresh0 |= (i << 8 as libc::c_int) as libc::c_uint;
                                (*s).cftab[uc as usize] += 1;
                                (*s).cftab[uc as usize];
                                i += 1;
                            }
                            (*s).tPos =
                                *((*s).tt).offset((*s).origPtr as isize) >> 8 as libc::c_int;
                            (*s).nblock_used = 0 as libc::c_int;
                            if (*s).blockRandomised != 0 {
                                (*s).rNToGo = 0 as libc::c_int;
                                (*s).rTPos = 0 as libc::c_int;
                                if (*s).tPos
                                    >= (100000 as libc::c_int as UInt32)
                                        .wrapping_mul((*s).blockSize100k as UInt32)
                                {
                                    return 1 as libc::c_int as Bool as Int32;
                                }
                                (*s).tPos = *((*s).tt).offset((*s).tPos as isize);
                                (*s).k0 = ((*s).tPos & 0xff as libc::c_int as libc::c_uint) as UChar
                                    as Int32;
                                (*s).tPos >>= 8 as libc::c_int;
                                (*s).nblock_used += 1;
                                (*s).nblock_used;
                                if (*s).rNToGo == 0 as libc::c_int {
                                    (*s).rNToGo = BZ2_rNums[(*s).rTPos as usize];
                                    (*s).rTPos += 1;
                                    (*s).rTPos;
                                    if (*s).rTPos == 512 as libc::c_int {
                                        (*s).rTPos = 0 as libc::c_int;
                                    }
                                }
                                (*s).rNToGo -= 1;
                                (*s).rNToGo;
                                (*s).k0 ^= if (*s).rNToGo == 1 as libc::c_int {
                                    1 as libc::c_int
                                } else {
                                    0 as libc::c_int
                                };
                            } else {
                                if (*s).tPos
                                    >= (100000 as libc::c_int as UInt32)
                                        .wrapping_mul((*s).blockSize100k as UInt32)
                                {
                                    return 1 as libc::c_int as Bool as Int32;
                                }
                                (*s).tPos = *((*s).tt).offset((*s).tPos as isize);
                                (*s).k0 = ((*s).tPos & 0xff as libc::c_int as libc::c_uint) as UChar
                                    as Int32;
                                (*s).tPos >>= 8 as libc::c_int;
                                (*s).nblock_used += 1;
                                (*s).nblock_used;
                            }
                        }
                        retVal = 0 as libc::c_int;
                        current_block = 3350591128142761507;
                        continue;
                    }
                }
            }
        }
        if current_block == 5649595406143318745 {
            if N >= 2 as libc::c_int * 1024 as libc::c_int * 1024 as libc::c_int {
                retVal = -(4 as libc::c_int);
                current_block = 3350591128142761507;
                continue;
            } else {
                if nextSym == 0 as libc::c_int {
                    es += (0 as libc::c_int + 1 as libc::c_int) * N;
                } else if nextSym == 1 as libc::c_int {
                    es += (1 as libc::c_int + 1 as libc::c_int) * N;
                }
                N *= 2 as libc::c_int;
                if groupPos == 0 as libc::c_int {
                    groupNo += 1;
                    if groupNo >= nSelectors {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                        continue;
                    } else {
                        groupPos = 50 as libc::c_int;
                        gSel = (*s).selector[groupNo as usize] as Int32;
                        gMinlen = (*s).minLens[gSel as usize];
                        gLimit = &mut *(*((*s).limit).as_mut_ptr().offset(gSel as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize)
                            as *mut Int32;
                        gPerm = &mut *(*((*s).perm).as_mut_ptr().offset(gSel as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize)
                            as *mut Int32;
                        gBase = &mut *(*((*s).base).as_mut_ptr().offset(gSel as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize)
                            as *mut Int32;
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
                    if j < 16 as libc::c_int {
                        current_block = 15451013008180677144;
                        continue 'c_10064;
                    }
                }
                3503188808869013853 => {
                    if i < nSelectors {
                        j = 0 as libc::c_int;
                        current_block = 16531797892856733396;
                        continue;
                    } else {
                        if nSelectors > 2 as libc::c_int + 900000 as libc::c_int / 50 as libc::c_int
                        {
                            nSelectors =
                                2 as libc::c_int + 900000 as libc::c_int / 50 as libc::c_int;
                        }
                        let mut pos: [UChar; 6] = [0; 6];
                        let mut tmp: UChar = 0;
                        let mut v_22: UChar = 0;
                        v_22 = 0 as libc::c_int as UChar;
                        while (v_22 as libc::c_int) < nGroups {
                            pos[v_22 as usize] = v_22;
                            v_22 = v_22.wrapping_add(1);
                        }
                        i = 0 as libc::c_int;
                        while i < nSelectors {
                            v_22 = (*s).selectorMtf[i as usize];
                            tmp = pos[v_22 as usize];
                            while v_22 as libc::c_int > 0 as libc::c_int {
                                pos[v_22 as usize] =
                                    pos[(v_22 as libc::c_int - 1 as libc::c_int) as usize];
                                v_22 = v_22.wrapping_sub(1);
                            }
                            pos[0 as libc::c_int as usize] = tmp;
                            (*s).selector[i as usize] = tmp;
                            i += 1;
                        }
                        t = 0 as libc::c_int;
                        current_block = 2488856075421756534;
                        break;
                    }
                }
                15415362524153386998 => {
                    if i < 16 as libc::c_int {
                        if (*s).inUse16[i as usize] != 0 {
                            j = 0 as libc::c_int;
                            current_block = 16953886395775657100;
                            continue;
                        }
                    } else {
                        makeMaps_d(s);
                        if (*s).nInUse == 0 as libc::c_int {
                            current_block = 12571193857528100212;
                            break;
                        } else {
                            current_block = 9416928054198617439;
                            break;
                        }
                    }
                }
                7746242308555130918 => {
                    (*s).len[t as usize][i as usize] = curr as UChar;
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
                    if i < 2 as libc::c_int + 900000 as libc::c_int / 50 as libc::c_int {
                        (*s).selectorMtf[i as usize] = j as UChar;
                    }
                    i += 1;
                    current_block = 3503188808869013853;
                    continue;
                }
                16531797892856733396 => {
                    if 1 as libc::c_int as Bool != 0 {
                        current_block = 15957329598978927534;
                        continue 'c_10064;
                    } else {
                        current_block = 10081471997089450706;
                        continue;
                    }
                }
                _ => {
                    if 1 as libc::c_int as Bool == 0 {
                        current_block = 7746242308555130918;
                        continue;
                    }
                    if !(curr < 1 as libc::c_int || curr > 20 as libc::c_int) {
                        current_block = 17216244326479313607;
                        continue 'c_10064;
                    }
                    retVal = -(4 as libc::c_int);
                    current_block = 3350591128142761507;
                    continue 'c_10064;
                }
            }
            i += 1;
            current_block = 15415362524153386998;
        }
        match current_block {
            9416928054198617439 => {
                alphaSize = (*s).nInUse + 2 as libc::c_int;
                current_block = 9434444550647791986;
            }
            12571193857528100212 => {
                retVal = -(4 as libc::c_int);
                current_block = 3350591128142761507;
            }
            _ => {
                if t < nGroups {
                    current_block = 11569294379105328467;
                    continue;
                }
                t = 0 as libc::c_int;
                while t < nGroups {
                    minLen = 32 as libc::c_int;
                    maxLen = 0 as libc::c_int;
                    i = 0 as libc::c_int;
                    while i < alphaSize {
                        if (*s).len[t as usize][i as usize] as libc::c_int > maxLen {
                            maxLen = (*s).len[t as usize][i as usize] as Int32;
                        }
                        if ((*s).len[t as usize][i as usize] as libc::c_int) < minLen {
                            minLen = (*s).len[t as usize][i as usize] as Int32;
                        }
                        i += 1;
                    }
                    BZ2_hbCreateDecodeTables(
                        &mut *(*((*s).limit).as_mut_ptr().offset(t as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize),
                        &mut *(*((*s).base).as_mut_ptr().offset(t as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize),
                        &mut *(*((*s).perm).as_mut_ptr().offset(t as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize),
                        &mut *(*((*s).len).as_mut_ptr().offset(t as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize),
                        minLen,
                        maxLen,
                        alphaSize,
                    );
                    (*s).minLens[t as usize] = minLen;
                    t += 1;
                }
                EOB = (*s).nInUse + 1 as libc::c_int;
                nblockMAX = 100000 as libc::c_int * (*s).blockSize100k;
                groupNo = -(1 as libc::c_int);
                groupPos = 0 as libc::c_int;
                i = 0 as libc::c_int;
                while i <= 255 as libc::c_int {
                    (*s).unzftab[i as usize] = 0 as libc::c_int;
                    i += 1;
                }
                let mut ii: Int32 = 0;
                let mut jj: Int32 = 0;
                let mut kk: Int32 = 0;
                kk = 4096 as libc::c_int - 1 as libc::c_int;
                ii = 256 as libc::c_int / 16 as libc::c_int - 1 as libc::c_int;
                while ii >= 0 as libc::c_int {
                    jj = 16 as libc::c_int - 1 as libc::c_int;
                    while jj >= 0 as libc::c_int {
                        (*s).mtfa[kk as usize] = (ii * 16 as libc::c_int + jj) as UChar;
                        kk -= 1;
                        jj -= 1;
                    }
                    (*s).mtfbase[ii as usize] = kk + 1 as libc::c_int;
                    ii -= 1;
                }
                nblock = 0 as libc::c_int;
                if groupPos == 0 as libc::c_int {
                    groupNo += 1;
                    if groupNo >= nSelectors {
                        retVal = -(4 as libc::c_int);
                        current_block = 3350591128142761507;
                        continue;
                    } else {
                        groupPos = 50 as libc::c_int;
                        gSel = (*s).selector[groupNo as usize] as Int32;
                        gMinlen = (*s).minLens[gSel as usize];
                        gLimit = &mut *(*((*s).limit).as_mut_ptr().offset(gSel as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize)
                            as *mut Int32;
                        gPerm = &mut *(*((*s).perm).as_mut_ptr().offset(gSel as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize)
                            as *mut Int32;
                        gBase = &mut *(*((*s).base).as_mut_ptr().offset(gSel as isize))
                            .as_mut_ptr()
                            .offset(0 as libc::c_int as isize)
                            as *mut Int32;
                    }
                }
                groupPos -= 1;
                zn = gMinlen;
                current_block = 13155828021133314705;
            }
        }
    }
    (*s).save_j = j;
    (*s).save_t = t;
    (*s).save_alphaSize = alphaSize;
    (*s).save_nGroups = nGroups;
    (*s).save_nSelectors = nSelectors;
    (*s).save_EOB = EOB;
    (*s).save_groupNo = groupNo;
    (*s).save_groupPos = groupPos;
    (*s).save_nextSym = nextSym;
    (*s).save_nblockMAX = nblockMAX;
    (*s).save_nblock = nblock;
    (*s).save_es = es;
    (*s).save_N = N;
    (*s).save_curr = curr;
    (*s).save_zt = zt;
    (*s).save_zn = zn;
    (*s).save_zvec = zvec;
    (*s).save_zj = zj;
    (*s).save_gSel = gSel;
    (*s).save_gMinlen = gMinlen;
    (*s).save_gLimit = gLimit;
    (*s).save_gBase = gBase;
    (*s).save_gPerm = gPerm;
    retVal
}

use ::libc;
use libc::{fprintf, FILE};
extern "C" {
    static mut stderr: *mut FILE;
}
use crate::blocksort::BZ2_blockSort;
use crate::bzlib::{BZ2_bz__AssertH__fail, Bool, EState, Int32, UChar, UInt16, UInt32};
use crate::huffman::{BZ2_hbAssignCodes, BZ2_hbMakeCodeLengths};
#[no_mangle]
pub unsafe extern "C" fn BZ2_bsInitWrite(mut s: *mut EState) {
    (*s).bsLive = 0 as libc::c_int;
    (*s).bsBuff = 0 as libc::c_int as UInt32;
}
unsafe extern "C" fn bsFinishWrite(mut s: *mut EState) {
    while (*s).bsLive > 0 as libc::c_int {
        *((*s).zbits).offset((*s).numZ as isize) = ((*s).bsBuff >> 24 as libc::c_int) as UChar;
        (*s).numZ += 1;
        (*s).numZ;
        (*s).bsBuff <<= 8 as libc::c_int;
        (*s).bsLive -= 8 as libc::c_int;
    }
}
#[inline]
unsafe extern "C" fn bsW(mut s: *mut EState, mut n: Int32, mut v: UInt32) {
    while (*s).bsLive >= 8 as libc::c_int {
        *((*s).zbits).offset((*s).numZ as isize) = ((*s).bsBuff >> 24 as libc::c_int) as UChar;
        (*s).numZ += 1;
        (*s).numZ;
        (*s).bsBuff <<= 8 as libc::c_int;
        (*s).bsLive -= 8 as libc::c_int;
    }
    (*s).bsBuff |= v << (32 as libc::c_int - (*s).bsLive - n);
    (*s).bsLive += n;
}
unsafe extern "C" fn bsPutUInt32(mut s: *mut EState, mut u: UInt32) {
    bsW(
        s,
        8 as libc::c_int,
        ((u >> 24 as libc::c_int) as libc::c_long & 0xff as libc::c_long) as UInt32,
    );
    bsW(
        s,
        8 as libc::c_int,
        ((u >> 16 as libc::c_int) as libc::c_long & 0xff as libc::c_long) as UInt32,
    );
    bsW(
        s,
        8 as libc::c_int,
        ((u >> 8 as libc::c_int) as libc::c_long & 0xff as libc::c_long) as UInt32,
    );
    bsW(
        s,
        8 as libc::c_int,
        (u as libc::c_long & 0xff as libc::c_long) as UInt32,
    );
}
unsafe extern "C" fn bsPutUChar(mut s: *mut EState, mut c: UChar) {
    bsW(s, 8 as libc::c_int, c as UInt32);
}
unsafe extern "C" fn makeMaps_e(mut s: *mut EState) {
    let mut i: Int32 = 0;
    (*s).nInUse = 0 as libc::c_int;
    i = 0 as libc::c_int;
    while i < 256 as libc::c_int {
        if (*s).inUse[i as usize] != 0 {
            (*s).unseqToSeq[i as usize] = (*s).nInUse as UChar;
            (*s).nInUse += 1;
            (*s).nInUse;
        }
        i += 1;
    }
}
unsafe extern "C" fn generateMTFValues(mut s: *mut EState) {
    let mut yy: [UChar; 256] = [0; 256];
    let mut i: Int32 = 0;
    let mut j: Int32 = 0;
    let mut zPend: Int32 = 0;
    let mut wr: Int32 = 0;
    let mut EOB: Int32 = 0;
    let mut ptr: *mut UInt32 = (*s).ptr;
    let mut block: *mut UChar = (*s).block;
    let mut mtfv: *mut UInt16 = (*s).mtfv;
    makeMaps_e(s);
    EOB = (*s).nInUse + 1 as libc::c_int;
    i = 0 as libc::c_int;
    while i <= EOB {
        (*s).mtfFreq[i as usize] = 0 as libc::c_int;
        i += 1;
    }
    wr = 0 as libc::c_int;
    zPend = 0 as libc::c_int;
    i = 0 as libc::c_int;
    while i < (*s).nInUse {
        yy[i as usize] = i as UChar;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < (*s).nblock {
        let mut ll_i: UChar = 0;
        j = (*ptr.offset(i as isize)).wrapping_sub(1 as libc::c_int as libc::c_uint) as Int32;
        if j < 0 as libc::c_int {
            j += (*s).nblock;
        }
        ll_i = (*s).unseqToSeq[*block.offset(j as isize) as usize];
        if yy[0 as libc::c_int as usize] as libc::c_int == ll_i as libc::c_int {
            zPend += 1;
        } else {
            if zPend > 0 as libc::c_int {
                zPend -= 1;
                while 1 as libc::c_int as Bool != 0 {
                    if zPend & 1 as libc::c_int != 0 {
                        *mtfv.offset(wr as isize) = 1 as libc::c_int as UInt16;
                        wr += 1;
                        (*s).mtfFreq[1 as libc::c_int as usize] += 1;
                        (*s).mtfFreq[1 as libc::c_int as usize];
                    } else {
                        *mtfv.offset(wr as isize) = 0 as libc::c_int as UInt16;
                        wr += 1;
                        (*s).mtfFreq[0 as libc::c_int as usize] += 1;
                        (*s).mtfFreq[0 as libc::c_int as usize];
                    }
                    if zPend < 2 as libc::c_int {
                        break;
                    }
                    zPend = (zPend - 2 as libc::c_int) / 2 as libc::c_int;
                }
                zPend = 0 as libc::c_int;
            }
            let mut rtmp: UChar = 0;
            let mut ryy_j: *mut UChar = std::ptr::null_mut::<UChar>();
            let mut rll_i: UChar = 0;
            rtmp = yy[1 as libc::c_int as usize];
            yy[1 as libc::c_int as usize] = yy[0 as libc::c_int as usize];
            ryy_j = &mut *yy.as_mut_ptr().offset(1 as libc::c_int as isize) as *mut UChar;
            rll_i = ll_i;
            while rll_i as libc::c_int != rtmp as libc::c_int {
                let mut rtmp2: UChar = 0;
                ryy_j = ryy_j.offset(1);
                rtmp2 = rtmp;
                rtmp = *ryy_j;
                *ryy_j = rtmp2;
            }
            yy[0 as libc::c_int as usize] = rtmp;
            j = ryy_j
                .offset_from(&mut *yy.as_mut_ptr().offset(0 as libc::c_int as isize) as *mut UChar)
                as libc::c_long as Int32;
            *mtfv.offset(wr as isize) = (j + 1 as libc::c_int) as UInt16;
            wr += 1;
            (*s).mtfFreq[(j + 1 as libc::c_int) as usize] += 1;
            (*s).mtfFreq[(j + 1 as libc::c_int) as usize];
        }
        i += 1;
    }
    if zPend > 0 as libc::c_int {
        zPend -= 1;
        while 1 as libc::c_int as Bool != 0 {
            if zPend & 1 as libc::c_int != 0 {
                *mtfv.offset(wr as isize) = 1 as libc::c_int as UInt16;
                wr += 1;
                (*s).mtfFreq[1 as libc::c_int as usize] += 1;
                (*s).mtfFreq[1 as libc::c_int as usize];
            } else {
                *mtfv.offset(wr as isize) = 0 as libc::c_int as UInt16;
                wr += 1;
                (*s).mtfFreq[0 as libc::c_int as usize] += 1;
                (*s).mtfFreq[0 as libc::c_int as usize];
            }
            if zPend < 2 as libc::c_int {
                break;
            }
            zPend = (zPend - 2 as libc::c_int) / 2 as libc::c_int;
        }
        zPend = 0 as libc::c_int;
    }
    *mtfv.offset(wr as isize) = EOB as UInt16;
    wr += 1;
    (*s).mtfFreq[EOB as usize] += 1;
    (*s).mtfFreq[EOB as usize];
    (*s).nMTF = wr;
}
unsafe extern "C" fn sendMTFValues(mut s: *mut EState) {
    let mut v: Int32 = 0;
    let mut t: Int32 = 0;
    let mut i: Int32 = 0;
    let mut j: Int32 = 0;
    let mut gs: Int32 = 0;
    let mut ge: Int32 = 0;
    let mut totc: Int32 = 0;
    let mut bt: Int32 = 0;
    let mut bc: Int32 = 0;
    let mut iter: Int32 = 0;
    let mut nSelectors: Int32 = 0;
    let mut alphaSize: Int32 = 0;
    let mut minLen: Int32 = 0;
    let mut maxLen: Int32 = 0;
    let mut selCtr: Int32 = 0;
    let mut nGroups: Int32 = 0;
    let mut nBytes: Int32 = 0;
    let mut cost: [UInt16; 6] = [0; 6];
    let mut fave: [Int32; 6] = [0; 6];
    let mut mtfv: *mut UInt16 = (*s).mtfv;
    if (*s).verbosity >= 3 as libc::c_int {
        fprintf(
            stderr,
            b"      %d in block, %d after MTF & 1-2 coding, %d+2 syms in use\n\0" as *const u8
                as *const libc::c_char,
            (*s).nblock,
            (*s).nMTF,
            (*s).nInUse,
        );
    }
    alphaSize = (*s).nInUse + 2 as libc::c_int;
    t = 0 as libc::c_int;
    while t < 6 as libc::c_int {
        v = 0 as libc::c_int;
        while v < alphaSize {
            (*s).len[t as usize][v as usize] = 15 as libc::c_int as UChar;
            v += 1;
        }
        t += 1;
    }
    if (*s).nMTF <= 0 as libc::c_int {
        BZ2_bz__AssertH__fail(3001 as libc::c_int);
    }
    if (*s).nMTF < 200 as libc::c_int {
        nGroups = 2 as libc::c_int;
    } else if (*s).nMTF < 600 as libc::c_int {
        nGroups = 3 as libc::c_int;
    } else if (*s).nMTF < 1200 as libc::c_int {
        nGroups = 4 as libc::c_int;
    } else if (*s).nMTF < 2400 as libc::c_int {
        nGroups = 5 as libc::c_int;
    } else {
        nGroups = 6 as libc::c_int;
    }
    let mut nPart: Int32 = 0;
    let mut remF: Int32 = 0;
    let mut tFreq: Int32 = 0;
    let mut aFreq: Int32 = 0;
    nPart = nGroups;
    remF = (*s).nMTF;
    gs = 0 as libc::c_int;
    while nPart > 0 as libc::c_int {
        tFreq = remF / nPart;
        ge = gs - 1 as libc::c_int;
        aFreq = 0 as libc::c_int;
        while aFreq < tFreq && ge < alphaSize - 1 as libc::c_int {
            ge += 1;
            aFreq += (*s).mtfFreq[ge as usize];
        }
        if ge > gs
            && nPart != nGroups
            && nPart != 1 as libc::c_int
            && (nGroups - nPart) % 2 as libc::c_int == 1 as libc::c_int
        {
            aFreq -= (*s).mtfFreq[ge as usize];
            ge -= 1;
        }
        if (*s).verbosity >= 3 as libc::c_int {
            fprintf(
                stderr,
                b"      initial group %d, [%d .. %d], has %d syms (%4.1f%%)\n\0" as *const u8
                    as *const libc::c_char,
                nPart,
                gs,
                ge,
                aFreq,
                100.0f64 * aFreq as libc::c_float as libc::c_double
                    / (*s).nMTF as libc::c_float as libc::c_double,
            );
        }
        v = 0 as libc::c_int;
        while v < alphaSize {
            if v >= gs && v <= ge {
                (*s).len[(nPart - 1 as libc::c_int) as usize][v as usize] =
                    0 as libc::c_int as UChar;
            } else {
                (*s).len[(nPart - 1 as libc::c_int) as usize][v as usize] =
                    15 as libc::c_int as UChar;
            }
            v += 1;
        }
        nPart -= 1;
        gs = ge + 1 as libc::c_int;
        remF -= aFreq;
    }
    iter = 0 as libc::c_int;
    while iter < 4 as libc::c_int {
        t = 0 as libc::c_int;
        while t < nGroups {
            fave[t as usize] = 0 as libc::c_int;
            t += 1;
        }
        t = 0 as libc::c_int;
        while t < nGroups {
            v = 0 as libc::c_int;
            while v < alphaSize {
                (*s).rfreq[t as usize][v as usize] = 0 as libc::c_int;
                v += 1;
            }
            t += 1;
        }
        if nGroups == 6 as libc::c_int {
            v = 0 as libc::c_int;
            while v < alphaSize {
                (*s).len_pack[v as usize][0 as libc::c_int as usize] =
                    (((*s).len[1 as libc::c_int as usize][v as usize] as libc::c_int)
                        << 16 as libc::c_int
                        | (*s).len[0 as libc::c_int as usize][v as usize] as libc::c_int)
                        as UInt32;
                (*s).len_pack[v as usize][1 as libc::c_int as usize] =
                    (((*s).len[3 as libc::c_int as usize][v as usize] as libc::c_int)
                        << 16 as libc::c_int
                        | (*s).len[2 as libc::c_int as usize][v as usize] as libc::c_int)
                        as UInt32;
                (*s).len_pack[v as usize][2 as libc::c_int as usize] =
                    (((*s).len[5 as libc::c_int as usize][v as usize] as libc::c_int)
                        << 16 as libc::c_int
                        | (*s).len[4 as libc::c_int as usize][v as usize] as libc::c_int)
                        as UInt32;
                v += 1;
            }
        }
        nSelectors = 0 as libc::c_int;
        totc = 0 as libc::c_int;
        gs = 0 as libc::c_int;
        while 1 as libc::c_int as Bool != 0 {
            if gs >= (*s).nMTF {
                break;
            }
            ge = gs + 50 as libc::c_int - 1 as libc::c_int;
            if ge >= (*s).nMTF {
                ge = (*s).nMTF - 1 as libc::c_int;
            }
            t = 0 as libc::c_int;
            while t < nGroups {
                cost[t as usize] = 0 as libc::c_int as UInt16;
                t += 1;
            }
            if nGroups == 6 as libc::c_int && 50 as libc::c_int == ge - gs + 1 as libc::c_int {
                let mut cost01: UInt32 = 0;
                let mut cost23: UInt32 = 0;
                let mut cost45: UInt32 = 0;
                let mut icv: UInt16 = 0;
                cost45 = 0 as libc::c_int as UInt32;
                cost23 = cost45;
                cost01 = cost23;
                icv = *mtfv.offset((gs + 0 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 1 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 2 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 3 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 4 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 5 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 6 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 7 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 8 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 9 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 10 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 11 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 12 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 13 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 14 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 15 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 16 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 17 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 18 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 19 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 20 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 21 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 22 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 23 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 24 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 25 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 26 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 27 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 28 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 29 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 30 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 31 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 32 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 33 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 34 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 35 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 36 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 37 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 38 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 39 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 40 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 41 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 42 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 43 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 44 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 45 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 46 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 47 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 48 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                icv = *mtfv.offset((gs + 49 as libc::c_int) as isize);
                cost01 = (cost01 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][0 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost23 = (cost23 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][1 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost45 = (cost45 as libc::c_uint)
                    .wrapping_add((*s).len_pack[icv as usize][2 as libc::c_int as usize])
                    as UInt32 as UInt32;
                cost[0 as libc::c_int as usize] =
                    (cost01 & 0xffff as libc::c_int as libc::c_uint) as UInt16;
                cost[1 as libc::c_int as usize] = (cost01 >> 16 as libc::c_int) as UInt16;
                cost[2 as libc::c_int as usize] =
                    (cost23 & 0xffff as libc::c_int as libc::c_uint) as UInt16;
                cost[3 as libc::c_int as usize] = (cost23 >> 16 as libc::c_int) as UInt16;
                cost[4 as libc::c_int as usize] =
                    (cost45 & 0xffff as libc::c_int as libc::c_uint) as UInt16;
                cost[5 as libc::c_int as usize] = (cost45 >> 16 as libc::c_int) as UInt16;
            } else {
                i = gs;
                while i <= ge {
                    let mut icv_0: UInt16 = *mtfv.offset(i as isize);
                    t = 0 as libc::c_int;
                    while t < nGroups {
                        cost[t as usize] = (cost[t as usize] as libc::c_int
                            + (*s).len[t as usize][icv_0 as usize] as libc::c_int)
                            as UInt16;
                        t += 1;
                    }
                    i += 1;
                }
            }
            bc = 999999999 as libc::c_int;
            bt = -(1 as libc::c_int);
            t = 0 as libc::c_int;
            while t < nGroups {
                if (cost[t as usize] as libc::c_int) < bc {
                    bc = cost[t as usize] as Int32;
                    bt = t;
                }
                t += 1;
            }
            totc += bc;
            fave[bt as usize] += 1;
            fave[bt as usize];
            (*s).selector[nSelectors as usize] = bt as UChar;
            nSelectors += 1;
            if nGroups == 6 as libc::c_int && 50 as libc::c_int == ge - gs + 1 as libc::c_int {
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 0 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 0 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 1 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 1 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 2 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 2 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 3 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 3 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 4 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 4 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 5 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 5 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 6 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 6 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 7 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 7 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 8 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 8 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 9 as libc::c_int) as isize) as usize] +=
                    1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 9 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 10 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 10 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 11 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 11 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 12 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 12 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 13 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 13 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 14 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 14 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 15 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 15 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 16 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 16 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 17 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 17 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 18 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 18 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 19 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 19 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 20 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 20 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 21 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 21 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 22 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 22 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 23 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 23 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 24 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 24 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 25 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 25 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 26 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 26 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 27 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 27 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 28 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 28 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 29 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 29 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 30 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 30 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 31 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 31 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 32 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 32 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 33 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 33 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 34 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 34 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 35 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 35 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 36 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 36 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 37 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 37 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 38 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 38 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 39 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 39 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 40 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 40 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 41 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 41 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 42 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 42 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 43 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 43 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 44 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 44 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 45 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 45 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 46 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 46 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 47 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 47 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 48 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 48 as libc::c_int) as isize) as usize];
                (*s).rfreq[bt as usize]
                    [*mtfv.offset((gs + 49 as libc::c_int) as isize) as usize] += 1;
                (*s).rfreq[bt as usize][*mtfv.offset((gs + 49 as libc::c_int) as isize) as usize];
            } else {
                i = gs;
                while i <= ge {
                    (*s).rfreq[bt as usize][*mtfv.offset(i as isize) as usize] += 1;
                    (*s).rfreq[bt as usize][*mtfv.offset(i as isize) as usize];
                    i += 1;
                }
            }
            gs = ge + 1 as libc::c_int;
        }
        if (*s).verbosity >= 3 as libc::c_int {
            fprintf(
                stderr,
                b"      pass %d: size is %d, grp uses are \0" as *const u8 as *const libc::c_char,
                iter + 1 as libc::c_int,
                totc / 8 as libc::c_int,
            );
            t = 0 as libc::c_int;
            while t < nGroups {
                fprintf(
                    stderr,
                    b"%d \0" as *const u8 as *const libc::c_char,
                    fave[t as usize],
                );
                t += 1;
            }
            fprintf(stderr, b"\n\0" as *const u8 as *const libc::c_char);
        }
        t = 0 as libc::c_int;
        while t < nGroups {
            BZ2_hbMakeCodeLengths(
                &mut *(*((*s).len).as_mut_ptr().offset(t as isize))
                    .as_mut_ptr()
                    .offset(0 as libc::c_int as isize),
                &mut *(*((*s).rfreq).as_mut_ptr().offset(t as isize))
                    .as_mut_ptr()
                    .offset(0 as libc::c_int as isize),
                alphaSize,
                17 as libc::c_int,
            );
            t += 1;
        }
        iter += 1;
    }
    if nGroups >= 8 as libc::c_int {
        BZ2_bz__AssertH__fail(3002 as libc::c_int);
    }
    if !(nSelectors < 32768 as libc::c_int
        && nSelectors <= 2 as libc::c_int + 900000 as libc::c_int / 50 as libc::c_int)
    {
        BZ2_bz__AssertH__fail(3003 as libc::c_int);
    }
    let mut pos: [UChar; 6] = [0; 6];
    let mut ll_i: UChar = 0;
    let mut tmp2: UChar = 0;
    let mut tmp: UChar = 0;
    i = 0 as libc::c_int;
    while i < nGroups {
        pos[i as usize] = i as UChar;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < nSelectors {
        ll_i = (*s).selector[i as usize];
        j = 0 as libc::c_int;
        tmp = pos[j as usize];
        while ll_i as libc::c_int != tmp as libc::c_int {
            j += 1;
            tmp2 = tmp;
            tmp = pos[j as usize];
            pos[j as usize] = tmp2;
        }
        pos[0 as libc::c_int as usize] = tmp;
        (*s).selectorMtf[i as usize] = j as UChar;
        i += 1;
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
        if maxLen > 17 as libc::c_int {
            BZ2_bz__AssertH__fail(3004 as libc::c_int);
        }
        if minLen < 1 as libc::c_int {
            BZ2_bz__AssertH__fail(3005 as libc::c_int);
        }
        BZ2_hbAssignCodes(
            &mut *(*((*s).code).as_mut_ptr().offset(t as isize))
                .as_mut_ptr()
                .offset(0 as libc::c_int as isize),
            &mut *(*((*s).len).as_mut_ptr().offset(t as isize))
                .as_mut_ptr()
                .offset(0 as libc::c_int as isize),
            minLen,
            maxLen,
            alphaSize,
        );
        t += 1;
    }
    let mut inUse16: [Bool; 16] = [0; 16];
    i = 0 as libc::c_int;
    while i < 16 as libc::c_int {
        inUse16[i as usize] = 0 as libc::c_int as Bool;
        j = 0 as libc::c_int;
        while j < 16 as libc::c_int {
            if (*s).inUse[(i * 16 as libc::c_int + j) as usize] != 0 {
                inUse16[i as usize] = 1 as libc::c_int as Bool;
            }
            j += 1;
        }
        i += 1;
    }
    nBytes = (*s).numZ;
    i = 0 as libc::c_int;
    while i < 16 as libc::c_int {
        if inUse16[i as usize] != 0 {
            bsW(s, 1 as libc::c_int, 1 as libc::c_int as UInt32);
        } else {
            bsW(s, 1 as libc::c_int, 0 as libc::c_int as UInt32);
        }
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < 16 as libc::c_int {
        if inUse16[i as usize] != 0 {
            j = 0 as libc::c_int;
            while j < 16 as libc::c_int {
                if (*s).inUse[(i * 16 as libc::c_int + j) as usize] != 0 {
                    bsW(s, 1 as libc::c_int, 1 as libc::c_int as UInt32);
                } else {
                    bsW(s, 1 as libc::c_int, 0 as libc::c_int as UInt32);
                }
                j += 1;
            }
        }
        i += 1;
    }
    if (*s).verbosity >= 3 as libc::c_int {
        fprintf(
            stderr,
            b"      bytes: mapping %d, \0" as *const u8 as *const libc::c_char,
            (*s).numZ - nBytes,
        );
    }
    nBytes = (*s).numZ;
    bsW(s, 3 as libc::c_int, nGroups as UInt32);
    bsW(s, 15 as libc::c_int, nSelectors as UInt32);
    i = 0 as libc::c_int;
    while i < nSelectors {
        j = 0 as libc::c_int;
        while j < (*s).selectorMtf[i as usize] as libc::c_int {
            bsW(s, 1 as libc::c_int, 1 as libc::c_int as UInt32);
            j += 1;
        }
        bsW(s, 1 as libc::c_int, 0 as libc::c_int as UInt32);
        i += 1;
    }
    if (*s).verbosity >= 3 as libc::c_int {
        fprintf(
            stderr,
            b"selectors %d, \0" as *const u8 as *const libc::c_char,
            (*s).numZ - nBytes,
        );
    }
    nBytes = (*s).numZ;
    t = 0 as libc::c_int;
    while t < nGroups {
        let mut curr: Int32 = (*s).len[t as usize][0 as libc::c_int as usize] as Int32;
        bsW(s, 5 as libc::c_int, curr as UInt32);
        i = 0 as libc::c_int;
        while i < alphaSize {
            while curr < (*s).len[t as usize][i as usize] as libc::c_int {
                bsW(s, 2 as libc::c_int, 2 as libc::c_int as UInt32);
                curr += 1;
            }
            while curr > (*s).len[t as usize][i as usize] as libc::c_int {
                bsW(s, 2 as libc::c_int, 3 as libc::c_int as UInt32);
                curr -= 1;
            }
            bsW(s, 1 as libc::c_int, 0 as libc::c_int as UInt32);
            i += 1;
        }
        t += 1;
    }
    if (*s).verbosity >= 3 as libc::c_int {
        fprintf(
            stderr,
            b"code lengths %d, \0" as *const u8 as *const libc::c_char,
            (*s).numZ - nBytes,
        );
    }
    nBytes = (*s).numZ;
    selCtr = 0 as libc::c_int;
    gs = 0 as libc::c_int;
    while 1 as libc::c_int as Bool != 0 {
        if gs >= (*s).nMTF {
            break;
        }
        ge = gs + 50 as libc::c_int - 1 as libc::c_int;
        if ge >= (*s).nMTF {
            ge = (*s).nMTF - 1 as libc::c_int;
        }
        if ((*s).selector[selCtr as usize] as libc::c_int) >= nGroups {
            BZ2_bz__AssertH__fail(3006 as libc::c_int);
        }
        if nGroups == 6 as libc::c_int && 50 as libc::c_int == ge - gs + 1 as libc::c_int {
            let mut mtfv_i: UInt16 = 0;
            let mut s_len_sel_selCtr: *mut UChar = &mut *(*((*s).len)
                .as_mut_ptr()
                .offset(*((*s).selector).as_mut_ptr().offset(selCtr as isize) as isize))
            .as_mut_ptr()
            .offset(0 as libc::c_int as isize)
                as *mut UChar;
            let mut s_code_sel_selCtr: *mut Int32 = &mut *(*((*s).code)
                .as_mut_ptr()
                .offset(*((*s).selector).as_mut_ptr().offset(selCtr as isize) as isize))
            .as_mut_ptr()
            .offset(0 as libc::c_int as isize)
                as *mut Int32;
            mtfv_i = *mtfv.offset((gs + 0 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 1 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 2 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 3 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 4 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 5 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 6 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 7 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 8 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 9 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 10 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 11 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 12 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 13 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 14 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 15 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 16 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 17 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 18 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 19 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 20 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 21 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 22 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 23 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 24 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 25 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 26 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 27 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 28 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 29 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 30 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 31 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 32 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 33 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 34 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 35 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 36 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 37 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 38 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 39 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 40 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 41 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 42 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 43 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 44 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 45 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 46 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 47 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 48 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
            mtfv_i = *mtfv.offset((gs + 49 as libc::c_int) as isize);
            bsW(
                s,
                *s_len_sel_selCtr.offset(mtfv_i as isize) as Int32,
                *s_code_sel_selCtr.offset(mtfv_i as isize) as UInt32,
            );
        } else {
            i = gs;
            while i <= ge {
                bsW(
                    s,
                    (*s).len[(*s).selector[selCtr as usize] as usize]
                        [*mtfv.offset(i as isize) as usize] as Int32,
                    (*s).code[(*s).selector[selCtr as usize] as usize]
                        [*mtfv.offset(i as isize) as usize] as UInt32,
                );
                i += 1;
            }
        }
        gs = ge + 1 as libc::c_int;
        selCtr += 1;
    }
    if selCtr != nSelectors {
        BZ2_bz__AssertH__fail(3007 as libc::c_int);
    }
    if (*s).verbosity >= 3 as libc::c_int {
        fprintf(
            stderr,
            b"codes %d\n\0" as *const u8 as *const libc::c_char,
            (*s).numZ - nBytes,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn BZ2_compressBlock(mut s: *mut EState, mut is_last_block: Bool) {
    if (*s).nblock > 0 as libc::c_int {
        (*s).blockCRC = !(*s).blockCRC;
        (*s).combinedCRC =
            (*s).combinedCRC << 1 as libc::c_int | (*s).combinedCRC >> 31 as libc::c_int;
        (*s).combinedCRC ^= (*s).blockCRC;
        if (*s).blockNo > 1 as libc::c_int {
            (*s).numZ = 0 as libc::c_int;
        }
        if (*s).verbosity >= 2 as libc::c_int {
            fprintf(
                stderr,
                b"    block %d: crc = 0x%08x, combined CRC = 0x%08x, size = %d\n\0" as *const u8
                    as *const libc::c_char,
                (*s).blockNo,
                (*s).blockCRC,
                (*s).combinedCRC,
                (*s).nblock,
            );
        }
        BZ2_blockSort(s);
    }
    (*s).zbits = &mut *((*s).arr2 as *mut UChar).offset((*s).nblock as isize) as *mut UChar;
    if (*s).blockNo == 1 as libc::c_int {
        BZ2_bsInitWrite(s);
        bsPutUChar(s, 0x42 as libc::c_int as UChar);
        bsPutUChar(s, 0x5a as libc::c_int as UChar);
        bsPutUChar(s, 0x68 as libc::c_int as UChar);
        bsPutUChar(s, (0x30 as libc::c_int + (*s).blockSize100k) as UChar);
    }
    if (*s).nblock > 0 as libc::c_int {
        bsPutUChar(s, 0x31 as libc::c_int as UChar);
        bsPutUChar(s, 0x41 as libc::c_int as UChar);
        bsPutUChar(s, 0x59 as libc::c_int as UChar);
        bsPutUChar(s, 0x26 as libc::c_int as UChar);
        bsPutUChar(s, 0x53 as libc::c_int as UChar);
        bsPutUChar(s, 0x59 as libc::c_int as UChar);
        bsPutUInt32(s, (*s).blockCRC);
        bsW(s, 1 as libc::c_int, 0 as libc::c_int as UInt32);
        bsW(s, 24 as libc::c_int, (*s).origPtr as UInt32);
        generateMTFValues(s);
        sendMTFValues(s);
    }
    if is_last_block != 0 {
        bsPutUChar(s, 0x17 as libc::c_int as UChar);
        bsPutUChar(s, 0x72 as libc::c_int as UChar);
        bsPutUChar(s, 0x45 as libc::c_int as UChar);
        bsPutUChar(s, 0x38 as libc::c_int as UChar);
        bsPutUChar(s, 0x50 as libc::c_int as UChar);
        bsPutUChar(s, 0x90 as libc::c_int as UChar);
        bsPutUInt32(s, (*s).combinedCRC);
        if (*s).verbosity >= 2 as libc::c_int {
            fprintf(
                stderr,
                b"    final combined CRC = 0x%08x\n   \0" as *const u8 as *const libc::c_char,
                (*s).combinedCRC,
            );
        }
        bsFinishWrite(s);
    }
}

use crate::bzlib::BZ2_bz__AssertH__fail;
use ::libc;
pub type Bool = libc::c_uchar;
#[no_mangle]
pub unsafe extern "C" fn BZ2_hbMakeCodeLengths(
    len: *mut u8,
    freq: *mut i32,
    alphaSize: i32,
    maxLen: i32,
) {
    let mut nNodes: i32;
    let mut nHeap: i32;
    let mut n1: i32;
    let mut n2: i32;
    let mut i: i32;
    let mut j: i32;
    let mut k: i32;
    let mut tooLong: Bool;
    let mut heap: [i32; 260] = [0; 260];
    let mut weight: [i32; 516] = [0; 516];
    let mut parent: [i32; 516] = [0; 516];
    i = 0 as libc::c_int;
    while i < alphaSize {
        weight[(i + 1 as libc::c_int) as usize] = (if *freq.offset(i as isize) == 0 as libc::c_int {
            1 as libc::c_int
        } else {
            *freq.offset(i as isize)
        }) << 8 as libc::c_int;
        i += 1;
    }
    while 1 as libc::c_int as Bool != 0 {
        nNodes = alphaSize;
        nHeap = 0 as libc::c_int;
        heap[0 as libc::c_int as usize] = 0 as libc::c_int;
        weight[0 as libc::c_int as usize] = 0 as libc::c_int;
        parent[0 as libc::c_int as usize] = -2 as libc::c_int;
        i = 1 as libc::c_int;
        while i <= alphaSize {
            parent[i as usize] = -1 as libc::c_int;
            nHeap += 1;
            heap[nHeap as usize] = i;
            let mut zz: i32;
            let tmp: i32;
            zz = nHeap;
            tmp = heap[zz as usize];
            while weight[tmp as usize] < weight[heap[(zz >> 1 as libc::c_int) as usize] as usize] {
                heap[zz as usize] = heap[(zz >> 1 as libc::c_int) as usize];
                zz >>= 1 as libc::c_int;
            }
            heap[zz as usize] = tmp;
            i += 1;
        }
        if nHeap >= 258 as libc::c_int + 2 as libc::c_int {
            BZ2_bz__AssertH__fail(2001 as libc::c_int);
        }
        while nHeap > 1 as libc::c_int {
            n1 = heap[1 as libc::c_int as usize];
            heap[1 as libc::c_int as usize] = heap[nHeap as usize];
            nHeap -= 1;
            let mut zz_0: i32;
            let mut yy: i32;
            let tmp_0: i32;
            zz_0 = 1 as libc::c_int;
            tmp_0 = heap[zz_0 as usize];
            while 1 as libc::c_int as Bool != 0 {
                yy = zz_0 << 1 as libc::c_int;
                if yy > nHeap {
                    break;
                }
                if yy < nHeap
                    && weight[heap[(yy + 1 as libc::c_int) as usize] as usize]
                        < weight[heap[yy as usize] as usize]
                {
                    yy += 1;
                }
                if weight[tmp_0 as usize] < weight[heap[yy as usize] as usize] {
                    break;
                }
                heap[zz_0 as usize] = heap[yy as usize];
                zz_0 = yy;
            }
            heap[zz_0 as usize] = tmp_0;
            n2 = heap[1 as libc::c_int as usize];
            heap[1 as libc::c_int as usize] = heap[nHeap as usize];
            nHeap -= 1;
            let mut zz_1: i32;
            let mut yy_0: i32;
            let tmp_1: i32;
            zz_1 = 1 as libc::c_int;
            tmp_1 = heap[zz_1 as usize];
            while 1 as libc::c_int as Bool != 0 {
                yy_0 = zz_1 << 1 as libc::c_int;
                if yy_0 > nHeap {
                    break;
                }
                if yy_0 < nHeap
                    && weight[heap[(yy_0 + 1 as libc::c_int) as usize] as usize]
                        < weight[heap[yy_0 as usize] as usize]
                {
                    yy_0 += 1;
                }
                if weight[tmp_1 as usize] < weight[heap[yy_0 as usize] as usize] {
                    break;
                }
                heap[zz_1 as usize] = heap[yy_0 as usize];
                zz_1 = yy_0;
            }
            heap[zz_1 as usize] = tmp_1;
            nNodes += 1;
            parent[n2 as usize] = nNodes;
            parent[n1 as usize] = parent[n2 as usize];
            weight[nNodes as usize] = ((weight[n1 as usize] as libc::c_uint
                & 0xffffff00 as libc::c_uint)
                .wrapping_add(weight[n2 as usize] as libc::c_uint & 0xffffff00 as libc::c_uint)
                | (1 as libc::c_int
                    + (if weight[n1 as usize] & 0xff as libc::c_int
                        > weight[n2 as usize] & 0xff as libc::c_int
                    {
                        weight[n1 as usize] & 0xff as libc::c_int
                    } else {
                        weight[n2 as usize] & 0xff as libc::c_int
                    })) as libc::c_uint) as i32;
            parent[nNodes as usize] = -1 as libc::c_int;
            nHeap += 1;
            heap[nHeap as usize] = nNodes;
            let mut zz_2: i32;
            let tmp_2: i32;
            zz_2 = nHeap;
            tmp_2 = heap[zz_2 as usize];
            while weight[tmp_2 as usize]
                < weight[heap[(zz_2 >> 1 as libc::c_int) as usize] as usize]
            {
                heap[zz_2 as usize] = heap[(zz_2 >> 1 as libc::c_int) as usize];
                zz_2 >>= 1 as libc::c_int;
            }
            heap[zz_2 as usize] = tmp_2;
        }
        if nNodes >= 258 as libc::c_int * 2 as libc::c_int {
            BZ2_bz__AssertH__fail(2002 as libc::c_int);
        }
        tooLong = 0 as libc::c_int as Bool;
        i = 1 as libc::c_int;
        while i <= alphaSize {
            j = 0 as libc::c_int;
            k = i;
            while parent[k as usize] >= 0 as libc::c_int {
                k = parent[k as usize];
                j += 1;
            }
            *len.offset((i - 1 as libc::c_int) as isize) = j as u8;
            if j > maxLen {
                tooLong = 1 as libc::c_int as Bool;
            }
            i += 1;
        }
        if tooLong == 0 {
            break;
        }
        i = 1 as libc::c_int;
        while i <= alphaSize {
            j = weight[i as usize] >> 8 as libc::c_int;
            j = 1 as libc::c_int + j / 2 as libc::c_int;
            weight[i as usize] = j << 8 as libc::c_int;
            i += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn BZ2_hbAssignCodes(
    code: *mut i32,
    length: *mut u8,
    minLen: i32,
    maxLen: i32,
    alphaSize: i32,
) {
    let mut n: i32;
    let mut vec: i32;
    let mut i: i32;
    vec = 0 as libc::c_int;
    n = minLen;
    while n <= maxLen {
        i = 0 as libc::c_int;
        while i < alphaSize {
            if *length.offset(i as isize) as libc::c_int == n {
                *code.offset(i as isize) = vec;
                vec += 1;
            }
            i += 1;
        }
        vec <<= 1 as libc::c_int;
        n += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn BZ2_hbCreateDecodeTables(
    limit: *mut i32,
    base: *mut i32,
    perm: *mut i32,
    length: *mut u8,
    minLen: i32,
    maxLen: i32,
    alphaSize: i32,
) {
    let mut pp: i32;
    let mut i: i32;
    let mut j: i32;
    let mut vec: i32;
    pp = 0 as libc::c_int;
    i = minLen;
    while i <= maxLen {
        j = 0 as libc::c_int;
        while j < alphaSize {
            if *length.offset(j as isize) as libc::c_int == i {
                *perm.offset(pp as isize) = j;
                pp += 1;
            }
            j += 1;
        }
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < 23 as libc::c_int {
        *base.offset(i as isize) = 0 as libc::c_int;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < alphaSize {
        let fresh0 = &mut (*base
            .offset((*length.offset(i as isize) as libc::c_int + 1 as libc::c_int) as isize));
        *fresh0 += 1;
        i += 1;
    }
    i = 1 as libc::c_int;
    while i < 23 as libc::c_int {
        let fresh1 = &mut (*base.offset(i as isize));
        *fresh1 += *base.offset((i - 1 as libc::c_int) as isize);
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < 23 as libc::c_int {
        *limit.offset(i as isize) = 0 as libc::c_int;
        i += 1;
    }
    vec = 0 as libc::c_int;
    i = minLen;
    while i <= maxLen {
        vec += *base.offset((i + 1 as libc::c_int) as isize) - *base.offset(i as isize);
        *limit.offset(i as isize) = vec - 1 as libc::c_int;
        vec <<= 1 as libc::c_int;
        i += 1;
    }
    i = minLen + 1 as libc::c_int;
    while i <= maxLen {
        *base.offset(i as isize) = ((*limit.offset((i - 1 as libc::c_int) as isize)
            + 1 as libc::c_int)
            << 1 as libc::c_int)
            - *base.offset(i as isize);
        i += 1;
    }
}

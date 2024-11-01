use std::cmp::Ordering;

use crate::{
    assert_h,
    bzlib::{EState, BZ_N_OVERSHOOT, BZ_N_QSORT, BZ_N_RADIX, FTAB_LEN},
};

/// Fallback O(N log(N)^2) sorting algorithm, for repetitive blocks      
#[inline]
unsafe fn fallbackSimpleSort(fmap: *mut u32, eclass: *mut u32, lo: i32, hi: i32) {
    let mut j: i32;
    let mut tmp: i32;
    let mut ec_tmp: u32;

    if lo == hi {
        return;
    }

    if hi - lo > 3 {
        for i in (lo..=hi - 4).rev() {
            tmp = *fmap.offset(i as isize) as i32;
            ec_tmp = *eclass.offset(tmp as isize);
            j = i + 4;
            while j <= hi && ec_tmp > *eclass.offset(*fmap.offset(j as isize) as isize) {
                *fmap.offset((j - 4) as isize) = *fmap.offset(j as isize);
                j += 4;
            }
            *fmap.offset((j - 4) as isize) = tmp as u32;
        }
    }

    for i in (lo..=hi - 1).rev() {
        tmp = *fmap.offset(i as isize) as i32;
        ec_tmp = *eclass.offset(tmp as isize);
        j = i + 1;
        while j <= hi && ec_tmp > *eclass.offset(*fmap.offset(j as isize) as isize) {
            *fmap.offset((j - 1) as isize) = *fmap.offset(j as isize);
            j += 1;
        }
        *fmap.offset((j - 1) as isize) = tmp as u32;
    }
}

const FALLBACK_QSORT_SMALL_THRESH: i32 = 10;
const FALLBACK_QSORT_STACK_SIZE: usize = 100;

unsafe fn fallbackQSort3(fmap: &mut [u32], eclass: *mut u32, loSt: i32, hiSt: i32) {
    let mut unLo: i32;
    let mut unHi: i32;
    let mut ltLo: i32;
    let mut gtHi: i32;
    let mut n: i32;
    let mut m: i32;
    let mut sp: usize;
    let mut lo: i32;
    let mut hi: i32;
    let mut r3: u32;
    let mut stackLo: [i32; FALLBACK_QSORT_STACK_SIZE] = [0; FALLBACK_QSORT_STACK_SIZE];
    let mut stackHi: [i32; FALLBACK_QSORT_STACK_SIZE] = [0; FALLBACK_QSORT_STACK_SIZE];

    macro_rules! fpush {
        ($lz:expr, $hz:expr) => {
            stackLo[sp] = $lz;
            stackHi[sp] = $hz;
            sp += 1;
        };
    }

    macro_rules! fvswap {
        ($zzp1:expr, $zzp2:expr, $zzn:expr) => {
            let mut yyp1: i32 = $zzp1;
            let mut yyp2: i32 = $zzp2;
            let mut yyn: i32 = $zzn;

            while (yyn > 0) {
                fmap.swap(yyp1 as usize, yyp2 as usize);
                yyp1 += 1;
                yyp2 += 1;
                yyn -= 1;
            }
        };
    }

    let mut r = 0u32;

    sp = 0;
    fpush!(loSt, hiSt);

    while sp > 0 {
        assert_h!(sp < FALLBACK_QSORT_STACK_SIZE - 1, 1004);

        // the `fpop` macro has one occurence, so it was inlined here
        sp -= 1;
        lo = stackLo[sp as usize];
        hi = stackHi[sp as usize];

        if hi - lo < FALLBACK_QSORT_SMALL_THRESH {
            fallbackSimpleSort(fmap.as_mut_ptr(), eclass, lo, hi);
            continue;
        }

        /* Random partitioning.  Median of 3 sometimes fails to
            avoid bad cases.  Median of 9 seems to help but
            looks rather expensive.  This too seems to work but
            is cheaper.  Guidance for the magic constants
            7621 and 32768 is taken from Sedgewick's algorithms
            book, chapter 35.
        */
        r = r.wrapping_mul(7621).wrapping_add(1).wrapping_rem(32768);
        let med = match r.wrapping_rem(3) {
            0 => *eclass.offset(fmap[lo as usize] as isize),
            1 => *eclass.offset(fmap[((lo + hi) >> 1) as usize] as isize),
            _ => *eclass.offset(fmap[hi as usize] as isize),
        };

        ltLo = lo;
        unLo = lo;

        gtHi = hi;
        unHi = hi;

        loop {
            while unLo <= unHi {
                match (*eclass.offset(fmap[unLo as usize] as isize)).cmp(&med) {
                    Ordering::Greater => break,
                    Ordering::Equal => {
                        fmap.swap(unLo as usize, ltLo as usize);
                        ltLo += 1;
                        unLo += 1;
                    }
                    Ordering::Less => {
                        unLo += 1;
                    }
                }
            }

            while unLo <= unHi {
                match (*eclass.offset(fmap[unLo as usize] as isize)).cmp(&med) {
                    Ordering::Less => break,
                    Ordering::Equal => {
                        fmap.swap(unHi as usize, gtHi as usize);
                        gtHi -= 1;
                        unHi -= 1;
                    }
                    Ordering::Greater => {
                        unHi -= 1;
                    }
                }
            }

            if unLo > unHi {
                break;
            }

            fmap.swap(unLo as usize, unHi as usize);
            unLo += 1;
            unHi -= 1;
        }

        debug_assert_eq!(unHi, unLo - 1, "fallbackQSort3(2)");

        if gtHi < ltLo {
            continue;
        }

        n = Ord::min(ltLo - lo, unLo - ltLo);
        fvswap!(lo, unLo - n, n);
        m = Ord::min(hi - gtHi, gtHi - unHi);
        fvswap!(unLo, hi - m + 1, m);

        n = lo + unLo - ltLo - 1 as libc::c_int;
        m = hi - (gtHi - unHi) + 1 as libc::c_int;

        if n - lo > hi - m {
            fpush!(lo, n);
            fpush!(m, hi);
        } else {
            fpush!(m, hi);
            fpush!(lo, n);
        }
    }
}

unsafe fn fallbackSort(fmap: *mut u32, eclass: *mut u32, bhtab: *mut u32, nblock: i32, verb: i32) {
    let fmap = core::slice::from_raw_parts_mut(fmap, nblock as usize);
    // let eclass8 = core::slice::from_raw_parts_mut(eclass as *mut u8, 4 * (nblock + BZ_N_OVERSHOOT) as usize);

    // bzip2 appears to use uninitalized memory. It all works out in the end, but is UB.
    core::ptr::write_bytes(bhtab, 0, FTAB_LEN);
    let bhtab = bhtab.cast::<[u32; FTAB_LEN]>().as_mut().unwrap();

    macro_rules! SET_BH {
        ($zz:expr) => {
            bhtab[$zz as usize >> 5] |= 1 << ($zz & 31);
        };
    }

    macro_rules! CLEAR_BH {
        ($zz:expr) => {
            bhtab[$zz as usize >> 5] &= !(1 << ($zz & 31));
        };
    }

    macro_rules! ISSET_BH {
        ($zz:expr) => {
            bhtab[$zz as usize >> 5] & 1u32 << ($zz & 31) != 0
        };
    }

    macro_rules! UNALIGNED_BH {
        ($zz:expr) => {
            ($zz & 0x01f) != 0
        };
    }

    macro_rules! WORD_BH {
        ($zz:expr) => {
            bhtab[$zz as usize >> 5]
        };
    }

    let mut ftab: [i32; 257] = [0; 257];
    let mut ftabCopy: [i32; 256] = [0; 256];
    let mut H: i32;
    let mut i: i32;
    let mut j: i32;
    let mut k: i32;
    let mut l: i32;
    let mut r: i32;
    let mut cc: i32;
    let mut cc1: i32;
    let mut nNotDone: i32;
    let eclass8: *mut u8 = eclass as *mut u8;

    /*--
       Initial 1-char radix sort to generate
       initial fmap and initial BH bits.
    --*/
    if verb >= 4 {
        eprintln!("        bucket sorting ...");
    }

    for i in 0..nblock {
        ftab[*eclass8.offset(i as isize) as usize] += 1;
    }

    ftabCopy[0..256].copy_from_slice(&ftab[0..256]);

    for i in 1..257 {
        ftab[i] += ftab[i - 1];
    }

    for i in 0..nblock as usize {
        j = *eclass8.add(i) as i32;
        k = ftab[j as usize] - 1;
        ftab[j as usize] = k;
        fmap[k as usize] = i as u32;
    }

    bhtab[0..2 + nblock as usize / 32].fill(0);

    for i in 0..256 {
        SET_BH!(ftab[i]);
    }

    /*--
       Inductively refine the buckets.  Kind-of an
       "exponential radix sort" (!), inspired by the
       Manber-Myers suffix array construction algorithm.
    --*/

    /*-- set sentinel bits for block-end detection --*/
    for i in 0..32 {
        SET_BH!(nblock + 2 * i);
        CLEAR_BH!(nblock + 2 * i + 1);
    }

    /*-- the log(N) loop --*/
    H = 1;
    loop {
        if verb >= 4 {
            eprint!("        depth {:>6} has ", H);
        }
        j = 0;
        i = 0;
        while i < nblock {
            if ISSET_BH!(i) {
                j = i;
            }
            k = fmap[i as usize].wrapping_sub(H as libc::c_uint) as i32;
            if k < 0 {
                k += nblock;
            }
            *eclass.offset(k as isize) = j as u32;
            i += 1;
        }
        nNotDone = 0;
        r = -1;
        loop {
            /*-- find the next non-singleton bucket --*/
            k = r + 1;
            while ISSET_BH!(k) && UNALIGNED_BH!(k) {
                k += 1;
            }
            if ISSET_BH!(k) {
                while WORD_BH!(k) == 0xffffffff {
                    k += 32;
                }
                while ISSET_BH!(k) {
                    k += 1;
                }
            }
            l = k - 1;
            if l >= nblock {
                break;
            }
            while !ISSET_BH!(k) && UNALIGNED_BH!(k) {
                k += 1;
            }
            if !ISSET_BH!(k) {
                while WORD_BH!(k) == 0x00000000 {
                    k += 32;
                }
                while !ISSET_BH!(k) {
                    k += 1;
                }
            }
            r = k - 1;
            if r >= nblock {
                break;
            }

            /*-- now [l, r] bracket current bucket --*/
            if r > l {
                nNotDone += r - l + 1;
                fallbackQSort3(fmap, eclass, l, r);

                /*-- scan bucket and generate header bits-- */
                cc = -1;
                for i in l..=r {
                    cc1 = *eclass.offset(fmap[i as usize] as isize) as i32;
                    if cc != cc1 {
                        SET_BH!(i);
                        cc = cc1;
                    }
                }
            }
        }
        if verb >= 4 {
            eprintln!("{:>6} unresolved strings", nNotDone);
        }
        H *= 2;
        if H > nblock || nNotDone == 0 {
            break;
        }
    }

    if verb >= 4 {
        eprintln!("        reconstructing block ...");
    }

    let mut j = 0;
    for i in 0..nblock {
        while ftabCopy[j] == 0 {
            j += 1;
        }
        ftabCopy[j] -= 1;
        *eclass8.offset(fmap[i as usize] as isize) = j as u8;
    }

    assert_h!(j < 256, 1005);
}

#[inline]
unsafe fn mainGtU(
    mut i1: u32,
    mut i2: u32,
    block: *mut u8,
    quadrant: *mut u16,
    nblock: u32,
    budget: &mut i32,
) -> bool {
    let mut k: i32;
    let mut c1: u8;
    let mut c2: u8;
    let mut s1: u16;
    let mut s2: u16;

    debug_assert_ne!(i1, i2, "mainGtU");

    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    k = nblock.wrapping_add(8 as libc::c_int as libc::c_uint) as i32;
    loop {
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        if i1 >= nblock {
            i1 = (i1 as libc::c_uint).wrapping_sub(nblock) as u32 as u32;
        }
        if i2 >= nblock {
            i2 = (i2 as libc::c_uint).wrapping_sub(nblock) as u32 as u32;
        }
        k -= 8 as libc::c_int;
        *budget -= 1;
        if k < 0 as libc::c_int {
            break false;
        }
    }
}
static INCS: [i32; 14] = [
    1 as libc::c_int,
    4 as libc::c_int,
    13 as libc::c_int,
    40 as libc::c_int,
    121 as libc::c_int,
    364 as libc::c_int,
    1093 as libc::c_int,
    3280 as libc::c_int,
    9841 as libc::c_int,
    29524 as libc::c_int,
    88573 as libc::c_int,
    265720 as libc::c_int,
    797161 as libc::c_int,
    2391484 as libc::c_int,
];
unsafe fn mainSimpleSort(
    ptr: *mut u32,
    block: *mut u8,
    quadrant: *mut u16,
    nblock: i32,
    lo: i32,
    hi: i32,
    d: i32,
    budget: &mut i32,
) {
    let mut i: i32;
    let mut j: i32;
    let mut h: i32;
    let bigN: i32;
    let mut hp: i32;
    let mut v: u32;
    bigN = hi - lo + 1 as libc::c_int;
    if bigN < 2 as libc::c_int {
        return;
    }
    hp = 0 as libc::c_int;
    while INCS[hp as usize] < bigN {
        hp += 1;
    }
    hp -= 1;
    while hp >= 0 as libc::c_int {
        h = INCS[hp as usize];
        i = lo + h;
        loop {
            if i > hi {
                break;
            }
            v = *ptr.offset(i as isize);
            j = i;
            while mainGtU(
                (*ptr.offset((j - h) as isize)).wrapping_add(d as libc::c_uint),
                v.wrapping_add(d as libc::c_uint),
                block,
                quadrant,
                nblock as u32,
                budget,
            ) {
                *ptr.offset(j as isize) = *ptr.offset((j - h) as isize);
                j -= h;
                if j <= lo + h - 1 as libc::c_int {
                    break;
                }
            }
            *ptr.offset(j as isize) = v;
            i += 1;
            if i > hi {
                break;
            }
            v = *ptr.offset(i as isize);
            j = i;
            while mainGtU(
                (*ptr.offset((j - h) as isize)).wrapping_add(d as libc::c_uint),
                v.wrapping_add(d as libc::c_uint),
                block,
                quadrant,
                nblock as u32,
                budget,
            ) {
                *ptr.offset(j as isize) = *ptr.offset((j - h) as isize);
                j -= h;
                if j <= lo + h - 1 as libc::c_int {
                    break;
                }
            }
            *ptr.offset(j as isize) = v;
            i += 1;
            if i > hi {
                break;
            }
            v = *ptr.offset(i as isize);
            j = i;
            while mainGtU(
                (*ptr.offset((j - h) as isize)).wrapping_add(d as libc::c_uint),
                v.wrapping_add(d as libc::c_uint),
                block,
                quadrant,
                nblock as u32,
                budget,
            ) {
                *ptr.offset(j as isize) = *ptr.offset((j - h) as isize);
                j -= h;
                if j <= lo + h - 1 as libc::c_int {
                    break;
                }
            }
            *ptr.offset(j as isize) = v;
            i += 1;
            if *budget < 0 as libc::c_int {
                return;
            }
        }
        hp -= 1;
    }
}
#[inline]
unsafe fn mmed3(mut a: u8, mut b: u8, c: u8) -> u8 {
    let t: u8;
    if a as libc::c_int > b as libc::c_int {
        t = a;
        a = b;
        b = t;
    }
    if b as libc::c_int > c as libc::c_int {
        b = c;
        if a as libc::c_int > b as libc::c_int {
            b = a;
        }
    }
    b
}

const MAIN_QSORT_SMALL_THRESH: i32 = 20;
const MAIN_QSORT_DEPTH_THRESH: i32 = BZ_N_RADIX + BZ_N_QSORT;
const MAIN_QSORT_STACK_SIZE: i32 = 100;

unsafe fn mainQSort3(
    ptr: *mut u32,
    block: *mut u8,
    quadrant: *mut u16,
    nblock: i32,
    loSt: i32,
    hiSt: i32,
    dSt: i32,
    budget: &mut i32,
) {
    let mut unLo: i32;
    let mut unHi: i32;
    let mut ltLo: i32;
    let mut gtHi: i32;
    let mut n: i32;
    let mut m: i32;
    let mut med: i32;
    let mut sp: i32;
    let mut lo: i32;
    let mut hi: i32;
    let mut d: i32;
    let mut stackLo: [i32; 100] = [0; 100];
    let mut stackHi: [i32; 100] = [0; 100];
    let mut stackD: [i32; 100] = [0; 100];
    let mut nextLo: [i32; 3] = [0; 3];
    let mut nextHi: [i32; 3] = [0; 3];
    let mut nextD: [i32; 3] = [0; 3];
    sp = 0 as libc::c_int;
    stackLo[sp as usize] = loSt;
    stackHi[sp as usize] = hiSt;
    stackD[sp as usize] = dSt;
    sp += 1;
    while sp > 0 as libc::c_int {
        assert_h!(sp < MAIN_QSORT_STACK_SIZE - 2, 1001);

        sp -= 1;
        lo = stackLo[sp as usize];
        hi = stackHi[sp as usize];
        d = stackD[sp as usize];
        if hi - lo < 20 as libc::c_int || d > 2 as libc::c_int + 12 as libc::c_int {
            mainSimpleSort(ptr, block, quadrant, nblock, lo, hi, d, budget);
            if *budget < 0 as libc::c_int {
                return;
            }
        } else {
            med = mmed3(
                *block.offset((*ptr.offset(lo as isize)).wrapping_add(d as libc::c_uint) as isize),
                *block.offset((*ptr.offset(hi as isize)).wrapping_add(d as libc::c_uint) as isize),
                *block.offset(
                    (*ptr.offset(((lo + hi) >> 1 as libc::c_int) as isize))
                        .wrapping_add(d as libc::c_uint) as isize,
                ),
            ) as i32;
            ltLo = lo;
            unLo = ltLo;
            gtHi = hi;
            unHi = gtHi;
            loop {
                loop {
                    if unLo > unHi {
                        break;
                    }
                    n =
                        *block
                            .offset((*ptr.offset(unLo as isize)).wrapping_add(d as libc::c_uint)
                                as isize) as i32
                            - med;
                    if n == 0 as libc::c_int {
                        let zztmp: i32 = *ptr.offset(unLo as isize) as i32;
                        *ptr.offset(unLo as isize) = *ptr.offset(ltLo as isize);
                        *ptr.offset(ltLo as isize) = zztmp as u32;
                        ltLo += 1;
                        unLo += 1;
                    } else {
                        if n > 0 as libc::c_int {
                            break;
                        }
                        unLo += 1;
                    }
                }
                loop {
                    if unLo > unHi {
                        break;
                    }
                    n =
                        *block
                            .offset((*ptr.offset(unHi as isize)).wrapping_add(d as libc::c_uint)
                                as isize) as i32
                            - med;
                    if n == 0 as libc::c_int {
                        let zztmp_0: i32 = *ptr.offset(unHi as isize) as i32;
                        *ptr.offset(unHi as isize) = *ptr.offset(gtHi as isize);
                        *ptr.offset(gtHi as isize) = zztmp_0 as u32;
                        gtHi -= 1;
                        unHi -= 1;
                    } else {
                        if n < 0 as libc::c_int {
                            break;
                        }
                        unHi -= 1;
                    }
                }
                if unLo > unHi {
                    break;
                }
                let zztmp_1: i32 = *ptr.offset(unLo as isize) as i32;
                *ptr.offset(unLo as isize) = *ptr.offset(unHi as isize);
                *ptr.offset(unHi as isize) = zztmp_1 as u32;
                unLo += 1;
                unHi -= 1;
            }
            if gtHi < ltLo {
                stackLo[sp as usize] = lo;
                stackHi[sp as usize] = hi;
                stackD[sp as usize] = d + 1 as libc::c_int;
                sp += 1;
            } else {
                n = if ltLo - lo < unLo - ltLo {
                    ltLo - lo
                } else {
                    unLo - ltLo
                };
                let mut yyp1: i32 = lo;
                let mut yyp2: i32 = unLo - n;
                let mut yyn: i32 = n;
                while yyn > 0 as libc::c_int {
                    let zztmp_2: i32 = *ptr.offset(yyp1 as isize) as i32;
                    *ptr.offset(yyp1 as isize) = *ptr.offset(yyp2 as isize);
                    *ptr.offset(yyp2 as isize) = zztmp_2 as u32;
                    yyp1 += 1;
                    yyp2 += 1;
                    yyn -= 1;
                }
                m = if hi - gtHi < gtHi - unHi {
                    hi - gtHi
                } else {
                    gtHi - unHi
                };
                let mut yyp1_0: i32 = unLo;
                let mut yyp2_0: i32 = hi - m + 1 as libc::c_int;
                let mut yyn_0: i32 = m;
                while yyn_0 > 0 as libc::c_int {
                    let zztmp_3: i32 = *ptr.offset(yyp1_0 as isize) as i32;
                    *ptr.offset(yyp1_0 as isize) = *ptr.offset(yyp2_0 as isize);
                    *ptr.offset(yyp2_0 as isize) = zztmp_3 as u32;
                    yyp1_0 += 1;
                    yyp2_0 += 1;
                    yyn_0 -= 1;
                }
                n = lo + unLo - ltLo - 1 as libc::c_int;
                m = hi - (gtHi - unHi) + 1 as libc::c_int;
                nextLo[0 as libc::c_int as usize] = lo;
                nextHi[0 as libc::c_int as usize] = n;
                nextD[0 as libc::c_int as usize] = d;
                nextLo[1 as libc::c_int as usize] = m;
                nextHi[1 as libc::c_int as usize] = hi;
                nextD[1 as libc::c_int as usize] = d;
                nextLo[2 as libc::c_int as usize] = n + 1 as libc::c_int;
                nextHi[2 as libc::c_int as usize] = m - 1 as libc::c_int;
                nextD[2 as libc::c_int as usize] = d + 1 as libc::c_int;
                if nextHi[0 as libc::c_int as usize] - nextLo[0 as libc::c_int as usize]
                    < nextHi[1 as libc::c_int as usize] - nextLo[1 as libc::c_int as usize]
                {
                    let mut tz: i32;
                    tz = nextLo[0 as libc::c_int as usize];
                    nextLo[0 as libc::c_int as usize] = nextLo[1 as libc::c_int as usize];
                    nextLo[1 as libc::c_int as usize] = tz;
                    tz = nextHi[0 as libc::c_int as usize];
                    nextHi[0 as libc::c_int as usize] = nextHi[1 as libc::c_int as usize];
                    nextHi[1 as libc::c_int as usize] = tz;
                    tz = nextD[0 as libc::c_int as usize];
                    nextD[0 as libc::c_int as usize] = nextD[1 as libc::c_int as usize];
                    nextD[1 as libc::c_int as usize] = tz;
                }
                if nextHi[1 as libc::c_int as usize] - nextLo[1 as libc::c_int as usize]
                    < nextHi[2 as libc::c_int as usize] - nextLo[2 as libc::c_int as usize]
                {
                    let mut tz_0: i32;
                    tz_0 = nextLo[1 as libc::c_int as usize];
                    nextLo[1 as libc::c_int as usize] = nextLo[2 as libc::c_int as usize];
                    nextLo[2 as libc::c_int as usize] = tz_0;
                    tz_0 = nextHi[1 as libc::c_int as usize];
                    nextHi[1 as libc::c_int as usize] = nextHi[2 as libc::c_int as usize];
                    nextHi[2 as libc::c_int as usize] = tz_0;
                    tz_0 = nextD[1 as libc::c_int as usize];
                    nextD[1 as libc::c_int as usize] = nextD[2 as libc::c_int as usize];
                    nextD[2 as libc::c_int as usize] = tz_0;
                }
                if nextHi[0 as libc::c_int as usize] - nextLo[0 as libc::c_int as usize]
                    < nextHi[1 as libc::c_int as usize] - nextLo[1 as libc::c_int as usize]
                {
                    let mut tz_1: i32;
                    tz_1 = nextLo[0 as libc::c_int as usize];
                    nextLo[0 as libc::c_int as usize] = nextLo[1 as libc::c_int as usize];
                    nextLo[1 as libc::c_int as usize] = tz_1;
                    tz_1 = nextHi[0 as libc::c_int as usize];
                    nextHi[0 as libc::c_int as usize] = nextHi[1 as libc::c_int as usize];
                    nextHi[1 as libc::c_int as usize] = tz_1;
                    tz_1 = nextD[0 as libc::c_int as usize];
                    nextD[0 as libc::c_int as usize] = nextD[1 as libc::c_int as usize];
                    nextD[1 as libc::c_int as usize] = tz_1;
                }
                stackLo[sp as usize] = nextLo[0 as libc::c_int as usize];
                stackHi[sp as usize] = nextHi[0 as libc::c_int as usize];
                stackD[sp as usize] = nextD[0 as libc::c_int as usize];
                sp += 1;
                stackLo[sp as usize] = nextLo[1 as libc::c_int as usize];
                stackHi[sp as usize] = nextHi[1 as libc::c_int as usize];
                stackD[sp as usize] = nextD[1 as libc::c_int as usize];
                sp += 1;
                stackLo[sp as usize] = nextLo[2 as libc::c_int as usize];
                stackHi[sp as usize] = nextHi[2 as libc::c_int as usize];
                stackD[sp as usize] = nextD[2 as libc::c_int as usize];
                sp += 1;
            }
        }
    }
}
unsafe fn mainSort(
    ptr: *mut u32,
    block: *mut u8,
    quadrant: *mut u16,
    ftab: *mut u32,
    nblock: i32,
    verb: i32,
    budget: &mut i32,
) {
    let mut i: i32;
    let mut j: i32;
    let mut k: i32;
    let mut ss: i32;
    let mut sb: i32;
    let mut runningOrder: [i32; 256] = [0; 256];
    let mut bigDone: [bool; 256] = [false; 256];
    let mut copyStart: [i32; 256] = [0; 256];
    let mut copyEnd: [i32; 256] = [0; 256];
    let mut c1: u8;
    let mut numQSorted: i32;
    let mut s: u16;
    if verb >= 4 as libc::c_int {
        eprintln!("        main sort initialise ...");
    }
    i = 65536 as libc::c_int;
    while i >= 0 as libc::c_int {
        *ftab.offset(i as isize) = 0 as libc::c_int as u32;
        i -= 1;
    }
    j = (*block.offset(0 as libc::c_int as isize) as libc::c_int) << 8 as libc::c_int;
    i = nblock - 1 as libc::c_int;
    while i >= 3 as libc::c_int {
        *quadrant.offset(i as isize) = 0 as libc::c_int as u16;
        j = j >> 8 as libc::c_int
            | (*block.offset(i as isize) as u16 as libc::c_int) << 8 as libc::c_int;
        let fresh4 = &mut (*ftab.offset(j as isize));
        *fresh4 = (*fresh4).wrapping_add(1);
        *quadrant.offset((i - 1 as libc::c_int) as isize) = 0 as libc::c_int as u16;
        j = j >> 8 as libc::c_int
            | (*block.offset((i - 1 as libc::c_int) as isize) as u16 as libc::c_int)
                << 8 as libc::c_int;
        let fresh5 = &mut (*ftab.offset(j as isize));
        *fresh5 = (*fresh5).wrapping_add(1);
        *quadrant.offset((i - 2 as libc::c_int) as isize) = 0 as libc::c_int as u16;
        j = j >> 8 as libc::c_int
            | (*block.offset((i - 2 as libc::c_int) as isize) as u16 as libc::c_int)
                << 8 as libc::c_int;
        let fresh6 = &mut (*ftab.offset(j as isize));
        *fresh6 = (*fresh6).wrapping_add(1);
        *quadrant.offset((i - 3 as libc::c_int) as isize) = 0 as libc::c_int as u16;
        j = j >> 8 as libc::c_int
            | (*block.offset((i - 3 as libc::c_int) as isize) as u16 as libc::c_int)
                << 8 as libc::c_int;
        let fresh7 = &mut (*ftab.offset(j as isize));
        *fresh7 = (*fresh7).wrapping_add(1);
        i -= 4 as libc::c_int;
    }
    while i >= 0 as libc::c_int {
        *quadrant.offset(i as isize) = 0 as libc::c_int as u16;
        j = j >> 8 as libc::c_int
            | (*block.offset(i as isize) as u16 as libc::c_int) << 8 as libc::c_int;
        let fresh8 = &mut (*ftab.offset(j as isize));
        *fresh8 = (*fresh8).wrapping_add(1);
        i -= 1;
    }
    i = 0 as libc::c_int;
    while i < 2 as libc::c_int + 12 as libc::c_int + 18 as libc::c_int + 2 as libc::c_int {
        *block.offset((nblock + i) as isize) = *block.offset(i as isize);
        *quadrant.offset((nblock + i) as isize) = 0 as libc::c_int as u16;
        i += 1;
    }
    if verb >= 4 as libc::c_int {
        eprintln!("        bucket sorting ...");
    }
    i = 1 as libc::c_int;
    while i <= 65536 as libc::c_int {
        let fresh9 = &mut (*ftab.offset(i as isize));
        *fresh9 = (*fresh9 as libc::c_uint)
            .wrapping_add(*ftab.offset((i - 1 as libc::c_int) as isize)) as u32
            as u32;
        i += 1;
    }
    s = ((*block.offset(0 as libc::c_int as isize) as libc::c_int) << 8 as libc::c_int) as u16;
    i = nblock - 1 as libc::c_int;
    while i >= 3 as libc::c_int {
        s = (s as libc::c_int >> 8 as libc::c_int
            | (*block.offset(i as isize) as libc::c_int) << 8 as libc::c_int) as u16;
        j = (*ftab.offset(s as isize)).wrapping_sub(1 as libc::c_int as libc::c_uint) as i32;
        *ftab.offset(s as isize) = j as u32;
        *ptr.offset(j as isize) = i as u32;
        s = (s as libc::c_int >> 8 as libc::c_int
            | (*block.offset((i - 1 as libc::c_int) as isize) as libc::c_int) << 8 as libc::c_int)
            as u16;
        j = (*ftab.offset(s as isize)).wrapping_sub(1 as libc::c_int as libc::c_uint) as i32;
        *ftab.offset(s as isize) = j as u32;
        *ptr.offset(j as isize) = (i - 1 as libc::c_int) as u32;
        s = (s as libc::c_int >> 8 as libc::c_int
            | (*block.offset((i - 2 as libc::c_int) as isize) as libc::c_int) << 8 as libc::c_int)
            as u16;
        j = (*ftab.offset(s as isize)).wrapping_sub(1 as libc::c_int as libc::c_uint) as i32;
        *ftab.offset(s as isize) = j as u32;
        *ptr.offset(j as isize) = (i - 2 as libc::c_int) as u32;
        s = (s as libc::c_int >> 8 as libc::c_int
            | (*block.offset((i - 3 as libc::c_int) as isize) as libc::c_int) << 8 as libc::c_int)
            as u16;
        j = (*ftab.offset(s as isize)).wrapping_sub(1 as libc::c_int as libc::c_uint) as i32;
        *ftab.offset(s as isize) = j as u32;
        *ptr.offset(j as isize) = (i - 3 as libc::c_int) as u32;
        i -= 4 as libc::c_int;
    }
    while i >= 0 as libc::c_int {
        s = (s as libc::c_int >> 8 as libc::c_int
            | (*block.offset(i as isize) as libc::c_int) << 8 as libc::c_int) as u16;
        j = (*ftab.offset(s as isize)).wrapping_sub(1 as libc::c_int as libc::c_uint) as i32;
        *ftab.offset(s as isize) = j as u32;
        *ptr.offset(j as isize) = i as u32;
        i -= 1;
    }
    bigDone.fill(false);
    i = 0 as libc::c_int;
    while i <= 255 as libc::c_int {
        runningOrder[i as usize] = i;
        i += 1;
    }
    let mut vv: i32;
    let mut h: i32 = 1 as libc::c_int;
    loop {
        h = 3 as libc::c_int * h + 1 as libc::c_int;
        if h > 256 as libc::c_int {
            break;
        }
    }
    loop {
        h /= 3 as libc::c_int;
        i = h;
        while i <= 255 as libc::c_int {
            vv = runningOrder[i as usize];
            j = i;
            while (*ftab.offset(
                ((runningOrder[(j - h) as usize] + 1 as libc::c_int) << 8 as libc::c_int) as isize,
            ))
            .wrapping_sub(
                *ftab.offset((runningOrder[(j - h) as usize] << 8 as libc::c_int) as isize),
            ) > (*ftab.offset(((vv + 1 as libc::c_int) << 8 as libc::c_int) as isize))
                .wrapping_sub(*ftab.offset((vv << 8 as libc::c_int) as isize))
            {
                runningOrder[j as usize] = runningOrder[(j - h) as usize];
                j -= h;
                if j <= h - 1 as libc::c_int {
                    break;
                }
            }
            runningOrder[j as usize] = vv;
            i += 1;
        }
        if h == 1 as libc::c_int {
            break;
        }
    }
    numQSorted = 0 as libc::c_int;
    i = 0 as libc::c_int;
    while i <= 255 as libc::c_int {
        ss = runningOrder[i as usize];
        j = 0 as libc::c_int;
        while j <= 255 as libc::c_int {
            if j != ss {
                sb = (ss << 8 as libc::c_int) + j;
                if *ftab.offset(sb as isize)
                    & ((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint
                    == 0
                {
                    let lo: i32 = (*ftab.offset(sb as isize)
                        & !((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint)
                        as i32;
                    let hi: i32 = (*ftab.offset((sb + 1 as libc::c_int) as isize)
                        & !((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint)
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        as i32;
                    if hi > lo {
                        if verb >= 4 as libc::c_int {
                            eprintln!(
                                "        qsort [{:#x}, {:#x}]   done {}   this {}",
                                ss,
                                j,
                                numQSorted,
                                hi - lo + 1 as libc::c_int,
                            );
                        }
                        mainQSort3(
                            ptr,
                            block,
                            quadrant,
                            nblock,
                            lo,
                            hi,
                            2 as libc::c_int,
                            budget,
                        );
                        numQSorted += hi - lo + 1 as libc::c_int;
                        if *budget < 0 as libc::c_int {
                            return;
                        }
                    }
                }
                let fresh10 = &mut (*ftab.offset(sb as isize));
                *fresh10 |= ((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint;
            }
            j += 1;
        }
        assert_h!(!bigDone[ss as usize], 1006);
        j = 0 as libc::c_int;
        while j <= 255 as libc::c_int {
            copyStart[j as usize] = (*ftab.offset(((j << 8 as libc::c_int) + ss) as isize)
                & !((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint)
                as i32;
            copyEnd[j as usize] =
                (*ftab.offset(((j << 8 as libc::c_int) + ss + 1 as libc::c_int) as isize)
                    & !((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint)
                    .wrapping_sub(1 as libc::c_int as libc::c_uint) as i32;
            j += 1;
        }
        j = (*ftab.offset((ss << 8 as libc::c_int) as isize)
            & !((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint) as i32;
        while j < copyStart[ss as usize] {
            k = (*ptr.offset(j as isize)).wrapping_sub(1 as libc::c_int as libc::c_uint) as i32;
            if k < 0 as libc::c_int {
                k += nblock;
            }
            c1 = *block.offset(k as isize);
            if !bigDone[c1 as usize] {
                let fresh11 = copyStart[c1 as usize];
                copyStart[c1 as usize] += 1;
                *ptr.offset(fresh11 as isize) = k as u32;
            }
            j += 1;
        }
        j = (*ftab.offset(((ss + 1 as libc::c_int) << 8 as libc::c_int) as isize)
            & !((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint)
            .wrapping_sub(1 as libc::c_int as libc::c_uint) as i32;
        while j > copyEnd[ss as usize] {
            k = (*ptr.offset(j as isize)).wrapping_sub(1 as libc::c_int as libc::c_uint) as i32;
            if k < 0 as libc::c_int {
                k += nblock;
            }
            c1 = *block.offset(k as isize);
            if !bigDone[c1 as usize] {
                let fresh12 = copyEnd[c1 as usize];
                copyEnd[c1 as usize] -= 1;
                *ptr.offset(fresh12 as isize) = k as u32;
            }
            j -= 1;
        }
        assert_h!(
            (copyStart[ss as usize]-1 == copyEnd[ss as usize])
                ||
                /* Extremely rare case missing in bzip2-1.0.0 and 1.0.1.
                   Necessity for this case is demonstrated by compressing
                   a sequence of approximately 48.5 million of character
                   251; 1.0.0/1.0.1 will then die here. */
                (copyStart[ss as usize] == 0 && copyEnd[ss as usize] == nblock-1),
            1007
        );
        j = 0 as libc::c_int;
        while j <= 255 as libc::c_int {
            let fresh13 = &mut (*ftab.offset(((j << 8 as libc::c_int) + ss) as isize));
            *fresh13 |= ((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint;
            j += 1;
        }
        bigDone[ss as usize] = true;
        if i < 255 as libc::c_int {
            let bbStart: i32 = (*ftab.offset((ss << 8 as libc::c_int) as isize)
                & !((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint)
                as i32;
            let bbSize: i32 = (*ftab.offset(((ss + 1 as libc::c_int) << 8 as libc::c_int) as isize)
                & !((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint)
                .wrapping_sub(bbStart as libc::c_uint) as i32;
            let mut shifts: i32 = 0 as libc::c_int;
            while bbSize >> shifts > 65534 as libc::c_int {
                shifts += 1;
            }
            j = bbSize - 1 as libc::c_int;
            while j >= 0 as libc::c_int {
                let a2update: i32 = *ptr.offset((bbStart + j) as isize) as i32;
                let qVal: u16 = (j >> shifts) as u16;
                *quadrant.offset(a2update as isize) = qVal;
                if a2update
                    < 2 as libc::c_int + 12 as libc::c_int + 18 as libc::c_int + 2 as libc::c_int
                {
                    *quadrant.offset((a2update + nblock) as isize) = qVal;
                }
                j -= 1;
            }
            assert_h!(((bbSize - 1) >> shifts) <= 65535, 1002);
        }
        i += 1;
    }
    if verb >= 4 as libc::c_int {
        eprintln!(
            "        {} pointers, {} sorted, {} scanned",
            nblock,
            numQSorted,
            nblock - numQSorted,
        );
    }
}

/// Pre:
///    nblock > 0
///    arr2 exists for [0 .. nblock-1 +N_OVERSHOOT]
///    ((UChar*)arr2)  [0 .. nblock-1] holds block
///    arr1 exists for [0 .. nblock-1]
///
/// Post:
///    ((UChar*)arr2) [0 .. nblock-1] holds block
///    All other areas of block destroyed
///    ftab [ 0 .. 65536 ] destroyed
///    arr1 [0 .. nblock-1] holds sorted order
pub unsafe fn BZ2_blockSort(s: &mut EState) {
    let ptr: *mut u32 = s.ptr;
    let block: *mut u8 = s.block;
    let ftab: *mut u32 = s.ftab;
    let nblock: i32 = s.nblock;
    let verb: i32 = s.verbosity;
    let quadrant: *mut u16;
    let mut budget: i32;
    let budgetInit: i32;
    let mut i: i32;
    if nblock < 10000 {
        fallbackSort(s.arr1, s.arr2, ftab, nblock, verb);
    } else {
        /* Calculate the location for quadrant, remembering to get
           the alignment right.  Assumes that &(block[0]) is at least
           2-byte aligned -- this should be ok since block is really
           the first section of arr2.
        */
        i = nblock + BZ_N_OVERSHOOT;
        if i & 1 != 0 {
            i += 1;
        }
        quadrant = block.offset(i as isize) as *mut u8 as *mut u16;

        /* (wfact-1) / 3 puts the default-factor-30
           transition point at very roughly the same place as
           with v0.1 and v0.9.0.
           Not that it particularly matters any more, since the
           resulting compressed stream is now the same regardless
           of whether or not we use the main sort or fallback sort.
        */
        let wfact = s.workFactor.clamp(1, 100);
        budgetInit = nblock * ((wfact - 1) / 3);
        budget = budgetInit;

        mainSort(ptr, block, quadrant, ftab, nblock, verb, &mut budget);

        if verb >= 3 {
            eprintln!(
                "      {} work, {} block, ratio {:5.2}",
                budgetInit - budget,
                nblock,
                ((budgetInit - budget) as libc::c_float
                    / (if nblock == 0 { 1 } else { nblock }) as libc::c_float)
                    as libc::c_double,
            );
        }

        if budget < 0 as libc::c_int {
            if verb >= 2 as libc::c_int {
                eprintln!("    too repetitive; using fallback sorting algorithm");
            }
            fallbackSort(s.arr1, s.arr2, ftab, nblock, verb);
        }
    }

    s.origPtr = -1 as libc::c_int;
    for i in 0..s.nblock {
        if *ptr.offset(i as isize) == 0 {
            s.origPtr = i;
            break;
        }
    }

    assert_h!(s.origPtr != -1, 1003);
}

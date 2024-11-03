use std::cmp::Ordering;

use crate::{
    assert_h,
    bzlib::{Arr2, EState, BZ_N_OVERSHOOT, BZ_N_QSORT, BZ_N_RADIX, FTAB_LEN},
};

/// Fallback O(N log(N)^2) sorting algorithm, for repetitive blocks      
#[inline]
fn fallbackSimpleSort(fmap: &mut [u32], eclass: &[u32], lo: i32, hi: i32) {
    let mut j: i32;
    let mut tmp: i32;
    let mut ec_tmp: u32;

    if lo == hi {
        return;
    }

    if hi - lo > 3 {
        for i in (lo..=hi - 4).rev() {
            tmp = fmap[i as usize] as i32;
            ec_tmp = eclass[tmp as usize];
            j = i + 4;
            while j <= hi && ec_tmp > eclass[fmap[j as usize] as usize] {
                fmap[(j - 4) as usize] = fmap[j as usize];
                j += 4;
            }
            fmap[(j - 4) as usize] = tmp as u32;
        }
    }

    for i in (lo..=hi - 1).rev() {
        tmp = fmap[i as usize] as i32;
        ec_tmp = eclass[tmp as usize];
        j = i + 1;
        while j <= hi && ec_tmp > eclass[fmap[j as usize] as usize] {
            fmap[(j - 1) as usize] = fmap[j as usize];
            j += 1;
        }
        fmap[(j - 1) as usize] = tmp as u32;
    }
}

const FALLBACK_QSORT_SMALL_THRESH: i32 = 10;
const FALLBACK_QSORT_STACK_SIZE: usize = 100;

fn fallbackQSort3(fmap: &mut [u32], eclass: &[u32], loSt: i32, hiSt: i32) {
    let mut unLo: i32;
    let mut unHi: i32;
    let mut ltLo: i32;
    let mut gtHi: i32;
    let mut n: i32;
    let mut m: i32;
    let mut sp: usize;
    let mut lo: i32;
    let mut hi: i32;
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
            fallbackSimpleSort(fmap, eclass, lo, hi);
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
        let index = match r.wrapping_rem(3) {
            0 => fmap[lo as usize],
            1 => fmap[((lo + hi) >> 1) as usize],
            _ => fmap[hi as usize],
        };
        let med = eclass[index as usize];

        ltLo = lo;
        unLo = lo;

        gtHi = hi;
        unHi = hi;

        loop {
            while unLo <= unHi {
                match eclass[fmap[unLo as usize] as usize].cmp(&med) {
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
                match eclass[fmap[unLo as usize] as usize].cmp(&med) {
                    Ordering::Greater => break,
                    Ordering::Equal => {
                        fmap.swap(unHi as usize, gtHi as usize);
                        gtHi -= 1;
                        unHi -= 1;
                    }
                    Ordering::Less => {
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

        n = lo + unLo - ltLo - 1;
        m = hi - (gtHi - unHi) + 1;

        if n - lo > hi - m {
            fpush!(lo, n);
            fpush!(m, hi);
        } else {
            fpush!(m, hi);
            fpush!(lo, n);
        }
    }
}

fn fallbackSort(
    fmap: &mut [u32],
    eclass: &mut [u32],
    bhtab: &mut [u32; FTAB_LEN],
    nblock: i32,
    verb: i32,
) {
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

    /*--
       Initial 1-char radix sort to generate
       initial fmap and initial BH bits.
    --*/
    if verb >= 4 {
        eprintln!("        bucket sorting ...");
    }

    fn to_eclass8(slice: &mut [u32]) -> &mut [u8] {
        // Safety: we're using a shorter piece of this slice with a type of a lower alignment and
        // the same initialization and validity behavior
        unsafe { core::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, slice.len()) }
    }

    {
        let eclass8 = &*to_eclass8(eclass);

        for e in eclass8.iter() {
            ftab[usize::from(*e)] += 1;
        }

        ftabCopy[0..256].copy_from_slice(&ftab[0..256]);

        for i in 1..257 {
            ftab[i] += ftab[i - 1];
        }

        for (i, e) in eclass8.iter().enumerate() {
            let j = usize::from(*e);
            k = ftab[j] - 1;
            ftab[j] = k;
            fmap[k as usize] = i as u32;
        }
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
            eclass[k as usize] = j as u32;
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
                    cc1 = eclass[fmap[i as usize] as usize] as i32;
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

    {
        let eclass8 = to_eclass8(eclass);

        let mut j = 0;
        for i in 0..nblock {
            while ftabCopy[j] == 0 {
                j += 1;
            }
            ftabCopy[j] -= 1;
            eclass8[fmap[i as usize] as usize] = j as u8;
        }
    }

    assert_h!(j < 256, 1005);
}

#[inline]
fn mainGtU(
    mut i1: u32,
    mut i2: u32,
    block: &[u8],
    quadrant: &mut [u16],
    nblock: u32,
    budget: &mut i32,
) -> bool {
    let mut k: i32;
    let mut c1: u8;
    let mut c2: u8;
    let mut s1: u16;
    let mut s2: u16;

    debug_assert_ne!(i1, i2, "mainGtU");

    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = block[i1 as usize];
    c2 = block[i2 as usize];
    if c1 != c2 {
        return c1 > c2;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    k = nblock.wrapping_add(8 as libc::c_int as libc::c_uint) as i32;
    loop {
        c1 = block[i1 as usize];
        c2 = block[i2 as usize];
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = quadrant[i1 as usize];
        s2 = quadrant[i2 as usize];
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = block[i1 as usize];
        c2 = block[i2 as usize];
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = quadrant[i1 as usize];
        s2 = quadrant[i2 as usize];
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = block[i1 as usize];
        c2 = block[i2 as usize];
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = quadrant[i1 as usize];
        s2 = quadrant[i2 as usize];
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = block[i1 as usize];
        c2 = block[i2 as usize];
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = quadrant[i1 as usize];
        s2 = quadrant[i2 as usize];
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = block[i1 as usize];
        c2 = block[i2 as usize];
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = quadrant[i1 as usize];
        s2 = quadrant[i2 as usize];
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = block[i1 as usize];
        c2 = block[i2 as usize];
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = quadrant[i1 as usize];
        s2 = quadrant[i2 as usize];
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = block[i1 as usize];
        c2 = block[i2 as usize];
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = quadrant[i1 as usize];
        s2 = quadrant[i2 as usize];
        if s1 != s2 {
            return s1 > s2;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = block[i1 as usize];
        c2 = block[i2 as usize];
        if c1 != c2 {
            return c1 > c2;
        }
        s1 = quadrant[i1 as usize];
        s2 = quadrant[i2 as usize];
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
fn mainSimpleSort(
    ptr: &mut [u32],
    block: &[u8],
    quadrant: &mut [u16],
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
            v = ptr[i as usize];
            j = i;
            while mainGtU(
                (ptr[(j - h) as usize]).wrapping_add(d as libc::c_uint),
                v.wrapping_add(d as libc::c_uint),
                block,
                quadrant,
                nblock as u32,
                budget,
            ) {
                ptr[j as usize] = ptr[(j - h) as usize];
                j -= h;
                if j <= lo + h - 1 as libc::c_int {
                    break;
                }
            }
            ptr[j as usize] = v;
            i += 1;
            if i > hi {
                break;
            }
            v = ptr[i as usize];
            j = i;
            while mainGtU(
                (ptr[(j - h) as usize]).wrapping_add(d as libc::c_uint),
                v.wrapping_add(d as libc::c_uint),
                block,
                quadrant,
                nblock as u32,
                budget,
            ) {
                ptr[j as usize] = ptr[(j - h) as usize];
                j -= h;
                if j <= lo + h - 1 as libc::c_int {
                    break;
                }
            }
            ptr[j as usize] = v;
            i += 1;
            if i > hi {
                break;
            }
            v = ptr[i as usize];
            j = i;
            while mainGtU(
                (ptr[(j - h) as usize]).wrapping_add(d as libc::c_uint),
                v.wrapping_add(d as libc::c_uint),
                block,
                quadrant,
                nblock as u32,
                budget,
            ) {
                ptr[j as usize] = ptr[(j - h) as usize];
                j -= h;
                if j <= lo + h - 1 as libc::c_int {
                    break;
                }
            }
            ptr[j as usize] = v;
            i += 1;
            if *budget < 0 as libc::c_int {
                return;
            }
        }
        hp -= 1;
    }
}

#[inline]
fn mmed3(mut a: u8, mut b: u8, c: u8) -> u8 {
    let t: u8;
    if a > b {
        t = a;
        a = b;
        b = t;
    }
    if b > c {
        b = c;
        if a > b {
            b = a;
        }
    }
    b
}

const MAIN_QSORT_SMALL_THRESH: i32 = 20;
const MAIN_QSORT_DEPTH_THRESH: i32 = BZ_N_RADIX + BZ_N_QSORT;
const MAIN_QSORT_STACK_SIZE: i32 = 100;

fn mainQSort3(
    ptr: &mut [u32],
    block: &[u8],
    quadrant: &mut [u16],
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
        if hi - lo < MAIN_QSORT_SMALL_THRESH || d > MAIN_QSORT_DEPTH_THRESH {
            mainSimpleSort(ptr, block, quadrant, nblock, lo, hi, d, budget);
            if *budget < 0 as libc::c_int {
                return;
            }
        } else {
            med = mmed3(
                block[(ptr[lo as usize]).wrapping_add(d as libc::c_uint) as usize],
                block[(ptr[hi as usize]).wrapping_add(d as libc::c_uint) as usize],
                block[((ptr[((lo + hi) >> 1 as libc::c_int) as usize])
                    .wrapping_add(d as libc::c_uint) as isize) as usize],
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
                    n = block[(ptr[unLo as usize]).wrapping_add(d as libc::c_uint) as usize] as i32
                        - med;
                    if n == 0 as libc::c_int {
                        let zztmp: i32 = ptr[unLo as usize] as i32;
                        ptr[unLo as usize] = ptr[ltLo as usize];
                        ptr[ltLo as usize] = zztmp as u32;
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
                    n = block[(ptr[unHi as usize]).wrapping_add(d as libc::c_uint) as usize] as i32
                        - med;
                    if n == 0 as libc::c_int {
                        let zztmp_0: i32 = ptr[unHi as usize] as i32;
                        ptr[unHi as usize] = ptr[gtHi as usize];
                        ptr[gtHi as usize] = zztmp_0 as u32;
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
                let zztmp_1: i32 = ptr[unLo as usize] as i32;
                ptr[unLo as usize] = ptr[unHi as usize];
                ptr[unHi as usize] = zztmp_1 as u32;
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
                    let zztmp_2: i32 = ptr[yyp1 as usize] as i32;
                    ptr[yyp1 as usize] = ptr[yyp2 as usize];
                    ptr[yyp2 as usize] = zztmp_2 as u32;
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
                    let zztmp_3: i32 = ptr[yyp1_0 as usize] as i32;
                    ptr[yyp1_0 as usize] = ptr[yyp2_0 as usize];
                    ptr[yyp2_0 as usize] = zztmp_3 as u32;
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
fn mainSort(
    ptr: &mut [u32],
    block: &mut [u8],
    quadrant: &mut [u16],
    ftab: &mut [u32; FTAB_LEN],
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

    // NOTE: the `ftab` has already been cleared in `BZ2_blockSort`.

    j = (block[0] as i32) << 8;
    i = nblock - 1 as libc::c_int;
    while i >= 3 as libc::c_int {
        j = j >> 8 | (block[i as usize] as u16 as libc::c_int) << 8;
        ftab[j as usize] += 1;

        j = j >> 8 | (block[(i - 1 as libc::c_int) as usize] as u16 as libc::c_int) << 8;
        ftab[j as usize] += 1;

        j = j >> 8 | (block[(i - 2 as libc::c_int) as usize] as u16 as libc::c_int) << 8;
        ftab[j as usize] += 1;

        j = j >> 8 | (block[(i - 3 as libc::c_int) as usize] as u16 as libc::c_int) << 8;
        ftab[j as usize] += 1;

        i -= 4;
    }

    while i >= 0 as libc::c_int {
        j = j >> 8 as libc::c_int | (block[i as usize] as u16 as libc::c_int) << 8 as libc::c_int;
        ftab[j as usize] += 1;
        i -= 1;
    }

    for i in 0..BZ_N_OVERSHOOT {
        block[nblock as usize + i] = block[i];
    }

    if verb >= 4 as libc::c_int {
        eprintln!("        bucket sorting ...");
    }

    /*-- Complete the initial radix sort --*/
    for i in 1..=65536 {
        ftab[i] += ftab[i - 1];
    }

    s = ((block[0 as libc::c_int as usize] as libc::c_int) << 8 as libc::c_int) as u16;
    i = nblock - 1 as libc::c_int;
    while i >= 3 as libc::c_int {
        s = (s as libc::c_int >> 8 | (block[i as usize] as libc::c_int) << 8) as u16;
        j = ftab[usize::from(s)] as i32 - 1;
        ftab[usize::from(s)] = j as u32;
        ptr[j as usize] = i as u32;

        s = (s as libc::c_int >> 8 | (block[(i - 1 as libc::c_int) as usize] as libc::c_int) << 8)
            as u16;
        j = ftab[usize::from(s)] as i32 - 1;
        ftab[usize::from(s)] = j as u32;
        ptr[j as usize] = i as u32 - 1;

        s = (s as libc::c_int >> 8 | (block[(i - 2 as libc::c_int) as usize] as libc::c_int) << 8)
            as u16;
        j = ftab[usize::from(s)] as i32 - 1;
        ftab[usize::from(s)] = j as u32;
        ptr[j as usize] = i as u32 - 2;

        s = (s as libc::c_int >> 8 | (block[(i - 3 as libc::c_int) as usize] as libc::c_int) << 8)
            as u16;
        j = ftab[usize::from(s)] as i32 - 1;
        ftab[usize::from(s)] = j as u32;
        ptr[j as usize] = i as u32 - 3;

        i -= 4 as libc::c_int;
    }

    while i >= 0 as libc::c_int {
        s = (s as libc::c_int >> 8 as libc::c_int
            | (block[i as usize] as libc::c_int) << 8 as libc::c_int) as u16;
        j = ftab[usize::from(s)] as i32 - 1;
        ftab[usize::from(s)] = j as u32;
        ptr[j as usize] = i as u32;

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

    macro_rules! BIGFREQ {
        ($b:expr) => {
            ftab[(($b) + 1) << 8] - ftab[($b) << 8]
        };
    }

    loop {
        h /= 3 as libc::c_int;
        i = h;
        while i <= 255 as libc::c_int {
            vv = runningOrder[i as usize];
            j = i;
            while BIGFREQ!(runningOrder[(j - h) as usize] as usize) > BIGFREQ!(vv as usize) {
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

    /*--
       The main sorting loop.
    --*/

    numQSorted = 0 as libc::c_int;

    for i in 0..=255 {
        /*--
           Process big buckets, starting with the least full.
           Basically this is a 3-step process in which we call
           mainQSort3 to sort the small buckets [ss, j], but
           also make a big effort to avoid the calls if we can.
        --*/
        ss = runningOrder[i as usize];

        const SETMASK: u32 = 1 << 21;
        const CLEARMASK: u32 = !SETMASK;

        /*--
           Step 1:
           Complete the big bucket [ss] by quicksorting
           any unsorted small buckets [ss, j], for j != ss.
           Hopefully previous pointer-scanning phases have already
           completed many of the small buckets [ss, j], so
           we don't have to sort them at all.
        --*/
        for j in 0..=255 {
            if j != ss {
                sb = (ss << 8 as libc::c_int) + j;
                if (!(ftab[sb as usize] & SETMASK)) != 0 {
                    let lo: i32 = (ftab[sb as usize] & CLEARMASK) as i32;
                    let hi: i32 = ((ftab[sb as usize + 1] & CLEARMASK) - 1) as i32;

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
                ftab[sb as usize] |= SETMASK;
            }
        }
        assert_h!(!bigDone[ss as usize], 1006);

        /*--
           Step 2:
           Now scan this big bucket [ss] so as to synthesise the
           sorted order for small buckets [t, ss] for all t,
           including, magically, the bucket [ss,ss] too.
           This will avoid doing Real Work in subsequent Step 1's.
        --*/
        {
            for j in 0..=255 {
                copyStart[j] = (ftab[(j << 8) + ss as usize] & CLEARMASK) as i32;
                copyEnd[j] = (ftab[(j << 8) + ss as usize + 1] & CLEARMASK) as i32 - 1;
            }

            j = (ftab[(ss as usize) << 8] & CLEARMASK) as i32;
            while j < copyStart[ss as usize] {
                k = (ptr[j as usize]).wrapping_sub(1) as i32;
                if k < 0 as libc::c_int {
                    k += nblock;
                }
                c1 = block[k as usize];
                if !bigDone[c1 as usize] {
                    let fresh11 = copyStart[c1 as usize];
                    copyStart[c1 as usize] += 1;
                    ptr[fresh11 as usize] = k as u32;
                }
                j += 1;
            }

            j = (ftab[(ss as usize + 1) << 8] & CLEARMASK) as i32 - 1;
            while j > copyEnd[ss as usize] {
                k = (ptr[j as usize]).wrapping_sub(1) as i32;
                if k < 0 as libc::c_int {
                    k += nblock;
                }
                c1 = block[k as usize];
                if !bigDone[c1 as usize] {
                    let fresh12 = copyEnd[c1 as usize];
                    copyEnd[c1 as usize] -= 1;
                    ptr[fresh12 as usize] = k as u32;
                }
                j -= 1;
            }
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

        for j in 0..=255 {
            ftab[(j << 8) + ss as usize] |= SETMASK
        }

        /*--
           Step 3:
           The [ss] big bucket is now done.  Record this fact,
           and update the quadrant descriptors.  Remember to
           update quadrants in the overshoot area too, if
           necessary.  The "if (i < 255)" test merely skips
           this updating for the last bucket processed, since
           updating for the last bucket is pointless.

           The quadrant array provides a way to incrementally
           cache sort orderings, as they appear, so as to
           make subsequent comparisons in fullGtU() complete
           faster.  For repetitive blocks this makes a big
           difference (but not big enough to be able to avoid
           the fallback sorting mechanism, exponential radix sort).

           The precise meaning is: at all times:

              for 0 <= i < nblock and 0 <= j <= nblock

              if block[i] != block[j],

                 then the relative values of quadrant[i] and
                      quadrant[j] are meaningless.

                 else {
                    if quadrant[i] < quadrant[j]
                       then the string starting at i lexicographically
                       precedes the string starting at j

                    else if quadrant[i] > quadrant[j]
                       then the string starting at j lexicographically
                       precedes the string starting at i

                    else
                       the relative ordering of the strings starting
                       at i and j has not yet been determined.
                 }
        --*/
        bigDone[ss as usize] = true;

        if i < 255 as libc::c_int {
            let bbStart: i32 = (ftab[(ss as usize) << 8] & CLEARMASK) as i32;
            let bbSize: i32 = (ftab[(ss as usize + 1) << 8] & CLEARMASK) as i32 - bbStart;
            let mut shifts: i32 = 0 as libc::c_int;

            while bbSize >> shifts > 65534 as libc::c_int {
                shifts += 1;
            }

            j = bbSize - 1 as libc::c_int;
            while j >= 0 as libc::c_int {
                let a2update: i32 = ptr[(bbStart + j) as usize] as i32;
                let qVal: u16 = (j >> shifts) as u16;
                quadrant[a2update as usize] = qVal;
                if (a2update as usize) < BZ_N_OVERSHOOT {
                    quadrant[(a2update + nblock) as usize] = qVal;
                }
                j -= 1;
            }

            assert_h!(((bbSize - 1) >> shifts) <= 65535, 1002);
        }
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
    let nblock = usize::try_from(s.nblock).unwrap();

    let ptr = s.arr1.ptr();

    // bzip2 appears to use uninitalized memory. It all works out in the end, but is UB.
    core::ptr::write_bytes(s.ftab, 0, FTAB_LEN);
    let ftab = s.ftab.cast::<[u32; FTAB_LEN]>().as_mut().unwrap();

    BZ2_blockSortHelp(ptr, &mut s.arr2, ftab, nblock, s.workFactor, s.verbosity);

    s.origPtr = -1 as libc::c_int;
    for i in 0..s.nblock {
        if ptr[i as usize] == 0 {
            s.origPtr = i;
            break;
        }
    }

    assert_h!(s.origPtr != -1, 1003);
}

unsafe fn BZ2_blockSortHelp(
    ptr: &mut [u32],
    arr2: &mut Arr2,
    ftab: &mut [u32; FTAB_LEN],
    nblock: usize,
    workFactor: i32,
    verbosity: i32,
) {
    if nblock < 10000 {
        let eclass = &mut arr2.arr2()[..nblock as usize];
        fallbackSort(ptr, eclass, ftab, nblock as i32, verbosity);
    } else {
        let (block, quadrant) = arr2.block_and_quadrant(nblock as usize);

        /* (wfact-1) / 3 puts the default-factor-30
           transition point at very roughly the same place as
           with v0.1 and v0.9.0.
           Not that it particularly matters any more, since the
           resulting compressed stream is now the same regardless
           of whether or not we use the main sort or fallback sort.
        */
        let wfact = workFactor.clamp(1, 100);
        let budgetInit = nblock as i32 * ((wfact - 1) / 3);
        let mut budget = budgetInit;

        mainSort(
            ptr,
            block,
            quadrant,
            ftab,
            nblock as i32,
            verbosity,
            &mut budget,
        );

        if verbosity >= 3 {
            eprintln!(
                "      {} work, {} block, ratio {:5.2}",
                budgetInit - budget,
                nblock,
                ((budgetInit - budget) as libc::c_float
                    / (if nblock == 0 { 1 } else { nblock }) as libc::c_float)
                    as libc::c_double,
            );
        }

        if budget < 0 {
            if verbosity >= 2 as libc::c_int {
                eprintln!("    too repetitive; using fallback sorting algorithm");
            }

            let eclass = &mut arr2.arr2()[..nblock as usize];
            fallbackSort(ptr, eclass, ftab, nblock as i32, verbosity);
        }
    }
}

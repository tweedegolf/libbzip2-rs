use crate::bzlib::{BZ2_bz__AssertH__fail, Bool, EState};

#[inline]
unsafe fn fallbackSimpleSort(fmap: *mut u32, eclass: *mut u32, lo: i32, hi: i32) {
    let mut i: i32;
    let mut j: i32;
    let mut tmp: i32;
    let mut ec_tmp: u32;
    if lo == hi {
        return;
    }
    if hi - lo > 3 as libc::c_int {
        i = hi - 4 as libc::c_int;
        while i >= lo {
            tmp = *fmap.offset(i as isize) as i32;
            ec_tmp = *eclass.offset(tmp as isize);
            j = i + 4 as libc::c_int;
            while j <= hi && ec_tmp > *eclass.offset(*fmap.offset(j as isize) as isize) {
                *fmap.offset((j - 4 as libc::c_int) as isize) = *fmap.offset(j as isize);
                j += 4 as libc::c_int;
            }
            *fmap.offset((j - 4 as libc::c_int) as isize) = tmp as u32;
            i -= 1;
        }
    }
    i = hi - 1 as libc::c_int;
    while i >= lo {
        tmp = *fmap.offset(i as isize) as i32;
        ec_tmp = *eclass.offset(tmp as isize);
        j = i + 1 as libc::c_int;
        while j <= hi && ec_tmp > *eclass.offset(*fmap.offset(j as isize) as isize) {
            *fmap.offset((j - 1 as libc::c_int) as isize) = *fmap.offset(j as isize);
            j += 1;
        }
        *fmap.offset((j - 1 as libc::c_int) as isize) = tmp as u32;
        i -= 1;
    }
}
unsafe fn fallbackQSort3(fmap: *mut u32, eclass: *mut u32, loSt: i32, hiSt: i32) {
    let mut unLo: i32;
    let mut unHi: i32;
    let mut ltLo: i32;
    let mut gtHi: i32;
    let mut n: i32;
    let mut m: i32;
    let mut sp: i32;
    let mut lo: i32;
    let mut hi: i32;
    let mut med: u32;
    let mut r: u32;
    let mut r3: u32;
    let mut stackLo: [i32; 100] = [0; 100];
    let mut stackHi: [i32; 100] = [0; 100];
    r = 0 as libc::c_int as u32;
    sp = 0 as libc::c_int;
    stackLo[sp as usize] = loSt;
    stackHi[sp as usize] = hiSt;
    sp += 1;
    while sp > 0 as libc::c_int {
        if sp >= 100 as libc::c_int - 1 as libc::c_int {
            BZ2_bz__AssertH__fail(1004 as libc::c_int);
        }
        sp -= 1;
        lo = stackLo[sp as usize];
        hi = stackHi[sp as usize];
        if hi - lo < 10 as libc::c_int {
            fallbackSimpleSort(fmap, eclass, lo, hi);
        } else {
            r = r
                .wrapping_mul(7621 as libc::c_int as libc::c_uint)
                .wrapping_add(1 as libc::c_int as libc::c_uint)
                .wrapping_rem(32768 as libc::c_int as libc::c_uint);
            r3 = r.wrapping_rem(3 as libc::c_int as libc::c_uint);
            if r3 == 0 as libc::c_int as libc::c_uint {
                med = *eclass.offset(*fmap.offset(lo as isize) as isize);
            } else if r3 == 1 as libc::c_int as libc::c_uint {
                med =
                    *eclass.offset(*fmap.offset(((lo + hi) >> 1 as libc::c_int) as isize) as isize);
            } else {
                med = *eclass.offset(*fmap.offset(hi as isize) as isize);
            }
            ltLo = lo;
            unLo = ltLo;
            gtHi = hi;
            unHi = gtHi;
            loop {
                while unLo <= unHi {
                    n = *eclass.offset(*fmap.offset(unLo as isize) as isize) as i32 - med as i32;
                    if n == 0 as libc::c_int {
                        let zztmp: i32 = *fmap.offset(unLo as isize) as i32;
                        *fmap.offset(unLo as isize) = *fmap.offset(ltLo as isize);
                        *fmap.offset(ltLo as isize) = zztmp as u32;
                        ltLo += 1;
                        unLo += 1;
                    } else {
                        if n > 0 as libc::c_int {
                            break;
                        }
                        unLo += 1;
                    }
                }
                while unLo <= unHi {
                    n = *eclass.offset(*fmap.offset(unHi as isize) as isize) as i32 - med as i32;
                    if n == 0 as libc::c_int {
                        let zztmp_0: i32 = *fmap.offset(unHi as isize) as i32;
                        *fmap.offset(unHi as isize) = *fmap.offset(gtHi as isize);
                        *fmap.offset(gtHi as isize) = zztmp_0 as u32;
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
                let zztmp_1: i32 = *fmap.offset(unLo as isize) as i32;
                *fmap.offset(unLo as isize) = *fmap.offset(unHi as isize);
                *fmap.offset(unHi as isize) = zztmp_1 as u32;
                unLo += 1;
                unHi -= 1;
            }
            if gtHi < ltLo {
                continue;
            }
            n = if ltLo - lo < unLo - ltLo {
                ltLo - lo
            } else {
                unLo - ltLo
            };
            let mut yyp1: i32 = lo;
            let mut yyp2: i32 = unLo - n;
            let mut yyn: i32 = n;
            while yyn > 0 as libc::c_int {
                let zztmp_2: i32 = *fmap.offset(yyp1 as isize) as i32;
                *fmap.offset(yyp1 as isize) = *fmap.offset(yyp2 as isize);
                *fmap.offset(yyp2 as isize) = zztmp_2 as u32;
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
                let zztmp_3: i32 = *fmap.offset(yyp1_0 as isize) as i32;
                *fmap.offset(yyp1_0 as isize) = *fmap.offset(yyp2_0 as isize);
                *fmap.offset(yyp2_0 as isize) = zztmp_3 as u32;
                yyp1_0 += 1;
                yyp2_0 += 1;
                yyn_0 -= 1;
            }
            n = lo + unLo - ltLo - 1 as libc::c_int;
            m = hi - (gtHi - unHi) + 1 as libc::c_int;
            if n - lo > hi - m {
                stackLo[sp as usize] = lo;
                stackHi[sp as usize] = n;
                sp += 1;
                stackLo[sp as usize] = m;
                stackHi[sp as usize] = hi;
                sp += 1;
            } else {
                stackLo[sp as usize] = m;
                stackHi[sp as usize] = hi;
                sp += 1;
                stackLo[sp as usize] = lo;
                stackHi[sp as usize] = n;
                sp += 1;
            }
        }
    }
}
unsafe fn fallbackSort(fmap: *mut u32, eclass: *mut u32, bhtab: *mut u32, nblock: i32, verb: i32) {
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
    let nBhtab: i32;
    let eclass8: *mut u8 = eclass as *mut u8;
    if verb >= 4 as libc::c_int {
        eprintln!("        bucket sorting ...");
    }
    i = 0 as libc::c_int;
    while i < 257 as libc::c_int {
        ftab[i as usize] = 0 as libc::c_int;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < nblock {
        ftab[*eclass8.offset(i as isize) as usize] += 1;
        ftab[*eclass8.offset(i as isize) as usize];
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < 256 as libc::c_int {
        ftabCopy[i as usize] = ftab[i as usize];
        i += 1;
    }
    i = 1 as libc::c_int;
    while i < 257 as libc::c_int {
        ftab[i as usize] += ftab[(i - 1 as libc::c_int) as usize];
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < nblock {
        j = *eclass8.offset(i as isize) as i32;
        k = ftab[j as usize] - 1 as libc::c_int;
        ftab[j as usize] = k;
        *fmap.offset(k as isize) = i as u32;
        i += 1;
    }
    nBhtab = 2 as libc::c_int + nblock / 32 as libc::c_int;
    i = 0 as libc::c_int;
    while i < nBhtab {
        *bhtab.offset(i as isize) = 0 as libc::c_int as u32;
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < 256 as libc::c_int {
        let fresh0 = &mut (*bhtab.offset((ftab[i as usize] >> 5 as libc::c_int) as isize));
        *fresh0 |= (1 as libc::c_int as u32) << (ftab[i as usize] & 31 as libc::c_int);
        i += 1;
    }
    i = 0 as libc::c_int;
    while i < 32 as libc::c_int {
        let fresh1 =
            &mut (*bhtab.offset(((nblock + 2 as libc::c_int * i) >> 5 as libc::c_int) as isize));
        *fresh1 |=
            (1 as libc::c_int as u32) << ((nblock + 2 as libc::c_int * i) & 31 as libc::c_int);
        let fresh2 = &mut (*bhtab.offset(
            ((nblock + 2 as libc::c_int * i + 1 as libc::c_int) >> 5 as libc::c_int) as isize,
        ));
        *fresh2 &= !((1 as libc::c_int as u32)
            << ((nblock + 2 as libc::c_int * i + 1 as libc::c_int) & 31 as libc::c_int));
        i += 1;
    }
    H = 1 as libc::c_int;
    loop {
        if verb >= 4 as libc::c_int {
            eprint!("        depth {:>6} has ", H);
        }
        j = 0 as libc::c_int;
        i = 0 as libc::c_int;
        while i < nblock {
            if *bhtab.offset((i >> 5 as libc::c_int) as isize)
                & (1 as libc::c_int as u32) << (i & 31 as libc::c_int)
                != 0
            {
                j = i;
            }
            k = (*fmap.offset(i as isize)).wrapping_sub(H as libc::c_uint) as i32;
            if k < 0 as libc::c_int {
                k += nblock;
            }
            *eclass.offset(k as isize) = j as u32;
            i += 1;
        }
        nNotDone = 0 as libc::c_int;
        r = -1 as libc::c_int;
        loop {
            k = r + 1 as libc::c_int;
            while *bhtab.offset((k >> 5 as libc::c_int) as isize)
                & (1 as libc::c_int as u32) << (k & 31 as libc::c_int)
                != 0
                && k & 0x1f as libc::c_int != 0
            {
                k += 1;
            }
            if *bhtab.offset((k >> 5 as libc::c_int) as isize)
                & (1 as libc::c_int as u32) << (k & 31 as libc::c_int)
                != 0
            {
                while *bhtab.offset((k >> 5 as libc::c_int) as isize) == 0xffffffff as libc::c_uint
                {
                    k += 32 as libc::c_int;
                }
                while *bhtab.offset((k >> 5 as libc::c_int) as isize)
                    & (1 as libc::c_int as u32) << (k & 31 as libc::c_int)
                    != 0
                {
                    k += 1;
                }
            }
            l = k - 1 as libc::c_int;
            if l >= nblock {
                break;
            }
            while *bhtab.offset((k >> 5 as libc::c_int) as isize)
                & (1 as libc::c_int as u32) << (k & 31 as libc::c_int)
                == 0
                && k & 0x1f as libc::c_int != 0
            {
                k += 1;
            }
            if *bhtab.offset((k >> 5 as libc::c_int) as isize)
                & (1 as libc::c_int as u32) << (k & 31 as libc::c_int)
                == 0
            {
                while *bhtab.offset((k >> 5 as libc::c_int) as isize)
                    == 0 as libc::c_int as libc::c_uint
                {
                    k += 32 as libc::c_int;
                }
                while *bhtab.offset((k >> 5 as libc::c_int) as isize)
                    & (1 as libc::c_int as u32) << (k & 31 as libc::c_int)
                    == 0
                {
                    k += 1;
                }
            }
            r = k - 1 as libc::c_int;
            if r >= nblock {
                break;
            }
            if r > l {
                nNotDone += r - l + 1 as libc::c_int;
                fallbackQSort3(fmap, eclass, l, r);
                cc = -1 as libc::c_int;
                i = l;
                while i <= r {
                    cc1 = *eclass.offset(*fmap.offset(i as isize) as isize) as i32;
                    if cc != cc1 {
                        let fresh3 = &mut (*bhtab.offset((i >> 5 as libc::c_int) as isize));
                        *fresh3 |= (1 as libc::c_int as u32) << (i & 31 as libc::c_int);
                        cc = cc1;
                    }
                    i += 1;
                }
            }
        }
        if verb >= 4 as libc::c_int {
            eprintln!("{:>6} unresolved strings", nNotDone);
        }
        H *= 2 as libc::c_int;
        if H > nblock || nNotDone == 0 as libc::c_int {
            break;
        }
    }
    if verb >= 4 as libc::c_int {
        eprintln!("        reconstructing block ...");
    }
    j = 0 as libc::c_int;
    i = 0 as libc::c_int;
    while i < nblock {
        while ftabCopy[j as usize] == 0 as libc::c_int {
            j += 1;
        }
        ftabCopy[j as usize] -= 1;
        ftabCopy[j as usize];
        *eclass8.offset(*fmap.offset(i as isize) as isize) = j as u8;
        i += 1;
    }
    if j >= 256 as libc::c_int {
        BZ2_bz__AssertH__fail(1005 as libc::c_int);
    }
}
#[inline]
unsafe fn mainGtU(
    mut i1: u32,
    mut i2: u32,
    block: *mut u8,
    quadrant: *mut u16,
    nblock: u32,
    budget: *mut i32,
) -> Bool {
    let mut k: i32;
    let mut c1: u8;
    let mut c2: u8;
    let mut s1: u16;
    let mut s2: u16;
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    c1 = *block.offset(i1 as isize);
    c2 = *block.offset(i2 as isize);
    if c1 != c2 {
        return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
    }
    i1 = i1.wrapping_add(1);
    i2 = i2.wrapping_add(1);
    k = nblock.wrapping_add(8 as libc::c_int as libc::c_uint) as i32;
    loop {
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return (s1 as libc::c_int > s2 as libc::c_int) as libc::c_int as Bool;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return (s1 as libc::c_int > s2 as libc::c_int) as libc::c_int as Bool;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return (s1 as libc::c_int > s2 as libc::c_int) as libc::c_int as Bool;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return (s1 as libc::c_int > s2 as libc::c_int) as libc::c_int as Bool;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return (s1 as libc::c_int > s2 as libc::c_int) as libc::c_int as Bool;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return (s1 as libc::c_int > s2 as libc::c_int) as libc::c_int as Bool;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return (s1 as libc::c_int > s2 as libc::c_int) as libc::c_int as Bool;
        }
        i1 = i1.wrapping_add(1);
        i2 = i2.wrapping_add(1);
        c1 = *block.offset(i1 as isize);
        c2 = *block.offset(i2 as isize);
        if c1 != c2 {
            return (c1 as libc::c_int > c2 as libc::c_int) as libc::c_int as Bool;
        }
        s1 = *quadrant.offset(i1 as isize);
        s2 = *quadrant.offset(i2 as isize);
        if s1 != s2 {
            return (s1 as libc::c_int > s2 as libc::c_int) as libc::c_int as Bool;
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
            break;
        }
    }
    0 as libc::c_int as Bool
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
    budget: *mut i32,
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
            ) != 0
            {
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
            ) != 0
            {
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
            ) != 0
            {
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
unsafe fn mainQSort3(
    ptr: *mut u32,
    block: *mut u8,
    quadrant: *mut u16,
    nblock: i32,
    loSt: i32,
    hiSt: i32,
    dSt: i32,
    budget: *mut i32,
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
        if sp >= 100 as libc::c_int - 2 as libc::c_int {
            BZ2_bz__AssertH__fail(1001 as libc::c_int);
        }
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
    budget: *mut i32,
) {
    let mut i: i32;
    let mut j: i32;
    let mut k: i32;
    let mut ss: i32;
    let mut sb: i32;
    let mut runningOrder: [i32; 256] = [0; 256];
    let mut bigDone: [Bool; 256] = [0; 256];
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
    i = 0 as libc::c_int;
    while i <= 255 as libc::c_int {
        bigDone[i as usize] = 0 as libc::c_int as Bool;
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
        if bigDone[ss as usize] != 0 {
            BZ2_bz__AssertH__fail(1006 as libc::c_int);
        }
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
            if bigDone[c1 as usize] == 0 {
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
            if bigDone[c1 as usize] == 0 {
                let fresh12 = copyEnd[c1 as usize];
                copyEnd[c1 as usize] -= 1;
                *ptr.offset(fresh12 as isize) = k as u32;
            }
            j -= 1;
        }
        if !(copyStart[ss as usize] - 1 as libc::c_int == copyEnd[ss as usize]
            || copyStart[ss as usize] == 0 as libc::c_int
                && copyEnd[ss as usize] == nblock - 1 as libc::c_int)
        {
            BZ2_bz__AssertH__fail(1007 as libc::c_int);
        }
        j = 0 as libc::c_int;
        while j <= 255 as libc::c_int {
            let fresh13 = &mut (*ftab.offset(((j << 8 as libc::c_int) + ss) as isize));
            *fresh13 |= ((1 as libc::c_int) << 21 as libc::c_int) as libc::c_uint;
            j += 1;
        }
        bigDone[ss as usize] = 1 as libc::c_int as Bool;
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
            if (bbSize - 1 as libc::c_int) >> shifts > 65535 as libc::c_int {
                BZ2_bz__AssertH__fail(1002 as libc::c_int);
            }
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
pub unsafe fn BZ2_blockSort(s: *mut EState) {
    let ptr: *mut u32 = (*s).ptr;
    let block: *mut u8 = (*s).block;
    let ftab: *mut u32 = (*s).ftab;
    let nblock: i32 = (*s).nblock;
    let verb: i32 = (*s).verbosity;
    let mut wfact: i32 = (*s).workFactor;
    let quadrant: *mut u16;
    let mut budget: i32;
    let budgetInit: i32;
    let mut i: i32;
    if nblock < 10000 as libc::c_int {
        fallbackSort((*s).arr1, (*s).arr2, ftab, nblock, verb);
    } else {
        i = nblock + (2 as libc::c_int + 12 as libc::c_int + 18 as libc::c_int + 2 as libc::c_int);
        if i & 1 as libc::c_int != 0 {
            i += 1;
        }
        quadrant = &mut *block.offset(i as isize) as *mut u8 as *mut u16;
        if wfact < 1 as libc::c_int {
            wfact = 1 as libc::c_int;
        }
        if wfact > 100 as libc::c_int {
            wfact = 100 as libc::c_int;
        }
        budgetInit = nblock * ((wfact - 1 as libc::c_int) / 3 as libc::c_int);
        budget = budgetInit;
        mainSort(ptr, block, quadrant, ftab, nblock, verb, &mut budget);
        if verb >= 3 as libc::c_int {
            eprintln!(
                "      {} work, {} block, ratio {:5.2}",
                budgetInit - budget,
                nblock,
                ((budgetInit - budget) as libc::c_float
                    / (if nblock == 0 as libc::c_int {
                        1 as libc::c_int
                    } else {
                        nblock
                    }) as libc::c_float) as libc::c_double,
            );
        }
        if budget < 0 as libc::c_int {
            if verb >= 2 as libc::c_int {
                eprintln!("    too repetitive; using fallback sorting algorithm");
            }
            fallbackSort((*s).arr1, (*s).arr2, ftab, nblock, verb);
        }
    }
    (*s).origPtr = -1 as libc::c_int;
    i = 0 as libc::c_int;
    while i < (*s).nblock {
        if *ptr.offset(i as isize) == 0 as libc::c_int as libc::c_uint {
            (*s).origPtr = i;
            break;
        } else {
            i += 1;
        }
    }
    if (*s).origPtr == -1 as libc::c_int {
        BZ2_bz__AssertH__fail(1003 as libc::c_int);
    }
}

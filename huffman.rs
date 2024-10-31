use crate::assert_h;

const BZ_MAX_ALPHA_SIZE: usize = 258;
const BZ_MAX_CODE_LEN: usize = 23;

#[inline]
const fn weight_of(zz0: i32) -> i32 {
    zz0 & 0xffffff00u32 as i32
}

#[inline]
const fn depth_of(zz1: i32) -> i32 {
    zz1 & 0xff
}

#[inline]
fn add_weights(zw1: i32, zw2: i32) -> i32 {
    (weight_of(zw1)).wrapping_add(weight_of(zw2)) | (1 + Ord::max(depth_of(zw1), depth_of(zw2)))
}

#[inline]
fn upheap(
    heap: &mut [i32; BZ_MAX_ALPHA_SIZE + 2],
    weight: &mut [i32; BZ_MAX_ALPHA_SIZE * 2],
    mut z: i32,
) {
    let tmp = heap[z as usize];
    while weight[tmp as usize] < weight[heap[(z >> 1) as usize] as usize] {
        heap[z as usize] = heap[(z >> 1) as usize];
        z >>= 1;
    }
    heap[z as usize] = tmp;
}

#[inline]
fn downheap(
    heap: &mut [i32; BZ_MAX_ALPHA_SIZE + 2],
    weight: &mut [i32; BZ_MAX_ALPHA_SIZE * 2],
    nHeap: i32,
    mut z: i32,
) {
    let tmp = heap[z as usize];
    loop {
        let mut yy = z << 1;
        if yy > nHeap {
            break;
        }
        if yy < nHeap
            && weight[heap[(yy + 1) as usize] as usize] < weight[heap[yy as usize] as usize]
        {
            yy += 1;
        }
        if weight[tmp as usize] < weight[heap[yy as usize] as usize] {
            break;
        }
        heap[z as usize] = heap[yy as usize];
        z = yy;
    }
    heap[z as usize] = tmp;
}

pub unsafe fn BZ2_hbMakeCodeLengths(len: &mut [u8], freq: &[i32], alphaSize: i32, maxLen: i32) {
    /*--
       Nodes and heap entries run from 1.  Entry 0
       for both the heap and nodes is a sentinel.
    --*/
    let mut nNodes: i32;
    let mut nHeap: i32;
    let mut j: i32;
    let mut heap = [0i32; BZ_MAX_ALPHA_SIZE + 2];
    let mut weight = [0i32; BZ_MAX_ALPHA_SIZE * 2];
    let mut parent = [0i32; BZ_MAX_ALPHA_SIZE * 2];

    for i in 0..alphaSize as usize {
        weight[i + 1] = (if freq[i] == 0 { 1 } else { freq[i] }) << 8;
    }

    loop {
        nNodes = alphaSize;
        nHeap = 0;

        heap[0] = 0;
        weight[0] = 0;
        parent[0] = -2;

        for i in 1..=alphaSize {
            parent[i as usize] = -1;
            nHeap += 1;
            heap[nHeap as usize] = i;
            upheap(&mut heap, &mut weight, nHeap);
        }

        assert_h!(nHeap < (BZ_MAX_ALPHA_SIZE as i32 + 2), 2001);

        while nHeap > 1 {
            let n1 = heap[1] as usize;
            heap[1] = heap[nHeap as usize];
            nHeap -= 1;
            downheap(&mut heap, &mut weight, nHeap, 1);
            let n2 = heap[1] as usize;
            heap[1] = heap[nHeap as usize];
            nHeap -= 1;
            downheap(&mut heap, &mut weight, nHeap, 1);
            nNodes += 1;
            parent[n1] = nNodes;
            parent[n2] = nNodes;
            weight[nNodes as usize] = add_weights(weight[n1], weight[n2]);
            parent[nNodes as usize] = -1;
            nHeap += 1;
            heap[nHeap as usize] = nNodes;
            upheap(&mut heap, &mut weight, nHeap);
        }

        assert_h!(nNodes < (BZ_MAX_ALPHA_SIZE as i32 * 2), 2002);

        let mut tooLong = false;
        for i in 1..=alphaSize {
            j = 0;
            let mut k = i;
            while parent[k as usize] >= 0 {
                k = parent[k as usize];
                j += 1;
            }
            len[i as usize - 1] = j as u8;
            if j > maxLen {
                tooLong = true;
            }
        }

        if !tooLong {
            break;
        }

        /* 17 Oct 04: keep-going condition for the following loop used
        to be 'i < alphaSize', which missed the last element,
        theoretically leading to the possibility of the compressor
        looping.  However, this count-scaling step is only needed if
        one of the generated Huffman code words is longer than
        maxLen, which up to and including version 1.0.2 was 20 bits,
        which is extremely unlikely.  In version 1.0.3 maxLen was
        changed to 17 bits, which has minimal effect on compression
        ratio, but does mean this scaling step is used from time to
        time, enough to verify that it works.

        This means that bzip2-1.0.3 and later will only produce
        Huffman codes with a maximum length of 17 bits.  However, in
        order to preserve backwards compatibility with bitstreams
        produced by versions pre-1.0.3, the decompressor must still
        handle lengths of up to 20. */

        for i in 1..=alphaSize {
            j = weight[i as usize] >> 8;
            j = 1 + j / 2;
            weight[i as usize] = j << 8;
        }
    }
}

pub fn BZ2_hbAssignCodes(
    code: &mut [i32],
    length: &[u8],
    minLen: i32,
    maxLen: i32,
    alphaSize: usize,
) {
    let mut vec: i32 = 0;
    for n in minLen..=maxLen {
        for i in 0..alphaSize {
            if length[i] as i32 == n {
                code[i] = vec;
                vec += 1;
            }
        }
        vec <<= 1;
    }
}

pub fn BZ2_hbCreateDecodeTables(
    limit: &mut [i32],
    base: &mut [i32],
    perm: &mut [i32],
    length: &mut [u8],
    minLen: i32,
    maxLen: i32,
    alphaSize: i32,
) {
    let alphaSize = usize::try_from(alphaSize).unwrap_or(0);

    let mut pp: i32 = 0;
    for i in minLen..=maxLen {
        for j in 0..alphaSize {
            if length[j] as i32 == i {
                perm[pp as usize] = j as i32;
                pp += 1;
            }
        }
    }

    base[0..BZ_MAX_CODE_LEN].fill(0);

    for i in 0..alphaSize {
        base[length[i] as usize + 1] += 1;
    }

    for i in 1..BZ_MAX_CODE_LEN {
        base[i] += base[i - 1];
    }

    limit[0..BZ_MAX_CODE_LEN].fill(0);

    let mut vec = 0;
    for i in minLen..=maxLen {
        vec += base[i as usize + 1] - base[i as usize];
        limit[i as usize] = vec - 1;
        vec <<= 1;
    }

    for i in minLen + 1..=maxLen {
        base[i as usize] = ((limit[i as usize - 1] + 1) << 1) - base[i as usize];
    }
}

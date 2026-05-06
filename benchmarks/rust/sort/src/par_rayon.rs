const SORT_CHUNK: usize = super::DIRECT_THRESHOLD;
const MERGE_CHUNK: usize = 2*super::DIRECT_THRESHOLD;

pub(super) fn sort(src: &mut [i32], buf: &mut [i32], usebuf: bool) {
    if usebuf && src.len() < SORT_CHUNK {
        src.sort();
        return;
    }

    let mid = src.len() / 2;
    let (sa, sb) = src.split_at_mut(mid);
    let (bufa, bufb) = buf.split_at_mut(mid);
    rayon::join(|| sort(sa, bufa, !usebuf), || sort(sb, bufb, !usebuf));

    if usebuf {
        merge(bufa, bufb, src);
    } else {
        merge(sa, sb, buf)
    }
    
}

fn merge(a: &mut [i32], b: &mut [i32], dest: &mut [i32]) {
    // Swap so a is always longer.
    let (a, b) = if a.len() > b.len() { (a, b) } else { (b, a) };
    if dest.len() < MERGE_CHUNK {
        seq_merge(a, b, dest);
        return;
    }

    // Find the middle element of the longer list, and
    // use binary search to find its location in the shorter list.
    let ma = a.len() / 2;
    let mb = match b.binary_search(&a[ma]) {
        Ok(i) => i,
        Err(i) => i,
    };

    let (a1, a2) = a.split_at_mut(ma);
    let (b1, b2) = b.split_at_mut(mb);
    let (d1, d2) = dest.split_at_mut(ma + mb);
    rayon::join(|| merge(a1, b1, d1), || merge(a2, b2, d2));
}

fn seq_merge(src_left: &[i32], src_right: &[i32], dest: &mut [i32]) {
    if src_right.is_empty() {
        dest.copy_from_slice(src_left);
        return;
    }
    let mut i = 0; // index for src_left
    let mut j = 0; // index for src_right
    let mut k = 0; // index for dest

    let left_len = src_left.len();
    let right_len = src_right.len();

    while i < left_len && j < right_len {
        let left = src_left[i];
        let right = src_right[j];
        if left <= right {
            dest[k] = left;
            i += 1;
        } else {
            dest[k] = right;
            j += 1;
        }
        k += 1;
    }

    if i < left_len {
        dest[k..].copy_from_slice(&src_left[i..]);
    } else if j < right_len {
        dest[k..].copy_from_slice(&src_right[j..]);
    }
}
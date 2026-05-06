use std::time::Instant;
use rayon::prelude::*;

use super::Real;

// Iterator that counts in interleaved bits.
// e.g. 0b0, 0b1, 0b100, 0b101, 0b10000, 0b10001, ...
struct SplayedBitsCounter {
    value: usize,
    max: usize,
}

impl SplayedBitsCounter {
    fn new(max: usize) -> Self {
        SplayedBitsCounter { value: 0, max }
    }
}

impl Iterator for SplayedBitsCounter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        // Return only odd bits.
        let prev = self.value & 0x5555_5555;
        if prev < self.max {
            // Set all even bits.
            self.value |= 0xaaaa_aaaa;
            // Add one, carrying through even bits.
            self.value += 1;
            Some(prev)
        } else {
            None
        }
    }
}

#[test]
fn test_splayed_counter() {
    let bits: Vec<usize> = SplayedBitsCounter::new(64).collect();
    assert_eq!(
        vec![0b0, 0b1, 0b100, 0b101, 0b10000, 0b10001, 0b10100, 0b10101],
        bits
    );
}

// Multiply the matrices laid out in z order.
// https://en.wikipedia.org/wiki/Z-order_curve
#[inline(never)]
pub fn seq_matmulz(a: &[Real], b: &[Real], dest: &mut [Real]) {
    // All inputs need to be the same length.
    assert!(a.len() == b.len() && a.len() == dest.len());
    // Input matrices must be square with each side a power of 2.
    assert!(a.len().count_ones() == 1 && a.len().trailing_zeros() % 2 == 0);
    // Zero dest, as it may be uninitialized.
    for d in dest.iter_mut() {
        *d = 0.0;
    }

    // Multiply in morton order
    // D[i,j] = sum for all k A[i,k] * B[k,j]
    let n = dest.len();
    for (ij, d) in dest.iter_mut().enumerate() {
        let i = ij & 0xaaaa_aaaa;
        let j = ij & 0x5555_5555;
        let mut sum = 0.0;
        for k in SplayedBitsCounter::new(n) {
            // sum += a[i, k] * b[k, j];
            sum += unsafe {
                // If n is a power of 4: i, j, or k should produce
                // no bits outside a valid index of n.
                a.get_unchecked(i | k) * b.get_unchecked(k << 1 | j)
            };
        }
        *d = sum;
    }
}

#[allow(clippy::identity_op)]
const MULT_CHUNK: usize = 1 * 1024;
const LINEAR_CHUNK: usize = 64 * 1024;

fn quarter_chunks(v: &[Real]) -> (&[Real], &[Real], &[Real], &[Real]) {
    let mid = v.len() / 2;
    let quarter = mid / 2;
    let (left, right) = v.split_at(mid);
    let (a, b) = left.split_at(quarter);
    let (c, d) = right.split_at(quarter);
    (a, b, c, d)
}

fn quarter_chunks_mut(v: &mut [Real]) -> (&mut [Real], &mut [Real], &mut [Real], &mut [Real]) {
    let mid = v.len() / 2;
    let quarter = mid / 2;
    let (left, right) = v.split_at_mut(mid);
    let (a, b) = left.split_at_mut(quarter);
    let (c, d) = right.split_at_mut(quarter);
    (a, b, c, d)
}

fn join4<F1, F2, F3, F4, R1, R2, R3, R4>(f1: F1, f2: F2, f3: F3, f4: F4) -> (R1, R2, R3, R4)
where
    F1: FnOnce() -> R1 + Send,
    R1: Send,
    F2: FnOnce() -> R2 + Send,
    R2: Send,
    F3: FnOnce() -> R3 + Send,
    R3: Send,
    F4: FnOnce() -> R4 + Send,
    R4: Send,
{
    let ((r1, r2), (r3, r4)) = rayon::join(|| rayon::join(f1, f2), || rayon::join(f3, f4));
    (r1, r2, r3, r4)
}

#[allow(clippy::too_many_arguments)]
fn join8<F1, F2, F3, F4, F5, F6, F7, F8, R1, R2, R3, R4, R5, R6, R7, R8>(
    f1: F1,
    f2: F2,
    f3: F3,
    f4: F4,
    f5: F5,
    f6: F6,
    f7: F7,
    f8: F8,
) -> (R1, R2, R3, R4, R5, R6, R7, R8)
where
    F1: FnOnce() -> R1 + Send,
    R1: Send,
    F2: FnOnce() -> R2 + Send,
    R2: Send,
    F3: FnOnce() -> R3 + Send,
    R3: Send,
    F4: FnOnce() -> R4 + Send,
    R4: Send,
    F5: FnOnce() -> R5 + Send,
    R5: Send,
    F6: FnOnce() -> R6 + Send,
    R6: Send,
    F7: FnOnce() -> R7 + Send,
    R7: Send,
    F8: FnOnce() -> R8 + Send,
    R8: Send,
{
    let (((r1, r2), (r3, r4)), ((r5, r6), (r7, r8))) = rayon::join(
        || rayon::join(|| rayon::join(f1, f2), || rayon::join(f3, f4)),
        || rayon::join(|| rayon::join(f5, f6), || rayon::join(f7, f8)),
    );
    (r1, r2, r3, r4, r5, r6, r7, r8)
}

// Multiply two square power of two matrices, given in Z-order.
pub fn matmulz(a: &[Real], b: &[Real], dest: &mut [Real]) {
    if a.len() <= MULT_CHUNK {
        seq_matmulz(a, b, dest);
        return;
    }

    // Allocate uninitialized scratch space.
    let mut tmp = raw_buffer(dest.len());

    let (a1, a2, a3, a4) = quarter_chunks(a);
    let (b1, b2, b3, b4) = quarter_chunks(b);
    {
        let (d1, d2, d3, d4) = quarter_chunks_mut(dest);
        let (t1, t2, t3, t4) = quarter_chunks_mut(&mut tmp[..]);
        // Multiply 8 submatrices
        join8(
            || matmulz(a1, b1, d1),
            || matmulz(a1, b2, d2),
            || matmulz(a3, b1, d3),
            || matmulz(a3, b2, d4),
            || matmulz(a2, b3, t1),
            || matmulz(a2, b4, t2),
            || matmulz(a4, b3, t3),
            || matmulz(a4, b4, t4),
        );
    }

    // Sum each quarter
    rmatsum(tmp.as_mut(), dest);
}

pub fn matmul_strassen(a: &[Real], b: &[Real], dest: &mut [Real]) {
    if a.len() <= MULT_CHUNK {
        seq_matmulz(a, b, dest);
        return;
    }

    // Naming taken from https://en.wikipedia.org/wiki/Strassen_algorithm
    let (a11, a12, a21, a22) = quarter_chunks(a);
    let (b11, b12, b21, b22) = quarter_chunks(b);
    // 7 submatrix multiplies.
    // Maybe the tree should be leaning the other way...
    let (m1, m2, m3, m4, m5, m6, m7, _) = join8(
        || strassen_add2_mul(a11, a22, b11, b22),
        || strassen_add_mul(a21, a22, b11),
        || strassen_sub_mul(b12, b22, a11),
        || strassen_sub_mul(b21, b11, a22),
        || strassen_add_mul(a11, a12, b22),
        || strassen_sub_add_mul(a21, a11, b11, b12),
        || strassen_sub_add_mul(a12, a22, b21, b22),
        || (),
    );

    // Sum results into dest.
    let (c11, c12, c21, c22) = quarter_chunks_mut(dest);
    join4(
        || strassen_sum_sub(&m1[..], &m4[..], &m7[..], &m5[..], c11),
        || strassen_sum(&m3[..], &m5[..], c12),
        || strassen_sum(&m2[..], &m4[..], c21),
        || strassen_sum_sub(&m1[..], &m3[..], &m6[..], &m2[..], c22),
    );
}

fn raw_buffer(n: usize) -> Vec<Real> {
    // A zero-initialized buffer is fast enough for our purposes.
    vec![0.0; n]
}

fn strassen_add2_mul(a1: &[Real], a2: &[Real], b1: &[Real], b2: &[Real]) -> Vec<Real> {
    let mut dest = raw_buffer(a1.len());
    let (a, b) = rayon::join(|| rtmp_sum(a1, a2), || rtmp_sum(b1, b2));
    matmul_strassen(&a[..], &b[..], &mut dest[..]);
    dest
}

fn strassen_sub_add_mul(a1: &[Real], a2: &[Real], b1: &[Real], b2: &[Real]) -> Vec<Real> {
    let mut dest = raw_buffer(a1.len());
    let (a, b) = rayon::join(|| rtmp_sub(a1, a2), || rtmp_sum(b1, b2));
    matmul_strassen(&a[..], &b[..], &mut dest[..]);
    dest
}

fn strassen_add_mul(a1: &[Real], a2: &[Real], b: &[Real]) -> Vec<Real> {
    let mut dest = raw_buffer(a1.len());
    let a = rtmp_sum(a1, a2);
    matmul_strassen(&a[..], b, &mut dest[..]);
    dest
}

fn strassen_sub_mul(b1: &[Real], b2: &[Real], a: &[Real]) -> Vec<Real> {
    let mut dest = raw_buffer(a.len());
    let b = rtmp_sub(b1, b2);
    matmul_strassen(a, &b[..], &mut dest[..]);
    dest
}

fn strassen_sum_sub(a: &[Real], b: &[Real], c: &[Real], s: &[Real], dest: &mut [Real]) {
    rcopy(a, dest);
    rmatsum(b, dest);
    rmatsum(c, dest);
    rmatsub(s, dest);
}

fn strassen_sum(a: &[Real], b: &[Real], dest: &mut [Real]) {
    rcopy(a, dest);
    rmatsum(b, dest);
}

fn rtmp_sum(a: &[Real], b: &[Real]) -> Vec<Real> {
    let mut tmp = raw_buffer(a.len());
    rcopy(a, &mut tmp[..]);
    rmatsum(b, &mut tmp[..]);
    tmp
}

fn rtmp_sub(a: &[Real], b: &[Real]) -> Vec<Real> {
    let mut tmp = raw_buffer(a.len());
    rcopy(a, &mut tmp[..]);
    rmatsub(b, &mut tmp[..]);
    tmp
}

// Any layout works, we're just adding by element.
fn rmatsum(src: &[Real], dest: &mut [Real]) {
    dest.par_iter_mut()
        .zip(src.par_iter())
        .for_each(|(d, s)| *d += *s);
}

fn rmatsub(src: &[Real], dest: &mut [Real]) {
    dest.par_iter_mut()
        .zip(src.par_iter())
        .for_each(|(d, s)| *d -= *s);
}

fn rcopy(src: &[Real], dest: &mut [Real]) {
    if dest.len() <= LINEAR_CHUNK {
        dest.copy_from_slice(src);
        return;
    }

    let mid = dest.len() / 2;
    let (s1, s2) = src.split_at(mid);
    let (d1, d2) = dest.split_at_mut(mid);
    rayon::join(|| rcopy(s1, d1), || rcopy(s2, d2));
}


pub(crate) fn strassen_main(depth: usize, dim: usize, num_threads: usize) {
    let full_dim: usize = 2_usize.pow(depth.try_into().unwrap()) * dim;
    let size = full_dim.next_power_of_two();
    if size != full_dim { println!("INPUT WAS NOT A POWER OF TWO !!"); }
    let n = size * size;
    let a: Vec<Real> = vec![1.0; n];
    let b: Vec<Real> = vec![2.0; n];
    let mut dest: Vec<Real> = vec![0.0; n];

    let cores = core_affinity::get_core_ids().unwrap();
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .start_handler(move |index| {
            let core_id = cores[index % cores.len()];
            let res = core_affinity::set_for_current(core_id);
            if !res {
                eprintln!("Could not pin worker thread id {:?}, continuing without pinning...", core_id.id);
            }
        })
        .build_global()
        .unwrap();

    let start = Instant::now();
    matmul_strassen(&a[..], &b[..], &mut dest[..]);
    let end = start.elapsed();
    
    // let exp = (size * 2) as Real;
    // let check = dest.iter().all(|&x| x == exp);
    // eprintln!("RAYON STRASSEN CORRECT : {}", check);

    let _version = 6;

    println!("{},{},{},{},{}", _version, num_threads, depth, dim, end.as_secs_f32());
}

pub(crate) fn z_main(depth: usize, dim: usize, num_threads: usize) {
    let full_dim: usize = 2_usize.pow(depth.try_into().unwrap()) * dim;
    let size = full_dim.next_power_of_two();
    if size != full_dim { println!("INPUT WAS NOT A POWER OF TWO !!"); }
    let n = size * size;
    let a: Vec<Real> = vec![1.0; n];
    let b: Vec<Real> = vec![2.0; n];
    let mut dest: Vec<Real> = vec![0.0; n];

    
    let cores = core_affinity::get_core_ids().unwrap();
    rayon::ThreadPoolBuilder::new()
    .num_threads(num_threads)
    .start_handler(move |index| {
        let core_id = cores[index % cores.len()];
        let res = core_affinity::set_for_current(core_id);
        if !res {
            eprintln!("Could not pin worker thread id {:?}, continuing without pinning...", core_id.id);
        }
    })
    .build_global()
    .unwrap();

    let start = Instant::now();
    matmulz(&a[..], &b[..], &mut dest[..]);
    let end = start.elapsed();

    // let exp = (size * 2) as Real;
    // let check = dest.iter().all(|&x| {
    //     if x != exp {
    //         eprintln!("ERROR in matrix!, exp = {}, value = {}", exp, x);
    //     }
    //     x == exp
    // });
    // eprintln!("RAYON-Z ORDER CORRECT : {}", check);

    let _version = 5;

    println!("{},{},{},{},{}", _version, num_threads, depth, dim, end.as_secs_f32());
}

/**
 * I got the implementation from https://github.com/kth-competitive-programming/kactl/blob/main/content/number-theory/FastEratosthenes.h
 * and modified it to work with Rust.
 * Author: Jakob Kogler, chilli, pajenegod
 * Date: 2020-04-12
 * License: CC0
 * Description: Prime sieve for generating all primes smaller than LIM.
 * Time: LIM=1e9 $\approx$ 1.5s
 * Status: Stress-tested
 * Details: Despite its n log log n complexity, segmented sieve is still faster
 * than other options, including bitset sieves and linear sieves. This is
 * primarily due to its low memory usage, which reduces cache misses. This
 * implementation skips even numbers.
 *
 * Benchmark can be found here: https://ideone.com/e7TbX4
 *
 * The line `for (int i=idx; i<S+L; idx = (i += p))` is done on purpose for performance reasons.
 * Se https://github.com/kth-competitive-programming/kactl/pull/166#discussion_r408354338
 */

/// Generate a vector of all primes up to 2.5e7
pub fn generate_primes() -> Vec<u32> {
    const LIM: usize = 25_000_000;
    let s = (LIM as f64).sqrt().round() as usize;
    let r = LIM / 2;
    let reserve = ((LIM as f64) / (LIM as f64).ln() * 1.1).ceil() as usize;
    let mut primes: Vec<u32> = Vec::with_capacity(reserve);
    primes.push(2);
    let mut sieve = vec![false; s + 1];
    let mut cp: Vec<(u32, usize)> = Vec::new();
    for i in (3..=s).step_by(2) {
        if !sieve[i] {
            let idx = i * i / 2;
            cp.push((i as u32, idx));
            for j in ((i * i)..=s).step_by(2 * i) {
                sieve[j] = true;
            }
        }
    }

    let mut block = vec![false; s];
    let mut l = 1;
    while l <= r {
        let block_size = if l + s - 1 <= r { s } else { r - l + 1 };
        block.fill(false);
        for &mut (p, ref mut idx) in cp.iter_mut() {
            if *idx < l {
                let diff = l - *idx;
                *idx += ((diff + p as usize - 1) / p as usize) * p as usize;
            }
            let mut i = *idx;
            while i < l + block_size {
                block[i - l] = true;
                i += p as usize;
            }
            *idx = i;
        }

        for i in 0..block_size {
            if !block[i] {
                primes.push(((l + i) * 2 + 1) as u32);
            }
        }
        l += s;
    }

    primes
}
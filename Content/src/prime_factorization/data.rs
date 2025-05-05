use once_cell::sync::OnceCell;
use rug::Integer;

use crate::number_theory::generate_primes;

use super::ecm::suyama::generate_parameters;


pub static ITERATIONS: usize = 200;
pub static SIZE: usize = 128;
pub static BOUNDS1: (usize, usize) = (50_000, 50 * 50_000);
pub static BOUNDS2: (usize, usize) = (500_000, 50 * 500_000);
pub static BLOCK_SIZE_1: usize = 2000;
pub static BLOCK_SIZE_2: usize = 5000;

pub struct PrimeFactorizeData {
    pub primes: Vec<u32>,
    pub gaps1: (Vec<usize>, Vec<usize>),
    pub s1: Vec<bool>,
    pub params1: [(u32, u32); ITERATIONS],
    pub gaps2: (Vec<usize>, Vec<usize>),
    pub s2: Vec<bool>,
    pub params2: [(u32, u32); ITERATIONS]
}

pub static DATA: OnceCell<PrimeFactorizeData> = OnceCell::new();

pub fn get_data() -> &'static PrimeFactorizeData {
    DATA.get_or_init(|| {
        let primes = generate_primes();
        let gaps1 = calculate_gaps(&primes, BLOCK_SIZE_1, BOUNDS1.1 as u32);
        let s1 = find_s(BOUNDS1.0 as u64, &primes);
        let gaps2 = calculate_gaps(&primes, BLOCK_SIZE_2, BOUNDS2.1 as u32);
        let s2 = find_s(BOUNDS2.0 as u64, &primes);
        let params1 = generate_parameters();
        let params2 = generate_parameters();

        PrimeFactorizeData {
            primes,
            gaps1,
            s1,
            params1,
            gaps2,
            s2,
            params2
        }
    })
}

fn calculate_gaps(primes: &Vec<u32>, block_size: usize, B2: u32) -> (Vec<usize>, Vec<usize>) {
    static INF: usize = 1_000_000;

    let half_block_size = block_size / 2;
    let mut values: Vec<usize> = Vec::with_capacity(half_block_size);  // it actually should have size phi(block_size)/2 + 1
    let mut index: Vec<usize> = vec![0; half_block_size + 1]; 

    // mark all the multiples of 2 and 5 as not needed, because their gcd with the block size != 1
    for i in (0..half_block_size).step_by(2) {
        index[i] = INF;
    }
    
    for i in (5..half_block_size).step_by(10) {
        index[i] = INF;
    }
    
    for i in 1..half_block_size {
        if index[i] == 0 {
            index[i] = values.len();
            values.push(i);
        }
    }

    let mut gaps: Vec<usize> = Vec::with_capacity(primes.len());

    let mut multiple = 0;
    for &p in primes {
        if p > B2 {
            break;
        }

        while multiple + (block_size as u32) < p {
            multiple += block_size as u32;
        }
        
        let mut v = p - multiple;
        if v > half_block_size as u32 {
            v = block_size as u32 - v;
        }

        gaps.push(index[v as usize]);
    }

    (values, gaps)
}

fn find_s(B1: u64, primes: &Vec<u32>) -> Vec<bool> {
    let mut s: Integer = Integer::ONE.clone();
    // For each prime, compute the highest power pᵉ with pᵉ ≤ B₁ and multiply s by pᵉ.
    for p in primes {
        let mut p_pow = *p as u64;  
        if p_pow > B1 {
            break;
        }
        while p_pow * (*p as u64) <= B1 {
            p_pow *= *p as u64;
        }
        s *= p_pow;
    }

    let n = s.significant_bits() - 1;
    let mut s_bits: Vec<bool> = Vec::with_capacity(n as usize);
    for i in (0..n).rev() {
        s_bits.push(s.get_bit(i));
    }

    s_bits
}
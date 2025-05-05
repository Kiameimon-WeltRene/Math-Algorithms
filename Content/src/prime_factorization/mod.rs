#![allow(non_snake_case)]
use std::{cell::RefCell, ops::ShrAssign};
use ecm::{ecm_trial, suyama::suyama_parameterization, MontgomeryPoint};
use pollards_rho::pollard_rho_brent;
use rug::{integer::IsPrime, Assign, Integer};


pub mod structs;
pub mod ecm;
pub mod pollards_rho;
pub mod data;
use data::{get_data, BLOCK_SIZE_1, BLOCK_SIZE_2, BOUNDS1, BOUNDS2, ITERATIONS, SIZE};
use structs::{Factor, FixedVec};

use crate::montgomery_mod_mult::Context;
// pub use self::structs::{BufferData, Instance};

fn trial_division(n: &mut Integer, factors: &mut Vec<(Integer, u32)>, primes: &Vec<u32>)  {
    for p in &primes[1..1230] { // skip 2 because it already has been factored, trial divide up to 1e4
        if n.is_divisible_u(*p) {
            factors.push((Integer::from(*p), 1));
            n.div_exact_u_mut(*p);
            while n.is_divisible_u(*p) {
                n.div_exact_u_mut(*p);
                factors.last_mut().unwrap().1 += 1;
            }
        }
    }
}

/// Reduces the value of n based on the prime factors we have found so far.
/// We iterate through the entries that aren't fully factorized (stored in temporary_factors)
/// and remove any prime factors that have been found from them.
fn find_exponents(n: &mut Integer, prime_factors: &mut FixedVec<Integer, SIZE>,
    factors: &mut Vec<(Integer, u32)>, temporary_factors: &mut FixedVec<Factor, SIZE>) {
    
    for i in  0..temporary_factors.len() { 
        let factor = temporary_factors.get_mut(i);
        let curval = &mut factor.n;
        let index = factor.idx;

        for idx in index..prime_factors.len() {
            let p = prime_factors.get(idx);
            while curval.is_divisible(p) {
                curval.div_exact_mut(p);
            }
        }
        factor.idx = 0;
    }
     
    for i in 0..prime_factors.len() {
        let p = prime_factors.get(i);
        let mut exponent = 1;
        n.div_exact_mut(p);
        while n.is_divisible(p) {
            n.div_exact_mut(p);
            exponent += 1;
        }
        factors.push((p.clone(), exponent));
    }

    prime_factors.clear();

}

thread_local! {
    static BUFFER: RefCell<(Integer, 
        FixedVec<Integer, SIZE>,
        FixedVec<Factor, SIZE>,
        [(MontgomeryPoint, Integer); ITERATIONS],
        [bool; SIZE],
        Factor,
        Context,
        
        )> = RefCell::new((
            Integer::new(),
            FixedVec::new(Integer::new()), 
            FixedVec::new(Factor::new()),
            std::array::from_fn(|_| (MontgomeryPoint::default(), Integer::new())),
            std::array::from_fn(|_| true),
            Factor::new(),
            Context::new(Integer::ONE.clone()),
        ));
}

struct Buffer;

impl Buffer {
    fn get_mut<F, R>(f: F) -> R
    where
        F: FnOnce(
            &mut Integer,
            &mut FixedVec<Integer, SIZE>,
            &mut FixedVec<Factor, SIZE>,
            &mut [(MontgomeryPoint, Integer); ITERATIONS],
            &mut [bool; SIZE],
            &mut Factor,
            &mut Context,
        ) -> R,
    {
        BUFFER.with(|cell| {
            let (n, prime_factors, temporary_factors,
                curves, failed_pollard, factor, ctx) = &mut *cell.borrow_mut();
            f(n, prime_factors, temporary_factors, curves, failed_pollard, factor, ctx)
        })
    }
}

/// Given an integer n, the function returns a vector of tuples (prime, exponent) for each prime factor of n.
pub fn prime_factorize(n_: &Integer) -> Vec<(Integer, u32)> {
    let data = get_data();
    let primes = &data.primes;
    let mut factors: Vec<(Integer, u32)> = Vec::new();
    
    Buffer::get_mut(|n, prime_factors, temporary_factors,
        curves, failed_pollard, factor, ctx| {

        temporary_factors.clear();
        // prime_factors: stores factors but without exponent
        // temporary_factors: stores the numbers that have yet to be fully factored
        // failed_pollard: stores the numbers that failed to get factored by pollard
        
        n.assign(n_);
        // removes the even factor
        if n.is_even() {
            let two_exponent = n.find_one(0).unwrap();
            factors.push((Integer::from(2), two_exponent));
            n.shr_assign(two_exponent);
        }
    
        // do trial division up to 1e4 remove small prime factors
        trial_division(n, &mut factors, primes);
    
        if n == Integer::ONE {
            return factors;
        }
        
        temporary_factors.next().update_all(&*n, prime_factors.len());
        temporary_factors.inc();
        // println!("temporary_factors: {:?}", temporary_factors.top());
        failed_pollard[0] = false;

        let mut index = 1;
        while index > 0 {
            index -= 1;
            // println!("index: {:?}", index);

            let curval = &temporary_factors.get(index).n;

            // println!("curval: {:?}", curval);

            if curval.is_probably_prime(20) != IsPrime::No {
                // println!("curval is prime: {:?}", curval);
                prime_factors.next().assign(curval);
                prime_factors.inc();

                failed_pollard[index] = true;
                temporary_factors.dec();
                temporary_factors.swap(index, temporary_factors.len());
                continue;
            }

            factor.n.assign(&temporary_factors.get(index).n);
            let mut value_changed = false;

            for i in factor.idx..prime_factors.len() {
                let p = prime_factors.get(i);
                while factor.n.is_divisible(p) {
                    value_changed = true;
                    factor.n.div_exact_mut(p);
                }
            }

            temporary_factors.get_mut(index).idx = prime_factors.len();
            if failed_pollard[index] && !value_changed {
                continue;  // if it failed pollard before and we haven't reduced it further, skip it
            }
            
            failed_pollard[index] = true;
            
            // update the ctx before calling pollard_rho_brent
            if temporary_factors.get(index).ctx.n == factor.n {
                // if the ctx is the same, just assign it to the factor
                factor.ctx.assign(&temporary_factors.get(index).ctx);
            } else {
                factor.update_ctx();
            }

            for _ in 0..3 {
                // println!("factor: {:?}", factor.n);
                // directly assign the result of pollard_rho_brent to the next entry in temporary_factors
                match pollard_rho_brent(&factor.n, &mut factor.ctx, &mut temporary_factors.next().n) {
                    None => continue,
                    Some(()) => {
                        // println!("found factor: {:?}", temporary_factors.next().n);
                        factor.n.div_exact_mut(&temporary_factors.next().n);
                        failed_pollard[index] = false;

                        // don't change the ctx-es yet, if its prime doing so is redundant
                        // changing the ctx is left to before calling pollard_rho_brent
                        temporary_factors.get_mut(index).n.assign(&factor.n);
                        temporary_factors.get_mut(index).idx = prime_factors.len();
                        
                        temporary_factors.next().idx = prime_factors.len();
                        // println!("factored result: {:?}", temporary_factors.next());
                        temporary_factors.inc();
                        
                        let len = temporary_factors.len();
                        if len > 1 && temporary_factors.get(index).n < temporary_factors.get(len - 1).n {
                            temporary_factors.swap(index, len - 1);
                        }
                        
                        index = len;
                        failed_pollard[index - 1] = false;

                        break;
                    },
                };
            }
        }
        
        find_exponents(n, prime_factors, &mut factors, temporary_factors);
        // println!("after pollard: {:?}\n left with n = {}", factors, n);
        
        // generate curve parameters.
        ctx.change_mod(n);
        suyama_parameterization(ctx, &data.params1, curves);
        // do 200 rounds of ECM with B1 = 5e4, B2 = 50 * B1 = 2.5e6
        ecm_trial(n, ctx, BOUNDS1.0, BOUNDS1.1, &data.params1, curves, &data.s1, temporary_factors,
            prime_factors, &primes, &data.gaps1.1, &data.gaps1.0);

        find_exponents(n, prime_factors, &mut factors, temporary_factors);
        
        if n == Integer::ONE {
            return factors;
        }
        
        // println!("after ecm with B1 = 5e4, B2 = 50 * B1: {:?}\n left with n = {}", factors, n);

        // println!("curves: {:?}", curves);
        // println!("so far we have: {:?}, {:?}", factors, temporary_factors);

        ctx.change_mod(n);
        suyama_parameterization(ctx, &data.params2, curves);
    
        // increase the bounds of ECM: B1 = 5e5, B2 = 50 * B1 = 2.5e7 
        ecm_trial(n, ctx, BOUNDS2.0, BOUNDS2.1, &data.params2, curves, &data.s2, temporary_factors,
            prime_factors, &primes, &data.gaps2.1, &data.gaps2.0);
    
        /*
        if !temporary_factors.is_empty() {
            println!("failed to fully factorize");
        }
        */
    
        find_exponents(n, prime_factors, &mut factors, temporary_factors);
        factors
    })
}
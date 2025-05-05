#![allow(non_snake_case, dead_code)]
use crate::montgomery_mod_mult::Context;
use std::cell::RefCell;
use rug::integer::IsPrime;
use rug::{Integer, Assign};
use super::structs::{Factor, FixedVec};
use super::{BLOCK_SIZE_1, BLOCK_SIZE_2, BOUNDS1, ITERATIONS, SIZE};

pub mod suyama;

#[derive(Clone, Debug, Default)]
pub struct MontgomeryPoint {
    // Represent a point in projective (X:Z) coordinates.
    pub X: Integer,
    pub Z: Integer,
}

impl MontgomeryPoint {
    pub fn assign(&mut self, other: &MontgomeryPoint) {
        self.X.assign(&other.X);
        self.Z.assign(&other.Z);
    }
}

thread_local! {
    static BUFFER_INTEGERS: RefCell<(Integer, Integer, Integer)> =
        RefCell::new((Integer::new(), Integer::new(), Integer::new()));
}

struct BufferIntegers;

impl BufferIntegers {
    fn get_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Integer, &mut Integer, &mut Integer) -> R,
    {
        BUFFER_INTEGERS.with(|cell| {
            let (a, b, c) = &mut *cell.borrow_mut();
            f(a, b, c)
        })
    }
}

/// Montgomery point doubling: given a point P (in projective coordinates), calculates 2P and stores the result in P.
fn point_double(P: &mut MontgomeryPoint, a24: &Integer, ctx: &mut Context) {
    BufferIntegers::get_mut(|a, b, _| {
        a.assign(&P.X);
        *a += ctx.wrap(&P.Z);
        ctx.square_mut(a);      // a = (P.X + P.Z)^2
        b.assign(&P.X);
        *b -= ctx.wrap(&P.Z);
        ctx.square_mut(b);      // b = (P.X - P.Z)^2

        P.X.assign(&*a);
        P.X *= ctx.wrap(&*b);   // P.X = a * b
        *a -= ctx.wrap(&*b);    // a = a - b
        P.Z.assign(&*a);   // P.Z = a
        
        *a *= ctx.wrap(a24);
        *a += ctx.wrap(&*b);
        P.Z *= ctx.wrap(&*a);  // P.Z = a * (b + a24 * a) 
    });
}


/// Montgomery differential addition: given two points P, Q and R (in projective coordinates), calculates P + Q
/// and stores the result in P. It is required that R = P - Q (= Q - P).
/// If R.Z != 1, multiply the returned point's X coordinate by R.Z.
fn point_add(P: &mut MontgomeryPoint, Q: &MontgomeryPoint, R: &MontgomeryPoint, ctx: &mut Context) {
    BufferIntegers::get_mut(|a, b, z| {
        a.assign(&P.X);
        b.assign(&P.X);
        *a += ctx.wrap(&P.Z);    // a = P.X + P.Z
        *b -= ctx.wrap(&P.Z);    // b = P.X - P.Z
        z.assign(&Q.X);
        *z -= ctx.wrap(&Q.Z);
        *a *= ctx.wrap(&*z);     // a = (Q.X - Q.Z) * (P.X + P.Z)
        
        z.assign(&Q.X);
        *z += ctx.wrap(&Q.Z);
        *b *= ctx.wrap(&*z);     // b = (Q.X + Q.Z) * (P.X - P.Z)
        
        P.X.assign(&*a);
        P.X += ctx.wrap(&*b);
        ctx.square_mut(&mut P.X);  // P.X = (a + b)^2
        
        P.Z.assign(&*a);
        P.Z -= ctx.wrap(&*b);
        ctx.square_mut(&mut P.Z);
        P.Z *= ctx.wrap(&R.X);     // P.Z = R.X (a - b)^2
    });
}

thread_local! {
    static BUFFER_POINTS: RefCell<(MontgomeryPoint, MontgomeryPoint)> =
        RefCell::new((MontgomeryPoint::default(), MontgomeryPoint::default()));
}

struct BufferPoints;

impl BufferPoints {
    fn get_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut MontgomeryPoint, &mut MontgomeryPoint) -> R,
    {
        BUFFER_POINTS.with(|cell| {
            let (P, Q) = &mut *cell.borrow_mut();
            f(P, Q)
        })
    }
}

/// Montgomery ladder for scalar multiplication. Given a point P, compute [s]P and [s + 1] P.
/// The result is stored in P0 and Q0 respectively.
fn montgomery_ladder(P0: &mut MontgomeryPoint, Q0: &mut MontgomeryPoint, s: u32, a24: &Integer, ctx: &mut Context) {
    BufferPoints::get_mut(|P, Q| {
        Q.assign(&*P0);
        P.assign(&*P0);
        point_double(P, &a24, ctx);

        for i in (0..(31 - s.leading_zeros())).rev() {
            if (s >> i) & 1 != 0 {
                point_add(Q, P, P0, ctx);
                Q.X *= ctx.wrap(&P0.Z);
                point_double(P, a24, ctx);
            } else {
                point_add(P, Q, P0, ctx);
                P.X *= ctx.wrap(&P0.Z);
                point_double(Q, a24, ctx);
            }
        }

        P0.assign(&*Q);
        Q0.assign(&*P);
    });
}


/// ECM Phase 1. We calculate s*P (s has been calculated beforehand).
fn ecm_phase1(ctx: &mut Context, P0: &mut MontgomeryPoint, a24: &Integer, s: &Vec<bool>) {
    // Montgomery ladder for scalar multiplication.
    // Given a point P, compute [s]P. In this ladder, the difference between the two
    // running points is always the initial P.
    BufferPoints::get_mut(|P, Q| {
        Q.assign(&*P0);
        P.assign(&*P0);
        point_double(P, a24, ctx);
        
        for b in s {
            if *b {
                point_add(Q, P, P0, ctx);
                point_double(P, a24, ctx);
            } else {
                point_add(P, Q, P0, ctx);
                point_double(Q, a24, ctx);
            }
        }

        P0.assign(&*Q);
    });
}

/// Precomputes jQ0 where j is odd, storing the results in the table.
/// Give the values of j in the values vector.
fn precompute_gaps(Q0: &mut MontgomeryPoint, Q2: &MontgomeryPoint, table: &mut [MontgomeryPoint; 2000], ctx: &mut Context, values: &Vec<usize>) {
    BufferPoints::get_mut(|P, Q| {
        let mut index = 0;
        let mut j = 1;
        P.assign(Q0);

        for val in values {
            while j < *val {
                Q.assign(P);
                P.assign(Q0);
                point_add(Q0, Q2, Q, ctx);
                Q0.X *= ctx.wrap(&Q.Z);
                j += 2;
            }
            table[index].assign(&*Q0);
            index += 1;
        }
    });
}


thread_local! {
    static PHASE_2_BUFFER: RefCell<([MontgomeryPoint; 2000], MontgomeryPoint, MontgomeryPoint, MontgomeryPoint)> =
        RefCell::new((
            std::array::from_fn(|_| (MontgomeryPoint::default())),
            MontgomeryPoint::default(),
            MontgomeryPoint::default(),
            MontgomeryPoint::default(),
        ));
}

struct Phase2Buffer;

impl Phase2Buffer {
    fn get_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut [MontgomeryPoint; 2000], &mut MontgomeryPoint, &mut MontgomeryPoint, &mut MontgomeryPoint) -> R,
    {
        PHASE_2_BUFFER.with(|cell| {
            let (table, Q2, R_prev, R) = &mut *cell.borrow_mut();
            f(table, Q2, R_prev, R)
        })
    }
}


fn ecm_iteration(ctx: &mut Context, n: &Integer, B1: usize, block_size: usize, Q: &mut MontgomeryPoint, a24: &Integer,
    primes: &Vec<u32>, start: usize, end: usize, gaps: &Vec<usize>, values: &Vec<usize>, s: &Vec<bool>, g: &mut Integer) {
    ecm_phase1(ctx, Q, a24, &s);
    g.assign(Q.Z.gcd_ref(n));
    if g != Integer::ONE && g != n {
        return;
    }

    g.assign(&ctx.r_mod_n);  // g = 1 in montgomery form

    let half_block_size = block_size / 2;
    Phase2Buffer::get_mut(|table, Q2, R_prev, R| {
        Q2.assign(&*Q);
        point_double(Q2, a24, ctx);  // Q2 = 2Q0 (duh)
        
        R.assign(&*Q);
        // precompute the gaps between any prime and the block
        precompute_gaps(Q, Q2, table, ctx, values);

        Q.assign(&*R);
        montgomery_ladder(Q, Q2, block_size as u32, a24, ctx);  // Q = block_size Q

        let mut c = ((B1 + half_block_size) / block_size) as i32;
        Q2.assign(&*Q);
        // R = the starting location: [(B1 + block_size/2) / block_size] Q
        // Q2 = 1 block behind the starting location: [(B1 + block_size/2) / block_size - 1] Q
        montgomery_ladder(Q2, R, c as u32 - 1, a24, ctx);

        c *= block_size as i32;  // c = the scalar of R: R = cQ (before we multiplied Q by block_size)
        let mut index = start;

        for &gap in &gaps[start..end] {
            let mut distance = primes[index] as i32 - c;  // the "distance" between our point and the next prime
            while distance > half_block_size as i32 {
                R_prev.assign(Q2);
                Q2.assign(R);
                point_add(R, Q, R_prev, ctx);  // move to the next block
                R.X *= ctx.wrap(&R_prev.Z);
                
                distance -= block_size as i32;
                c += block_size as i32;
            }

            BufferIntegers::get_mut(|x, y, _| {    
                x.assign(&R.X);
                *x *= ctx.wrap(&table[gap].Z);
                y.assign(&R.Z);
                *y *= ctx.wrap(&table[gap].X);
                
                *x -= ctx.wrap(&*y);
                *g *= ctx.wrap(&*x);  // g *= R.X * table[gap].Z - table[gap].X * R.Z
            });
            
            if primes[index] % block_size as u32 > half_block_size as u32 {
                index += 1;
                continue;
            }

            index += 1;
        }
        
        g.gcd_mut(n);
    });
}

fn print_curve(curve: &(MontgomeryPoint, Integer), ctx: &mut Context) {
    println!("Curve: X: {}, Z: {}, a24: {}", ctx.from_montgomery(&curve.0.X), ctx.from_montgomery(&curve.0.Z), ctx.from_montgomery(&curve.1));
}

struct Buffer;

thread_local! {
    static BUFFER: RefCell<Integer> =
        RefCell::new(
            Integer::new(),
        );
}

impl Buffer {
    fn get_mut<F, R>(f: F) -> R 
    where
        F: FnOnce(&mut Integer) -> R,
    {
        BUFFER.with(|cell| {
            let value = &mut *cell.borrow_mut();
            f(value)
        })
    }
}

/// Given bounds B1 and B2, it runs 200 iterations of ECM (both phase 1 and 2).
/// Any prime factors found will be inserted into the prime_factors vector.
/// Insert the number to be factorised in the temporary_factors vector.
pub fn ecm_trial(n: &Integer, ctx_n: &mut Context, B1: usize, B2: usize, params: &[(u32, u32)], curves: &mut [(MontgomeryPoint, Integer); ITERATIONS],
    s: &Vec<bool>, temporary_factors: &mut FixedVec<Factor, SIZE>, prime_factors: &mut FixedVec<Integer, SIZE>,
    primes: &Vec<u32>, gaps: &Vec<usize>, values: &Vec<usize>) {
    let block_size = if B1 == BOUNDS1.0 {
        BLOCK_SIZE_1
    } else {
        BLOCK_SIZE_2
    };
    let print_curve_parameters = true;  // set to true to print the curve parameters

    let start = primes.partition_point(|&x| x < B1 as u32);
    let end = primes.partition_point(|&x| x <= B2 as u32);
    Buffer::get_mut(|result| {
        let mut i = 0;
        while i < ITERATIONS && !temporary_factors.is_empty() {
            let curve = &mut curves[i];
            // print_curve(&curve, ctx_n);
            i += 1;
            
            let factor = temporary_factors.top();
            let curval = &mut factor.n;
            let index = &mut factor.idx;
            let ctx = &mut factor.ctx;
            
            // check if we have found a prime factor from other iterations of ECM that also divides the current value 
            for idx in *index..prime_factors.len() {
                let p = prime_factors.get(idx);
                while curval.is_divisible(p) {
                        curval.div_exact_mut(p);
                }
            }
            
            if *curval == 1 {
                temporary_factors.dec();
                i -= 1;  // reuse the current curve
                continue;
            }

            *index = prime_factors.len();  // we have tested division up to this point

            while curval.is_perfect_square() {
                curval.sqrt_mut();
            }
            
            if curval.is_probably_prime(20) != IsPrime::No {
                prime_factors.next().assign(&*curval);
                prime_factors.inc();
                temporary_factors.dec();
                i -= 1;  // reuse the current curve
                continue;
            }
            
            // update the factor data
            factor.idx = prime_factors.len();
            if ctx.n != *curval {
                ctx.change_mod(curval);
            }
            
            // change the curve to the new modulus if necessary
            if curval != n {
                // println!("changing curve to {}", curval);
                ctx_n.from_montgomery_mut(&mut curve.0.X);
                ctx_n.from_montgomery_mut(&mut curve.0.Z);
                ctx_n.from_montgomery_mut(&mut curve.1);
                curve.0.X %= &*curval;
                curve.0.Z %= &*curval;
                curve.1 %= &*curval;
                ctx.to_montgomery_mut(&mut curve.0.X);
                ctx.to_montgomery_mut(&mut curve.0.Z);
                ctx.to_montgomery_mut(&mut curve.1);
            }
    
            // println!("current: {}", curval);
            ecm_iteration(ctx, curval, B1, block_size, &mut curve.0, &curve.1, &primes, start, end, &gaps, &values, &s, result);
            
            
            // if *result != 1 {
            //     println!("found: {}", result);
            //     println!("\n");
            //     print_curve(curve, ctx);
            // }
    
            if result == Integer::ONE || result == curval {
                // the current curve failed to find a factor
                // println!("failed: {}", result);
                continue;
            }

            if print_curve_parameters {
                println!("Bounds: {} {}", B1, B2);
                println!("DATA: {}, {}", params[i - 1].0, params[i - 1].1);
                println!("result: {}, curval: {}", result, curval);
            }
            // don't update the ctx, leave that to before calling ecm_iteration
            curval.div_exact_mut(result);

            temporary_factors.next().update_n_and_index(&*result, prime_factors.len());
            temporary_factors.inc();

            let len = temporary_factors.len();
            if len > 1 && temporary_factors.get(len - 2).n < temporary_factors.get(len - 1).n {
                temporary_factors.swap(len - 2, len - 1);
            }
        }
    })
}
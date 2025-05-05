#![allow(non_snake_case)]

use std::{cell::RefCell, cmp::max};

use rug::{Assign, Integer};

use rand::{rng, Rng};

use crate::{montgomery_mod_mult::Context, prime_factorization::ITERATIONS};

use super::MontgomeryPoint;

pub fn generate_parameters() -> [(u32, u32); ITERATIONS] {
    let mut rng = rng();

    let mut params: [(u32, u32); ITERATIONS] = std::array::from_fn(|_| (0, 0));

    for item in params.iter_mut() {
        let sigma: u16 = rng.random();
        item.0 = sigma.into();
        item.0 = max(item.0, 6);  // sigma must be > 5
        item.1 = 4 * item.0;
        item.0 *= item.0;
        item.0 -= 5;                     // item.0 is now u
    }

    params
}

struct Buffer;

thread_local! {
    static BUFFER: RefCell<([Integer; ITERATIONS], Integer, Integer)> =
        RefCell::new((
            std::array::from_fn(|_| Integer::new()),
            Integer::new(),
            Integer::new()
        ));
}

impl Buffer {
    fn get_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut [Integer; ITERATIONS], &mut Integer, &mut Integer) -> R,
    {
        BUFFER.with(|cell| {
            let (arr, value, value2) = &mut *cell.borrow_mut();
            f(arr, value, value2)
        })
    }
}

/// Generates n curves and starting points defined by Suyama's parameterization.
/// Generating them in batches is faster.
pub fn suyama_parameterization(ctx: &mut Context, params: &[(u32, u32)], curves: &mut [(MontgomeryPoint, Integer)]) {    
    let mont_16 = ctx.to_montgomery(&Integer::from(16));
    let mont_3 = ctx.to_montgomery(&Integer::from(3));

    for i in 0..ITERATIONS {
        let (P, val) = &mut curves[i];
        P.X.assign(params[i].0);
        ctx.to_montgomery_mut(&mut P.X);
        P.Z.assign(params[i].1);
        ctx.to_montgomery_mut(&mut P.Z); // P is now (u, v) in Montgomery form

        val.assign(&P.X);
        ctx.cube_mut(val);
        *val *= ctx.wrap(&mont_16);
        *val *= ctx.wrap(&P.Z);  // curves[i].1 is the denominator of a24 in suyama's parameterization
    }

    Buffer::get_mut(|arr, value, value2| {
        let prod = &mut *arr;
        
        // Invert the denominators together    
        (*prod)[0].assign(&curves[0].1);
        for i in 1..ITERATIONS {
            let [prod_i_1, prod_i] = (*prod).get_disjoint_mut([i-1, i]).unwrap();
            prod_i.assign(&*prod_i_1);
            *prod_i *= ctx.wrap(&curves[i].1);
        }

        // It shouldn't be the case that their gcd with n is not 1, since pollard would have sieved out small factors 
        value.assign(&(*prod)[ITERATIONS - 1]);
        ctx.invert_mut(value).unwrap();
        
        // calculate mod inverses (the denominators of a)
        for i in (0..ITERATIONS - 1).rev() {
            prod[i] *= ctx.wrap(&*value);  // prod[i] is now (curves[i + 1].1)^-1 in montgomery form
            *value *= ctx.wrap(&curves[i + 1].1);  // value is now (product of curves[0].1 ... curves[i].1)^-1 in montgomery form
            curves[i + 1].1.assign(&prod[i]);
        }
        curves[0].1.assign(&*value);

        let (w, y) = (&mut *value, &mut *value2);
    
        for i in 0..ITERATIONS {
            let (P, a24) = &mut curves[i];
            w.assign(&P.X);  // W = u
            y.assign(&P.Z);  // Y = v
            
            P.Z -= ctx.wrap(&P.X);
            ctx.cube_mut(&mut P.Z);  // Z = (v - u)^3
            *w *= ctx.wrap(&mont_3); 
            *w += ctx.wrap(&*y);       // W = 3u + v
            P.Z *= ctx.wrap(&*w);      // P.Z is now the numerator of a
    
            ctx.square_mut(&mut P.X);
            ctx.square_mut(&mut P.X);  // X = u^4
            P.X *= ctx.wrap(&mont_16);  // recall: a24 = (16 u^3 v)^-1
            P.X *= ctx.wrap(&*a24);    // X = u * v^-1
            ctx.cube_mut(&mut P.X);    // X = (u * v^-1)^3
            
            *a24 *= ctx.wrap(&P.Z);
            P.Z.assign(&ctx.r_mod_n);  // Z = 1
        }
    });
    
}

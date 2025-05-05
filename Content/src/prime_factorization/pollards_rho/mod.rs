use std::{cell::RefCell, cmp::min};
use rug::{rand::RandState, Assign, Integer};

use crate::montgomery_mod_mult::Context;


thread_local! {
    static RAND_STATE: RefCell<RandState<'static>> = RefCell::new(RandState::new());
}

/// Computes the next value in the sequence: f(y) = (y^2 + 1) mod n.
fn f(x: &mut Integer, c: &Integer, ctx: &mut Context) {
    ctx.square_mut(x);
    *x += ctx.wrap(c);
}

thread_local! {
    static BUFFER_INTEGERS: RefCell<(Integer, Integer, Integer, Integer, Integer)> =
        RefCell::new((Integer::new(), Integer::new(), Integer::new(), Integer::new(), Integer::new()));
}

struct BufferIntegers;

impl BufferIntegers {
    fn get_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Integer, &mut Integer, &mut Integer, &mut Integer, &mut Integer) -> R,
    {
        BUFFER_INTEGERS.with(|cell| {
            let (x, y, ys, c, t) = &mut *cell.borrow_mut();
            f(x, y, ys, c, t)
        })
    }
}

/// Implements Pollard's Rho algorithm with Brent's cycle detection for integer factorization.
///
/// ## Arguments
/// * `n` - The composite number to factorize (must be positive and odd).
/// * `ctx` - A Context with n as the modulus
/// ## Returns
/// * `Some(factor)` - A non-trivial factor of `n` if found.
/// * `None` - If the algorithm fails to find a factor after a reasonable number of iterations.
/// 
/// ## Notes
/// you need to provide it all the variables 
pub fn pollard_rho_brent(n: &Integer, ctx: &mut Context, g: &mut Integer) -> Option<()> {
    // println!("running pollard. n: {}", n);
    g.assign(0);

    BufferIntegers::get_mut(|x, y, ys, c, t| {
        
        RAND_STATE.with(|rand_state| {
            let mut rng = rand_state.borrow_mut();
            c.assign(Integer::random_bits(10, &mut *rng));
            y.assign(Integer::random_bits(10, &mut *rng));
        });

        ctx.to_montgomery_mut(c);
        ctx.to_montgomery_mut(y);

        let iterations = 4096;
        let mut r = 1;
        for _ in 0..19 {
            x.assign(&*y);
    
            // Advance y by r steps
            for _ in 0..r {
                f(y, &c, ctx);
            }

            let mut k = 0;
            while k < r && *g < 2 {
                g.assign(&ctx.r_mod_n);
                ys.assign(&*y);

                // Accumulate product of differences
                for _ in 0.. min(iterations, r - k){
                    f(y, &c, ctx);
                    t.assign(&*x);
                    *x -= ctx.wrap(&*y);
                    *g *= ctx.wrap(&*x);
                    x.assign(&*t);
                }

                g.gcd_mut(n); // note that here, g is no longer in montgomery form as n is odd
                k += iterations;
            }

            if *g > 1 {
                break;
            }
            r <<= 1;  // r *= 2
        }

        if *g == 1 {
            // println!("Failed to find a factor");
            return None;
        }

        if *g == *n {
            // Fallback: try to find a factor
            // println!("rarely should happen");
            for _ in 0..128 {
                g.assign(&ctx.r_mod_n);
                for _ in 0..128 {
                    f(ys, &c, ctx);
                    t.assign(&*x);
                    *x -= ctx.wrap(&*ys);
                    *g *= ctx.wrap(&*x);
                    x.assign(&*t);
                }
                g.gcd_mut(n);
                if *g > 1 && *g < *n {
                    return Some(());
                }
            }
        }

        if *g == *n {
            // println!("Failed to find a factor");
            return None;
        }

        Some(())
    })
}
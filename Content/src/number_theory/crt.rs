use std::{
    borrow::Borrow, cell::RefCell
};


use rug::{Assign, Integer};

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
            let (g, x, y) = &mut *cell.borrow_mut();
            f(g, x, y)
        })
    }
}
// to use:
// let buffer = get_buffer();
// let x = buffer.get_ints();

/// Solves the Chinese Remainder Theorem for two congruences: x ≡ a (mod m) and x ≡ b (mod n).
///
/// Given integers a, m, b, n, finds x and M such that x ≡ a (mod m), x ≡ b (mod n), and M is the
/// modulus of the solution (typically lcm(m, n) / gcd(m, n)). Returns None if no solution exists.
///
/// # Arguments
/// * `a` - First residue (x ≡ a mod m).
/// * `m` - First modulus.
/// * `b` - Second residue (x ≡ b mod n).
/// * `n` - Second modulus.
///
/// # Returns
/// * `Some((x, m))` - Solution x and modulus M if a solution exists.
/// * `None` - If the congruences have no solution (i.e., (b - a) is not divisible by gcd(m, n)).
///
/// # Type Constraints
/// * `a`, `m`, `b`, `n`: Must be either `Integer`` or `&Integer`.
pub fn chinese_remainder_theorem<A, M, B, N>(a: A, m: M, b: B, n: N) -> Option<(Integer, Integer)> 
where 
    A: Into<Integer>, M: Into<Integer>, B: Borrow<Integer>, N: Borrow<Integer>,
{
    let (mut a, mut m, b, n) = (a.into(), m.into(), b.borrow(), n.borrow());
    chinese_remainder_theorem_mut(&mut a, &mut m, b, n).map(|_| (a, m))
}

/// Mutable version of the Chinese Remainder Theorem, updating a and m in place.
///
/// Solves x ≡ a (mod m) and x ≡ b (mod n), updating `a` to the solution and `m` to the new modulus.
/// Uses the extended GCD to find coefficients and checks if a solution exists.
///
/// # Arguments
/// * `a` - First residue, updated to the solution x.
/// * `m` - First modulus, updated to the new modulus (lcm(m, n) / gcd(m, n)).
/// * `b` - Second residue.
/// * `n` - Second modulus.
///
/// # Returns
/// * `Some(())` - If a solution is found, with `a` and `m` updated.
/// * `None` - If no solution exists (i.e., (b - a) is not divisible by gcd(m, n)).
pub fn chinese_remainder_theorem_mut(a: &mut Integer, m: &mut Integer, b: &Integer, n: &Integer) -> Option<()> {
    // Get temporary buffers for GCD computation
    BufferIntegers::get_mut(|g, x, y| {
        // Compute GCD(m, n) and Bézout coefficients: g = gcd(m, n), m*x + n*y = g
        g.assign(&*m);
        x.assign(n);
        g.extended_gcd_mut(x, y);
    
        // y = b - a
        y.assign(b - &*a);
    
        // Check if solution exists: (b - a) must be divisible by gcd(m, n)
        if !y.is_divisible(&g) {
            return None;
        }
    
        // Compute solution: ((b - a) * x % n / g * m) + a, and store solution in `a`
        *y *= &*x;  *y %= n;
        y.div_exact_mut(&g);
        *y *= &*m;
        *a += &*y;
    
        // Update modulus: m = m * n / g
        *m *= n;
        m.div_exact_mut(&g);
    
        if a.is_negative() {
            *a += &*m;
        }
    
        Some(())
    })
}



#[cfg(test)]
mod tests {
    use super::*;
    use rug::{rand::RandState, Integer};
    /// Generates a random positive Integer in [1, max]
    fn random_integer(rng: &mut RandState, max: &Integer) -> Integer {
        Integer::from(max.random_below_ref(rng)) + 1
    }

    /// Tests CRT solution against direct verification
    fn test_crt_case(a: &Integer, m: &Integer, b: &Integer, n: &Integer) {
        if let Some((x, modulus)) = chinese_remainder_theorem(a, m, b, n) {
            // Verify x ≡ a mod m
            assert!(x.is_congruent(&a, &m), "x ≡ a mod m failed for a={a}, m={m}, b={b}, n={n}");
            
            // Verify x ≡ b mod n
            assert!(x.is_congruent(&b, &n), "x ≡ b mod n failed for a={a}, m={m}, b={b}, n={n}");
            
            // Verify M is valid
            assert!(modulus == m.clone().lcm(n), "Invalid modulus M");
        } else {
            // Verify no solution should exist
            let g = Integer::from(m.gcd_ref(n));
            let difference = Integer::from(b - a);
            assert!(!difference.is_divisible(&g), 
                "CRT returned None but solution exists for a={a}, m={m}, b={b}, n={n}");
        }
    }
    #[test]
    fn test_crt() {
        let mut rng = RandState::new();
        let iterations = 1_000_000;
        let bits = 300;
        for _ in 0..iterations {
            let m = Integer::from(Integer::random_bits(bits, &mut rng));
            let n = Integer::from(Integer::random_bits(bits, &mut rng));
            let a = random_integer(&mut rng, &m);
            let b = random_integer(&mut rng, &n);
            test_crt_case(&a, &m, &b, &n);
        }
    }
}
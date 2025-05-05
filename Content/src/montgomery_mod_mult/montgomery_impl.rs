use std::ops::{AddAssign, MulAssign, ShrAssign, SubAssign};

use rug::{
    Assign, Integer,
    ops::{NegAssign, SubFrom},
};

use super::WrapWithCtx;

/// Montgomery multiplication context holding precomputed constants
/// for efficient modular arithmetic operations.
///
/// Contains:
/// - n: The modulus
/// - n2: 2*n (used for keeping values in [0, 2n) range)
/// - n_inv: -n⁻¹ mod r (for Montgomery reduction)
/// - r: A power of 2 > 4n
/// - r_squared_mod_n: r² mod n (for conversion to Montgomery form)
/// - r_bit_length: Bit length of r (aligned to 32-bit words)
#[derive(Debug, Clone)]
pub struct Context {
    pub n: Integer,           // Modulus
    n2: Integer,              // 2 * n
    n_inv: Integer,           // -n^(-1) mod r
    pub r_mod_n: Integer,     // r mod n
    r_squared_mod_n: Integer, // r^2 mod n
    r_cubed_mod_n: Integer,    // r^3 mod n
    r_bit_length: u32,        // Bit length of r
    t: Integer,               // A scratch buffer for storing values used in intermediate calculations
    t2: Integer
}

impl Context {
    /// Creates a new Montgomery context for the given modulus.
    /// # Arguments
    /// * `n` - The modulus (must be odd and > 1)
    pub fn new(n: Integer) -> Self {
        // debug_// assert!(n.is_odd() && n > 1, "Modulus must be odd and > 1");

        // n2 = 2 * n
        let n2 = Integer::from(2 * &n);

        // Calculate r as a power of 2, aligned to 32-bit words for performance
        let r_bit_length = (n.significant_bits() + 2).next_multiple_of(gmp_mpfr_sys::gmp::LIMB_BITS as u32);

        // Compute n_inv = n⁻¹ mod r using Hensel lifting
        let mut n_inv: Integer = n.clone();
        let mut accuracy = 3;
        let mut temp = Integer::new();

        while accuracy < r_bit_length {
            accuracy *= 2;
            temp.assign(&n_inv * &n);
            temp.sub_from(2);
            n_inv *= &temp;
        }
        n_inv.keep_bits_mut(r_bit_length);
        n_inv.neg_assign(); // n_inv = -n⁻¹ mod r

        // Calculate r^2 mod n
        let mut r_squared_mod_n = Integer::ZERO;
        r_squared_mod_n.set_bit(r_bit_length, true); // r
        n_inv += &r_squared_mod_n; // make n_inv positive

        r_squared_mod_n.set_bit(r_bit_length, false); // set back to 0
        r_squared_mod_n.set_bit(2 * r_bit_length, true); // r^2
        r_squared_mod_n %= &n; // r_squared_mod_n is r^2 mod n

        // perform reduction on r^2 to get r mod n
        let mut r_mod_n = r_squared_mod_n.clone();
        let mut t = r_mod_n.clone();
        t.keep_bits_mut(r_bit_length);
        t *= &n_inv;
        t.keep_bits_mut(r_bit_length);
        t *= &n;
        r_mod_n += &t;
        r_mod_n.shr_assign(r_bit_length);

        // perform reduction on r^4 to get r^3 mod n
        let mut r_cubed_mod_n = r_squared_mod_n.clone() * &r_squared_mod_n;
        let mut t2 = r_cubed_mod_n.clone();
        t2.keep_bits_mut(r_bit_length);
        t2 *= &n_inv;
        t2.keep_bits_mut(r_bit_length);
        t2 *= &n;
        r_cubed_mod_n += &t2;
        r_cubed_mod_n.shr_assign(r_bit_length);

        Self {
            n,
            n2,
            n_inv,
            r_mod_n,
            r_squared_mod_n,
            r_cubed_mod_n,
            r_bit_length,
            t,
            t2
        }
    }

    /// Performs Montgomery reduction: x * r^(-1) mod n. Assumes x < r * n.
    /// Result is in [0, 2n).
    #[inline]
    pub fn reduce<X: Into<Integer>>(&mut self, x: X) -> Integer {
        let mut x = x.into();
        self.reduce_mut(&mut x);
        x
    }

    /// Performs Montgomery reduction in-place: x * r^(-1) mod n. Assumes x < r * n.
    /// Result is in [0, 2n).
    #[inline]
    pub fn reduce_mut(&mut self, x: &mut Integer) {
        // assert!(x < &mut self.n2.clone().square());
        self.t.assign(x.keep_bits_ref(self.r_bit_length)); // x mod r
        self.t *= &self.n_inv;
        self.t.keep_bits_mut(self.r_bit_length);
        self.t *= &self.n;
        *x += &self.t;
        x.shr_assign(self.r_bit_length); // x /= r
        // assert!(*x < self.n2);
        // assert!(!x.is_negative());
    }

    /// Montgomery multiplication: computes a * b in Montgomery form.
    /// Both a and b must be in Montgomery representation.
    #[inline]
    pub fn mul<A: Into<Integer>, B>(&mut self, a: A, b: B) -> Integer
    where
        Integer: MulAssign<B>,
    {
        let mut a = a.into();
        self.mul_assign(&mut a, b);
        a
    }

    /// In-place Montgomery multiplication: computes a *= b in Montgomery form.
    #[inline]
    pub fn mul_assign<B>(&mut self, a: &mut Integer, b: B)
    where
        Integer: MulAssign<B>,
    {
        *a *= b;
        // assert!(*a < self.n2.clone().square());
        self.reduce_mut(a);
    }
    
    #[inline]
    pub fn invert<A: Into<Integer>>(&mut self, a: A) -> Option<Integer> {
        let mut a = a.into();
        self.invert_mut(&mut a)?;
        Some(self.from_montgomery(&a))
    }

    #[inline]
    pub fn invert_mut(&mut self, a: &mut Integer) -> Option<()> {
        if a.invert_mut(&self.n).is_err() {
            return None;
        }
        
        *a *= &self.r_cubed_mod_n;
        self.reduce_mut(a);

        Some(())
    }

    /// Squares a number in Montgomery form.
    #[inline]
    pub fn square<X: Into<Integer>>(&mut self, x: X) -> Integer {
        let mut x = x.into();
        self.square_mut(&mut x);
        x
    }

    /// Squares a number in Montgomery form in-place.
    #[inline]
    pub fn square_mut(&mut self, a: &mut Integer) {
        // assert!(*a < self.n2);
        a.square_mut();
        self.reduce_mut(a);
    }

    /// Cubes a number in Montgomery form.
    #[inline]
    pub fn cube<X: Into<Integer>>(&mut self, x: X) -> Integer {
        let mut x = x.into();
        self.cube_mut(&mut x);
        x
    }

    /// Cubes a number in Montgomery form in-place.
    #[inline]
    pub fn cube_mut(&mut self, a: &mut Integer) {
        // assert!(*a < self.n2);
        self.t2.assign(&*a);
        self.square_mut(a);
        // assert!(*a < self.n2);
        *a *= &self.t2;
        self.reduce_mut(a);
    }

    /// Add by 1 in Montgomery Form.
    #[inline]
    pub fn increment<X: Into<Integer>>(&mut self, x: X) -> Integer {
        let mut x = x.into();
        self.increment_mut(&mut x);
        x
    }

    /// Add by 1 in Montgomery form in-place.
    #[inline]
    pub fn increment_mut(&mut self, x: &mut Integer) {
        *x += &self.r_mod_n;
        if *x == self.n2 {
            *x = Integer::ZERO;
        }
    }

    /// Addition in Montgomery form, ensures result < 2n.
    #[inline]
    pub fn add<A: Into<Integer>, B>(&mut self, a: A, b: B) -> Integer
    where
        Integer: AddAssign<B>,
    {
        let mut a = a.into();
        self.add_assign(&mut a, b);
        a
    }

    /// In-place addition in Montgomery form, ensures result < 2n.
    #[inline]
    pub fn add_assign<B>(&mut self, a: &mut Integer, b: B)
    where
        Integer: AddAssign<B>,
    {
        *a += b;
        if *a >= self.n2 {
            *a -= &self.n2;
        }
    }

    /// Subtract by 1 in Montgomery Form.
    #[inline]
    pub fn decrement<X: Into<Integer>>(&mut self, x: X) -> Integer {
        let mut x = x.into();
        self.decrement_mut(&mut x);
        x
    }

    /// Subtract by 1 in Montgomery Form in-place.
    #[inline]
    pub fn decrement_mut(&mut self, x: &mut Integer) {
        if *x == Integer::ZERO {
            x.assign(&self.n - Integer::ONE);
        } else {
            *x -= &self.r_mod_n;
        }
    }

    /// Subtraction in Montgomery form, ensures non-negative result.
    #[inline]
    pub fn sub<A: Into<Integer>, B>(&mut self, a: A, b: B) -> Integer
    where
        Integer: SubAssign<B>,
    {
        let mut a = a.into();
        self.sub_assign(&mut a, b);
        a
    }

    /// In-place subtraction in Montgomery form, ensures result < 2n.
    #[inline]
    pub fn sub_assign<B>(&mut self, a: &mut Integer, b: B)
    where
        Integer: SubAssign<B>,
    {
        *a -= b;
        if a.is_negative() {
            *a += &self.n2;
        }
    }

    /// Converts a number to Montgomery form: x * r mod n.
    /// It is assumed that x < 2n.
    #[inline]
    pub fn to_montgomery<X: Into<Integer>>(&mut self, x: X) -> Integer {
        let mut x = x.into();
        self.to_montgomery_mut(&mut x);
        x
    }

    /// Converts a number to Montgomery form: x * r mod n.
    /// It is assumed that x < 2n.
    #[inline]
    pub fn to_montgomery_mut(&mut self, x: &mut Integer) {
        // assert!(x < &mut self.n2.clone());
        x.mul_assign(&self.r_squared_mod_n);
        self.reduce_mut(x);
    }

    /// Converts from Montgomery form to standard form.
    #[inline]
    pub fn from_montgomery<X: Into<Integer>>(&mut self, x: X) -> Integer {
        let mut x = x.into();
        self.from_montgomery_mut(&mut x);
        x
    }

    /// Converts from Montgomery form to standard form.
    /// The result will be in the range [0, n).
    #[inline]
    pub fn from_montgomery_mut(&mut self, x: &mut Integer) {
        self.reduce_mut(x);
        if *x >= self.n {
            *x -= &self.n;
        }
    }

    pub fn modulus(&mut self) -> Integer {
        self.n.clone()
    }

    pub fn one(&mut self) -> Integer {
        self.r_mod_n.clone()
    }

    /// Changes the modulus to a new value.
    pub fn change_mod(&mut self, n: &Integer) {
        self.n.assign(n);
        
        // n2 = 2 * n
        self.n2.assign(2 * n);

        // Calculate r as a power of 2, aligned to 32-bit words for performance
        self.r_bit_length = (n.significant_bits() + 2).next_multiple_of(gmp_mpfr_sys::gmp::LIMB_BITS as u32);

        // Compute n_inv = n⁻¹ mod r using Hensel lifting
        self.n_inv.assign(n);
        let mut accuracy = 3;

        while accuracy < self.r_bit_length {
            accuracy *= 2;
            self.t.assign(&self.n_inv * n);
            self.t.sub_from(2);
            self.n_inv *= &self.t;
        }
        self.n_inv.keep_bits_mut(self.r_bit_length);
        self.n_inv.neg_assign(); // n_inv = -n⁻¹ mod r

        // Calculate r^2 mod n
        self.r_squared_mod_n = Integer::ZERO;
        self.r_squared_mod_n.set_bit(self.r_bit_length, true); // r
        self.n_inv += &self.r_squared_mod_n; // make n_inv positive

        self.r_squared_mod_n.set_bit(self.r_bit_length, false); // set back to 0
        self.r_squared_mod_n.set_bit(2 * self.r_bit_length, true); // r^2
        self.r_squared_mod_n %= n; // r_squared_mod_n is r^2 mod n

        // perform reduction on r^2 to get r mod n
        self.r_mod_n.assign(&self.r_squared_mod_n);
        self.t.assign(&self.r_mod_n);
        self.t.keep_bits_mut(self.r_bit_length);
        self.t *= &self.n_inv;
        self.t.keep_bits_mut(self.r_bit_length);
        self.t *= n;
        self.r_mod_n += &self.t;
        self.r_mod_n.shr_assign(self.r_bit_length);

        // perform reduction on r^4 to get r^3 mod n
        self.r_cubed_mod_n.assign(&self.r_squared_mod_n * &self.r_squared_mod_n);
        self.t2.assign(&self.r_cubed_mod_n);
        self.t2.keep_bits_mut(self.r_bit_length);
        self.t2 *= &self.n_inv;
        self.t2.keep_bits_mut(self.r_bit_length);
        self.t2 *= n;
        self.r_cubed_mod_n += &self.t2;
        self.r_cubed_mod_n.shr_assign(self.r_bit_length);
    }

    pub(crate) fn assign(&mut self, other: &Context) {
        self.n.assign(&other.n);
        self.n2.assign(&other.n2);
        self.n_inv.assign(&other.n_inv);
        self.r_mod_n.assign(&other.r_mod_n);
        self.r_squared_mod_n.assign(&other.r_squared_mod_n);
        self.r_cubed_mod_n.assign(&other.r_cubed_mod_n);
        self.r_bit_length = other.r_bit_length;
    }

    /// Wraps the value in a wrapper to support operator overloading
    #[inline]
    pub fn wrap<'a, X>(&'a mut self, x: X) -> X::Output
    where
        X: WrapWithCtx<'a>,
    {
        x.wrap(self)
    }
}

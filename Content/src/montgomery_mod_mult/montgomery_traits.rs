use std::ops::{Add, Mul, Sub};

use rug::Integer;

use super::Context;

// ===== Wrapper Types =====

/// Wrapper for owned Integer + Context
pub struct MontgomeryOwned<'a>(pub Integer, pub &'a mut Context);

/// Wrapper for borrowed Integer + Context
pub struct MontgomeryRef<'a>(pub &'a Integer, pub &'a mut Context);

// ===== Operator Implementations =====

macro_rules! impl_montgomery_op_assign {
    ($trait:ident, $method:ident, $op:ident) => {
        // &mut a += &b
        impl<'a> std::ops::$trait<MontgomeryRef<'a>> for Integer {
            fn $method(&mut self, rhs: MontgomeryRef<'a>) {
                rhs.1.$op(self, rhs.0);
            }
        }

        // &mut a += b
        impl<'a> std::ops::$trait<MontgomeryOwned<'a>> for Integer {
            fn $method(&mut self, rhs: MontgomeryOwned<'a>) {
                rhs.1.$op(self, rhs.0);
            }
        }
    };
}

macro_rules! impl_montgomery_op {
    ($trait:ident, $method:ident, $op:ident) => {
        // a + &b
        impl<'a> $trait<&'a Integer> for MontgomeryOwned<'a> {
            type Output = Integer;
            fn $method(self, rhs: &'a Integer) -> Integer {
                self.1.$op(self.0, rhs)
            }
        }

        // a + b
        impl<'a> $trait<Integer> for MontgomeryOwned<'a> {
            type Output = Integer;
            fn $method(self, rhs: Integer) -> Integer {
                self.1.$op(self.0, rhs)
            }
        }

        // &a + &b
        impl<'a> $trait<&'a Integer> for MontgomeryRef<'a> {
            type Output = Integer;
            fn $method(self, rhs: &'a Integer) -> Integer {
                self.1.$op(self.0, rhs)
            }
        }
    };
}

macro_rules! impl_montgomery_op_commutative {
    ($trait:ident, $method:ident, $op:ident) => {
        // &a + b, assuming its a + b = b + a. this avoids cloning.
        impl<'a> $trait<Integer> for MontgomeryRef<'a> {
            type Output = Integer;
            fn $method(self, rhs: Integer) -> Integer {
                self.1.$op(rhs, self.0)
            }
        }
    };
}

// Implement for all operations
// I have no choice but to implement it the other way around (overload on Integer rather than on the wrapper)
impl_montgomery_op_assign!(AddAssign, add_assign, add_assign);
impl_montgomery_op_assign!(MulAssign, mul_assign, mul_assign);
impl_montgomery_op_assign!(SubAssign, sub_assign, sub_assign);

impl_montgomery_op!(Add, add, add);
impl_montgomery_op_commutative!(Add, add, add);
impl_montgomery_op!(Mul, mul, mul);
impl_montgomery_op_commutative!(Mul, mul, mul);
impl_montgomery_op!(Sub, sub, sub);

// Implement &a - b separately:
// Use b to store b - &a (to avoid cloning), then taking the negative of that gives a - b.
impl<'a> Sub<Integer> for MontgomeryRef<'a> {
    type Output = Integer;
    fn sub(self, rhs: Integer) -> Integer {
        -self.1.sub(rhs, self.0)
    }
}

// Assign trait

// ===== Convenience Methods =====

pub trait WrapWithCtx<'a> {
    type Output;
    fn wrap(self, ctx: &'a mut Context) -> Self::Output;
}

// For immutable reference
impl<'a> WrapWithCtx<'a> for &'a Integer {
    type Output = MontgomeryRef<'a>;
    fn wrap(self, ctx: &'a mut Context) -> Self::Output {
        MontgomeryRef(self, ctx)
    }
}

// For owned value
impl<'a> WrapWithCtx<'a> for Integer {
    type Output = MontgomeryOwned<'a>;
    fn wrap(self, ctx: &'a mut Context) -> Self::Output {
        MontgomeryOwned(self, ctx)
    }
}

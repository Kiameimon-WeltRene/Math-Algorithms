use std::{cell::RefCell, str::FromStr};

use rug::{Integer, rand::RandState};

#[cfg(test)]
use super::Context;

// Thread-local random number generator for tests
thread_local! {
    static RAND_STATE: RefCell<RandState<'static>> = RefCell::new(RandState::new());
}

// Generate a random Integer less than the modulus
fn random_below(modulus: &Integer) -> Integer {
    RAND_STATE.with(|rand_state| {
        let mut rand = rand_state.borrow_mut();
        Integer::random_below(modulus.clone(), &mut *rand)
    })
}

// Number of test cases to run for each test
const TEST_CASES: usize = 1000000;

#[test]
fn test_addition() {
    let mut modulus = random_below(&Integer::from_str("1000000000000000000000000000000").unwrap());
    if modulus.is_even() {
        modulus += 1;
    }
    let mut ctx = Context::new(modulus.clone());

    for _ in 0..TEST_CASES {
        // Generate random numbers
        let a = random_below(&modulus);
        let b = random_below(&modulus);

        // Convert to Montgomery form
        let mont_a = ctx.to_montgomery(&a);
        let mont_b = ctx.to_montgomery(&b);

        // Perform addition in Montgomery form
        let mont_sum = ctx.wrap(&mont_a) + &mont_b;

        // Convert back to standard form
        let result = ctx.from_montgomery(mont_sum);

        // Compute expected result
        let mut expected = Integer::from(&a + &b) % &modulus;
        if expected.is_negative() {
            expected += &modulus;
        }

        assert_eq!(result, expected, "Addition failed for a={} b={}", a, b);
    }
}

#[test]
fn test_subtraction() {
    let mut modulus = random_below(&Integer::from_str("1000000000000000000000000000000").unwrap());
    if modulus.is_even() {
        modulus += 1;
    }
    let mut ctx = Context::new(modulus.clone());

    for _ in 0..TEST_CASES {
        // Generate random numbers
        let a = random_below(&modulus);
        let b = random_below(&modulus);

        // Convert to Montgomery form
        let mont_a = ctx.to_montgomery(&a);
        let mont_b = ctx.to_montgomery(&b);

        // Perform subtraction in Montgomery form
        let mont_diff = ctx.wrap(&mont_a) - &mont_b;

        // Convert back to standard form
        let result = ctx.from_montgomery(mont_diff);

        // Compute expected result
        let mut expected = Integer::from(&a - &b) % &modulus;
        if expected.is_negative() {
            expected += &modulus;
        }

        assert_eq!(result, expected, "Subtraction failed for a={} b={}", a, b);
    }
}

#[test]
fn test_multiplication() {
    let mut modulus = random_below(&Integer::from_str("1000000000000000000000000000000").unwrap());
    if modulus.is_even() {
        modulus += 1;
    }
    let mut ctx = Context::new(modulus.clone());

    for _ in 0..TEST_CASES {
        // Generate random numbers
        let a = random_below(&modulus);
        let b = random_below(&modulus);

        // Convert to Montgomery form
        let mont_a = ctx.to_montgomery(&a);
        let mont_b = ctx.to_montgomery(&b);

        // Perform multiplication in Montgomery form
        let mont_prod = ctx.wrap(&mont_a) * &mont_b;

        // Convert back to standard form
        let result = ctx.from_montgomery(mont_prod);

        // Compute expected result
        let expected = Integer::from(&a * &b) % &modulus;

        assert_eq!(result, expected, "Multiplication failed for a={} b={}", a, b);
    }
}

#[test]
fn test_in_place_operations() {
    let mut modulus = random_below(&Integer::from_str("1000000000000000000000000000000").unwrap());
    if modulus.is_even() {
        modulus += 1;
    }

    let mut ctx = Context::new(modulus.clone());
    for _ in 0..TEST_CASES {
        // Generate random numbers
        let a = random_below(&modulus);
        let b = random_below(&modulus);

        // Convert to Montgomery form
        let mont_a = ctx.to_montgomery(&a);
        let mont_b = ctx.to_montgomery(&b);

        // Test multiplication: (a *= b)
        let mut mont_prod = mont_a.clone();
        mont_prod *= ctx.wrap(&mont_b);
        let prod_result = ctx.from_montgomery(&mont_prod);
        let prod_expected = Integer::from(&a * &b) % &modulus;

        // Test addition: a += c
        let mut mont_sum = mont_a.clone();
        mont_sum += ctx.wrap(&mont_b);
        let sum_result = ctx.from_montgomery(&mont_sum);
        let mut sum_expected = Integer::from(&a + &b) % &modulus;
        if sum_expected.is_negative() {
            sum_expected += &modulus;
        }

        // Test subtraction: ((a * b) + c) - b
        let mut mont_diff = mont_a.clone();
        mont_diff -= ctx.wrap(&mont_b);
        let diff_result = ctx.from_montgomery(&mont_diff);
        let mut diff_expected = Integer::from(&a - &b) % &modulus;
        if diff_expected.is_negative() {
            diff_expected += &modulus;
        }

        assert_eq!(prod_result, prod_expected, "Multiplication in test_all failed");
        assert_eq!(sum_result, sum_expected, "Addition in test_all failed");
        assert_eq!(diff_result, diff_expected, "Subtraction in test_all failed");
    }
}

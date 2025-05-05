use std::time::Instant;

use rug::{Integer, rand::RandState};

use super::Context;

/// Benchmarks modular addition using standard and Montgomery arithmetic.
///
/// # Arguments
/// * `iterations` - Number of additions to perform.
/// *279* `bits` - Bit size of the n and random operands.
fn benchmark_addition(iterations: usize, bits: u32) {
    // Initialize random number generator
    let mut rng = RandState::new();

    // Generate a random odd n
    let mut n = Integer::from(Integer::random_bits(bits, &mut rng));
    n.set_bit(bits - 1, true);
    n.set_bit(0, true); // Ensure n is odd

    // Generate random values less than n
    let mut testcases: Vec<Integer> = (0..iterations)
        .map(|_| Integer::from(Integer::random_bits(bits, &mut rng)))
        .collect();

    // Print benchmark header
    println!("\n=== Modular Addition Benchmark ===");
    println!("Iterations: {}, Bit Size: {}", iterations, bits);

    // Standard modular addition: a += y; if a >= n { a -= n }
    let mut a1 = Integer::ZERO;
    let start1 = Instant::now();
    for tc in &testcases {
        a1 += tc;
        if a1 >= n {
            a1 -= &n;
        }
    }
    let duration1 = start1.elapsed();
    let ns_per_op1 = duration1.as_nanos() / iterations as u128;

    // Montgomery modular addition
    let mut ctx = Context::new(n.clone());
    // Convert values to Montgomery form
    let start2 = Instant::now();
    for tc in &mut testcases {
        ctx.to_montgomery_mut(tc);
    }
    let conversion_duration = start2.elapsed();

    // Perform Montgomery additions
    let mut a2 = Integer::ZERO;
    let start3 = Instant::now();
    for tc in &testcases {
        a2 += ctx.wrap(tc);
    }
    let duration3 = start3.elapsed();
    let ns_per_op2 = duration3.as_nanos() / iterations as u128;

    // Verify results
    assert_eq!(a1, ctx.from_montgomery(a2));

    let ratio = ns_per_op2 as f64 / ns_per_op1 as f64;

    println!("Standard Addition:        {:>8} ns/op", ns_per_op1);
    println!("Montgomery Conversion:    {:>8} ns", conversion_duration.as_nanos());
    println!("Montgomery Addition:      {:>8} ns/op", ns_per_op2);
    println!("Ratio (Montgomery/Standard): {:.2}x", ratio);
}

/// Benchmarks modular multiplication using standard and Montgomery arithmetic.
///
/// # Arguments
/// * `iterations` - Number of multiplications to perform.
/// * `bits` - Bit size of the modulus and random operands.
fn benchmark_multiplication(iterations: usize, bits: u32) {
    // Initialize random number generator
    let mut rng = RandState::new();

    // Generate a random odd modulus
    let mut n = Integer::from(Integer::random_bits(bits, &mut rng));
    n.set_bit(0, true); // Ensure n is odd
    n.set_bit(bits - 1, true);

    // Generate random values less than n
    let mut testcases: Vec<Integer> = (0..iterations)
        .map(|_| Integer::from(Integer::random_bits(bits, &mut rng)))
        .collect();

    // Print benchmark header
    println!("\n=== Modular Multiplication Benchmark ===");
    println!("Iterations: {}, Bit Size: {}", iterations, bits);

    // Standard modular multiplication: a *= b; a %= n
    let mut val1 = Integer::ONE.clone();
    let start1 = Instant::now();
    for tc in &testcases {
        val1 *= tc;
        val1 %= &n;
    }
    let duration1 = start1.elapsed();
    let ns_per_op1 = duration1.as_nanos() / iterations as u128;

    // Montgomery modular multiplication
    let mut ctx = Context::new(n.clone());
    // Convert values to Montgomery form
    let start2 = Instant::now();
    for tc in &mut testcases {
        ctx.to_montgomery_mut(tc);
    }
    let conversion_duration = start2.elapsed();

    // Perform Montgomery multiplications
    let mut val2 = ctx.one();
    let start3 = Instant::now();
    for tc in &testcases {
        val2 *= ctx.wrap(tc);
    }
    let duration3 = start3.elapsed();
    let ns_per_op2 = duration3.as_nanos() / iterations as u128;

    // Verify results
    assert_eq!(val1, ctx.from_montgomery(val2));

    let ratio = ns_per_op2 as f64 / ns_per_op1 as f64;

    println!("Standard Multiplication:   {:>8} ns/op", ns_per_op1);
    println!("Montgomery Conversion:    {:>8} ns", conversion_duration.as_nanos());
    println!("Montgomery Multiplication: {:>8} ns/op", ns_per_op2);
    println!("Ratio (Montgomery/Standard): {:.2}x", ratio);
}

/// Runs benchmarks for modular addition and multiplication using standard and Montgomery arithmetic.
///
/// # Arguments
/// * `iterations` - Number of operations to perform in each benchmark.
/// * `bits` - Bit size of the n and random operands.
pub fn benchmark_montgomery(iterations: usize, bits: u32) {
    benchmark_addition(iterations, bits);
    benchmark_multiplication(iterations, bits);
}

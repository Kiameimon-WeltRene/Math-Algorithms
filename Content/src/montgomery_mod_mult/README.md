
# Montgomery Modular Multiplication in Rust

This implementation provides an efficient Montgomery modular multiplication (MMM) system using the `rug` crate for arbitrary-precision arithmetic. It is designed for scenarios requiring fast repeated modular multiplications, such as cryptography or number-theoretic algorithms.

## Overview

The `Context` struct holds precomputed constants and provides methods for performing Montgomery modular arithmetic. It works by converting numbers into a special representation (Montgomery form), where modular arithmetic can be done without using "slower" operations like `/` and `%`. After performing the modular arithmetic with the numbers in Montgomery form, you can convert the numbers back to normal.
Note that this implementation works only for odd moduli.

### Key Concepts
- **Montgomery Form**: A number $x$ is represented as $x \cdot r \mod n$, where $r$ is a power of two greater than $n$. We write this as $\bar{x}$. The following properties hold:
    - $\overline{x + y} \mod n = \bar{x} + \bar{y} \mod n$
    - $\overline{x - y} \mod n = \bar{x} - \bar{y} \mod n$
    - $\overline{x \cdot y} \mod n = \bar{x} \cdot \bar{y} \cdot r^{-1} \mod n$, where $r^{-1}$ is the [modular inverse](https://en.wikipedia.org/wiki/Modular_multiplicative_inverse) of $r \mod n$ (exists if $n$ is odd)

- **Reduction**: For any $x < r \cdot n$, Montgomery Reduction computes $x \cdot r^{-1} \mod n$ — without using `/` or `%`.
    - To multiply $\bar{x}$ and $\bar{y}$ in Montgomery form, multiply them and apply Montgomery Reduction.

- **To Montgomery Form**: If $x < n$, convert $x$ by applying Montgomery Reduction on $x \cdot R$, where $R = r^2 \mod n$.
    - This avoids using `%` to compute $x \cdot r \mod n`.

- **From Montgomery Form**: To convert $\bar{x}$ back to normal form, apply Montgomery Reduction on $\bar{x}$.

All of the above avoids using `/` or `%`, except for a single `%` used when creating a new `Context` (to compute $R = r^2 \mod n$).


If you'd like a more detailed explanation on how MMM works, the [CP-Algorithms](https://cp-algorithms.com/algebra/montgomery_multiplication.html) site explains it well. You can also find a more thorough description on [Wikipedia](https://en.wikipedia.org/wiki/Montgomery_modular_multiplication).

### Modifications
I’ve made some changes to the typical Montgomery Modular Multiplication (MMM) implementation to improve performance (this is also described in the Wikipedia page):

- **Montgomery Form Range**: Instead of restricting numbers in Montgomery form to the range $[0, n)$, I allow them to be in $[0, 2n)$.  
    - This removes the need for an `if`-statement in Montgomery Reduction.
    - A number $x$ in Montgomery form could be either $x \cdot r \mod n$ or that value plus $n$.  
        - Keep this in mind when comparing two Montgomery-form numbers for equality.

- **Modified $r$ Value**:
    - $r$ is chosen so that $r > 4n$, instead of just $r > n$.
        - This ensures that the result of Montgomery Reduction stays within $[0, 2n)$ for inputs less than $(2n)^2$ (since the product of two Montgomery-form numbers can now be up to $(2n-1)^2$).
    - The number of significant bits in $r$ is also set to a multiple of the [limb bit](https://en.wikipedia.org/wiki/Word_(computer_architecture)).
        - This would have allowed faster computation because multiplication modulo $r$ would now be as simple as multiplying up to a certain limb (rather than calculate everything, then keep the necessary limbs), but gmp has removed the `mullo_n` function for some reason...

 

## `Context` Struct

```rs
pub struct Context {
    pub n: Integer,           // Modulus
    n2: Integer,              // 2 * n
    n_inv: Integer,           // -n^(-1) mod r
    pub r_mod_n: Integer,     // r mod n, which is also 1 in Montgomery form 
    r_squared_mod_n: Integer, // r^2 mod n
    r_cubed_mod_n: Integer,   // r^3 mod n
    r_bit_length: u32,        // Bit length of r
    t: Integer,               // A scratch buffer for storing values used in intermediate calculations
    t2: Integer,              // A scratch buffer for storing values used in intermediate calculations
}
```
**NOTE**: do NOT modify or move the values of `n` and `r_mod_n`. They are made public for your convenience if you need a reference to them. 

## Methods


### Setup

- `Context::new(n)`: Initializes the context with modulus `n`.

### Conversions

- `to_montgomery(x)`, `from_montgomery(x)`: Converts `x` to and from Montgomery form respectively.
- `to_montgomery_mut(x)`, `from_montgomery_mut(x)`: The operation is directly applied on `x`.

### Core Arithmetic

- `mul(a, b)`, `add(a, b)`, `sub(a, b)`: Multiplication, addition and subtraction in Montgomery form.
- `mul_assign(a, b)`, `add_assign(a, b)`, `sub_assign(a, b)`:  The operation is directly applied on `a`. In other words, `a *= b`, `a += b`, and `a -= b` (in Montgomery form).

### Utility Operations

- `increment(x)`, `decrement(x)`: Adds or subtracts 1 in Montgomery form.
- `increment_mut(x)`, `decrement_mut(x)`: The operation is directly applied on `x`.
- `square(x)`, `cube(x)`: Squares or cubes a value in Montgomery form.
- `square_mut(x)`, `cube_mut(x)`: The operation is directly applied on `x`.
- `invert(x)`: calculates the modular inverse of `x` in Montgomery form (if it exists).
- `invert_mut(x)`: The operation is directly applied on `x`.
- `modulus()`, `one()` returns the value of `n` and `r_mod_n` respectively (note that `r_mod_n` is the Montgomery form of `1`).

## Operator Overloading

The `MontgomeryTraits` module provides wrapper types and operator overloads for ergonomic arithmetic:

- `MontgomeryOwned<'a>(Integer, &'a Context)`
- `MontgomeryRef<'a>(&'a Integer, &'a Context)`

These wrappers enable syntax like:

```rs
let mut ctx = Context::new(Integer::from(7));
let a = ctx.to_montgomery(Integer::from(5));
let b = ctx.to_montgomery(Integer::from(3));
let mont_result = ctx.wrap(a) + &b;
let result = ctx.from_montgomery(mont_result);
```

Unfortunately, Rust does not allow this to be applied to assign operators such as `+=` and `*=`. In other words:
```rs
let mut ctx = Context::new(Integer::from(7));
let mut a = ctx.to_montgomery(Integer::from(5));
let b = ctx.to_montgomery(Integer::From(3));
ctx.wrap(&mut a) += &b; // this is not supported by rust as the LHS is not an lvalue
```

As a result, I have instead resorted to overloading the +=, *= and -= operations on `Integer` and `&Integer` for both `MontgomeryOwned` and `MontgomeryRef`. Thus:
 ```rs
let mut ctx = Context::new(Integer::from(7));
let mut a = ctx.to_montgomery(Integer::from(5));
let b = ctx.to_montgomery(Integer::From(3));
a += ctx.wrap(&b); // this does a += &b
```

## Usage Example

```rs
use rug::Integer;
pub mod MontgomeryModMult;
use crate::MontgomeryModMult::Context;

fn main() {
    let modulus = Integer::from(97);
    let mut ctx = Context::new(modulus.clone());

    let a = ctx.to_montgomery(Integer::from(52));
    let b = ctx.to_montgomery(Integer::from(77));
    let mut result = ctx.wrap(&a) * &b;
    let normal = ctx.from_montgomery(result.clone());

    println!("Result: {}", normal); // (52 * 77) mod 97 = 22

    let c = ctx.to_montgomery(Integer::from(63));
    result -= ctx.wrap(c);  // subtracts c from result
    ctx.from_montgomery_mut(&mut result); // converts result back to normal form
    println!("Result: {}", result); // 22 - 63 mod 97 = 61
    
    ctx.to_montgomery_mut(&mut result); // converts result to montgomery form
    ctx.increment_mut(&mut result); // increments result by 1
    let result_squared = ctx.square(result);
    let value = ctx.from_montgomery(result_squared);
    println!("Result: {}", value); // (61 + 1)^2 mod 97 = 61

}
```
As expected, the program would print:
```
Result: 27
Result: 61
Result: 61
```

## Future Improvements

Several enhancements could be made to improve flexibility and usability:

- **More Utility Functions**: 
    - Add commonly used operations such as modular exponentiation, modular inversion, and batch operations (e.g. sum/product of numbers in Montgomery form in a container).
    - Implement helper methods for comparisons, zero-checking, and conversions between different forms (e.g., standard to Montgomery and vice versa).

- **Generic Integer Support**: 
    - Modify the `Context` struct to support other integer types beyond `rug::Integer`, such as native Rust integers (`u64`, `u128`) or other big integer libraries.
    - Use traits to abstract over the integer implementation, allowing easy swapping depending on performance or dependency needs.
    - Support MMM for even moduli.

- **Performance Optimization**:
    - I have implemented a benchkarking function (you can find it under `Benchmark.rs`). Running the following code:
    ```rs
    let iterations = 10_000_000;
    for bits in (100..1000).step_by(100) {
        benchmark_montgomery(iterations, bits);
    }
    ```
    Gives the result shown in `BenchmarkResults.txt`. To my disappointment, benchmarks show that my current implementation is about **15% slower** than directly using the modulo operation, even for large integers. However, I hope that with a deeper understanding of computer architecture and instruction-level optimizations, I can eventually make it significantly faster. My goal is to fine-tune the implementation so that multiplication outperforms direct modulo operations, particularly for integers larger than 200 bits.


- **Better Error Handling**:
    - Introduce a more robust error system for edge cases like non-invertible modulus or invalid input ranges, as well as performing montgomery operations on two numbers in montgomery form with respect to different moduli.

- **Testing & Documentation**:
  - Expand test coverage to ensure correctness across a wide range of inputs.
  - Provide detailed documentation and usage examples to help users integrate the module easily.


## Notes
- `Context` **must** be declared as mutable. This allows it to modify the variable `t`, which is used as a scratch buffer for intermediate calculations.
- **AVOID** performing operations on two numbers in montgomery form if their moduli differ.
- Always convert inputs to Montgomery form before using any of the operations.
- If you need to check for equality of two numbers `a` and `b` in Montgomery form, check if `a == b` or if `b - a == ctx.modulus()` (assuming b > a). 
- After calculations, convert back with `from_montgomery` to get the final result in the standard form.

# Discrete Logarithm (Pohlig–Hellman) in Rust

This project implements the **Pohlig–Hellman algorithm** to solve the discrete logarithm problem:

> Given integers $g$, $h$, and $n$, find an integer $x$ such that:  
> $g^x ≡ h \mod n$

This implementation works for composite moduli $n$, assuming that it is smooth (meaning, it's a product of small prime factors).

---

## Assumptions and Limitations

- This algorithm **assumes all prime factors of n fit within a `u64`**.
- It will **panic** if any factor overflows `u64`.
- If $n$ has large prime factors, the algorithm becomes inefficient or unusable. Therefore, the constraint that all factors must fit within a `u64` was put in place.
- The current implementation is **not optimized for memory allocation** (see [Future Improvements](#future-improvements)).

---

## ⏱️ Performance

- For moduli `n` where is smooth (say, all factors are smaller than `1e9`) and factorizes quickly, this implementation can solve logs in **under a second** for 200–300 bit moduli.
- For larger or less-smooth `n`, execution time increases significantly. See the [Wikipedia page](https://en.wikipedia.org/wiki/Pohlig%E2%80%93Hellman_algorithm) on its time complexity.

---

## 🔁 Return Value

The function returns a tuple:

```rust
Option<(exponent, period)>
```

If it unwraps to `Some`, This means all solutions to the discrete logarithm `x` satisfy:

```math
x \equiv \text{exponent} \mod \text{period}
```

So any `x = exponent + period * k` is a valid solution.

If it unwraps to `None`, then there does not exist a solution.

---

## Example Code

```rust
use rug::Integer;
pub mod discrete_logarithm;
use math_algorithms::discrete_logarithm::discrete_log;

fn verify_solution(g: &Integer, h: &Integer, n: &Integer, result: &Option<(Integer, Integer)>) {
    match result {
        Some((exponent, period)) => {
            // Compute g^exponent mod n
            let computed_h = g.clone().pow_mod(exponent, n).unwrap();
            println!(
                "Result: exponent = {}, period = {}, g^{} mod n = {} (expected h = {})",
                exponent, period, exponent, computed_h, h
            );
            assert_eq!(computed_h, *h, "Computed h does not match expected h");
            // verify period: g^period ≡ 1 mod n, therefore g^(exponent + period * k) = (g^exponent) * g^(period * k)
            // (g^exponent) * (g^period) * k) = (g^exponent) * 1^k = g^exponent = h mod n
            let g_to_period = g.clone().pow_mod(period , n).unwrap();
            println!("Verification: g^{} mod n = {}", period, g_to_period);
            assert_eq!(g_to_period, Integer::from(1), "Period does not satisfy g^period ≡ 1 mod n");
        }
        None => {
            println!("No solution exists for g = {}, h = {}, n = {}", g, h, n);
            // No direct way to verify non-existence, but we assume the function is correct. You can use online calculators to verify
        }
    }
}

fn main() {
    // Small Example 1: g = 2, h = 8, n = 17 (solution exists: 2^3 = 8 mod 17)
    println!("\nExample 1: g = 2, h = 8, n = 17");
    let g = Integer::from(2);
    let h = Integer::from(8);
    let n = Integer::from(17);
    let result = discrete_log(g.clone(), h.clone(), n.clone());
    verify_solution(&g, &h, &n, &result);

    // Small Example 2: g = 3, h = 5, n = 7 (no solution)
    println!("\nExample 2: g = 3, h = 5, n = 7");
    let g = Integer::from(3);
    let h = Integer::from(5);
    let n = Integer::from(7);
    let result = discrete_log(g.clone(), h.clone(), n.clone());
    verify_solution(&g, &h, &n, &result);

    // Small Example 3: g = 2, h = 4, n = 15 (solution exists: 2^2 = 4 mod 15)
    println!("\nExample 3: g = 2, h = 4, n = 15");
    let g = Integer::from(2);
    let h = Integer::from(4);
    let n = Integer::from(15); // 15 = 3 * 5, both < 2^64
    let result = discrete_log(g.clone(), h.clone(), n.clone());
    verify_solution(&g, &h, &n, &result);

    // Larger Example: n = 10007 * 10009 = 100160063, g = 5, h = 5^12345 mod n
    println!("\nLarger Example: n = 100160063, g = 5, h = 5^12345 mod n");
    let n = Integer::from(10007) * Integer::from(10009); // n = 100160063
    let g = Integer::from(5);
    let exponent = Integer::from(12345);
    let h = g.clone().pow_mod(&exponent, &n).unwrap();
    let result = discrete_log(g.clone(), h.clone(), n.clone());
    verify_solution(&g, &h, &n, &result);

}
```

Output:
```
Example 1: g = 2, h = 8, n = 17
Result: exponent = 3, period = 8, g^3 mod n = 8 (expected h = 8)
Verification: g^8 mod n = 1

Example 2: g = 3, h = 5, n = 7
Result: exponent = 5, period = 6, g^5 mod n = 5 (expected h = 5)
Verification: g^6 mod n = 1

Example 3: g = 2, h = 4, n = 15
Result: exponent = 2, period = 4, g^2 mod n = 4 (expected h = 4)
Verification: g^4 mod n = 1

Larger Example: n = 100160063, g = 5, h = 5^12345 mod n
Result: exponent = 12345, period = 12517506, g^12345 mod n = 39366816 (expected h = 39366816)
Verification: g^12517506 mod n = 1
```

---

## Further Reading

- [Wikipedia: Pohlig–Hellman algorithm](https://en.wikipedia.org/wiki/Pohlig–Hellman_algorithm)
- [Handbook of Applied Cryptography – Chapter 3: Number-Theoretic Problems](https://cacr.uwaterloo.ca/hac/about/chap3.pdf)
- I made a slight modification to the [Pollard's Rho for discrete logarithms](https://en.wikipedia.org/wiki/Pollard's_rho_algorithm_for_logarithms) to solve discrete logarithms with prime order (then, polhig hellman merges the results to find the answer modulo `n`).

---

## Future Improvements

- **Memory allocation optimization**: Currently, vectors, Integers and context structs are not reused efficiently. A better allocator or buffer reuse system would improve performance.
- **A better algorithm**: There are better algorithms for solving the discrete logarithm problem that I could consider implementing instead.

## Additional Note
While implementing this algorithm, I constantly used [this marvellous site](https://www.alpertron.com.ar/DILOG.HTM) to double-check my program's results while debugging it. In doing so, I chanced upon a bug on said website, where it reported the wrong period. I have submitted a bug report to the owner of the website.
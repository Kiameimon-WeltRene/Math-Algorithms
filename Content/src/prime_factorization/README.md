# Prime factorization function

`prime_factorize(&Integer) -> Vec<(Integer, u32)>`

This function takes a reference to a [`rug::Integer`](https://docs.rs/rug/latest/rug/struct.Integer.html) and returns its **prime factorization** as a list of pairs `(prime, exponent)`.

---

## How It Works

The algorithm combines **trial division**, **Pollard's Rho**, and **ECM (Elliptic Curve Method)** to efficiently factor composite numbers of various sizes.

### Walkthrough:

1. **Precomputation (one-time cost):**

    - Generates a list of small primes up to **2.5×10⁷**.
    - Prepares ECM parameters (elliptic curve state, buffer allocations).

2. **Trial Division:**

    - Tries all primes up to **10,000** to quickly remove small factors.

3. **Pollard’s Rho:**

    - For each remaining factor, runs **Pollard’s Rho 3 times** in an attempt to find smaller nontrivial divisors.

4. **ECM (Elliptic Curve Method):**

    - If Pollard's Rho fails, the algorithm switches to ECM.
    - Performs two passes:
        * **Pass 1:** 200 iterations with `B₁ = 50,000` and `B₂ = 2,500,000`.
        * **Pass 2:** 200 iterations with `B₁ = 500,000` and `B₂ = 25,000,000`.

This staged approach ensures a good balance of **speed** and **depth** of factoring.

---

## Memory Optimization

To minimize allocation overhead:

- Most large data structures (e.g., curves, buffers, temporary values) are stored in **thread-local storage**.
- These are **initialized once** on the first call, then **reused** across all subsequent calls to `prime_factorize`, up until the program ends.

This approach dramatically reduces per-call overhead, especially in batch factorization tasks.

---

## Return Format

The result is returned as:

```rs
Vec<(Integer, u32)>
```

Each entry represents a **prime factor** and its **exponent**.

---

## Performance
This algorithm factors numbers with less than 30 digits with ease- typically within a second.
As it mainly relies on ECM for large numbers, its performance would depend on the size of the prime factors of the input (rather than the size of the input itself). If all (except one) of the prime factors are all within `25` digits, the function is likely to succeed within a minute (so even if you threw it the product of a hundred 20-digit primes, it should factor it fairly quickly).

## Further Reading
Prime factorization is a vast topic in Cryptography and Mathematics. I'm honestly unsure what to even link here...
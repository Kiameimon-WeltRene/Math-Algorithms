# A Prime Factorization & Discrete Log CLI Tool

This program provides a **simple command-line interface** to either:

- **Prime factorize** an integer using trial division, Pollard’s Rho, and ECM.
- Compute a **discrete logarithm** using the **Pohlig–Hellman algorithm**.

---

## How to Use

After compiling and running the program (`cargo run --release`), you’ll be prompted with:

```
Enter 1 for prime factorization, 2 for discrete log:
```

### Option 1: Prime Factorization

Choose `1` and input an integer `n` when prompted:

```
Enter n: 1234567891011121314151617181920
```

You will receive output in the form of a `Vec<(Integer, u32)>`, showing the prime factorization of `n`:

```
[(2, 4), (3, 1), (5, 1), (823, 1), (5790586843, 1), (1308443533279, 1)]
```

### Option 2: Discrete Logarithm

Choose `2` and input integers `g`, `h`, and `n` when prompted:

```
Enter g: 3
Enter h: 81
Enter n: 1009
```

If the discrete log exists, you’ll receive a result:

```
Discrete log result: 4
 + 252k
```

This means:

> Any `x ≡ 4 mod 252` satisfies `g^x ≡ h mod n`.

---

## Project Structure

### `discrete_log/`

Contains the implementation of the **Pohlig–Hellman** algorithm.
- Assumes all prime factors of the group order fit within a `u64`, and will **panic** if not.

### `prime_factorize/`

Includes the full factorization engine, which combines:

- Trial division (up to 10,000),
- Pollard’s Rho (Brent-style),
- ECM (with two B1/B2 phases).

### `montgomery_mod_mult/`

Contains Montgomery modular multiplication logic used by both folders above.

### `number_theory/`

Helper utility functions for `discrete_log` and `prime_factorize`

### Other files and folders

Various other scripts and modules are present for:

* Benchmarking
* Stress testing
* Profiling and performance tuning

They are **not necessary** to understand or use the core functionality. All the main features are inside the `Contents/src` folder.

---

## Notes

* The project may contain **incomplete comments, leftover print statements, or test code**.
* This is intentional, as the project is **actively in development**.
* Future updates will include cleanup and optimization.
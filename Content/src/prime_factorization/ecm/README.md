# Elliptic Curve Method (ECM) – Montgomery Curve Implementation

This project implements the **Elliptic Curve Method (ECM)** for integer factorization using **Montgomery curves**. It is designed to be fast and efficient, especially when searching for smaller prime factors (20–60 digits) of large composite numbers.

## What Is ECM?

The **Elliptic Curve Method** is a probabilistic algorithm for finding a **nontrivial factor** of a composite number `n`. It works by performing arithmetic over randomly chosen elliptic curves modulo `n`, hoping that calculations will "accidentally" reveal a factor of `n`.

Unlike traditional factorization methods, ECM is especially effective when `n` has a **small prime factor**, even if `n` itself is very large. In other words, how long it takes to factorize a number depends on the size of the prime factors, not `n`.

### How It Works (Heavily Simplified)

ECM consists of two main phases:

#### 📌 Phase 1: Smoothness Search

1. Choose a **random Montgomery curve** (via something called Suyama's Parameterization) and a point on it.
2. "Multiply" the point by powers of small primes up to a bound `B1`.
3. If the gcd of the point's `Z` coordinate and `n` is not `1` and is not `n`, you have found a factor!

#### 🔁 Phase 2: Extension Search

1. Continue from the point in Phase 1.
2. Use primes between `B1` and a larger bound `B2`.
3. Use "differences" between points to increase the chance of discovering a factor that was missed in Phase 1.

If all goes well, you'll get a factor of `n`. If not, you can retry with a new random curve.

## Why Montgomery Curves?

Montgomery curves are a special form of elliptic curves that allow for **faster arithmetic**, specifically **point doubling and addition**. This reduces the time it takes to run an iteration of ECM. 

### Additional notes
If you are familiar with [Pollard's p-1 factorization algorithm](https://en.wikipedia.org/wiki/Pollard%27s_p_%E2%88%92_1_algorithm), it really isn't very different from it. This technique of factorizing a number applies to any mathematical structure of a specific form (called a [cyclic group](https://en.wikipedia.org/wiki/Cyclic_group)).

I tried writing an more detailed explanation, but it is difficult to do so in a way that someone with little knowledge on elementary number theory can understand... 

## Further Reading
I have referenced many sources in the making of this program, and there are much more which I have yet to cover (for example, ECM with Edwards curves instead, which has shown to be more performant than Montgomery curves).
- [Wikipedia – Elliptic Curve Method](https://en.wikipedia.org/wiki/Lenstra_elliptic-curve_factorization)
- [Wikipedia – Montgomery Curves](https://en.wikipedia.org/wiki/Montgomery_curve)
- [A paper describing an implementation of the Elliptic Curve Method with Montgomery Curves](https://www.hyperelliptic.org/tanja/SHARCS/talks06/Gaj.pdf)
- [Database on Montgomery Curve operations](https://www.hyperelliptic.org/EFD/g1p/auto-montgom-xz.html#doubling-dbl-1987-m-3)
- [This wonderful website by Dario Alpern, which I used to compare the results of my program while debugging](https://www.alpertron.com.ar/ECM.HTM)
# Pollard's Rho (with Brent's Modification)

This project implements **Pollard’s Rho algorithm with Brent’s Modification** to factorize composite numbers quickly—especially when small factors are expected.

It is typically one of the first steps in a factorization pipeline, before switching to more powerful algorithms like ECM, GNFS and SIQS.

## What Is Pollard's Rho?

Pollard’s Rho is a probabilistic algorithm for finding a **nontrivial factor** of a composite number `n`. It uses a "pseudorandom" function to generate a sequence of numbers modulo `n`, hoping that the sequence at some will repeat itself modulo `p`, where `p` is some nontrivial factor of `n` (in other words, there is a cycle in the sequence), before it repeats itself modulo `n`. If this happens, then taking the difference of the values would be a multiple of `p` , but not `n`, and taking the gcd would produce the factor. 

## 🔁 Brent’s Improvement

Instead of checking for cycles using Floyd's “tortoise and hare” method, **Brent’s modification** speeds things up by:

- Doubling the step size (`r`) every iteration
- Using a batched GCD check that reduces the number of expensive `gcd` computations

## Limits & Strategy

This implementation runs **up to** $r = 2^18$ (\~262,000 iterations). If it fails to factor the number, the `prime_factorize` function will proceed to use ECM. This allows Pollard to pick off the "smaller-sized" prime factors before proceeding to factor the rest with ECM.

## Further Reading

- [Wikipedia – Pollard’s Rho Algorithm](https://en.wikipedia.org/wiki/Pollard%27s_rho_algorithm)
- [Brent’s paper on his modification](https://maths-people.anu.edu.au/brent/pd/rpb051i.pdf)
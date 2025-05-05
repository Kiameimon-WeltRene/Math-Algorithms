use rug::{Integer, ops::Pow};

/// Calculates euler's totient function, AKA the group order.
/// Provide it with the prime factorization of a the number which you wish to compute euler's totient for
/// I realised I ultimately didn't need this function, but its still nice to keep it for future use
/// (I didn't need it because I can already determine the factorization of phi_m, so I just need to multiply it up)
fn phi(factorization: &Vec<(u64, u32)>) -> Integer {
    let mut order = Integer::ONE.clone();
    for (p, e) in factorization {
        order *= Integer::from(*p).pow(e - 1) * (p - 1);
    }
    order
}

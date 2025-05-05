pub mod crt;
pub mod generate_primes;

pub use self::crt::chinese_remainder_theorem;
pub use self::crt::chinese_remainder_theorem_mut;
pub use self::generate_primes::generate_primes;

// to use:
// let buffer = get_buffer();
// let x = buffer.get_ints();
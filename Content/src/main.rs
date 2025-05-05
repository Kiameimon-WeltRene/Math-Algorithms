use math_algorithms::{discrete_logarithm::discrete_log, prime_factorization::prime_factorize};
use rug::{
    integer::IsPrime, rand::RandState, Integer
};

// pub mod montgomery_mod_mult;
// pub mod number_theory;
// mod pollards_rho;

// mod DiscreteLog;
// mod PrimeFactorize;

// use crate::{
//     montgomery_mod_mult::Context, number_theory::chinese_remainder_theorem_mut,
//     pollards_rho::pollard_rho_brent,
// };

// use DiscreteLog::discrete_log;
// use PrimeFactorize::prime_factorize;

/// Generates an n-bit semiprime (product of two primes of similar bit length)
pub fn generate_semiprime(bits: &u32, rand: &mut RandState<'static>) -> Integer {
    let half_bits = bits / 2;
    let remaining_bits = bits - half_bits;
    
    loop {
        // Generate two primes of appropriate sizes
        let p = generate_prime(half_bits, rand);
        let q = generate_prime(remaining_bits, rand);
        
        let n = p * q;
        
        // Ensure we got exactly n bits (sometimes product is n-1 bits)
        if n.significant_bits() == *bits {
            return n;
        }
    }
}

/// Helper function to generate a probable prime of specified bit length
fn generate_prime(bits: u32, rand: &mut RandState) -> Integer {
    loop {
        let mut candidate = Integer::from(Integer::random_bits(bits, rand));
        // Set highest and lowest bits to ensure proper bit length and oddness
        candidate.set_bit(bits - 1, true);
        candidate.set_bit(0, true);
        
        if candidate.is_probably_prime(30) != IsPrime::No {
            return candidate;
        }
    }
}
// /// Generate a random smooth integer: all prime factors are < 2^48
// /// `bits`: the approximate bit length of the result
// /// `rand`: random state
// fn generate_smooth_integer(bits: u32, rand: &mut RandState) -> Integer {
//     let mut result = Integer::ONE.clone();
//     let mut total_bits = 0;

//     while total_bits < bits {
//         // Generate a small prime <= 48 bits
//         let prime_bits = rand.below(48) + 1; // Random bit size in range [1, 32]
//         let prime = generate_prime(prime_bits, rand);
//         // Choose a large exponent to boost its contribution to bit length
//         let exp = rand.below(30) + 1; // Choose exponent in a reasonable range

//         let power = Integer::from(&prime).pow(exp);
//         result *= &power;

//         total_bits = result.significant_bits();
//     }

//     result
// }


use std::io::{self, Write};

fn read_integer(prompt: &str) -> Integer {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().parse::<Integer>().expect("Invalid integer input")
}


fn main() {
    println!("Enter 1 for prime factorization, 2 for discrete log:");
    let mut mode_input = String::new();
    io::stdin().read_line(&mut mode_input).unwrap();
    let mode = mode_input.trim();

    match mode {
        "1" => {
            let n = read_integer("Enter n: ");
            println!("{:?}", prime_factorize(&n));
        }
        "2" => {
            let g = read_integer("Enter g: ");
            let h = read_integer("Enter h: ");
            let n = read_integer("Enter n: ");
            match discrete_log(g, h, n) {
                Some(result) => println!("Discrete log result: {}\n + {}k", result.0, result.1),
                None => println!("Discrete log does not exist"),
            };
        }
        _ => {
            println!("Invalid choice. Please enter 1 or 2.");
        }
    }


    // let time = Instant::now();
    // for i in 0..iterations {
    //     let mut n = generate_semiprime(&bits, &mut rng);
    //     n.set_bit(bits - 1, true);
    //     n.set_bit(0, true);
    //     let timer = Instant::now();
    //     println!("trial {} factorizing: {}", i, n);
    //     println!("result: {:?}, ", prime_factorize(&n));
    //     println!("time taken: {:?}", timer.elapsed());
    // }

    // println!("Time taken: {:?}", time.elapsed());
    
   // let n = Integer::from(Integer::random_bits(bits, &mut rng));
    // use this more lol    

    // let bits = 128 as u32;
    // let iterations = 5000;
    // let mut rng = RandState::new();

    // let start3 = Instant::now();
    // let mut stuff = BufferIntegersStruct::new();
    // stuff.bench_struct();
    // println!("Time for function: {:9.3}ms", start3.elapsed().as_secs_f64() * 1000.0);

    // let mut rand = RandState::new();
    // let n = Integer::from_str("5388347234974863733545610468527190698478824948887878090038192").unwrap();
    // let g = Integer::from_str("55037740078947060580632269411584297057859832540715390754701").unwrap();
    // let h: Integer = Integer::from_str("4115201084249034135885137082860845586135872127786048209043269").unwrap();
    // println!("ans: {}", discrete_log(g.clone(), h.clone(), n.clone()).unwrap());
    // let mut factors = convert_factors_u64(prime_factorize(n.clone())).unwrap();
    // let phi_n = phi(&factors);
    // factors = convert_factors_u64(prime_factorize(phi_n.clone())).unwrap();
    // println!("haha I'm crying {}", find_order(&n, &phi_n, &g, &factors))

    // let g = Integer::from_str("2206345404660033224707626148194737062738224119214220246856408289464526442963502281179938189703153398335700249888443038258201876").unwrap();
    // let h = Integer::from_str("2526990524322045969053458552300405579938997072794559914845931654802277411870596548947350114573351050527668094845093179190198550").unwrap();
    // let n = Integer::from_str("4157068119802570964406561528384156097087175239276756364972147800000386088267740156237775387873484813179624438366432517899975633").unwrap();
    // let ctx = Context::new(n.clone());
    // println!("ans: {}", pollard_rho_dlog(&g, &h, &117153192201553, &n, &ctx));

    // let bits = 100;
    // let trials = 10;
    // let mut failed = 0;
    // let mut rand = RandState::new();
    // let mut testcase: Vec<(Integer, Context)> = Vec::with_capacity(trials);
    // for _ in 0..trials {
    //     let mut n = Integer::from(Integer::random_bits(bits, &mut rand));
    //     n.set_bit(0, true);
    //     while n.is_probably_prime(20) == IsPrime::No {
    //         n = Integer::from(Integer::random_bits(bits/2, &mut rand));
    //         n.set_bit(0, true);
    //     }
    //     n *= generate_prime(bits/2, &mut rand);
    //     let ctx = Context::new(n.clone());
    //     testcase.push((n, ctx));
    // }

    // let start = Instant::now();
    // for count in 0..trials {
    //     let (n, ctx) = testcase.pop().unwrap();
    //     let res = match pollard_rho(&n) {
    //         Some(val) => val,
    //         None => Integer::NEG_ONE.clone(),
    //     };
    //     if res == -1 {
    //         failed += 1;
    //     } else if !n.is_divisible(&res) {
    //         println!("error! {}", n);
    //         break;
    //     }
    //     println!("progress: {}", count);

    // }
    // let duration = start.elapsed();
    // println!("Done! time taken: {:?}, failed: {}", duration, failed);

    // let bits = 200; // number of bits
    // let trials = 1000;
    // let mut testcase: Vec<(Integer, Integer, Integer)> = Vec::with_capacity(trials);
    // let mut rand = RandState::new();
    // for _ in 0..trials {
    //     let mut g = Integer::from(Integer::random_bits(bits, &mut rand));
    //     let n = generate_smooth_integer(bits, &mut rand);
    //     g %= &n;
    //     let h: Integer = g.clone().pow_mod(&Integer::from(Integer::random_bits(bits, &mut rand)), &n).unwrap();
    //     testcase.push((g, h, n));
    // }

    // let mut failed = 0;
    // println!("starting tests!");

    // let total_start = Instant::now();

    // for (g, h, n) in testcase {
    //     let start_time = Instant::now();
    //     println!("g = {}, h = {}, n = {}", g, h, n);
    //     let res = match discrete_log(g.clone(), h.clone(), n.clone()) {
    //         Some(val) => val,
    //         None => Integer::NEG_ONE.clone(),
    //     };
    //     let elapsed = start_time.elapsed();
    //     if res == *Integer::NEG_ONE || g.clone().pow_mod(&res, &n).unwrap() != h {
    //         println!("failed... g = {}, h = {}, n = {}, result = {}", g, h, n, res);
    //         break;
    //     }
    //     failed += 1;
    //     println!("Passed! Progress: {}, Time taken: {:?}", failed, elapsed);
    // }
    // let total_elapsed = total_start.elapsed();

    // println!("it took {:?} to calculate the discrete log of {} testcases, each of size approximately {} bits", total_elapsed, trials, bits);
    // // println!("Of all the testcases, {} of them failed...", failed);
}

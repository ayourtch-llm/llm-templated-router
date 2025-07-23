fn main() {
    let mut primes = Vec::new();

    while primes.len() < 100 {
        let mut is_prime = true;
        for &p in &primes {
            if p * p > candidate {
                break;
            }
            if candidate % p == 0 {
                is_prime = false;
                break;
            }
        }
        if is_prime {
            primes.push(candidate);
        }
        candidate += 1;
    }

    for p in primes {
        println!("{}", p);
    }
}

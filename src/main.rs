use std::{env, process::exit};

use num::{range_inclusive, BigInt, BigRational, FromPrimitive};

fn prime(n: &BigRational) -> bool {
    let zero = &BigRational::from_usize(0).unwrap();
    let one = &BigRational::from_usize(1).unwrap();
    let two = &BigRational::from_usize(2).unwrap();
    let three = &BigRational::from_usize(3).unwrap();
    let five = &BigRational::from_usize(5).unwrap();
    let six = &BigRational::from_usize(6).unwrap();
    let seven = &BigRational::from_usize(7).unwrap();

    if n == two || n == three || n == five || n == seven {
        return true;
    }

    if n == one || (n > seven && (n % five == *zero || n % seven == *zero || n % two == *zero || n % three == *zero)) {
        return false;
    }

    if ((n - one) / six).is_integer() || ((n + one) / six).is_integer() {
        let start = BigInt::from(1);
        let stop = n.to_integer();

        for index in range_inclusive(start, stop) {
            let factorsix = &(six * index);
            let fivebase = &(n / (five + factorsix));
            let sevenbase = &(n / (seven + factorsix));

            if (fivebase > one && fivebase.is_integer()) || (sevenbase > one && sevenbase.is_integer()) {
                return false;
            }

            if factorsix > n {
                break;
            }
        }

        return true;
    }

    false
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let buffer = args[1..args.len()].join("");

    if buffer.is_empty() {
        eprintln!("input is empty");
        exit(-1);
    }

    match buffer.trim().parse::<BigInt>() {
        Err(error) => {
            eprintln!("{error}");
            exit(-1);
        }
        Ok(value) => {
            let ratio = &BigRational::from_integer(value);

            match prime(ratio) {
                true => println!("{ratio} is prime"),
                false => println!("{ratio} is composite")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use num::{BigInt, BigRational};
    use indicatif::ParallelProgressIterator;
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

    use crate::prime;

    #[test]
    fn matches_precomputed_primes() {
        let mut lines = include_str!("10000.txt")
            .split('\n')
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        let count = lines.len() as u64;

        lines.sort();

        lines.par_iter().progress_count(count).for_each(|line| {
            let value = line.parse::<BigInt>().unwrap();
            let ratio = BigRational::from_integer(value);
            let is_prime = prime(&ratio);

            assert!(is_prime);
        });
    }
}

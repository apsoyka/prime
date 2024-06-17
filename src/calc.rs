use std::sync::Mutex;

use indicatif::{MultiProgress, ProgressBar};
use num::{range_inclusive, BigInt, BigRational, FromPrimitive, ToPrimitive};
use num_rational::Ratio;
use once_cell::sync::Lazy;
use rayon::iter::{ParallelBridge, ParallelIterator};

enum FactoringResult {
    HasFactors,
    NoFactors,
}

struct NumericConstants<'a> {
    zero: &'a Ratio<BigInt>,
    one: &'a Ratio<BigInt>,
    two: &'a Ratio<BigInt>,
    three: &'a Ratio<BigInt>,
    five: &'a Ratio<BigInt>,
    six: &'a Ratio<BigInt>,
    seven: &'a Ratio<BigInt>,
}

static CONSTANTS: Lazy<NumericConstants> = Lazy::new(|| {
    static ZERO: Lazy<Ratio<BigInt>> = Lazy::new(|| BigRational::from_usize(0).unwrap());
    static ONE: Lazy<Ratio<BigInt>> = Lazy::new(|| BigRational::from_usize(1).unwrap());
    static TWO: Lazy<Ratio<BigInt>> = Lazy::new(|| BigRational::from_usize(2).unwrap());
    static THREE: Lazy<Ratio<BigInt>> = Lazy::new(|| BigRational::from_usize(3).unwrap());
    static FIVE: Lazy<Ratio<BigInt>> = Lazy::new(|| BigRational::from_usize(5).unwrap());
    static SIX: Lazy<Ratio<BigInt>> = Lazy::new(|| BigRational::from_usize(6).unwrap());
    static SEVEN: Lazy<Ratio<BigInt>> = Lazy::new(|| BigRational::from_usize(7).unwrap());

    NumericConstants {
        zero: &ZERO,
        one: &ONE,
        two: &TWO,
        three: &THREE,
        five: &FIVE,
        six: &SIX,
        seven: &SEVEN
    }
});

pub fn prime(number: &BigInt, multi_progress: Option<&MultiProgress>) -> bool {
    let NumericConstants {
        zero,
        one,
        two,
        three,
        five,
        six,
        seven
    } = *CONSTANTS;

    let ratio = &BigRational::from_integer(number.clone());

    if ratio == two || ratio == three || ratio == five || ratio == seven {
        return true;
    }

    if ratio == one || (ratio > seven && (ratio % five == *zero || ratio % seven == *zero || ratio % two == *zero || ratio % three == *zero)) {
        return false;
    }

    if ((ratio - one) / six).is_integer() || ((ratio + one) / six).is_integer() {
        let start = BigInt::from(1);
        let max = BigInt::from_u64(u64::MAX).unwrap();
        let chunk_count = number.min(&max).to_u64().unwrap();
        let chunk_size = &(ratio / max).ceil();

        let progress_bar = multi_progress.clone()
            .and_then(|value| Some(value.add(ProgressBar::new(chunk_count))));
        let range = range_inclusive(start, number.clone());
        let chunk_index = Mutex::new(BigInt::from(0));

        let result = range.par_bridge().try_for_each(|index| {
            let binding = &index;

            let factorsix = &(six * binding);
            let fivebase = &(ratio / (five + factorsix));
            let sevenbase = &(ratio / (seven + factorsix));

            if (fivebase > one && fivebase.is_integer()) || (sevenbase > one && sevenbase.is_integer()) {
                return Err(FactoringResult::HasFactors);
            }

            if factorsix > ratio {
                return Err(FactoringResult::NoFactors);
            }

            let new_index = (&BigRational::from_integer(index) / chunk_size).floor().to_integer();
            let mut guard = chunk_index.lock().unwrap();

            if new_index > *guard {
                *guard = new_index;

                if progress_bar.is_some() { progress_bar.as_ref().unwrap().inc(1); }
            }

            Ok(())
        });

        if multi_progress.is_some() && progress_bar.is_some() {
            let progress_bar = progress_bar.unwrap();
            let multi_progress = multi_progress.unwrap();

            progress_bar.finish();
            multi_progress.remove(&progress_bar);
        }

        match result {
            Err(FactoringResult::HasFactors) => return false,
            Err(FactoringResult::NoFactors) | Ok(()) => return true
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use num::BigInt;
    use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar};
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
        let multi_progress = MultiProgress::new();
        let progress_bar = multi_progress.add(ProgressBar::new(count));

        lines.sort();

        lines.par_iter().progress_with(progress_bar).for_each(|line| {
            let number = line.parse::<BigInt>().unwrap();
            let is_prime = prime(&number, None);

            assert!(is_prime);
        });
    }
}

mod config;

use std::{process::exit, sync::atomic::{AtomicU64, Ordering}};

use clap::Parser;
use config::{setup_logging, Arguments};
use indicatif::{MultiProgress, ProgressBar};
use log::{error, info};
use num::{range_inclusive, BigInt, BigRational, FromPrimitive};
use rayon::iter::{ParallelBridge, ParallelIterator};

type UnitResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

enum FactoringResult {
    HasFactors,
    NoFactors,
}

trait First<T> {
    fn first(&self) -> Option<&T>;
}

impl<T> First<T> for Vec<T> {
    fn first(&self) -> Option<&T> {
        self.get(0)
    }
}

fn prime(number: &BigInt, multi_progress: Option<MultiProgress>) -> bool {
    let zero = &BigRational::from_usize(0).unwrap();
    let one = &BigRational::from_usize(1).unwrap();
    let two = &BigRational::from_usize(2).unwrap();
    let three = &BigRational::from_usize(3).unwrap();
    let five = &BigRational::from_usize(5).unwrap();
    let six = &BigRational::from_usize(6).unwrap();
    let seven = &BigRational::from_usize(7).unwrap();

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
        let min = number.min(&max);
        let (_, binding) = min.to_u64_digits();
        let chunk_count = binding.first().unwrap();
        let chunk_size = (ratio / max).ceil().to_integer();
        let (_, binding) = chunk_size.to_u64_digits();
        let chunk_size = binding.first().unwrap();

        let progress_bar = multi_progress.clone()
            .and_then(|value| Some(value.add(ProgressBar::new(*chunk_count))));
        let chunk_index = AtomicU64::new(0);
        let range = range_inclusive(start, number.clone());

        let result = range.par_bridge().try_for_each(|index| {
            let index = &index;
            let factorsix = &(six * index);
            let fivebase = &(ratio / (five + factorsix));
            let sevenbase = &(ratio / (seven + factorsix));

            if (fivebase > one && fivebase.is_integer()) || (sevenbase > one && sevenbase.is_integer()) {
                return Err(FactoringResult::HasFactors);
            }

            if factorsix > ratio {
                return Err(FactoringResult::NoFactors);
            }

            let (_, binding) = (index / chunk_size).to_u64_digits();
            let new_index = binding.first().unwrap();

            if progress_bar.is_some() && new_index > &chunk_index.load(Ordering::Relaxed) {
                progress_bar.as_ref().unwrap().inc(*chunk_size);
                chunk_index.store(*new_index, Ordering::Relaxed);
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

fn main() -> UnitResult {
    let arguments = Arguments::parse();
    let multi_progress = setup_logging(&arguments.verbosity)?;

    match arguments.number.parse::<BigInt>() {
        Err(error) => {
            error!("{error}");
            exit(-1);
        }
        Ok(value) => {
            match prime(&value, Some(multi_progress)) {
                true => info!("{value} is prime"),
                false => info!("{value} is composite")
            }
        }
    }

    Ok(())
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

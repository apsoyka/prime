mod config;
mod calc;

use std::{fs::read_to_string, io::{stdin, Read}, path::PathBuf};

use clap::Parser;
use config::{setup_logging, Arguments};
use indicatif::ProgressBar;
use log::{debug, info};
use num::{BigInt, FromPrimitive};
use calc::prime;
use scope_functions::Apply;

type ReadResult = Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>>;
type UnitResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn read(path: Option<PathBuf>) -> ReadResult {
    match path {
        Some(path) => {
            let buffer = read_to_string(path)?;
            let output = buffer
                .split('\n')
                .map(|value| value.trim())
                .filter_map(|value| (!value.is_empty()).then_some(value.to_owned()))
                .collect();

            Ok(output)
        }
        None => {
            let mut stdin = stdin();
            let mut buffer = String::new();

            stdin.read_to_string(&mut buffer)?;

            let output = buffer
                .split('\n')
                .map(|value| value.trim())
                .filter_map(|value| (!value.is_empty()).then_some(value.to_owned()))
                .collect();

            Ok(output)
        }
    }
}

fn main() -> UnitResult {
    let arguments = Arguments::parse();
    let multi_progress = setup_logging(&arguments.verbosity)?;
    let input = read(arguments.input_file)?;
    let numbers = input
        .iter()
        .map(|string| string.parse::<BigInt>())
        .collect::<Vec<_>>();
    let count = numbers.len() as u64;
    let max = BigInt::from_usize(9999999999).unwrap();

    let progress_bar = multi_progress.add(ProgressBar::new(count));

    for number in numbers {
        let number = number?;

        let formatted = if number > max {
            number.to_string().apply_mut(|value| value.truncate(10)).to_owned() + "..."
        }
        else {
            number.to_string()
        };

        match prime(&number, Some(&multi_progress)) {
            true => info!("{formatted} -> PRIME"),
            false => debug!("{formatted} -> COMPOSITE")
        }

        progress_bar.inc(1);
    }

    progress_bar.finish();
    multi_progress.remove(&progress_bar);

    Ok(())
}

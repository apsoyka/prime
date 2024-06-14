mod config;
mod calc;

use std::process::exit;

use clap::Parser;
use config::{setup_logging, Arguments};
use log::{error, info};
use num::BigInt;
use calc::prime;

type UnitResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

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

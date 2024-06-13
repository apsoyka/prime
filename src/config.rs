use clap::{Args, Parser};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use log::LevelFilter;

type MultiProgressResult = Result<MultiProgress, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true, propagate_version = true)]
pub struct Arguments {
    #[command(flatten)]
    pub verbosity: Verbosity,

    #[arg(help = "A positive integer in the set of the natural numbers")]
    pub number: String,
}

#[derive(Args)]
#[group(multiple = false)]
pub struct Verbosity {
    #[arg(short = 'd', long = "debug", help = "Enable debugging output")]
    pub debug: bool,

    #[arg(short = 'v', long = "verbose", help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(short = 'q', long = "quiet", help = "Suppress informational messages")]
    pub quiet: bool
}

impl Verbosity {
    fn to_filter(&self) -> LevelFilter {
        if self.debug { LevelFilter::Trace }
        else if self.verbose { LevelFilter::Debug }
        else if self.quiet { LevelFilter::Warn }
        else { LevelFilter::Info }
    }
}

pub fn setup_logging(verbosity: &Verbosity) -> MultiProgressResult {
    let filter = verbosity.to_filter();

    let logger = env_logger::builder()
        .filter_level(filter)
        .format_level(true)
        .format_target(false)
        .format_module_path(false)
        .format_timestamp_secs()
        .parse_default_env()
        .build();

    let multi_progress = MultiProgress::new();

    LogWrapper::new(multi_progress.clone(), logger).try_init()?;

    Ok(multi_progress)
}

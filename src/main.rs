use anyhow::Result;
use clap::{ArgAction, ColorChoice, Parser, ValueHint};
use std::path::PathBuf;
use tracing::Level;

mod gcode;
mod preprocess;
mod slicers;
mod types;

/// Preprocess G-Code files to inject fine-grained acceleration control commands per feature.
///
/// Current supported slicers:{n}
///   * SuperSlicer{n}
#[derive(clap::Parser, Debug)]
#[clap(author, about, version, name = "Preprocess Cancellation", color=ColorChoice::Auto)]
pub(crate) struct Cli {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, action=ArgAction::Count)]
    verbose: u8,
    /// G-code input files
    #[clap(value_hint=ValueHint::FilePath, num_args=1..)]
    pub gcode: Vec<PathBuf>,
}

fn setup_logging(verbose: u8) -> Result<()> {
    let log_level = match verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    // Logging
    tracing_subscriber::fmt().with_max_level(log_level).init();

    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    setup_logging(args.verbose)?;

    for filename in args.gcode {
        tracing::debug!("Processing GCode file: {}", filename.to_string_lossy());

        let result = preprocess::file(&filename);

        match result {
            Ok(_) => {
                tracing::info!("Successfully processed {}", filename.to_string_lossy());
            }
            Err(e) => {
                tracing::error!(
                    "Error processing file {}: {}",
                    &filename.to_string_lossy(),
                    e
                );
                anyhow::bail!("Error: {e}");
            }
        }
    }

    Ok(())
}

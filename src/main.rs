use clap::Parser;

pub mod cli;

use crate::cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if let Err(err) = cli.exec() {
        err.display_cli_errors()?;
        std::process::exit(1);
    }
    Ok(())
}

mod database;
mod run;
mod search;
mod paths;

use clap::{Parser, Subcommand};
use database::DatabaseCommandes;

use crate::{
    run::{RunCommand, run}, search::{SearchCommand, cli_search},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Global verbosity flag
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn exec(self) -> crate::run::errors::run_errors::Result<()> {
        match self.command {
            Commands::Database(args) => {
                args.parse();
            }
            Commands::Search(args) => {
                cli_search(args)?;
            },
            Commands::Run(args) => {
                run(&args.path, args.method)?;
            },
        }
        Ok(())
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Manage database
    #[command(subcommand)]
    Database(DatabaseCommandes),

    /// Search entry in imported databases
    Search(SearchCommand),

    /// Execute inventory, impact assessment and life cycle assessment
    Run(RunCommand),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if let Err(err) = cli.exec() {
        err.display_cli_errors()?;
        std::process::exit(1);
    }
    Ok(())
}

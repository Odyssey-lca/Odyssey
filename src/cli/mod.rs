mod database;
mod run;
mod search;

use clap::{Parser, Subcommand};
use database::DatabaseCommandes;

use crate::cli::{
    run::{run_lca, RunCommand},
    search::{cli_search, SearchCommand},
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
    pub fn exec(self) {
        match self.command {
            Commands::Database(args) => {
                args.parse();
            }
            Commands::Search(args) => match cli_search(args) {
                Ok(()) => {}
                Err(e) => eprintln!("Error while searching: {}", e),
            },
            Commands::Run(args) => match run_lca(&args.path, args.method) {
                Ok(()) => {}
                Err(e) => eprintln!("Error while runing: {}", e),
            },
        }
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

use clap::Args;
use search::Search;
use serde::{Deserialize, Serialize};
use search::errors::Result;

use crate::{paths::SEARCH_PATH, search::format::format_result};
mod format;

#[derive(Debug, clap::ValueEnum, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum ResFormat {
    #[default]
    Default,
    Compact,
    Json,
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct SearchCommand {
    /// Adds files to myapp
    #[arg(short, long)]
    pub unit: Option<String>,

    #[arg(short, long)]
    pub location: Option<String>,

    #[arg(short, long)]
    pub database: Option<String>,

    #[arg(short, long, default_value = "default")]
    pub format: ResFormat,

    #[arg(short, long, default_value_t = 10)]
    pub number: usize,

    pub query: String,
}

pub fn cli_search(args: SearchCommand) -> Result<()> {
    let search_results = Search::load(&SEARCH_PATH)?;
    let search_results = search_results.search(
        &args.query,
        args.database.as_deref(),
        args.location.as_deref(),
        args.unit.as_deref(),
        false,
        Some(args.number),
    )?;
    print!("{}", format_result(search_results, args.format));
    Ok(())
}

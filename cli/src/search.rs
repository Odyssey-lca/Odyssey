use clap::Args;
use search::Search;

use crate::paths::SEARCH_PATH;

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

    #[arg(short, long, default_value_t = false)]
    pub json: bool,

    pub query: String,
}

pub fn cli_search(args: SearchCommand) -> tantivy::Result<()> {
    let search_results = Search::load(&SEARCH_PATH)?;
    let search_results = if args.json {
        search_results.search_for_json(
            &args.query,
            args.database.as_deref(),
            args.location.as_deref(),
            args.unit.as_deref(),
        )?
    } else {
        search_results.search(
            &args.query,
            args.database.as_deref(),
            args.location.as_deref(),
            args.unit.as_deref(),
        )?
    };
    search_results.iter().for_each(|s| println!("{}", s.1));
    Ok(())
}

use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Args;
use comput::lca::Database;
use console::style;
use indicatif::ProgressBar;
use databases::ecospold2::Ecoinvent;
use errors::Result;
use search::Search;
use serde::{Deserialize, Serialize};

use crate::paths::{DATABASES_FILE, DATABASES_PATH, SEARCH_PATH};

use crate::database::{DatabaseKind, delete::{RemoveDatabaseArgs, remove_database}};

#[derive(Debug, Args, Serialize, Deserialize, Clone)]
pub struct ImportDatabaseArgs {
    /// Optional output file
    #[arg(short, long, default_value = "none")]
    pub version: String,

    /// Optional output file
    #[arg(short, long)]
    pub path: PathBuf,

    pub kind: DatabaseKind,
}

pub fn import_database(infos: ImportDatabaseArgs) -> Result<()> {
  let remove_data = RemoveDatabaseArgs{kind: infos.kind.clone(), version: infos.version.clone()};
  let res = try_import_database(infos);
  if res.is_err() {
    remove_database(remove_data)?;
  }
  res
}

fn with_spinner<F, T>(name: &str, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let bar = ProgressBar::new_spinner().with_message(name.to_string());
    bar.enable_steady_tick(Duration::from_millis(100));
    let result = f();
    match result {
        Ok(_) => bar.finish_with_message(format!("{} {}", style("✓").green(), name)),
        Err(_) => bar.finish_with_message(format!("{} {}", style("✖").red(), name)),
    };
    result
}

fn try_import_database(mut infos: ImportDatabaseArgs) -> Result<()> {
    std::fs::create_dir_all(&*DATABASES_PATH)?;

    // Save cache of database in global folder
    let data_path = Path::new(&infos.path);
    let database = with_spinner("Parsing database", || {
        match infos.kind {
            DatabaseKind::Ecoinvent => Ecoinvent::load(&infos.version, data_path, &DATABASES_PATH),
        }
    })?;

    // Register database in databases.json file
    with_spinner("Registering database", || register_database(&mut infos))?;

    // Index search
    with_spinner("Indexing database", || {
      std::fs::create_dir_all(&*SEARCH_PATH)?;
      let search = Search::load(&SEARCH_PATH)?;
      search.index_database(database.list_candidates())?;
      Ok(())
    })?;
    Ok(())
}

fn register_database(infos: &mut ImportDatabaseArgs) -> Result<()> {
    let mut databases: Vec<ImportDatabaseArgs> = vec![];
    if let Ok(f) = File::open(&*DATABASES_FILE) {
        let reader = BufReader::new(&f);
        databases = serde_json::from_reader(reader)?;
        if databases
            .iter()
            .any(|e| e.kind == infos.kind && e.version == infos.version)
        {
            return Ok(());
        }
    }
    let f = File::create(&*DATABASES_FILE)?;
    infos.path = std::fs::canonicalize(&infos.path)?;
    databases.push(infos.to_owned());
    let mut writer = BufWriter::new(f);
    serde_json::to_writer_pretty(&mut writer, &databases)?;
    writer.flush()?;
    Ok(())
}

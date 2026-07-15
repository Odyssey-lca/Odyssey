use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
};

use clap::Args;
use errors::Result;
use search::Search;
use serde::{Deserialize, Serialize};

use crate::paths::{DATABASES_FILE, DATABASES_PATH, SEARCH_PATH};

use crate::database::{import::ImportDatabaseArgs, DatabaseKind};

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct RemoveDatabaseArgs {
    /// Optional output file
    #[arg(short, long, default_value = "none")]
    pub(crate) version: String,

    pub(crate) kind: DatabaseKind,
}

pub fn remove_database(infos: RemoveDatabaseArgs) -> Result<()> {
    std::fs::create_dir_all(&*DATABASES_PATH)?;

    // Remove database from databases.json file
    unregister_database(&infos.kind, &infos.version)?;

    // Delete cache
    let name = format!("{:?}_{}", infos.kind, infos.version);
    let cache_path = &*DATABASES_PATH.join(format!("{:?}", infos.kind)).join(&infos.version);
    std::fs::remove_file(cache_path)?;

    // Delete search index
    std::fs::create_dir_all(&*SEARCH_PATH)?;
    let mut search = Search::load(&SEARCH_PATH)?;
    search.delete_database(&name)?;
    Ok(())
}


pub fn unregister_database(kind: &DatabaseKind, version: &str) -> Result<()> {
  let f = File::open(&*DATABASES_FILE)?;
  let reader = BufReader::new(&f);
  let mut databases: Vec<ImportDatabaseArgs> = serde_json::from_reader(reader)?;
  let f = File::create(&*DATABASES_FILE)?;
  if let Some(index) = databases
      .iter()
      .position(|d| d.kind == *kind && d.version == version)
  {
      databases.remove(index);
  } else {
      eprintln!("No database found!");
      return Ok(());
  }
  let mut writer = BufWriter::new(f);
  if databases.is_empty() {
      write!(&mut writer, "[]")?;
  } else {
      serde_json::to_writer_pretty(&mut writer, &databases)?;
  }
  writer.flush()?;
  Ok(())
}

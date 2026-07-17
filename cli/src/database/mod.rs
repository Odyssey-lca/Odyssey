use std::{fs::File, io::BufReader};

use clap::Subcommand;
use errors::Result;

use crate::paths::DATABASES_FILE;
use serde::{Deserialize, Serialize};

use crate::database::{
    delete::{remove_database, RemoveDatabaseArgs},
    import::{import_database, ImportDatabaseArgs},
};
mod delete;
mod import;

#[derive(Subcommand, Debug)]
#[command(args_conflicts_with_subcommands = true)]
pub enum DatabaseCommandes {
    /// Adds files to myapp
    Import(ImportDatabaseArgs),
    List,
    Remove(RemoveDatabaseArgs),
}

#[derive(Debug, clap::ValueEnum, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseKind {
    Ecoinvent,
}

impl DatabaseCommandes {
    pub fn parse(self) {
        let res = match self {
            DatabaseCommandes::Import(args) => import_database(args),
            DatabaseCommandes::List => list_databases(),
            DatabaseCommandes::Remove(args) => remove_database(args),
        };
        match res {
            Ok(()) => {}
            Err(e) => eprintln!("{}", e),
        }
    }
}

pub fn list_databases() -> Result<()> {
    let f = File::open(&*DATABASES_FILE)?;
    let reader = BufReader::new(&f);
    let databases: Vec<ImportDatabaseArgs> = serde_json::from_reader(reader)?;
    for d in databases {
        println!("{:?} {}", d.kind, d.version);
    }
    Ok(())
}

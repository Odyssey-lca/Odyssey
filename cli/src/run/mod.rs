use std::path::PathBuf;
use std::{
    collections::HashMap,
    path::Path,
};

use clap::Args;
use comput::impacts::ImpactCategory;
use comput::lca::Database;
use databases::load_database;
use matrices::MappedVector;
use search::Search;
use units::parser::parse_unit;
use units::unit::Unit;

pub mod parser;
pub mod errors;
use errors::{Result, RunError};

use crate::paths::DATABASES_PATH;
use crate::run::errors::diagnose_missing_exchange_error;
use crate::run::parser::parse_activity;

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct RunCommand {
    pub path: PathBuf,

    #[arg(short, long, default_value_t = String::from("ef31"))]
    pub method: String,
}

#[derive(Debug)]
pub struct DatabaseInfos {
    name: String,
    version: String,
}

#[derive(Debug)]
pub enum ExchangeLink {
    File { file: String },
    Database { database: DatabaseInfos },
}

#[derive(Debug)]
pub struct Exchange {
    link: ExchangeLink,
    location: Option<String>,
    unit: Option<String>,
    name: Option<String>,
    amount: f64,
    source_path: PathBuf,
    line: (usize, usize),
}

#[derive(Debug)]
pub struct Activity {
    exchanges: Vec<Exchange>,
}

fn import_from_database(
    database_infos: &DatabaseInfos,
    databases: &mut HashMap<String, Box<dyn Database>>,
    rfs: &mut HashMap<String, MappedVector<String>>,
    search: &Search,
    exchange: &Exchange,
    parent_amount: f64,
) -> Result<()> {
    let database_name = format!("{}_{}", database_infos.name, database_infos.version);
    let exchange_name = exchange.name.clone().unwrap();
    let unit = exchange.unit.clone();
    let search_results = search.search(
        &exchange_name,
        Some(&database_name),
        exchange.location.as_deref(),
        unit.as_deref(),
        None,
    )?;
    match &search_results[..] {
        [] => return diagnose_missing_exchange_error(database_infos, search, exchange),
        [a] => {
            let database = databases
                .entry(database_name.clone())
                .or_insert(load_database(
                    &database_infos.name,
                    &database_infos.version,
                    &DATABASES_PATH,
                )?);

            let candidate = database.get_candidate(&a.id).unwrap();

            let local_rf = rfs
                .entry(database_name)
                .or_insert(database.empty_reference_flow());

            let exchange_amount = unit
                .and_then(|u| parse_unit(&u))
                .map(|u| Unit {
                    dimension: u.dimension,
                    scale_to_si: u.scale_to_si * exchange.amount,
                    substance: u.substance,
                })
                .and_then(|u| u.convert(&candidate.unit))
                .map(|u| u.scale_to_si)
                .unwrap_or(exchange.amount);

            local_rf
                .set(a.id.clone(), parent_amount * exchange_amount)
                .unwrap();
        }
        _ => {
            return Err(RunError::MultipleExchange {
                path: exchange.source_path.clone(),
                line: exchange.line,
            })
        }
    }
    Ok(())
}

fn import_from_file(
    path: &Path,
    databases: &mut HashMap<String, Box<dyn Database>>,
    rfs: &mut HashMap<String, MappedVector<String>>,
    search: &Search,
    parent_amount: f64,
) -> Result<()> {
    let activity: Activity = parse_activity(path)?;
    for e in activity.exchanges {
        import_flow(&e, databases, rfs, search, parent_amount)?;
    }
    Ok(())
}

fn import_flow(
    e: &Exchange,
    databases: &mut HashMap<String, Box<dyn Database>>,
    rfs: &mut HashMap<String, MappedVector<String>>,
    search: &Search,
    parent_amount: f64,
) -> Result<()> {
    match &e.link {
        ExchangeLink::File { file } => import_from_file(
            Path::new(file),
            databases,
            rfs,
            search,
            parent_amount * e.amount,
        )
        .map_err(|err| match err {
            RunError::IoError(_) => {
                RunError::FileError { path: e.source_path.clone(), line: e.line }
            }
            _ => err,
        })?,
        ExchangeLink::Database { database } => {
            import_from_database(database, databases, rfs, search, e, parent_amount)?
        }
    }
    Ok(())
}

pub fn run_lca(path: &Path, method: String) -> Result<()> {
    let search = Search::load(&crate::paths::SEARCH_PATH)?;

    let activity: Activity = parse_activity(path)?;

    let mut global_res = ImpactCategory::get_empty_vector(&method);
    print!("\"flow\"");
    for i in 0..global_res.values.len() {
        if let Some(ic) = global_res.mapping.get_by_right(&i) {
            print!(";{:?}", ic);
        }
    }
    println!();

    let mut databases: HashMap<String, Box<dyn Database>> = HashMap::new();
    for e in activity.exchanges {
        let mut res = ImpactCategory::get_empty_vector(&method);
        let mut rfs: HashMap<String, MappedVector<String>> = HashMap::new();
        import_flow(&e, &mut databases, &mut rfs, &search, 1f64)?;

        for (db, rf) in rfs.iter() {
            res += databases.get_mut(db).unwrap().lca(rf, &method)?;
        }
        global_res += res.clone();
        print!("{:?}", e.name.unwrap_or("None".to_string()));
        for i in 0..res.values.len() {
            if res.mapping.get_by_right(&i).is_some() {
                print!(";{:.4e}", res.values[i])
            }
        }
        println!();
    }

    print!("\"all\"");
    for i in 0..global_res.values.len() {
        if global_res.mapping.get_by_right(&i).is_some() {
            print!(";{:.4e}", global_res.values[i])
        }
    }
    println!();

    Ok(())
}

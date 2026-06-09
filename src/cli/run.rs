use std::path::PathBuf;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use clap::Args;
use mapped_sparse_matrix::MappedVector;
use odyssey::comput::impacts::ImpactCategory;
use odyssey::utils::search::Search;
use odyssey::{comput::lca::Database, errors::Result, parsers::load_database};
use serde::{Deserialize, Serialize};
use units_conversion::parser::parse_unit;
use units_conversion::unit::Unit;

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct RunCommand {
    pub path: PathBuf,

    #[arg(short, long, default_value_t = String::from("ef31"))]
    pub method: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseInfos {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExchangeLink {
    File { file: String },
    Database { database: DatabaseInfos },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Exchange {
    #[serde(flatten)]
    link: ExchangeLink,
    location: Option<String>,
    unit: Option<String>,
    name: Option<String>,
    amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
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
    let id = search.search_for_ids(
        &exchange_name,
        Some(&database_name),
        exchange.location.as_deref(),
        unit.as_deref(),
    )?;
    match &id[..] {
        [] => panic!("No matching activity for {}", exchange_name),
        [a] => {
            let database = databases
                .entry(database_name.clone())
                .or_insert(load_database(
                    &database_infos.name,
                    &database_infos.version,
                )?);

            let candidate = database.get_candidate(a).unwrap();

            let local_rf = rfs
                .entry(database_name)
                .or_insert(database.empty_reference_flow());

            let exchange_amount = unit
                .and_then(|u| parse_unit(&u))
                .map(|u| Unit {
                    dimension: u.dimension,
                    scale_to_si: u.scale_to_si * exchange.amount,
                })
                .and_then(|u| u.convert(&candidate.unit))
                .map(|u| u.scale_to_si)
                .unwrap_or(exchange.amount);

            local_rf.set(a.clone(), parent_amount * exchange_amount).unwrap();
        }
        _ => panic!("Multiple matching activities for {}", exchange_name),
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
    let file = File::open(path)?;
    let reader = BufReader::new(&file);
    let activity: Activity = serde_yaml::from_reader(reader)?;

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
        ExchangeLink::File { file } => {
            import_from_file(Path::new(file), databases, rfs, search, parent_amount * e.amount)?
        }
        ExchangeLink::Database { database } => {
            import_from_database(database, databases, rfs, search, e, parent_amount)?
        }
    }
    Ok(())
}

pub fn run_lca(path: &Path, method: String) -> Result<()> {
    let search = Search::new()?;

    let file = File::open(path)?;
    let reader = BufReader::new(&file);
    let activity: Activity = serde_yaml::from_reader(reader)?;

    let mut global_res = ImpactCategory::get_empty_vector(&method);
    print!("\"flow\"");
    for i in 0..global_res.values.len() {
        if let Some(ic) = global_res.mapping.get_by_right(&i) {
            print!(";{:?}", ic);
        }
    }
    println!();

    for e in activity.exchanges {
        let mut res = ImpactCategory::get_empty_vector(&method);
        let mut databases: HashMap<String, Box<dyn Database>> = HashMap::new();
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

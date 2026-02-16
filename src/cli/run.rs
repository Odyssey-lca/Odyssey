use std::path::PathBuf;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use clap::Args;
use odyssey::comput::impacts::ImpactCategory;
use odyssey::utils::search::Search;
use odyssey::{
    comput::lca::Database, errors::Result, parsers::load_database, utils::matrix::MappedVector,
};
use serde::{Deserialize, Serialize};

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
    amount: f64,
) -> Result<()> {
    let database_name = format!("{}_{}", database_infos.name, database_infos.version);
    let exchange_name = exchange.name.clone().unwrap();
    let id = search.search_for_ids(
        &exchange_name,
        Some(&database_name),
        exchange.location.as_deref(),
        exchange.unit.as_deref(),
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

            let local_rf = rfs
                .entry(database_name)
                .or_insert(database.empty_reference_flow());

            local_rf.set(a.clone(), amount * exchange.amount).unwrap();
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
    amount: f64,
) -> Result<()> {
    let file = File::open(path)?;
    let reader = BufReader::new(&file);
    let activity: Activity = serde_yaml::from_reader(reader)?;

    for e in activity.exchanges {
        import_flow(&e, databases, rfs, search, amount)?;
    }
    Ok(())
}

fn import_flow(
    e: &Exchange,
    databases: &mut HashMap<String, Box<dyn Database>>,
    rfs: &mut HashMap<String, MappedVector<String>>,
    search: &Search,
    amount: f64,
) -> Result<()> {
    match &e.link {
        ExchangeLink::File { file } => {
            import_from_file(Path::new(file), databases, rfs, search, amount * e.amount)?
        }
        ExchangeLink::Database { database } => {
            import_from_database(database, databases, rfs, search, e, amount)?
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

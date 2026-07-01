use std::path::PathBuf;
use std::{collections::HashMap, fs::File, io::{BufReader, Read}, path::Path};
use marked_yaml::parse_yaml;

use clap::Args;
use mapped_sparse_matrix::MappedVector;
use odyssey::comput::impacts::ImpactCategory;
use odyssey::utils::search::Search;
use odyssey::{comput::lca::Database, parsers::load_database};
use units_conversion::parser::parse_unit;
use units_conversion::unit::Unit;
use crate::cli::errors::{CliError, Result};

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

pub fn parse_activity(path: &Path) -> Result<Activity> {

    let file = File::open(path)?; 
    let mut reader = BufReader::new(file);
    let mut file_yaml = String::new();
    reader.read_to_string(&mut file_yaml)?;

    let node = parse_yaml(0, &file_yaml).map_err(|err| {
        
        let error_line = match &err {
            marked_yaml::LoadError::TopLevelMustBeMapping(m) => m.line(),
            marked_yaml::LoadError::TopLevelMustBeSequence(m) => m.line(),
            marked_yaml::LoadError::UnexpectedAnchor(m) => m.line(),
            marked_yaml::LoadError::MappingKeyMustBeScalar(m) => m.line(),
            marked_yaml::LoadError::UnexpectedTag(m) => m.line(),
            marked_yaml::LoadError::ScanError(m, _) => m.line(),
            marked_yaml::LoadError::DuplicateKey(_) => 0,
        };
        
        CliError::YamlLoadError {
            path: path.to_path_buf(),
            line: (error_line, error_line),
            details: err.to_string(),
        }
    })?;

    let top_mapping = node.as_mapping().ok_or_else(|| CliError::InvalidYamlFormat {
        path: path.to_path_buf(),
        line: (0,0),
        details: "error: the root of the document must be a YAML mapping".to_string(),
    })?;
 
    let exchanges_node = top_mapping.get("exchanges").ok_or_else(|| CliError::InvalidYamlFormat {
        path: path.to_path_buf(),
        line: (0,0),
        details: "error: missing root key 'exchanges'".to_string(),
    })?;

    let exchanges_seq = exchanges_node.as_sequence().ok_or_else(|| CliError::InvalidYamlFormat {
        path: path.to_path_buf(),
        line: (0,0),
        details: "The 'exchanges' key must contain a sequence (list) of items".to_string(),
    })?;

    let mut activity: Vec<Exchange> = Vec::new();

    for exchange_node in exchanges_seq.iter() {
        
        let start_line = exchange_node.span().start()
            .map(|marker| marker.line())
            .unwrap_or(0);
        
        let end_line = exchange_node.span().end()
            .map(|marker| marker.line())
            .unwrap_or(0);

        let line = (start_line, end_line);
        
        let exchange_map = exchange_node.as_mapping().ok_or_else(|| CliError::InvalidYamlFormat {
            path: path.to_path_buf(),
            line,
            details: "Each exchange item must be a mapping block".to_string(),
        })?;

        // Requiered fields
        let name = Some(exchange_map.get("name")
        .ok_or_else(|| CliError::MissingExchangeName{ path: path.to_path_buf(), line: line })?
        .as_scalar()
        .unwrap()
        .as_str()
        .to_string());
        
        let link = if let Some(file_node) = exchange_map.get("file") {
            ExchangeLink::File {
                file: file_node
                    .as_scalar()
                    .unwrap()
                    .as_str().
                    to_string(),
            }
        } else if let Some(db_node) = exchange_map.get("database") {
            let db_map = db_node.as_mapping().ok_or_else(|| CliError::InvalidYamlFormat {
                path: path.to_path_buf(), line, details: "'database' must be a mapping block".to_string()
            })?;
            let db_name = db_map.get("name")
                .ok_or_else(|| CliError::MissingDatabaseName { path: path.to_path_buf(), line: line })?
                .as_scalar()
                .unwrap()
                .as_str()
                .to_string();
            let db_version = db_map.get("version")
                .ok_or_else(|| CliError::MissingDatabaseVersion { path: path.to_path_buf(), line: line })?
                .as_scalar()
                .unwrap()
                .as_str()
                .to_string();

            ExchangeLink::Database { database: DatabaseInfos { name: db_name, version: db_version }}
        } else {
            return Err(CliError::MissingExchangeLink{ path: path.to_path_buf(), line: line })
        };

        let amount = exchange_map.get("amount")
            .ok_or_else(|| CliError::MissingExchangeAmount{ path: path.to_path_buf(), line: line })?
            .as_scalar()
            .unwrap()
            .as_str()
            .parse::<f64>()
            .map_err(|_| CliError::AmountError { path: path.to_path_buf(), line: line })?;

        // Optionnal fields
        let location = exchange_map.get("location").map(|n| n.as_scalar().unwrap().as_str().to_string());
        let unit = exchange_map.get("unit").map(|n| n.as_scalar().unwrap().as_str().to_string());

        activity.push(Exchange {
            link: link,
            location: location,
            unit: unit,
            name: name,
            amount: amount,
            source_path: path.to_path_buf(),
            line: line,
        });
    }

    Ok(Activity { exchanges: activity })
}

fn diagnose_missing_exchange_error(
    database_infos: &DatabaseInfos,
    search: &Search,
    exchange: &Exchange,
) -> Result<()> {

    let database_name = format!("{}_{}", database_infos.name, database_infos.version);
    let exchange_name = exchange.name.clone().unwrap_or_default();

    let name_found = !search.search_for_ids(&exchange_name, None, None, None)?.is_empty();
    if !name_found {
        return Err(CliError::NameError { path: exchange.source_path.clone(), line: exchange.line });
    }

    let database_found = !search.search_for_ids(&exchange_name, Some(&database_name), None, None)?.is_empty();
    if !database_found {
        return Err(CliError::DatabaseError { path: exchange.source_path.clone(), line: exchange.line });
    }

    if exchange.location.is_some() {
        let location_found = !search.search_for_ids(&exchange_name, Some(&database_name), exchange.location.as_deref(), None)?.is_empty();
        if !location_found {
            return Err(CliError::LocationError { path: exchange.source_path.clone(), line: exchange.line });
        }
    }

    if exchange.unit.is_some() {
        let unit_found = !search.search_for_ids(&exchange_name, Some(&database_name), None, exchange.unit.as_deref())?.is_empty();
        if !unit_found {
            return Err(CliError::UnitError { path: exchange.source_path.clone(), line: exchange.line });
        }
    }

    Err(CliError::MissingExchange { path: exchange.source_path.clone(), line: exchange.line })
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
        [] => return diagnose_missing_exchange_error(database_infos, search, exchange),
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
        _ => return Err(CliError::MultipleExchange{ path: exchange.source_path.clone(), line: exchange.line }),
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
        ExchangeLink::File { file } => {
            import_from_file(Path::new(file), databases, rfs, search, parent_amount * e.amount)
                .map_err(|err| match err {
                    CliError::IoError(_) => CliError::FileError {
                        path: e.source_path.clone(),
                        line: e.line
                    },
                    _ => err,
                })?
        }
        ExchangeLink::Database { database } => {
            import_from_database(database, databases, rfs, search, e, parent_amount)?
        }
    }
    Ok(())
}

pub fn run_lca(path: &Path, method: String) -> Result<()> {
    let search = Search::new()?;

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

use comput::lca::{Activity, ActivityInfos, ExchangeLink};
use databases::{DatabaseInfos, impacts::ImpactMethod};
use marked_yaml::{Node, parse_yaml, types::MarkedMappingNode};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};
use units::parser::parse_unit;

use crate::run::{FileLocator, errors::run_errors::RunError};

use super::errors::run_errors::Result;

fn parse_activity_infos(
    map: &MarkedMappingNode,
    locator: &FileLocator,
    amount: f64,
) -> Result<ActivityInfos<FileLocator>> {
    let name = map
        .get("name")
        .ok_or_else(|| RunError::MissingExchangeName(locator.clone()))?
        .as_scalar()
        .ok_or_else(|| RunError::WrongFieldFormat {
            error_locator: locator.clone(),
            field_name: "name".to_string(),
        })?
        .as_str()
        .to_string();
    let location = match map.get("location") {
        None => Ok(None),
        Some(l) => l
            .as_scalar()
            .ok_or_else(|| RunError::WrongFieldFormat {
                error_locator: locator.clone(),
                field_name: "location".to_string(),
            })
            .map(|l| Some(l.as_str().to_string())),
    }?;
    let unit = match map.get("unit") {
        None => Ok(None),
        Some(u) => u
            .as_scalar()
            .ok_or_else(|| RunError::WrongFieldFormat {
                error_locator: locator.clone(),
                field_name: "unit".to_string(),
            })
            .map(|u| Some(u.as_str().to_string())),
    }?;
    Ok(ActivityInfos { name, location, unit, amount, locator: locator.clone() })
}

fn check_equality<T>(lhs: &Option<T>, rhs: &Option<T>) -> bool
where
    T: PartialEq,
{
    match (lhs, rhs) {
        (None, _) | (_, None) => true,
        (Some(lhs), Some(rhs)) => lhs == rhs,
    }
}

/// This function merges the activity and the exchange infos.
/// It has two purpose :
/// -  Merge information that are only present in one place.
/// -  Provide the activity with the amount and locator information
///    from the exchange.
fn merge_activiy_infos(
    activity_infos: ActivityInfos<FileLocator>,
    exchange_infos: ActivityInfos<FileLocator>,
) -> ActivityInfos<FileLocator> {
    ActivityInfos {
        name: exchange_infos.name,
        location: activity_infos.location.or(exchange_infos.location),
        unit: activity_infos.unit.or(exchange_infos.unit),
        amount: exchange_infos.amount,
        locator: exchange_infos.locator,
    }
}

fn check_unit_conversion(
    activity_unit: String,
    exchange_unit: String,
    mut activity: Activity<FileLocator>,
) -> Result<ExchangeLink<FileLocator>> {
    let parsed_activity_unit = parse_unit(&activity_unit);
    let parsed_exchange_unit = parse_unit(&exchange_unit);
    match (&parsed_exchange_unit, &parsed_activity_unit) {
        (None, _) | (_, None) => Err(RunError::UnitMismatch {
            error_locator: activity.activity_infos.locator,
            activity_unit,
        }),
        (Some(au), Some(eu)) => {
            if let Some(conversion_factor) = au.convert(eu) {
                activity.activity_infos.amount *= conversion_factor;
                Ok(ExchangeLink::Activity(activity))
            } else {
                Err(RunError::UnitMismatch {
                    error_locator: activity.activity_infos.locator,
                    activity_unit,
                })
            }
        }
    }
}

fn parse_file_exchange(
    map: &MarkedMappingNode,
    exchange_infos: ActivityInfos<FileLocator>,
) -> Result<ExchangeLink<FileLocator>> {
    let file_node = map.get("file").unwrap();
    let path = file_node.as_scalar().unwrap().as_str();
    let path = Path::new(path);
    let mut activity = parse_activity(path)?;
    if exchange_infos.name != activity.activity_infos.name {
        return Err(RunError::NameMismatch {
            error_locator: exchange_infos.locator,
            activity_name: activity.activity_infos.name,
        });
    }
    if !check_equality(&exchange_infos.location, &activity.activity_infos.location) {
        return Err(RunError::LocationMismatch {
            error_locator: exchange_infos.locator,
            activity_location: activity.activity_infos.location.unwrap(),
        });
    }
    let exchange_unit = exchange_infos.unit.clone();
    activity.activity_infos = merge_activiy_infos(activity.activity_infos, exchange_infos);
    if !check_equality(&exchange_unit, &activity.activity_infos.unit) {
        let activity_unit = activity.activity_infos.unit.clone().unwrap();
        return check_unit_conversion(activity_unit, exchange_unit.unwrap(), activity);
    }
    Ok(ExchangeLink::Activity(activity))
}

fn parse_database_exchange(
    map: &MarkedMappingNode,
    activity_infos: ActivityInfos<FileLocator>,
) -> Result<ExchangeLink<FileLocator>> {
    let db_node = map.get("database").unwrap();
    let db_map = db_node.as_mapping().ok_or(RunError::InvalidYamlFormat {
        error_locator: activity_infos.locator.clone(),
        details: "'database' must be a mapping block".to_string(),
    })?;
    let db_name = get_string(
        db_map,
        "name",
        || RunError::MissingDatabaseName(activity_infos.locator.clone()),
        || RunError::MissingDatabaseName(activity_infos.locator.clone()),
    )?;
    let db_name = correct_database_name(db_name, &activity_infos.locator.clone())?;
    let db_version = get_string(
        db_map,
        "version",
        || RunError::MissingDatabaseVersion(activity_infos.locator.clone()),
        || RunError::MissingDatabaseVersion(activity_infos.locator.clone()),
    )?;
    Ok(ExchangeLink::Database {
        database_infos: DatabaseInfos { name: db_name, version: db_version },
        activity_infos,
    })
}

pub fn parse_activity(path: &Path) -> Result<Activity<FileLocator>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut file_yaml = String::new();
    reader.read_to_string(&mut file_yaml)?;
    let error_locator = FileLocator { lines: (0, 0), path: path.to_path_buf() };

    let node = parse_yaml(0, &file_yaml).map_err(|err| (err, path))?;

    let top_mapping = node
        .as_mapping()
        .ok_or_else(|| RunError::InvalidYamlFormat {
            error_locator: error_locator.clone(),
            details: "error: the root of the document must be a YAML mapping".to_string(),
        })?;
    let exchanges_node =
        top_mapping
            .get("exchanges")
            .ok_or_else(|| RunError::InvalidYamlFormat {
                error_locator: error_locator.clone(),
                details: "error: missing root key 'exchanges'".to_string(),
            })?;

    let exchanges_seq =
        exchanges_node
            .as_sequence()
            .ok_or_else(|| RunError::InvalidYamlFormat {
                error_locator: error_locator.clone(),
                details: "The 'exchanges' key must contain a sequence (list) of items".to_string(),
            })?;

    let activity_infos = parse_activity_infos(top_mapping, &error_locator, 1.)?;

    let mut exchanges: Vec<ExchangeLink<FileLocator>> = Vec::new();

    for exchange_node in exchanges_seq.iter() {
        let lines = get_exchange_lines(exchange_node);
        let error_locator = FileLocator { lines, path: path.to_path_buf() };

        let exchange_map =
            exchange_node
                .as_mapping()
                .ok_or_else(|| RunError::InvalidYamlFormat {
                    error_locator: error_locator.clone(),
                    details: "Each exchange item must be a mapping block".to_string(),
                })?;

        let amount = get_string(
            exchange_map,
            "amount",
            || RunError::MissingExchangeAmount(error_locator.clone()),
            || RunError::AmountError(error_locator.clone()),
        )?
        .parse::<f64>()
        .map_err(|_| RunError::AmountError(error_locator.clone()))?;

        let activity_infos = parse_activity_infos(exchange_map, &error_locator, amount)?;

        let exchange = if exchange_map.contains_key("file") && exchange_map.contains_key("database")
        {
            Err(RunError::BothDatabaseAndFile(error_locator.clone()))
        } else if exchange_map.contains_key("file") {
            parse_file_exchange(exchange_map, activity_infos)
        } else if exchange_map.contains_key("database") {
            parse_database_exchange(exchange_map, activity_infos)
        } else {
            Err(RunError::MissingExchangeLink(error_locator.clone()))
        }?;

        exchanges.push(exchange);
    }
    Ok(Activity { activity_infos, exchanges })
}

fn get_exchange_lines(node: &Node) -> (usize, usize) {
    let start_line = node.span().start().map(|marker| marker.line()).unwrap_or(0);
    let end_line = node.span().end().map(|marker| marker.line()).unwrap_or(0);
    (start_line, end_line)
}

fn get_string<F1, F2>(
    map: &MarkedMappingNode,
    field: &str,
    missing_error: F1,
    parse_error: F2,
) -> Result<String>
where
    F1: FnOnce() -> RunError,
    F2: FnOnce() -> RunError,
{
    Ok(map
        .get(field)
        .ok_or_else(missing_error)?
        .as_scalar()
        .ok_or_else(parse_error)?
        .as_str()
        .to_string())
}

pub fn correct_database_name(name: String, locator: &FileLocator) -> Result<String> {
    match name.to_lowercase().as_str() {
        "ecoinvent" => Ok("Ecoinvent".to_string()),
        _ => Err(RunError::UnknownDatabase(locator.clone())),
    }
}

pub fn parse_method(name: String) -> Result<ImpactMethod> {
    match name.to_lowercase().as_str() {
        "ef31" => Ok(ImpactMethod::EF31),
        "traci" => Ok(ImpactMethod::TRACI),
        "cml" => Ok(ImpactMethod::CML),
        _ => Err(RunError::UnknownMethod(name)),
    }
}

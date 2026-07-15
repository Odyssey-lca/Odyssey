use std::{fs::File, io::{BufReader, Read}, path::Path};
use marked_yaml::parse_yaml;

use super::{DatabaseInfos, Exchange, ExchangeLink, Activity, errors::{Result, RunError}};

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

        RunError::YamlLoadError {
            path: path.to_path_buf(),
            line: (error_line, error_line),
            details: err.to_string(),
        }
    })?;

    let top_mapping = node
        .as_mapping()
        .ok_or_else(|| RunError::InvalidYamlFormat {
            path: path.to_path_buf(),
            line: (0, 0),
            details: "error: the root of the document must be a YAML mapping".to_string(),
        })?;

    let exchanges_node =
        top_mapping
            .get("exchanges")
            .ok_or_else(|| RunError::InvalidYamlFormat {
                path: path.to_path_buf(),
                line: (0, 0),
                details: "error: missing root key 'exchanges'".to_string(),
            })?;

    let exchanges_seq =
        exchanges_node
            .as_sequence()
            .ok_or_else(|| RunError::InvalidYamlFormat {
                path: path.to_path_buf(),
                line: (0, 0),
                details: "The 'exchanges' key must contain a sequence (list) of items".to_string(),
            })?;

    let mut activity: Vec<Exchange> = Vec::new();

    for exchange_node in exchanges_seq.iter() {
        let start_line = exchange_node
            .span()
            .start()
            .map(|marker| marker.line())
            .unwrap_or(0);

        let end_line = exchange_node
            .span()
            .end()
            .map(|marker| marker.line())
            .unwrap_or(0);

        let line = (start_line, end_line);

        let exchange_map =
            exchange_node
                .as_mapping()
                .ok_or_else(|| RunError::InvalidYamlFormat {
                    path: path.to_path_buf(),
                    line,
                    details: "Each exchange item must be a mapping block".to_string(),
                })?;

        // Requiered fields
        let name = Some(
            exchange_map
                .get("name")
                .ok_or(RunError::MissingExchangeName { path: path.to_path_buf(), line })?
                .as_scalar()
                .ok_or(RunError::MissingExchangeName { path: path.to_path_buf(), line })?
                .as_str()
                .to_string(),
        );

        let link = if let Some(file_node) = exchange_map.get("file") {
            ExchangeLink::File { file: file_node.as_scalar().unwrap().as_str().to_string() }
        } else if let Some(db_node) = exchange_map.get("database") {
            let db_map = db_node.as_mapping().ok_or(RunError::InvalidYamlFormat {
                path: path.to_path_buf(),
                line,
                details: "'database' must be a mapping block".to_string(),
            })?;
            let db_name = db_map
                .get("name")
                .ok_or_else(|| RunError::MissingDatabaseName { path: path.to_path_buf(), line })?
                .as_scalar()
                .unwrap()
                .as_str()
                .to_string();
            let db_version = db_map
                .get("version")
                .ok_or_else(|| RunError::MissingDatabaseVersion { path: path.to_path_buf(), line })?
                .as_scalar()
                .unwrap()
                .as_str()
                .to_string();

            ExchangeLink::Database {
                database: DatabaseInfos { name: db_name, version: db_version },
            }
        } else {
            return Err(RunError::MissingExchangeLink { path: path.to_path_buf(), line });
        };

        let amount = exchange_map
            .get("amount")
            .ok_or(RunError::MissingExchangeAmount { path: path.to_path_buf(), line })?
            .as_scalar()
            .unwrap()
            .as_str()
            .parse::<f64>()
            .map_err(|_| RunError::AmountError { path: path.to_path_buf(), line })?;

        // Optionnal fields
        let location = exchange_map
            .get("location")
            .map(|n| n.as_scalar().unwrap().as_str().to_string());
        let unit = exchange_map
            .get("unit")
            .map(|n| n.as_scalar().unwrap().as_str().to_string());

        activity.push(Exchange {
            link,
            location,
            unit,
            name,
            amount,
            source_path: path.to_path_buf(),
            line,
        });
    }

    Ok(Activity { exchanges: activity })
}

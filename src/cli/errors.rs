use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use colored::Colorize;

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(thiserror::Error, Debug)]
pub enum CliError {
    #[error(transparent)] OdysseyError(#[from] odyssey::errors::OdysseyErrors),
    #[error(transparent)] IoError(#[from] std::io::Error),
    #[error(transparent)] TantivyError(#[from] tantivy::TantivyError),

    #[error("load error in during yaml parsing: {details}")] YamlLoadError { path: PathBuf, line: (usize, usize), details: String },
    #[error("invalid yaml format: {details}")] InvalidYamlFormat { path: PathBuf, line: (usize, usize), details: String },

    #[error("missing exchange name in {path} at line {line:?}")] MissingExchangeName { path: PathBuf, line: (usize, usize) },
    #[error("missing exchange link in {path} at line {line:?}")] MissingExchangeLink { path: PathBuf, line: (usize, usize) },
    #[error("missing database name in {path} at line {line:?}")] MissingDatabaseName { path: PathBuf, line: (usize, usize) },
    #[error("missing database version in {path} at line {line:?}")] MissingDatabaseVersion { path: PathBuf, line: (usize, usize) },
    #[error("missing exchange amount in {path} at line {line:?}")] MissingExchangeAmount { path: PathBuf, line: (usize, usize) },
    #[error("invalid exchange amount format in {path} at line {line:?}")] AmountError { path: PathBuf, line: (usize, usize) },
    #[error("invalid exchange file in {path} at line {line:?}")] FileError { path: PathBuf, line: (usize, usize) },
    #[error("exchange flow not found in {path} at line {line:?}")] MissingExchange { path: PathBuf, line: (usize, usize) },
    #[error("invalid exchange name in {path} at line {line:?}")] NameError { path: PathBuf, line: (usize, usize) },
    #[error("invalid exchange database in {path} at line {line:?}")] DatabaseError { path: PathBuf, line: (usize, usize) },
    #[error("invalid exchange location in {path} at line {line:?}")] LocationError { path: PathBuf, line: (usize, usize) },
    #[error("invalid exchange unit in {path} at line {line:?}")] UnitError { path: PathBuf, line: (usize, usize) },
    #[error("multiple exchanges found in {path} at line {line:?}")] MultipleExchange { path: PathBuf, line: (usize, usize) },
}

impl CliError {

    pub fn display_cli_errors(&self) -> std::io::Result<()> {

        match self {
            CliError::OdysseyError(err) => { eprintln!("engine error: {err}"); return Ok(()); }
            CliError::IoError(err) => { eprintln!("io error: {err}"); return Ok(()); }
            CliError::TantivyError(err) => { eprintln!("tantivy error: {err}"); return Ok(()); }
            _ => {}
        }

        let (path, line, error_msg, help_msg, suggestion) = match self {
            Self::YamlLoadError { path, line, details } => (path, line, "error: error during yaml parsing", details.as_str(), ""),
            Self::InvalidYamlFormat { path, line, details } => (path, line, "error: invalid YAML structure", details.as_str(), ""),
            Self::MissingExchangeName { path, line } => (path, line, "error: missing exchange name", "every exchange must have a unique identifier name", "name: \"your_exchange_name\""),
            Self::MissingExchangeLink { path, line } => (path, line, "error: missing exchange link", "an exchange requires a data source definition, define either `file` OR `database`", "database:\n            name: \"your_database_name\"\n            version: \"0.0.0\"\n    OR:\n         file: \"path/to/file.yaml\"\n  "),
            Self::MissingDatabaseName { path, line } => (path, line, "error: missing database name", "the `database` configuration block is incomplete, define a database name", "name: \"your_database_name\""),
            Self::MissingDatabaseVersion { path, line } => (path, line, "error: missing database version", "the `database` configuration block is incomplete, define a database version", "version: \"0.0.0\""),
            Self::MissingExchangeAmount { path, line } => (path, line, "error: missing exchange amount", "the numerical quantity `amount` for the flow is missing", "amount: 0.0"),
            Self::AmountError { path, line } => (path, line, "error: invalid exchange amount format", "the amount could not be parsed into a 64-bit float (f64)", "amount"),
            Self::FileError { path, line } => (path, line, "error: invalid exchange file", "Failed to open or read the exchange file. Verify that the file path exists and is accessible.", "file"),
            Self::MissingExchange { path, line } => (path, line, "error: exchange flow not found", "the search query returned zero results in the current database | try searching for the exchange using: odyssey search \"your_exchange_name\"", ""),
            Self::NameError { path, line } => (path, line, "error: exchange flow not found", "the search query returned zero results in the current database (the 'name' is likely incorrect) | try searching for the exchange using: odyssey search \"your_exchange_name\"", ""),
            Self::DatabaseError { path, line } => (path, line, "error: exchange flow not found", "the search query returned zero results in the current database (the 'database' is likely incorrect) | try searching for the exchange using: odyssey search \"your_exchange_name\" -d \"database.name_database.version\"", "database"),
            Self::LocationError { path, line } => (path, line, "error: exchange flow not found", "the search query returned zero results in the current database (the 'location' filter is likely incorrect) | try searching for the exchange using: odyssey search \"your_exchange_name\" -l \"your_location\"", "location"),
            Self::UnitError { path, line } => (path, line, "error: exchange flow not found", "the search query returned zero results in the current database (the 'unit' filter is likely incorrect) | try searching for the exchange using: odyssey search \"your_exchange_name\" -l \"your_unit\"", "unit"),
            Self::MultipleExchange { path, line } => (path, line, "error: multiple exchanges found", "the search query returned multiple matching entries in the database | restrict the search by adding filters (e.g. location, unit) | try searching for the exchange using: odyssey search \"your_exchange_name\"", ""),
            _ => unreachable!(),
        };

        eprintln!("\n{}", error_msg.red().bold());
        eprintln!(" {} {}:{}\n", "-->".blue().bold(), path.display(), line.0);

        let file = File::open(path)?;
        let lines: Vec<String> = BufReader::new(file).lines()
            .skip(line.0.saturating_sub(1))
            .take(line.1.saturating_sub(line.0).max(1))
            .flatten()
            .collect();

        let margin = " ".repeat(line.1.to_string().len());

        for (index, content) in lines.iter().enumerate() {

            let trimmed_content = content.trim_start();
            if trimmed_content.is_empty() || trimmed_content.starts_with('#') { continue; }
            let curr_line = line.0 + index;

            match self {
                Self::AmountError { .. } | Self::LocationError { .. } | Self::UnitError { .. } | Self::FileError { .. } | Self::DatabaseError { .. } => {
                    let suggestion_with_space = format!("{} :", suggestion);
                    let suggestion_without_space = format!("{}:", suggestion);
                    if content.contains(&suggestion_with_space) || content.contains(&suggestion_without_space) {
                        eprintln!("{curr_line} |{content}      {}", "<-- error occurred here".yellow());
                    } else {
                        eprintln!("{curr_line} |{content}");
                    }
                }
                Self::MissingDatabaseName { .. } | Self::MissingDatabaseVersion { .. } => {
                    eprintln!("{curr_line} |{content}");
                    if content.contains("database:") {
                        eprintln!("{}{margin}|       {}", "+".green(), suggestion.green());
                    }
                }
                _ => eprintln!("{curr_line} |{content}"),
            }
        }

        if matches!(self, Self::MissingExchangeName { .. } | Self::MissingExchangeLink { .. } | Self::MissingExchangeAmount { .. } | Self::MissingExchange { .. } | Self::MultipleExchange { .. }) && !suggestion.is_empty() {
            eprintln!("{}{margin}|    {}", "+".green(), suggestion.green());
        }

        eprintln!("\n{} {}", "= help:".blue().bold(), help_msg);
        Ok(())
    }
}
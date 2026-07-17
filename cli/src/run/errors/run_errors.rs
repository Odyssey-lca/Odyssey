use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use comput::errors::ComputError;

use crate::run::FileLocator;
use crate::run::errors::comput_errors::display_comput_errors;
use crate::run::errors::format_errors::{
    show_added_line_suggestion, show_error_msg, show_help, show_line_addition_suggestion,
    show_line_change_suggestions, show_line_with_suggestion_arrow,
};

pub type Result<T> = std::result::Result<T, RunError>;

#[rustfmt::skip]
#[derive(thiserror::Error, Debug)]
pub enum RunError {
    #[error(transparent)]
    OdysseyError(#[from] errors::OdysseyErrors),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    TantivyError(#[from] tantivy::TantivyError),
    #[error(transparent)]
    SearchError(#[from] search::errors::SearchErrors),
    #[error(transparent)]
    ComputError(#[from] ComputError<FileLocator>),

    #[error("load error in during yaml parsing in {error_locator:?}: {details}")]
    YamlLoadError { error_locator: FileLocator, details: String },
    #[error("invalid yaml format in {error_locator:?}: {details}")]
    InvalidYamlFormat { error_locator: FileLocator, details: String },

    #[error("missing exchange name in {0:?}")]
    MissingExchangeName (FileLocator),
    #[error("wrong {field_name} format in {error_locator:?}")]
    WrongFieldFormat{error_locator: FileLocator, field_name: String},
    #[error("exchange in {0:?} contains both database and file links")]
    BothDatabaseAndFile (FileLocator),
    #[error("missing exchange link in {0:?}")]
    MissingExchangeLink (FileLocator),
    #[error("missing database name in {0:?}")]
    MissingDatabaseName (FileLocator),
    #[error("missing database version in {0:?}")]
    MissingDatabaseVersion (FileLocator),
    #[error("unknown database in {0:?}")]
    UnknownDatabase (FileLocator),
    #[error("unknown method {0}")]
    UnknownMethod(String),
    #[error("missing exchange amount in {0:?}")]
    MissingExchangeAmount (FileLocator),
    #[error("invalid exchange amount format in {0:?}")]
    AmountError (FileLocator),
    #[error("invalid exchange file in {0:?}")]
    FileError (FileLocator),


    #[error("unit mismatch between exchange and activity at {error_locator:?}")]
    UnitMismatch { error_locator: FileLocator, activity_unit: String },
    #[error("location mismatch between exchange and activity at {error_locator:?}")]
    LocationMismatch { error_locator: FileLocator, activity_location: String },
    #[error("name mismatch between exchange and activity at {error_locator:?}")]
    NameMismatch { error_locator: FileLocator, activity_name: String  },
}

impl RunError {
    pub fn display_cli_errors(&self) -> std::io::Result<()> {
        match self {
            Self::OdysseyError(err) => {
                eprintln!("engine error: {err}");
                return Ok(());
            }
            Self::IoError(err) => {
                eprintln!("io error: {err}");
                return Ok(());
            }
            Self::TantivyError(err) => {
                eprintln!("tantivy error: {err}");
                return Ok(());
            }
            Self::SearchError(err) => {
                eprintln!("{err}");
                return Ok(());
            }
            Self::UnknownMethod(err) => {
                eprintln!("{err}");
                return Ok(());
            }
            Self::ComputError(err) => {
                display_comput_errors(err)?;
                return Ok(());
            }
            _ => {}
        }

        let (error_locator, error_msg, help_msg, field, suggestion) = match self {
            Self::YamlLoadError { error_locator, details } => (
                error_locator,
                "error: error during yaml parsing",
                details.as_str(),
                "",
                "",
            ),
            Self::InvalidYamlFormat { error_locator, details } => (
                error_locator,
                "error: invalid YAML structure",
                details.as_str(),
                "",
                "",
            ),
            Self::BothDatabaseAndFile(error_locator) => (
                error_locator,
                "error: exchange contains both database and file links",
                "every exchange must take its values from file or from a database, not both",
                "",
                "Remove either the database or the file section",
            ),
            Self::MissingExchangeName(error_locator) => (
                error_locator,
                "error: missing exchange name",
                "every exchange must have a unique identifier name",
                "name",
                "name: \"your_exchange_name\"",
            ),
            Self::WrongFieldFormat{ error_locator, field_name } => (
                error_locator,
                "error: wrong field format",
                "this field should be a string",
                field_name.as_str(),
                "",
            ),
            Self::MissingExchangeLink(error_locator) => (
                error_locator,
                "error: missing exchange link",
                "an exchange requires a data source definition, define either `file` OR `database`",
                "database",
                "database:\n            name: \"your_database_name\"\n            version: \"0.0.0\"\n    OR:\n         file: \"path/to/file.yaml\"\n  ",
            ),
            Self::MissingDatabaseName(error_locator) => (
                error_locator,
                "error: missing database name",
                "the `database` configuration block is incomplete, define a database name",
                "name",
                "name: \"your_database_name\"",
            ),
            Self::UnknownDatabase(error_locator) => (
                error_locator,
                "error: unknown database",
                "the `database` provided is not supported by Odyssey",
                "",
                "",
            ),
            Self::MissingDatabaseVersion(error_locator) => (
                error_locator,
                "error: missing database version",
                "the `database` configuration block is incomplete, define a database version",
                "version",
                "version: \"0.0.0\"",
            ),
            Self::MissingExchangeAmount(error_locator) => (
                error_locator,
                "error: missing exchange amount",
                "the numerical quantity `amount` for the flow is missing",
                "amount",
                "amount: 0.0",
            ),
            Self::AmountError(error_locator) => (
                error_locator,
                "error: invalid exchange amount format",
                "the amount could not be parsed into a 64-bit float (f64)",
                "amount",
                "",
            ),
            Self::FileError(error_locator) => (
                error_locator,
                "error: invalid exchange file",
                "Failed to open or read the exchange file. Verify that the file path exists and is accessible.",
                "file",
                "",
            ),
            Self::UnitMismatch { error_locator, activity_unit } => (
                error_locator,
                "error: unit mismatch between exchange and its defining activity and no conversion is possible",
                "consider changing the unit of that exchange to match that of the linked activity",
                "unit",
                activity_unit.as_str(),
            ),
            Self::LocationMismatch { error_locator, activity_location } => (
                error_locator,
                "error: location mismatch between exchange and its defining activity and no conversion is possible",
                "consider changing the location of that exchange to match that of the linked activity",
                "location",
                activity_location.as_str()
            ),
            Self::NameMismatch { error_locator, activity_name } => (
                error_locator,
                "error: name mismatch between exchange and its defining activity and no conversion is possible",
                "consider changing the name of that exchange to match that of the linked activity",
                "name",
                activity_name.as_str(),
            ),
            Self::TantivyError(_)
            | Self::OdysseyError(_)
            | Self::IoError(_)
            | Self::SearchError(_)
            | Self::UnknownMethod(_)
            | Self::ComputError(_) => unreachable!(),
        };

        show_error_msg(error_msg, error_locator);

        let file = File::open(error_locator.path.clone())?;
        let lines: Vec<String> = BufReader::new(file)
            .lines()
            .skip(error_locator.lines.0.saturating_sub(1))
            .take(
                error_locator
                    .lines
                    .1
                    .saturating_sub(error_locator.lines.0)
                    .max(1),
            )
            .flatten()
            .collect();

        let margin = " ".repeat(error_locator.lines.1.to_string().len());

        for (index, content) in lines.iter().enumerate() {
            let trimmed_content = content.trim_start();
            if trimmed_content.is_empty() || trimmed_content.starts_with('#') {
                continue;
            }
            let curr_line = error_locator.lines.0 + index;

            match self {
                Self::AmountError { .. } | Self::FileError { .. } | Self::WrongFieldFormat { .. } => {
                    show_line_with_suggestion_arrow(content, curr_line, field);
                }
                Self::UnitMismatch { .. }
                | Self::LocationMismatch { .. }
                | Self::NameMismatch { .. } => {
                    show_line_change_suggestions(content, curr_line, field, suggestion);
                }
                Self::MissingDatabaseName { .. } | Self::MissingDatabaseVersion { .. } => {
                    show_line_addition_suggestion(content, curr_line, suggestion, &margin);
                }
                _ => eprintln!("{curr_line} |{content}"),
            }
        }

        if matches!(
            self,
            Self::MissingExchangeName { .. }
                | Self::MissingExchangeLink { .. }
                | Self::MissingExchangeAmount { .. }
        ) && !suggestion.is_empty()
        {
            show_added_line_suggestion(suggestion, &margin);
        }

        show_help(help_msg);
        Ok(())
    }
}

impl From<(marked_yaml::LoadError, &Path)> for RunError {
    fn from(err: (marked_yaml::LoadError, &Path)) -> Self {
        let error_line = match &err.0 {
            marked_yaml::LoadError::TopLevelMustBeMapping(m) => m.line(),
            marked_yaml::LoadError::TopLevelMustBeSequence(m) => m.line(),
            marked_yaml::LoadError::UnexpectedAnchor(m) => m.line(),
            marked_yaml::LoadError::MappingKeyMustBeScalar(m) => m.line(),
            marked_yaml::LoadError::UnexpectedTag(m) => m.line(),
            marked_yaml::LoadError::ScanError(m, _) => m.line(),
            marked_yaml::LoadError::DuplicateKey(_) => 0,
        };

        RunError::YamlLoadError {
            error_locator: FileLocator {
                path: err.1.to_path_buf(),
                lines: (error_line, error_line),
            },
            details: err.0.to_string(),
        }
    }
}

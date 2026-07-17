use std::{
    fs::File,
    io::{BufRead, BufReader},
};
use comput::errors::{ComputError};

use crate::run::{FileLocator, errors::format_errors::{show_added_line_suggestion, show_error_msg, show_help, show_line, show_line_with_suggestion_arrow}};

pub fn display_comput_errors(error: &ComputError<FileLocator>) -> std::io::Result<()> {
    match error {
        ComputError::OdysseyError(err) => {
            eprintln!("engine error: {err}");
            return Ok(());
        }
        ComputError::IoError(err) => {
            eprintln!("io error: {err}");
            return Ok(());
        }
        ComputError::SearchError(err) => {
            eprintln!("{err}");
            return Ok(());
        }
        _ => {}
    }

    let (error_locator, error_msg, help_msg, suggestion) = match error {
        ComputError::MissingExchange(error_locator) => (
            error_locator,
            "error: exchange flow not found",
            "the search query returned zero results in the current database | try searching for the exchange using: odyssey search \"your_exchange_name\"",
            "",
        ),
        ComputError::NameError(error_locator) => (
            error_locator,
            "error: exchange flow not found",
            "the search query returned zero results in the current database (the 'name' is likely incorrect) | try searching for the exchange using: odyssey search \"your_exchange_name\"",
            "",
        ),
        ComputError::DatabaseError(error_locator) => (
            error_locator,
            "error: exchange flow not found",
            "the search query returned zero results in the current database (the 'database' is likely incorrect) | try searching for the exchange using: odyssey search \"your_exchange_name\" -d \"database.name_database.version\"",
            "database",
        ),
        ComputError::LocationError(error_locator) => (
            error_locator,
            "error: exchange flow not found",
            "the search query returned zero results in the current database (the 'location' filter is likely incorrect) | try searching for the exchange using: odyssey search \"your_exchange_name\" -l \"your_location\"",
            "location",
        ),
        ComputError::UnitError(error_locator) => (
            error_locator,
            "error: exchange flow not found",
            "the search query returned zero results in the current database (the 'unit' filter is likely incorrect) | try searching for the exchange using: odyssey search \"your_exchange_name\" -l \"your_unit\"",
            "unit",
        ),
        ComputError::MultipleExchange(error_locator) => (
            error_locator,
            "error: multiple exchanges found",
            "the search query returned multiple matching entries in the database | restrict the search by adding filters (e.g. location, unit) | try searching for the exchange using: odyssey search \"your_exchange_name\"",
            "",
        ),
        ComputError::UnknownDatabase(error_locator) => (
            error_locator,
            "error: a database was not found",
            "",
            "",
        ),
        ComputError::InternalError(error_locator) => (
            error_locator,
            "error: an internal error occured due to the following exchange",
            "",
            "",
        ),
        ComputError::OdysseyError(_) | ComputError::IoError(_) | ComputError::SearchError(_) => unreachable!(),
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

        match error {
            ComputError::LocationError { .. }
            | ComputError::UnitError { .. }
            | ComputError::DatabaseError { .. } => {
                show_line_with_suggestion_arrow(content, curr_line, suggestion);
            }
            _ => show_line(content, curr_line),
        }
    }

    if matches!(
        error,
          ComputError::MissingExchange { .. }
            | ComputError::MultipleExchange { .. }
    ) && !suggestion.is_empty()
    {
        show_added_line_suggestion(suggestion, &margin);
    }

    show_help(help_msg);
    Ok(())
}

use search::SearchResult;
use serde_json::json;

use crate::search::ResFormat;

fn to_json(sr: &SearchResult) -> String {
    let json = if let Some(location) = &sr.location {
        json!({"database": {"name": sr.database_name, "version": sr.database_version}, "name": sr.name, "location": location, "unit": sr.unit})
    } else {
        json!({"database": {"name": sr.database_name, "version": sr.database_version}, "name": sr.name, "unit": sr.unit})
    };
    format!("{}", json)
}

fn to_inline(sr: &SearchResult) -> String {
    if let Some(location) = &sr.location {
        format!(
            "[{} v{}] {} {} {}\n",
            sr.database_name, sr.database_version, sr.name, location, sr.unit
        )
    } else {
        format!(
            "[{} v{}] {} {}\n",
            sr.database_name, sr.database_version, sr.name, sr.unit
        )
    }
}

fn to_column(sr: &SearchResult) -> String {
    use colored::Colorize;

    let mut output = String::new();

    output.push_str(&format!("{}\n", sr.name));

    output.push_str(&format!(
        " | {} {}\n",
        "Database:".bold(),
        format!("{} v{}", sr.database_name, sr.database_version).bright_white()
    ));

    if let Some(location) = &sr.location {
        output.push_str(&format!(
            " | {} {}\n",
            "Location:".bold(),
            location.bright_white()
        ));
    }

    output.push_str(&format!(
        " | {} {}\n\n",
        "Unit:".bold(),
        sr.unit.bright_white()
    ));

    output
}

pub fn format_result(results: Vec<SearchResult>, format: ResFormat) -> String {
    let mut output = String::new();
    match format {
        ResFormat::Default => results
            .iter()
            .for_each(|sr| output.push_str(&to_column(sr))),
        ResFormat::Compact => results
            .iter()
            .for_each(|sr| output.push_str(&to_inline(sr))),
        ResFormat::Json => results.iter().for_each(|sr| output.push_str(&to_json(sr))),
    }
    output
}

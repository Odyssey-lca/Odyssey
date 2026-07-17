use colored::Colorize;

use crate::run::FileLocator;

pub fn show_line_with_suggestion_arrow(line: &str, line_number: usize, field: &str) {
    let suggestion_without_space = format!("{}:", field);
    if line.replace(" ", "").contains(&suggestion_without_space) {
        eprintln!(
            "{line_number} |{line}      {}",
            "<-- error occurred here".yellow()
        );
    } else {
        show_line(line, line_number);
    }
}

pub fn show_line_change_suggestions(line: &str, line_number: usize, field: &str, replacement: &str) {
    let suggestion_without_space = format!("{}:", field);
    if line.replace(" ", "").contains(&suggestion_without_space) {
        eprintln!("{}", format!("{line_number} |{line}").red().strikethrough());
        eprintln!("{}", format!("{line_number} |    {field}: \"{replacement}\"").green());
    } else {
        show_line(line, line_number);
    }
}

pub fn show_line(line: &str, line_number: usize) {
  eprintln!("{line_number} |{line}");
}

pub fn show_added_line_suggestion(suggestion: &str, margin: &str) {
  eprintln!("{}{margin}|       {}", "+".green(), suggestion.green());
}

pub fn show_line_addition_suggestion(line: &str, line_number: usize, suggestion: &str, margin: &str) {
    show_line(line, line_number);
    if line.replace(" ", "").contains("database:") {
        show_added_line_suggestion(suggestion, margin);
    }
}

pub fn show_error_msg(error: &str, error_locator: &FileLocator) {
    eprintln!("{}", error.red().bold());
    eprintln!(
        " {} {}:{}\n",
        "-->".blue().bold(),
        error_locator.path.display(),
        error_locator.lines.0
    );
}

pub fn show_help(help: &str) {
  eprintln!("\n{} {}", "= help:".blue().bold(), help);
}


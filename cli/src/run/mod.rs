use std::fmt::{self, Debug};
use std::path::Path;
use std::path::PathBuf;

use crate::paths::{DATABASES_PATH, SEARCH_PATH};
use crate::run::errors::run_errors::Result;
use crate::run::parser::{parse_activity, parse_method};
use clap::Args;
use comput::lca::{Activity, compute_lca_detailed};

pub mod errors;
pub mod parser;

#[derive(Clone)]
pub struct FileLocator {
    path: PathBuf,
    lines: (usize, usize),
}

impl Debug for FileLocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}:{}:0-{}.100", self.path, self.lines.0, self.lines.1)
    }
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct RunCommand {
    pub path: PathBuf,

    #[arg(short, long, default_value = "ef31")]
    pub method: String,
}

pub fn run(path: &Path, method: String) -> Result<()> {
    let activity: Activity<FileLocator> = parse_activity(path)?;
    let method = parse_method(method)?;
    let full_res = compute_lca_detailed(activity, method, &DATABASES_PATH, &SEARCH_PATH)?;
    print!("flow");
    let all_res = full_res.get("all").unwrap();
    for i in 0..all_res.values.len() {
        if let Some(ic) = all_res.mapping.get_by_right(&i) {
            print!(";{:?}", ic);
        }
    }
    println!();
    for (flow_name, res) in full_res.iter() {
        if flow_name == "all" {
            continue;
        }
        print!("{}", flow_name);
        for i in 0..res.values.len() {
            if res.mapping.get_by_right(&i).is_some() {
                print!(";{:.4e}", res.values[i])
            }
        }
        println!();
    }
    print!("all");
    for i in 0..all_res.values.len() {
        if all_res.mapping.get_by_right(&i).is_some() {
            print!(";{:.4e}", all_res.values[i])
        }
    }
    println!();

    Ok(())
}

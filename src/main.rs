use std::cmp::min;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use roogle_engine::exec::QueryExecutor;
use roogle_engine::types::Query;
use roogle_index::types::Index;

#[derive(StructOpt, Debug)]
struct Config {
    #[structopt(short, long)]
    krate: Option<String>,

    #[structopt(short, long, parse(from_os_str))]
    index: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    query: PathBuf,
}

fn read_json(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path.as_ref()).expect("failed in reading file")
}

fn main() {
    let cfg = Config::from_args();
    let index: Index =
        serde_json::from_str(&read_json(cfg.index)).expect("failed in deserializing index");
    let query: Query =
        serde_json::from_str(&read_json(cfg.query)).expect("failed in deserializing query");

    let krate = match cfg.krate {
        Some(krate) => krate,
        None => {
            let keys = index.crates.keys();
            if keys.len() == 1 {
                // In this case, though the user didn't pass `--krate`, we can infer what crate to use.
                keys.into_iter().next().unwrap().clone()
            } else {
                panic!("Please specify which crate to query in, using `--krate` argument.")
            }
        }
    };

    let qe = QueryExecutor::new(krate, index);
    let results = qe.exec(&query);
    for i in 0..min(results.len(), 3) {
        println!("{:#?}", results[i]);
    }
}

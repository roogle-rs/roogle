use std::path::{Path, PathBuf};

use nom::error::ErrorKind;
use rustyline::Editor;
use structopt::StructOpt;

use roogle_engine::exec::QueryExecutor;
use roogle_engine::parse::parse_query;
use roogle_index::types::Index;

#[derive(StructOpt, Debug)]
struct Config {
    #[structopt(short, long)]
    krate: Option<String>,

    #[structopt(short, long, parse(from_os_str))]
    index: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    query: Option<PathBuf>,
}

fn read_json(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path.as_ref()).expect("failed in reading file")
}

fn main() {
    let cfg = Config::from_args();
    let index: Index =
        serde_json::from_str(&read_json(cfg.index)).expect("failed in deserializing index");

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
    match cfg.query {
        None => repl(qe),
        Some(query) => {
            let query =
                serde_json::from_str(&read_json(query)).expect("failed in deserializing query");
            let results = qe.exec(&query);
            results
                .iter()
                .take(1)
                .for_each(|result| println!("{:#?}", result));
        }
    }
}

fn repl(qe: QueryExecutor) {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let query = parse_query::<(&str, ErrorKind)>(&line)
                    .expect("parse failed")
                    .1;
                println!("query={:?}", &query);
                let results = qe.exec(&query);
                results
                    .iter()
                    .take(1)
                    .for_each(|result| println!("{:#?}", result));
            }
            _ => panic!("repl failed"),
        }
    }
}

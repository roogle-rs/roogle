use core::panic;
use std::fs;
use std::path::{Path, PathBuf};

use env_logger as logger;
use rustdoc_types::Crate;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use structopt::StructOpt;

use roogle_engine::{exec::QueryExecutor, parse::parse_query, types::Crates};

#[derive(StructOpt, Debug)]
struct Config {
    #[structopt(short, long, parse(from_os_str))]
    index: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    query: Option<PathBuf>,
}

fn read_json(path: impl AsRef<Path>) -> String {
    std::fs::read_to_string(path.as_ref()).expect("failed in reading file")
}

fn main() {
    logger::init();

    let cfg = Config::from_args();

    let krates: Vec<Crate> = fs::read_dir(cfg.index)
        .unwrap()
        .map(Result::unwrap)
        .map(|entry| serde_json::from_str(&read_json(entry.path())).unwrap())
        .collect();
    let krates = Crates::from(krates);

    let qe = QueryExecutor::new(krates);

    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());

                let query = parse_query(&line).expect("failed in parsing query").1;
                let items = qe.exec(query);
                for item in items.iter().take(3) {
                    println!("{:?}", item.path.join("::"));
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            _ => panic!("exitted repl"),
        }
    }
}

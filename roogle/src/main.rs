#[macro_use]
extern crate rocket;

use std::collections::HashMap;

use anyhow::{Context, Result};
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    response::content,
    State,
};
use tracing::debug;

use roogle_engine::{query::parse::parse_query, search::Scope, Index};

#[get("/search", data = "<query>")]
fn search(
    query: &str,
    index: &State<Index>,
) -> Result<content::Json<String>, rocket::response::Debug<anyhow::Error>> {
    let query = parse_query(query)
        .ok()
        .context(format!("parsing query `{}` failed", query))?
        .1;
    debug!(?query);

    let hits = index
        .search(
            &query,
            Scope::Set(vec![
                "alloc".to_owned(),
                "core".to_owned(),
                "std".to_owned(),
            ]),
            0.4, // NOTE(hkmatsumoto): Just a temporal value; maybe needs discussion in the future.
        )
        .with_context(|| format!("search with query `{:?}` failed", query))?;
    let hits = hits
        .into_iter()
        .inspect(|hit| debug!(?hit.name, ?hit.link, similarities = ?hit.similarities(), score = ?hit.similarities().score()))
        .take(30)
        .collect::<Vec<_>>();

    Ok(content::Json(
        serde_json::to_string(&hits).context("serializing search result failed")?,
    ))
}

#[launch]
fn rocket() -> _ {
    init_logger();

    let index = index().unwrap();
    rocket::build()
        .attach(Cors)
        .manage(index)
        .mount("/", routes![search])
}

fn init_logger() {
    use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

    let filter = match std::env::var("ROOGLE_LOG") {
        Ok(env) => EnvFilter::new(env),
        _ => return,
    };
    let layer = tracing_tree::HierarchicalLayer::default()
        .with_indent_lines(true)
        .with_indent_amount(2)
        .with_ansi(true)
        .with_targets(true);
    tracing_subscriber::Registry::default()
        .with(filter)
        .with(layer)
        .init();
}

fn index() -> Result<Index> {
    let crates = std::fs::read_dir("roogle-index/crate")
        .context("failed to read index files")?
        .map(|entry| {
            let entry = entry?;
            let json = std::fs::read_to_string(entry.path())
                .with_context(|| format!("failed to read `{:?}`", entry.file_name()))?;
            let krate = serde_json::from_str(&json)
                .with_context(|| format!("failed to deserialize `{:?}`", entry.file_name()))?;
            let file_name = entry
                .path()
                .with_extension("")
                .file_name()
                .with_context(|| format!("failed to get file name from `{:?}`", entry.path()))?
                .to_str()
                .context("failed to get `&str` from `&OsStr`")?
                .to_owned();
            Ok((file_name, krate))
        })
        .collect::<Result<HashMap<_, _>>>();
    crates.map(|crates| Index { crates })
}

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _: &'r rocket::Request<'_>, res: &mut rocket::Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        res.set_header(Header::new("Access-Control-Allow-Methods", "GET"));
        res.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
        res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

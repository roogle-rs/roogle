#[macro_use]
extern crate rocket;

use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    response::content,
    State,
};
use structopt::StructOpt;
use tracing::{debug, warn};

use roogle_engine::{query::parse::parse_query, search::Scope, Index};
use roogle_util::shake;

#[get("/search?<scope>", data = "<query>", rank = 2)]
fn search_with_data(
    query: &str,
    scope: &str,
    index: &State<Index>,
    scopes: &State<Scopes>,
) -> Result<content::Json<String>, rocket::response::Debug<anyhow::Error>> {
    search(query, scope, index, scopes)
}

#[get("/search?<scope>&<query>")]
fn search(
    query: &str,
    scope: &str,
    index: &State<Index>,
    scopes: &State<Scopes>,
) -> Result<content::Json<String>, rocket::response::Debug<anyhow::Error>> {
    let scope = match scope.split(':').collect::<Vec<_>>().as_slice() {
        ["set", set] => scopes
            .inner()
            .sets
            .get(*set)
            .context(format!("set `{}` not found", set))?,
        ["crate", krate] => scopes
            .inner()
            .krates
            .get(*krate)
            .context(format!("krate `{}` not found", krate))?,
        _ => Err(anyhow!("parsing scope `{}` failed", scope))?,
    };
    debug!(?scope);

    let query = parse_query(query)
        .ok()
        .context(format!("parsing query `{}` failed", query))?
        .1;
    debug!(?query);

    let hits = index
        .search(
            &query,
            scope.clone(),
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

#[get("/scopes")]
fn scopes(
    scopes: &State<Scopes>,
) -> Result<content::Json<String>, rocket::response::Debug<anyhow::Error>> {
    let mut result = vec![];
    for set in scopes.inner().sets.keys() {
        result.push(format!("set:{}", set));
    }
    for krate in scopes.inner().krates.keys() {
        result.push(format!("crate:{}", krate));
    }

    Ok(content::Json(
        serde_json::to_string(&result).context("serializing scopes failed")?,
    ))
}

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, name = "INDEX", default_value = "roogle-index")]
    index: PathBuf,
}

#[launch]
fn rocket() -> _ {
    init_logger();

    let opt = Opt::from_args();

    let index = make_index(&opt).unwrap();
    let scopes = make_scopes(&opt).unwrap();
    rocket::build()
        .attach(Cors)
        .manage(index)
        .manage(scopes)
        .mount("/", routes![search, search_with_data, scopes])
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

fn make_index(opt: &Opt) -> Result<Index> {
    let crates = std::fs::read_dir(format!("{}/crate", opt.index.display()))
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
            Ok((file_name, shake(krate)))
        })
        .filter_map(|res: Result<_, anyhow::Error>| {
            if let Err(ref e) = res {
                warn!("parsing a JSON file skipped: {}", e);
            }
            res.ok()
        })
        .collect::<HashMap<_, _>>();
    Ok(Index { crates })
}

struct Scopes {
    sets: HashMap<String, Scope>,
    krates: HashMap<String, Scope>,
}

fn make_scopes(opt: &Opt) -> Result<Scopes> {
    let krates: HashMap<String, Scope> =
        std::fs::read_dir(format!("{}/crate", opt.index.display()))
            .context("failed to read crate files")?
            .map(|entry| {
                let entry = entry?;
                let path = entry.path();
                let krate = path.file_stem().unwrap().to_str().unwrap(); // SAFETY: files in `roogle-index` has a name.

                Ok((krate.to_owned(), Scope::Crate(krate.to_owned())))
            })
            .filter_map(|res: Result<_, anyhow::Error>| {
                if let Err(ref e) = res {
                    warn!("registering a scope skipped: {}", e)
                }
                res.ok()
            })
            .collect();
    let sets: HashMap<String, Scope> =
        match std::fs::read_dir(format!("{}/set", opt.index.display())) {
            Err(e) => {
                warn!("registering sets skipped: {}", e);
                HashMap::default()
            }
            Ok(entry) => {
                entry
                    .map(|entry| {
                        let entry = entry?;
                        let path = entry.path();
                        let json = std::fs::read_to_string(&path)
                            .context(format!("failed to read `{:?}`", path))?;
                        let set = path.file_stem().unwrap().to_str().unwrap().to_owned(); // SAFETY: files in `roogle-index` has a name.
                        let krates = serde_json::from_str::<Vec<String>>(&json)
                            .context(format!("failed to deserialize set `{}`", &set))?;

                        Ok((set, Scope::Set(krates)))
                    })
                    .filter_map(|res: Result<_, anyhow::Error>| {
                        if let Err(ref e) = res {
                            warn!("registering a scope skipped: {}", e)
                        }
                        res.ok()
                    })
                    .collect()
            }
        };
    Ok(Scopes { sets, krates })
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

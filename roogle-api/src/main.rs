#[macro_use]
extern crate rocket;
use rocket::http::Header;
use rocket::response::content;
use rocket::State;
use rocket::fairing::{Fairing, Info, Kind};
use serde::Deserialize;

use roogle_engine::exec::QueryExecutor;
use roogle_engine::parse::parse_query;
use roogle_engine::types::Crates;
use rustdoc_types::Crate;

#[get("/", data = "<query>")]
fn index(query: &str, qe: &State<QueryExecutor>) -> content::Json<String> {
    index_with_query(query, qe)
}

#[get("/?<query>")]
fn index_with_query(query: &str, qe: &State<QueryExecutor>) -> content::Json<String> {
    let query = parse_query(query).expect("failed to parse query").1;
    let items: Vec<_> = qe
        .exec(query)
        .into_iter()
        .take(30)
        .collect();
    content::Json(serde_json::to_string(&items).unwrap())
}

#[launch]
fn rocket() -> _ {
    let qe = QueryExecutor::new(krates());
    rocket::build()
        .attach(Cors)
        .manage(qe)
        .mount("/", routes![index, index_with_query])
}

fn krates() -> Crates {
    let krates: Vec<_> = std::fs::read_dir("assets/")
        .expect("failed to read directory")
        .map(Result::unwrap)
        .map(|entry| {
            let json = std::fs::read_to_string(entry.path()).expect("failed to read file");
            let mut deserializer = serde_json::Deserializer::from_str(&json);
            deserializer.disable_recursion_limit();
            Crate::deserialize(&mut deserializer).expect("failed to deserialize")
        })
        .collect();

    Crates::from(krates)
}

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "CORS",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _: &'r rocket::Request<'_>, res: &mut rocket::Response<'r>) {
        res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        res.set_header(Header::new("Access-Control-Allow-Methods", "GET"));
        res.set_header(Header::new("Access-Control-Allow-Headers", "Content-Type"));
        res.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[macro_use]
extern crate rocket;
use rocket::response::content;
use rocket::State;

use roogle_engine::exec::QueryExecutor;
use roogle_engine::parse::parse_query;
use roogle_engine::types::Crates;

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
        .map(|item| item.name.as_ref())
        .collect();
    content::Json(serde_json::to_string(&items).unwrap())
}

#[launch]
fn rocket() -> _ {
    let qe = QueryExecutor::new(krates());
    rocket::build()
        .manage(qe)
        .mount("/", routes![index, index_with_query])
}

fn krates() -> Crates {
    let krates: Vec<_> = std::fs::read_dir("assets/")
        .expect("failed to read directory")
        .map(Result::unwrap)
        .map(|entry| {
            serde_json::from_str(
                &std::fs::read_to_string(entry.path()).expect("failed to read file"),
            )
            .expect("failed to deserialize")
        })
        .collect();

    Crates::from(krates)
}

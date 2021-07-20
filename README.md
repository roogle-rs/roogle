# Roogle
Roogle is a Rust API search engine, which allows you to search functions by names and type signatures.

## Example
```sh
$ cargo build --release
$ cargo run --release --quiet -- --index assets/index/answer_of_everything.json --query assets/query/answer_of_everything.json
```
# Roogle
Roogle is a Rust API search engine, which allows you to search functions by names and type signatures.

## Progress
Current available features are listed below.

### Available Queries
- [x] Function queries
- [ ] Method queries

### Available Types to query
- [x] Primitive types
- [ ] Generic types
- [ ] Custom types
- [ ] and more...

## Example
```sh
$ cargo build --release
$ cargo run --release --quiet -- --index assets/index/answer_of_everything.json --query assets/query/answer_of_everything.json
```

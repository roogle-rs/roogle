# Roogle
Roogle is a Rust API search engine, which allows you to search functions by names and type signatures.

## Progress
Current available features are listed below.

### Available Queries
- [x] Function queries
- [x] Method queries

### Available Types to query
- [x] Primitive types
- [ ] Generic types
  - [x] Without bounds and where predicates (e.g., `<T>`)
  - [ ] With bounds (e.g., `<T: Copy>`)
  - [ ] With where predicates
- [x] Custom types
  - [x] Without generic args (e.g., `IpAddr`)
  - [x] With generic args (e.g., `Vec<T>`, `Option<T>`)
- [ ] Other types

## Example (REPL)
```sh
$ cargo build --release
$ cargo run --release --quiet -- --index assets/index/core.json
> fn (Option<T>) -> bool
> fn (Option<Option<T>>) -> Option<T>
> fn (Option<T>, Option<T>) -> Option<T>
> fn (Option<Result<T, E>>) -> Result<Option<T>, E>
```

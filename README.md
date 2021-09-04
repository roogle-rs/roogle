# Roogle
Roogle is a Rust API search engine, which allows you to search functions by names and type signatures.

## Progress
Current available features are listed below.

### Available Queries
- [x] Function queries
- [x] Method queries

### Available Types to Query
- [x] Primitive types
- [ ] Generic types
  - [x] Without bounds and where predicates (e.g., `<T>`)
  - [ ] With bounds (e.g., `<T: Copy>`)
  - [ ] With where predicates
- [x] Custom types
  - [x] Without generic args (e.g., `IpAddr`)
  - [x] With generic args (e.g., `Vec<T>`, `Option<T>`)
- [ ] Other types

## Example
[Online hands-on](https://roogle.hkmatsumoto.com)

Note that onlines hands-on is much slower than the repl example below. You may need to wait 10 seconds to get a query completed.

## Example (REPL)
```sh
$ cargo run --release --bin roogle -- --index assets/
> fn (Option<T>) -> bool
> fn (Option<Option<T>>) -> Option<T>
> fn (Option<T>, Option<T>) -> Option<T>
> fn (Option<Result<T, E>>) -> Result<Option<T>, E>
> fn (&mut Vec<T>, T)
```

## Example (API Server)
```sh
$ cargo run --release --bin roogle-api
$ curl -X GET \
      -H "Content-type: application/json" \
      -d "fn (Option<Result<T, E>>) -> Result<Option<T>, E>" \
      "localhost:8000/" # On another shell session
```

### Indexing a 3rd party crate
Before running the command below, build [custom rustdoc](https://github.com/hkmatsumoto/rust/tree/rustdoc-roogle) and register `stage1`.
See [this section](https://rustc-dev-guide.rust-lang.org/getting-started.html#building-and-testing-rustdoc) for buliding rustdoc,
and [this section](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html#creating-a-rustup-toolchain)
for registering the toolchain.

```sh
$ cargo run --release --bin index_crate -- <crate-name>
```
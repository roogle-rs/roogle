# Roogle
Roogle is a Rust API search engine, which allows you to search functions by names and type signatures.

## Progress

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
```sh
$ cargo r --release
$ curl -X GET \
      -d "fn (Option<Result<T, E>>) -> Result<Option<T>, E>" \
      "localhost:8000/?" # On another shell session
```

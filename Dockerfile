FROM rust:latest AS chef
RUN cargo install cargo-chef
WORKDIR /usr/src/roogle

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /usr/src/roogle/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:buster-slim AS runtime
WORKDIR /usr/src/roogle
COPY --from=builder /usr/src/roogle/target/release/roogle /usr/local/bin
COPY --from=builder /usr/src/roogle/roogle-index roogle-index

ARG ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_ADDRESS=${ROCKET_ADDRESS}

CMD ["/usr/local/bin/roogle"]

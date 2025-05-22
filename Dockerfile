FROM rust:1-bullseye as builder
WORKDIR /usr/src/
RUN cargo new econ-society-backend --vcs none
WORKDIR /usr/src/econ-society-backend
COPY Cargo.toml ./
COPY rust-toolchain ./
RUN cargo update
RUN cargo build --release

# 为了充分利用docker的缓存
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release


FROM debian:bullseye-slim
RUN apt-get update && apt-get install libpq5 curl -y
COPY --from=builder /usr/src/econ-society-backend/target/release/econ-society-backend /usr/local/bin/econ-society-backend
COPY Rocket.toml /usr/local/bin/

CMD ["econ-society-backend"]

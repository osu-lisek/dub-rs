FROM rust as rust-builder

WORKDIR /app
RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install musl-dev musl-tools -y
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN mkdir ./src && echo 'fn main() { println!("Dummy!"); }' > ./src/main.rs
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN rm -rf ./src
COPY ./src ./src
RUN touch -a -m ./src/main.rs
COPY .sqlx /app/.sqlx
RUN cargo build --release --target x86_64-unknown-linux-musl


FROM alpine

RUN apk add --no-cache libssl3 curl

WORKDIR /app

COPY --from=rust-builder /app/target/x86_64-unknown-linux-musl/release/dub-rs /app/

CMD ["/app/dub-rs"]

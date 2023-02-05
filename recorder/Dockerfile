FROM rust:alpine AS builder
WORKDIR /usr/src/
RUN apk add musl-dev
RUN cargo new "recorder"
WORKDIR /usr/src/recorder
COPY build.rs sqlx-data.json Cargo.toml Cargo.lock ./
COPY src ./src 
RUN cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=builder /usr/local/cargo/bin/recorder /usr/local/bin/recorder
CMD ["recorder"]

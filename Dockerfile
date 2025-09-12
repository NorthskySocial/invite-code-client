FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml Trunk.toml index.html ./

RUN rustup target add wasm32-unknown-unknown

RUN cargo install trunk --locked

COPY assets ./assets

COPY src ./src

RUN trunk build --release

FROM nginx:mainline-alpine AS server

COPY --from=builder /app/dist /usr/share/nginx/html

LABEL org.opencontainers.image.source=https://github.com/NorthskySocial/invite-code-client
LABEL org.opencontainers.image.description="Invite Code Client"
LABEL org.opencontainers.image.licenses=MIT

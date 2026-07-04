FROM rust:1.96.1-alpine3.24 AS builder
WORKDIR /app
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml,readonly \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock,readonly \
    --mount=type=bind,source=src,target=src,readonly \
    --mount=type=bind,source=crates,target=crates,readonly \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release && cp target/release/typscord /typscord

FROM gcr.io/distroless/static-debian13:nonroot-amd64
COPY --from=builder /typscord /
ENV PORT="3000"
CMD ["/typscord"]

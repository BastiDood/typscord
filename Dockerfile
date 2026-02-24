FROM rust:1.93.1-slim-trixie AS builder
WORKDIR /app
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=crates,target=crates \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release && cp target/release/typscord /typscord

FROM gcr.io/distroless/static-debian13:nonroot-amd64
COPY --from=builder /typscord /
EXPOSE 3000
ENV PORT="3000"
CMD ["/typscord"]

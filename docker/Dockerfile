FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /checkmade
RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall -y trunk

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /checkmade/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin checkmade-server

WORKDIR /checkmade/app
RUN trunk build --release

FROM gcr.io/distroless/cc-debian13:nonroot AS runtime
WORKDIR /checkmade
COPY --from=builder /checkmade/target/release/checkmade-server /usr/local/bin/server
COPY --from=builder /checkmade/app/dist /checkmade/static
CMD ["/usr/local/bin/server"]
FROM rust:1-bookworm

# Add non-root user
RUN adduser --disabled-password --gecos "" dev

# Install Rust components
RUN rustup component add rustfmt clippy && rustup target add wasm32-unknown-unknown

# Install Node
RUN apt-get update && apt-get install nodejs npm -y
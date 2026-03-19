# Rust Actix Test Dockerfile
FROM rust:1.75

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo.toml and source
COPY adapters/rust/actix/Cargo.toml /app/Cargo.toml
COPY adapters/rust/actix/src /app/src

# Build and test
RUN cargo test --lib

CMD ["cargo", "test", "--lib"]

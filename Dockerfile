# x402 Agent SDK - Main Dockerfile
# For building and testing the core Rust library

FROM rust:1.75

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy project files
COPY Cargo.toml /app/
COPY src /app/src

# Build dependencies
RUN cargo fetch

# Run tests with coverage
RUN cargo test --lib

# Build WASM
RUN cargo install wasm-pack
RUN wasm-pack build --target web --out-dir pkg

CMD ["cargo", "test", "--lib", "--", "--nocapture"]

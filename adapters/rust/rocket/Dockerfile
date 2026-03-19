# Rust Rocket Test Dockerfile
FROM rust:1.75

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY adapters/rust/rocket/Cargo.toml /app/Cargo.toml
COPY adapters/rust/rocket/src /app/src

CMD ["cargo", "test", "--lib"]

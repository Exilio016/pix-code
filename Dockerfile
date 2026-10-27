# ----------------------------------------------------
# Stage 1: Build the application
# ----------------------------------------------------
FROM rust:1.99-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire project source code
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src src

# Build the release binary
RUN cargo build --release

# ----------------------------------------------------
# Stage 2: Minimal runtime environment
# ----------------------------------------------------
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-privileged user for security
RUN useradd -ms /bin/bash appuser
USER appuser

# Copy the compiled binary from the builder stage
# (Replace 'myapp' with your actual package name)
COPY --from=builder /app/target/release/pix-code /usr/local/bin/pix-code

# Run the binary
CMD ["/usr/local/bin/pix-code"]

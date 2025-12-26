# Build stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ffmpeg \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/radio-recorder-server .

# Create directories
RUN mkdir -p recordings

# Environment
ENV PORT=3000
ENV RECORDINGS_DIR=/app/recordings
ENV RUST_LOG=info

EXPOSE 3000

# Volume for recordings and database
VOLUME ["/app/recordings", "/app/data"]

CMD ["./radio-recorder-server"]

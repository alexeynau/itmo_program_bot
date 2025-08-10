# Stage 1: Build stage
FROM rust:1.88 as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src
COPY data ./data

# Build the application in release mode
RUN cargo build --release --bin tg_bot

# Stage 2: Runtime stage
FROM debian:bookworm-slim

# Install CA certificates and other runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1001 appuser

# Set the working directory
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/tg_bot /app/tg_bot

# Copy the data directory if needed at runtime
COPY --from=builder /app/data /app/data

# Change ownership to the non-root user
RUN chown -R appuser:appuser /app

# Switch to the non-root user
USER appuser

# Run the binary
CMD ["./tg_bot"]
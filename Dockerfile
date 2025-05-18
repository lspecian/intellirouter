# Stage 1: Builder
FROM rust:1.82-slim as builder

# Create a new empty shell project
WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev build-essential g++ libfontconfig1-dev \
    wget xfonts-75dpi xfonts-base && \
    wget https://github.com/wkhtmltopdf/packaging/releases/download/0.12.6-1/wkhtmltox_0.12.6-1.buster_amd64.deb && \
    dpkg -i wkhtmltox_0.12.6-1.buster_amd64.deb || true && \
    apt-get -f install -y && \
    rm wkhtmltox_0.12.6-1.buster_amd64.deb && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml rust-toolchain.toml ./

# Copy source code
COPY src/ ./src/
COPY config/ ./config/
COPY tests/ ./tests/
COPY examples/ ./examples/

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl-dev libfontconfig1 \
    wget xfonts-75dpi xfonts-base && \
    wget https://github.com/wkhtmltopdf/packaging/releases/download/0.12.6-1/wkhtmltox_0.12.6-1.buster_amd64.deb && \
    dpkg -i wkhtmltox_0.12.6-1.buster_amd64.deb || true && \
    apt-get -f install -y && \
    rm wkhtmltox_0.12.6-1.buster_amd64.deb && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/intellirouter /app/intellirouter

# Copy configuration files
COPY config/ /app/config/

# Set environment variables
ENV INTELLIROUTER_ENVIRONMENT=production
ENV INTELLIROUTER__SERVER__HOST=0.0.0.0
ENV INTELLIROUTER__SERVER__PORT=8080

# Expose the application port
EXPOSE 8080

# Set the entrypoint
ENTRYPOINT ["/app/intellirouter"]

# Default command
CMD ["run", "--role", "all"]
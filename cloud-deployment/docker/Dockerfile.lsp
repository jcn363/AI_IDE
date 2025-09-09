# Multi-stage build for LSP Server container
FROM rust:1.91.0-slim as builder

# Install system dependencies for LSP servers
RUN apt-get update && apt-get install -y \
    git \
    curl \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/rust-ai-ide

# Copy workspace files
COPY . .

# Build the LSP crate specifically (release mode for performance)
RUN cargo build --release --package rust-ai-ide-lsp --bin lsp-server || \
    cargo build --release --package rust-ai-ide-lsp

# Runtime stage (minimal)
FROM debian:bookworm-slim

# Install runtime dependencies for language servers
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    git \
    python3 \
    python3-pip \
    nodejs \
    npm \
    golang-go \
    && rm -rf /var/lib/apt/lists/*

# Install language servers
RUN npm install -g typescript typescript-language-server pyright @typescript-eslint/eslint-plugin @typescript-eslint/parser

# Install Python language server
RUN pip3 install pyright pylsp python-lsp-server

# Create non-root user
RUN useradd --create-home --shell /bin/bash lsp-user

# Set working directory
WORKDIR /home/lsp-user

# Copy binary from builder
COPY --from=builder /usr/src/rust-ai-ide/target/release/lsp-server /usr/local/bin/lsp-server

# Copy configs if any
COPY --from=builder /usr/src/rust-ai-ide/cloud-deployment/docker/config.yaml /home/lsp-user/config.yaml

# Change ownership
RUN chown -R lsp-user:lsp-user /home/lsp-user

# Switch to non-root user
USER lsp-user

# Expose port for LSP communication (typically 8080 or custom port)
EXPOSE 8080

# Health check endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Entry point
ENTRYPOINT ["/usr/local/bin/lsp-server"]

# Default arguments
CMD ["--config", "/home/lsp-user/config.yaml", "--port", "8080"]
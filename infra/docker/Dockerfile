# Multi-stage Dockerfile for Enterprise Management Suite (EMS).
# Build with the repository root as the Docker context:
# docker build -f infra/docker/Dockerfile .
# Run with your local config:
# docker run --env-file config/.env -p 5002:5002 ems-app

# Stage 1: Build React Frontend
FROM node:22-alpine AS frontend-builder

WORKDIR /app/frontend

# Copy package files for dependency caching
COPY apps/client/package*.json ./
RUN npm ci

# Copy frontend source and build
COPY apps/client/ ./
RUN npm run build

# Stage 2: Build Rust Backend
FROM rust:1.87.0-slim AS backend-builder

# Install system dependencies for Diesel and PostgreSQL
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/backend

# Copy Cargo.toml first
COPY apps/server/Cargo.toml ./
COPY apps/server/Cargo.lock ./

# Create dummy crate targets for dependency resolution.
RUN mkdir src && echo "" > src/lib.rs && echo "fn main() {}" > src/main.rs

# Fetch and build dependencies using the checked-in lockfile.
RUN cargo fetch --locked && cargo build --release --locked --bin server && rm -rf src

# Copy backend source and build
COPY apps/server/ ./
RUN touch src/lib.rs src/main.rs && cargo build --release --locked --bin server

# Stage 3: Runtime Image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN useradd -r -s /bin/false ems

WORKDIR /app

# Match the config layout used by run.py and the Rust dotenv loader.
RUN mkdir -p /app/config

# Copy built backend binary
COPY --from=backend-builder /app/backend/target/release/server ./server

# Copy built frontend static files
COPY --from=frontend-builder /app/frontend/dist ./static

# Copy configuration template. Provide real secrets via env vars,
# docker compose env_file, or a mounted /app/config/.env file.
COPY config/.env.example ./config/.env.example

# Set ownership and permissions
RUN chown -R ems:ems /app
USER ems

# Expose ports
EXPOSE 5002

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5002/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV BACKEND_PORT=5002
ENV STATIC_FILES_DIR=/app/static
ENV ENV_FILE=/app/config/.env

# Run the application
CMD ["./server"] 

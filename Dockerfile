# Multi-stage Dockerfile for Rust + Python project with PyO3 and Maturin

# Stage 1: Build stage
FROM rust:1.75-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    python3 \
    python3-pip \
    python3-dev \
    build-essential \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml pyproject.toml ./

# Install maturin for building Python extensions
RUN pip3 install maturin

# Copy source code
COPY src/ ./src/

# Build the Python extension
RUN maturin build --release

# Stage 2: Runtime stage
FROM python:3.11-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libgcc-s1 \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the built wheel from builder stage
COPY --from=builder /app/target/wheels/*.whl ./

# Install the Python package
RUN pip3 install *.whl

# Create a non-root user for security
RUN useradd --create-home --shell /bin/bash app \
    && chown -R app:app /app
USER app

# Set environment variables
ENV PYTHONPATH=/app
ENV PYTHONUNBUFFERED=1

# Default command
CMD ["python3", "-c", "import kappa_apk; print('kappa_apk module loaded successfully.')"] 
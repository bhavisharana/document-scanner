# --- Stage 1: Build the Rust binary ---
FROM rust:bookworm AS builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Copy real source code and build
COPY src ./src
RUN touch src/main.rs && cargo build --release

# --- Stage 2: Minimal Runtime Image ---
FROM debian:bookworm-slim AS runtime

# Install native Tesseract OCR and Poppler utilities (for PDF rasterization)
RUN apt-get update && apt-get install -y \
    tesseract-ocr \
    tesseract-ocr-eng \
    poppler-utils \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the compiled binary and frontend dist from builder stage
COPY --from=builder /app/target/release/document_scanner /app/document_scanner
COPY frontend/dist /app/frontend/dist

# Server environment defaults
ENV PORT=8081 \
    HOST=0.0.0.0 \
    RUST_LOG=info,document_scanner=info \
    TESSERACT_PATH=/usr/bin/tesseract

EXPOSE 8081

CMD ["/app/document_scanner"]

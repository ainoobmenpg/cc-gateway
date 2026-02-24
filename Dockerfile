# Build stage
FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/cc-core/Cargo.toml crates/cc-core/
COPY crates/cc-tools/Cargo.toml crates/cc-tools/
COPY crates/cc-mcp/Cargo.toml crates/cc-mcp/
COPY crates/cc-schedule/Cargo.toml crates/cc-schedule/
COPY crates/cc-discord/Cargo.toml crates/cc-discord/
COPY crates/cc-telegram/Cargo.toml crates/cc-telegram/
COPY crates/cc-whatsapp/Cargo.toml crates/cc-whatsapp/
COPY crates/cc-imessage/Cargo.toml crates/cc-imessage/
COPY crates/cc-signal/Cargo.toml crates/cc-signal/
COPY crates/cc-slack/Cargo.toml crates/cc-slack/
COPY crates/cc-line/Cargo.toml crates/cc-line/
COPY crates/cc-browser/Cargo.toml crates/cc-browser/
COPY crates/cc-voice/Cargo.toml crates/cc-voice/
COPY crates/cc-dashboard/Cargo.toml crates/cc-dashboard/
COPY crates/cc-email/Cargo.toml crates/cc-email/
COPY crates/cc-api/Cargo.toml crates/cc-api/
COPY crates/cc-ws/Cargo.toml crates/cc-ws/
COPY crates/cc-gateway/Cargo.toml crates/cc-gateway/

# Create dummy main.rs to build dependencies
RUN mkdir -p crates/cc-core/src crates/cc-tools/src crates/cc-mcp/src crates/cc-schedule/src \
    crates/cc-discord/src crates/cc-telegram/src crates/cc-whatsapp/src crates/cc-imessage/src \
    crates/cc-signal/src crates/cc-slack/src crates/cc-line/src crates/cc-browser/src \
    crates/cc-voice/src crates/cc-dashboard/src crates/cc-email/src crates/cc-api/src \
    crates/cc-ws/src crates/cc-gateway/src && \
    echo "fn main() {}" > crates/cc-gateway/src/main.rs && \
    for crate in cc-core cc-tools cc-mcp cc-schedule cc-discord cc-telegram cc-whatsapp cc-imessage \
                 cc-signal cc-slack cc-line cc-browser cc-voice cc-dashboard cc-email cc-api cc-ws; do \
        echo "pub fn main() {}" > "crates/${crate}/src/lib.rs"; \
    done

# Build dependencies
RUN cargo build --release

# Copy actual source code
COPY crates/ crates/

# Build binary
RUN cargo build --release --bin cc-gateway

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

# Create non-root user
RUN addgroup -g 1000 ccgateway && \
    adduser -u 1000 -G ccgateway -s /bin/sh -D ccgateway

# Copy binary from builder
COPY --from=builder /app/target/release/cc-gateway /usr/local/bin/

# Set ownership
RUN chown -R ccgateway:ccgateway /app

# Switch to non-root user
USER ccgateway

# Expose default port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Default environment
ENV RUST_LOG=info
ENV API_PORT=3000

ENTRYPOINT ["cc-gateway"]
CMD ["serve"]

# Stage 1: Build
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev gcc make openssl-dev pkgconfig

WORKDIR /usr/src/lumina
COPY . .

# Build the CLI in release mode
RUN cargo build --release --bin lumina-cli

# Stage 2: Runtime
FROM alpine:3.19

# Add standard library dependencies if needed
RUN apk add --no-cache libgcc

# Copy binary from builder
COPY --from=builder /usr/src/lumina/target/release/lumina-cli /usr/local/bin/lumina

# Default command
ENTRYPOINT ["lumina"]
CMD ["--help"]

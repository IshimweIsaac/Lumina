# Stage 1: Build
FROM rust:1.84-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev gcc g++ make openssl-dev pkgconfig cmake

WORKDIR /usr/src/lumina

# 1. Copy only the dependency manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/lumina-lexer/Cargo.toml crates/lumina-lexer/
COPY crates/lumina-parser/Cargo.toml crates/lumina-parser/
COPY crates/lumina-analyzer/Cargo.toml crates/lumina-analyzer/
COPY crates/lumina-diagnostics/Cargo.toml crates/lumina-diagnostics/
COPY crates/lumina-runtime/Cargo.toml crates/lumina-runtime/
COPY crates/lumina_ffi/Cargo.toml crates/lumina_ffi/
COPY crates/lumina-cli/Cargo.toml crates/lumina-cli/
COPY crates/lumina-wasm/Cargo.toml crates/lumina-wasm/
COPY crates/lumina-lsp/Cargo.toml crates/lumina-lsp/
COPY crates/lumina-cluster/Cargo.toml crates/lumina-cluster/

# 2. Create dummy source files (Bins need main.rs, Libs need lib.rs)
RUN mkdir -p crates/lumina-lexer/src \
    && mkdir -p crates/lumina-parser/src \
    && mkdir -p crates/lumina-analyzer/src \
    && mkdir -p crates/lumina-diagnostics/src \
    && mkdir -p crates/lumina-runtime/src \
    && mkdir -p crates/lumina_ffi/src \
    && mkdir -p crates/lumina-cli/src \
    && mkdir -p crates/lumina-wasm/src \
    && mkdir -p crates/lumina-lsp/src \
    && mkdir -p crates/lumina-cluster/src \
    && echo "fn main() {}" > crates/lumina-cli/src/main.rs \
    && echo "fn main() {}" > crates/lumina-lsp/src/main.rs \
    && touch crates/lumina-lexer/src/lib.rs \
    && touch crates/lumina-parser/src/lib.rs \
    && touch crates/lumina-analyzer/src/lib.rs \
    && touch crates/lumina-diagnostics/src/lib.rs \
    && touch crates/lumina-runtime/src/lib.rs \
    && touch crates/lumina_ffi/src/lib.rs \
    && touch crates/lumina-wasm/src/lib.rs \
    && touch crates/lumina-cluster/src/lib.rs

# 3. Build dependencies (cached layer) - Using correct binary name 'lumina'
RUN cargo build --release --bin lumina

# 4. Copy real source
COPY . .

# 5. Build real CLI - Using correct binary name 'lumina'
RUN cargo build --release --bin lumina

# Stage 2: Runtime
FROM alpine:3.19
RUN apk add --no-cache libgcc
COPY --from=builder /usr/src/lumina/target/release/lumina /usr/local/bin/lumina
ENTRYPOINT ["lumina"]
CMD ["--help"]

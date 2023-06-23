########################
#    Backend Builder   #
########################
FROM rust:1.70.0-slim-bookworm as builder

# Install dependencies
RUN apt update && apt install -y libsqlite3-dev libpq-dev

# Create a new empty shell project
RUN USER=root cargo new --bin sidestore-id-backend
WORKDIR /sidestore-id-backend

# Copy manifest files
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Build and cache the dependencies
RUN cargo build --release
RUN rm src/*.rs

# Copy source code
COPY ./src ./src
COPY ./migrations ./migrations
COPY ./diesel.toml ./diesel.toml

# Build for release
RUN rm ./target/release/deps/sidestore_id_backend* 
RUN cargo build --release


########################
# Production Container #
########################
FROM debian:bookworm-slim

# Install dependencies
RUN apt update && apt install -y --no-install-recommends sqlite3 libpq-dev && rm -rf /var/lib/apt/lists/*

# Copy compiled build from builder
WORKDIR /app
COPY --from=builder /sidestore-id-backend/target/release/sidestore-id-backend .

# Starup command
CMD ["./sidestore-id-backend"]

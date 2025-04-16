# ---- Build stage ----
FROM rust:1.86-alpine AS builder

# Install dependencies for compiling
RUN apk add --no-cache musl-dev openssl-dev pkgconf build-base sqlite-dev

# Create app directory
WORKDIR /app

# Now copy real source
COPY . .

# Build real project
RUN cargo build --release

# ---- Runtime stage ----
FROM alpine:latest

# Install dynamic libs needed (sqlite, openssl if required)
RUN apk add --no-cache sqlite-libs

# Copy only final binary
COPY --from=builder /app/target/release/nixscheduler-engine /usr/local/bin/nixscheduler-engine

# Copy runtime assets
COPY data/jobs.db /app/data/jobs.db
COPY statics /app/statics
COPY .env /app/.env

WORKDIR /app

# Expose default port
EXPOSE 8888

# Run the scheduler
CMD ["nixscheduler-engine"]
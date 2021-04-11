# We are using four stages: the first computes the recipe file, 
# the second caches our dependencies, the third builds the binary 
# and the fourth is our runtime environment.  As long as our 
# dependencies do not change the recipe.json file will stay the same, 
# therefore the outcome of cargo chef cook--release --recipe-path recipe.json
# will be cached, massively speeding up our builds. 
#
# We are taking advantage of how Docker layer caching interacts with 
# multi-stage builds: the COPY . . statement in the planner stage will 
# invalidate the cache for the planner container, but it will not
# invalidate the cache for the cacher container as long as the checksum
# of the recipe.json returned by cargo chef prepare does not change.

FROM lukemathwalker/cargo-chef AS planner
WORKDIR /app
COPY . .
# Compute a lock-like file for project
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef AS cacher
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
# Build project dependencies only
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust AS builder
WORKDIR /app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
ENV SQLX_OFFLINE true
# Build app leveraging the cached deps
RUN cargo build --release --bin zero2prod

FROM debian:buster-slim AS runtime
WORKDIR /app
# Install OpenSSL as it's dynamically linked by some deps
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
# Copy the compiled binary from builder to runtime environment
COPY --from=builder /app/target/release/zero2prod zero2prod
# Bring the configurations file
COPY configurations configurations
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
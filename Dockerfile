# Stage 1: Extract ca-certificates
FROM debian:bookworm-slim AS certs
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Stage 2: Final minimal image
FROM scratch

# Copy ca-certificates from first stage
COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

WORKDIR /app

# Copy application files
COPY ./target/prod-release/hot-or-not-web-leptos-ssr .
COPY ./target/prod-release/hash.txt .
COPY ./target/site ./site

# Environment variables
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_ENV="production"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_HASH_FILES="true"
# ENV LEPTOS_TAILWIND_VERSION="v4.0.9"

EXPOSE 8080

ENTRYPOINT ["./hot-or-not-web-leptos-ssr"]

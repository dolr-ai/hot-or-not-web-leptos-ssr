FROM scratch

WORKDIR /app
COPY ./target/x86_64-unknown-linux-musl/debug/hot-or-not-web-leptos-ssr .
COPY ./target/x86_64-unknown-linux-musl/debug/hash.txt .
COPY ./target/site ./site

ENV LEPTOS_SITE_ROOT="site"

ENV LEPTOS_ENV="production"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_HASH_FILES="true"
# ENV LEPTOS_TAILWIND_VERSION="v4.0.9"

# Sentry release version will be set at runtime by deployment scripts
# ENV SENTRY_RELEASE="${GITHUB_SHA:0:7}"

EXPOSE 8080

CMD ["./hot-or-not-web-leptos-ssr"]

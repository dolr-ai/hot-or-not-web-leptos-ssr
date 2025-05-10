FROM debian:bookworm-slim 

WORKDIR /app
COPY ./target/prod-release/hot-or-not-web-leptos-ssr .
COPY ./target/prod-release/hash.txt .
COPY ./target/site ./site

RUN chmod +x hot-or-not-web-leptos-ssr

ENV LEPTOS_SITE_ROOT="site"

ENV LEPTOS_ENV="production"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_HASH_FILES="true"
# ENV LEPTOS_TAILWIND_VERSION="v4.0.9"
EXPOSE 8080

CMD ["./hot-or-not-web-leptos-ssr"]

FROM alpine:3.21 

RUN apk upgrade --update-cache --available && \
    apk add openssl && \
    rm -rf /var/cache/apk/*

WORKDIR /app
COPY ./target/x86_64-unknown-linux-musl/prod-release/hot-or-not-web-leptos-ssr .
COPY ./target/x86_64-unknown-linux-musl/prod-release/hash.txt .

COPY ./target/site ./site
ENV LEPTOS_SITE_ROOT="site"

ENV LEPTOS_ENV="production"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_HASH_FILES="true"
# ENV LEPTOS_TAILWIND_VERSION="v4.0.9"
EXPOSE 8080

CMD ["./hot-or-not-web-leptos-ssr"]

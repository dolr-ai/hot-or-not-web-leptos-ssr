# Dockerfile for Fly Redis proxy sidecar
FROM alpine:3.19

# Install dependencies
RUN apk add --no-cache expect curl netcat-openbsd

# Install flyctl
RUN curl -L https://fly.io/install.sh | sh
ENV PATH="/root/.fly/bin:${PATH}"

WORKDIR /app

# Copy scripts
COPY entrypoint.sh /entrypoint.sh
COPY fly_redis_proxy.exp /app/fly_redis_proxy.exp
RUN chmod +x /entrypoint.sh /app/fly_redis_proxy.exp

EXPOSE 16379

ENTRYPOINT ["/entrypoint.sh"]

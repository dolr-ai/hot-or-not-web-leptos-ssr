#!/bin/bash

echo "Starting Jaeger in Docker..."
docker run -d --name jaeger \
  -p 8502:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  -e COLLECTOR_OTLP_ENABLED=true \
  jaegertracing/all-in-one:latest

echo "Waiting for Jaeger to start..."
sleep 5

echo "Jaeger UI available at: http://localhost:8502"
echo "OTLP gRPC endpoint: localhost:4317"
echo "OTLP HTTP endpoint: localhost:4318"

echo ""
echo "To view traces:"
echo "1. Open http://localhost:8502 in your browser"
echo "2. Select 'yral_ssr' from the Service dropdown"
echo "3. Click 'Find Traces'"
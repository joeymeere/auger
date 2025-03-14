#!/bin/bash

PORT=8180
RPC_URL="https://api.mainnet-beta.solana.com"
API_KEYS="dev-api-key"
IMAGE_NAME="auger-server"

while [[ $# -gt 0 ]]; do
  case $1 in
    --port)
      PORT="$2"
      shift 2
      ;;
    --rpc-url)
      RPC_URL="$2"
      shift 2
      ;;
    --api-keys)
      API_KEYS="$2"
      shift 2
      ;;
    --build)
      BUILD=true
      shift
      ;;
    --help)
      echo "Usage: $0 [options]"
      echo "Options:"
      echo "  --port PORT        Port to expose (default: 8180)"
      echo "  --rpc-url URL      Solana RPC URL (default: https://api.mainnet-beta.solana.com)"
      echo "  --api-keys KEYS    Comma-separated list of API keys (default: dev-api-key)"
      echo "  --build            Build the Docker image before running"
      echo "  --help             Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

if [ "$BUILD" = true ]; then
  echo "Building Docker: $IMAGE_NAME"
  docker build -t "$IMAGE_NAME" .
fi

echo "Starting Auger on port $PORT"

docker run -p "$PORT:8180" \
  -e SOLANA_RPC_URL="$RPC_URL" \
  -e API_KEYS="$API_KEYS" \
  -e RUST_LOG=info \
  "$IMAGE_NAME" 
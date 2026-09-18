#!/bin/sh
set -eu
cd "$(dirname "$0")"

# Start backend Axum server on port 3000 if not running.
if ! curl -sf -o /dev/null --max-time 1 http://127.0.0.1:3000/health; then
  echo "Starting Axum backend server..."
  cargo run --manifest-path ../backend/Cargo.toml >>/tmp/backend-startup.log 2>&1 &
fi

# Start frontend dev server on port 8080 if not running
if ! curl -sf -o /dev/null --max-time 1 http://127.0.0.1:8080/ || true; then
  echo "Starting React frontend dev server..."
  npm run dev >>/tmp/app-startup.log 2>&1 &
fi

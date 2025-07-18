#!/bin/bash

# PNRS Benchmark Script
# Tests the performance of the npm registry proxy

set -e

PNRS_URL="${PNRS_URL:-http://localhost:8000}"
CONCURRENT_REQUESTS="${CONCURRENT_REQUESTS:-10}"
TOTAL_REQUESTS="${TOTAL_REQUESTS:-100}"

echo "🚀 PNRS Benchmark"
echo "=================="
echo "Server: $PNRS_URL"
echo "Concurrent requests: $CONCURRENT_REQUESTS"
echo "Total requests: $TOTAL_REQUESTS"
echo ""

# Check if server is running
echo "📡 Checking server health..."
if ! curl -s "$PNRS_URL/" > /dev/null; then
    echo "❌ Server is not responding at $PNRS_URL"
    echo "Please start PNRS first: cargo run"
    exit 1
fi
echo "✅ Server is healthy"
echo ""

# Test packages to benchmark
PACKAGES=(
    "express"
    "lodash"
    "react"
    "vue"
    "axios"
)

echo "🔥 Running benchmarks..."
echo ""

for package in "${PACKAGES[@]}"; do
    echo "Testing package: $package"
    
    # Benchmark package metadata
    echo -n "  Package metadata: "
    ab -n $TOTAL_REQUESTS -c $CONCURRENT_REQUESTS -q "$PNRS_URL/$package" 2>/dev/null | \
        grep "Requests per second" | awk '{print $4 " req/sec"}'
    
    # Benchmark specific version (if available)
    echo -n "  Version metadata: "
    ab -n 50 -c 5 -q "$PNRS_URL/$package/latest" 2>/dev/null | \
        grep "Requests per second" | awk '{print $4 " req/sec"}' || echo "N/A"
    
    echo ""
done

echo "🎯 Testing tarball download performance..."
echo -n "  Lodash tarball: "
ab -n 10 -c 2 -q "$PNRS_URL/lodash/-/lodash-4.17.21.tgz" 2>/dev/null | \
    grep "Requests per second" | awk '{print $4 " req/sec"}'

echo ""
echo "✅ Benchmark complete!"
echo ""
echo "💡 Tips for better performance:"
echo "   - Run in release mode: cargo run --release"
echo "   - Increase worker threads with ROCKET_WORKERS env var"
echo "   - Use a reverse proxy like nginx for production"

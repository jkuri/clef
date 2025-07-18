#!/bin/bash

# Performance comparison between PNRS and direct npm registry access

set -e

PNRS_URL="${PNRS_URL:-http://localhost:8000}"
NPM_REGISTRY="${NPM_REGISTRY:-https://registry.npmjs.org}"
REQUESTS="${REQUESTS:-50}"
CONCURRENCY="${CONCURRENCY:-5}"

echo "⚡ PNRS vs NPM Registry Performance Comparison"
echo "=============================================="
echo "PNRS URL: $PNRS_URL"
echo "NPM Registry: $NPM_REGISTRY"
echo "Requests: $REQUESTS"
echo "Concurrency: $CONCURRENCY"
echo ""

# Test packages
PACKAGES=("express" "lodash" "react")

# Check if PNRS is running
echo "🔍 Checking PNRS availability..."
if ! curl -s "$PNRS_URL/" > /dev/null; then
    echo "❌ PNRS is not running at $PNRS_URL"
    echo "Please start PNRS first: cargo run"
    exit 1
fi
echo "✅ PNRS is running"
echo ""

# Function to run benchmark and extract requests per second
benchmark() {
    local url=$1
    local name=$2
    
    echo -n "  $name: "
    local result=$(ab -n $REQUESTS -c $CONCURRENCY -q "$url" 2>/dev/null | grep "Requests per second" | awk '{print $4}')
    if [ -n "$result" ]; then
        echo "${result} req/sec"
        echo $result
    else
        echo "Failed"
        echo "0"
    fi
}

echo "🏁 Running benchmarks..."
echo ""

total_pnrs=0
total_npm=0
count=0

for package in "${PACKAGES[@]}"; do
    echo "Testing package: $package"
    
    # Benchmark PNRS
    pnrs_result=$(benchmark "$PNRS_URL/$package" "PNRS")
    
    # Benchmark direct NPM
    npm_result=$(benchmark "$NPM_REGISTRY/$package" "NPM Direct")
    
    # Calculate improvement
    if [ "$npm_result" != "0" ] && [ "$pnrs_result" != "0" ]; then
        improvement=$(echo "scale=1; ($pnrs_result - $npm_result) / $npm_result * 100" | bc -l 2>/dev/null || echo "N/A")
        if [ "$improvement" != "N/A" ]; then
            echo "  Improvement: ${improvement}%"
        fi
        
        total_pnrs=$(echo "$total_pnrs + $pnrs_result" | bc -l)
        total_npm=$(echo "$total_npm + $npm_result" | bc -l)
        count=$((count + 1))
    fi
    
    echo ""
done

# Calculate averages
if [ $count -gt 0 ]; then
    avg_pnrs=$(echo "scale=2; $total_pnrs / $count" | bc -l)
    avg_npm=$(echo "scale=2; $total_npm / $count" | bc -l)
    overall_improvement=$(echo "scale=1; ($avg_pnrs - $avg_npm) / $avg_npm * 100" | bc -l)
    
    echo "📊 Summary"
    echo "=========="
    echo "Average PNRS performance: ${avg_pnrs} req/sec"
    echo "Average NPM performance: ${avg_npm} req/sec"
    echo "Overall improvement: ${overall_improvement}%"
    echo ""
fi

echo "🎯 Tarball Download Comparison"
echo "==============================="

# Test tarball download
echo "Testing lodash tarball download..."
pnrs_tarball=$(benchmark "$PNRS_URL/lodash/-/lodash-4.17.21.tgz" "PNRS Tarball")
npm_tarball=$(benchmark "$NPM_REGISTRY/lodash/-/lodash-4.17.21.tgz" "NPM Tarball")

if [ "$npm_tarball" != "0" ] && [ "$pnrs_tarball" != "0" ]; then
    tarball_improvement=$(echo "scale=1; ($pnrs_tarball - $npm_tarball) / $npm_tarball * 100" | bc -l 2>/dev/null || echo "N/A")
    if [ "$tarball_improvement" != "N/A" ]; then
        echo "Tarball improvement: ${tarball_improvement}%"
    fi
fi

echo ""
echo "💡 Notes:"
echo "   - Performance may vary based on network conditions"
echo "   - PNRS adds minimal overhead for proxying"
echo "   - Geographic location affects upstream latency"
echo "   - First requests may be slower due to cold start"
echo ""
echo "🚀 To improve PNRS performance:"
echo "   - Run with --release flag: cargo run --release"
echo "   - Use a reverse proxy with caching"
echo "   - Deploy closer to your users"

#!/bin/bash

# Spatio Benchmark Runner
# This script runs benchmarks and generates results for the README

set -e

echo "🚀 Spatio Benchmark Runner"
echo "=============================="

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the Spatio root directory"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo not found. Please install Rust first."
    exit 1
fi

# Check if benchmark directory exists
if [ ! -d "benches" ]; then
    echo "❌ Error: benches/ directory not found. This project doesn't have benchmarks set up."
    exit 1
fi

echo "📋 Step 1: Building benchmark generator..."
cd scripts

# Build the benchmark generator
if cargo build --release --bin generate_benchmarks 2>/dev/null; then
    echo "✅ Benchmark generator built successfully"
else
    echo "❌ Error: Failed to build benchmark generator"
    echo "💡 Make sure you have a Cargo.toml in the scripts directory"
    exit 1
fi

cd ..

echo "🔧 Step 2: Running benchmarks (this may take several minutes)..."
echo "💡 This will test various operations including:"
echo "   - Basic key-value operations"
echo "   - Spatial indexing and queries"
echo "   - Trajectory operations"
echo "   - Concurrent operations"
echo "   - Large dataset performance"
echo ""

# Run benchmarks with better error handling
if cargo bench 2>&1; then
    echo "✅ Benchmarks completed successfully"
else
    echo "❌ Error: Benchmark execution failed"
    echo "💡 Try running 'cargo bench' manually to see detailed errors"
    exit 1
fi

echo ""
echo "📊 Step 3: Generating benchmark results..."
cd scripts

if cargo run --release --bin generate_benchmarks; then
    echo "✅ Benchmark results generated successfully"
else
    echo "❌ Error: Failed to generate benchmark results"
    echo "💡 Creating sample results instead..."
    cargo run --release --bin create_sample_results
    echo "⚠️  Using sample data - run actual benchmarks for real performance data"
fi

cd ..

echo ""
echo "✅ Benchmark process completed!"
echo ""
echo "📄 Generated files:"
if [ -f "scripts/BENCHMARK_RESULTS.md" ]; then
    echo "   ✅ scripts/BENCHMARK_RESULTS.md (full results)"
elif [ -f "scripts/SAMPLE_BENCHMARK_RESULTS.md" ]; then
    echo "   📊 scripts/SAMPLE_BENCHMARK_RESULTS.md (sample results)"
fi

if [ -f "scripts/benchmark_snippet.md" ]; then
    echo "   ✅ scripts/benchmark_snippet.md (for README)"
elif [ -f "scripts/sample_benchmark_snippet.md" ]; then
    echo "   📊 scripts/sample_benchmark_snippet.md (sample for README)"
fi

echo ""
echo "📝 Quick README update:"
echo "   ./scripts/update_readme.sh"
echo ""
echo "📝 Manual update steps:"
echo "   1. Review the generated benchmark results"
echo "   2. Copy content from benchmark_snippet.md"
echo "   3. Replace the section between <!-- BENCHMARK_RESULTS_START --> and <!-- BENCHMARK_RESULTS_END --> in README.md"
echo "   4. Commit the updated files"
echo ""
echo "💡 Tips:"
echo "   - Run on a quiet system for most accurate results"
echo "   - Results may vary between runs and systems"
echo "   - Large dataset benchmarks take the longest time"

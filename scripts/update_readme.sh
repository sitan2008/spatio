#!/bin/bash

# Update README with benchmark results
# This script replaces the benchmark section in README.md with fresh results

set -e

echo "📝 Spatio README Updater"
echo "============================"

# Check if we're in the right directory
if [ ! -f "README.md" ]; then
    echo "❌ Error: README.md not found. Please run this script from the Spatio root directory"
    exit 1
fi

# Check if benchmark snippet exists
if [ ! -f "scripts/sample_benchmark_snippet.md" ] && [ ! -f "scripts/benchmark_snippet.md" ]; then
    echo "❌ Error: No benchmark snippet found. Please run benchmarks first:"
    echo "   ./scripts/run_benchmarks.sh"
    echo "   or create sample results with:"
    echo "   cd scripts && cargo run --bin create_sample_results"
    exit 1
fi

# Use real benchmark results if available, otherwise use sample
SNIPPET_FILE="scripts/benchmark_snippet.md"
if [ ! -f "$SNIPPET_FILE" ]; then
    SNIPPET_FILE="scripts/sample_benchmark_snippet.md"
    echo "📊 Using sample benchmark results (run ./scripts/run_benchmarks.sh for real results)"
else
    echo "📊 Using real benchmark results"
fi

# Backup original README
cp README.md README.md.backup
echo "💾 Created backup: README.md.backup"

# Extract content between markers from snippet
START_MARKER="<!-- BENCHMARK_RESULTS_START -->"
END_MARKER="<!-- BENCHMARK_RESULTS_END -->"

# Extract the benchmark content (everything between and including markers)
BENCHMARK_CONTENT=$(sed -n "/$START_MARKER/,/$END_MARKER/p" "$SNIPPET_FILE")

if [ -z "$BENCHMARK_CONTENT" ]; then
    echo "❌ Error: Could not find benchmark markers in $SNIPPET_FILE"
    exit 1
fi

# Create temporary file for new README
TEMP_README=$(mktemp)

# Process README.md line by line
IN_BENCHMARK_SECTION=false
while IFS= read -r line; do
    if [[ $line == *"$START_MARKER"* ]]; then
        # Start of benchmark section - output our new content
        echo "$BENCHMARK_CONTENT" >> "$TEMP_README"
        IN_BENCHMARK_SECTION=true
    elif [[ $line == *"$END_MARKER"* ]]; then
        # End of benchmark section - we already included the end marker
        IN_BENCHMARK_SECTION=false
    elif [[ $IN_BENCHMARK_SECTION == false ]]; then
        # Outside benchmark section - keep original line
        echo "$line" >> "$TEMP_README"
    fi
    # Skip lines inside benchmark section (they're replaced)
done < README.md

# Replace original README with updated version
mv "$TEMP_README" README.md

echo "✅ README.md updated successfully!"
echo ""
echo "📋 Changes made:"
echo "   - Updated benchmark results section"
echo "   - Preserved all other content"
echo "   - Backup saved as README.md.backup"
echo ""
echo "🔍 Next steps:"
echo "   1. Review the changes: git diff README.md"
echo "   2. Commit the updated README: git add README.md && git commit -m 'Update benchmark results'"
echo "   3. If something went wrong: mv README.md.backup README.md"
echo ""
echo "💡 Tip: You can also manually review the benchmark content in $SNIPPET_FILE"

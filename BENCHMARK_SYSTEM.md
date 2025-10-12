# 📊 Spatio Benchmark System

A comprehensive, automated benchmark generation system that creates dynamic performance documentation for the SpatioLite project.

## 🎯 Overview

The SpatioLite Benchmark System automatically:
- ✅ Runs comprehensive performance tests using Criterion.rs
- ✅ Generates formatted performance tables with system information
- ✅ Updates README.md with current benchmark results
- ✅ Provides multiple workflows for different use cases
- ✅ Creates sample data for testing and demonstrations

## 🚀 Quick Start

```bash
# Complete workflow: benchmark + README update
just bench

# Just update README with existing results
just bench-update

# Create sample results for testing
just bench-sample

# Get detailed help
just bench-help
```

## 📁 System Architecture

```
scripts/
├── run_benchmarks.sh          # 🎯 Main benchmark orchestrator
├── update_readme.sh           # 📝 README content updater
├── generate_benchmarks.rs     # 🔧 Benchmark result parser
├── create_sample_results.rs   # 📊 Sample data generator
├── benchmark_help.sh          # ❓ Comprehensive help system
├── Cargo.toml                 # 📦 Script dependencies
└── README.md                  # 📚 Detailed documentation
```

## 🔄 Workflow Types

### 1. Complete Benchmark Run (Recommended)
```bash
./scripts/run_benchmarks.sh
```
**Duration**: 5-15 minutes
**Output**: Real performance data + updated README
**Use Case**: Before releases, after performance changes

### 2. Quick README Update
```bash
./scripts/update_readme.sh
```
**Duration**: Seconds
**Output**: Updated README with existing data
**Use Case**: Documentation updates, formatting changes

### 3. Sample Data Generation
```bash
cd scripts && cargo run --bin create_sample_results
```
**Duration**: Seconds
**Output**: Realistic sample benchmark data
**Use Case**: Testing, demos, when real benchmarks fail

## 📊 Benchmark Categories

| Category | Icon | Operations Tested | Key Metrics |
|----------|------|------------------|-------------|
| **Basic Operations** | 🔧 | insert, get, batch | Ops/sec, latency |
| **Spatial Operations** | 📍 | geohash, S2, points | Spatial throughput |
| **Trajectory Operations** | 📈 | trajectory CRUD | Time-series performance |
| **Concurrent Operations** | 🧵 | multi-threaded ops | Concurrency efficiency |
| **High Throughput** | ⚡ | sustained operations | Peak performance |
| **Large Datasets** | 📊 | 1K-100K records | Scalability limits |
| **Persistence** | 💾 | AOF writes/sync | Storage performance |
| **Spatial Indexing** | 🗂️ | indexed vs linear | Index effectiveness |
| **TTL Operations** | ⏰ | expiring data | TTL overhead |

## 📈 Generated Output

### Primary Files
- **`BENCHMARK_RESULTS.md`** - Complete performance report with system info
- **`benchmark_snippet.md`** - README-ready content with comment markers
- **`README.md`** - Automatically updated Performance section

### Sample Output Format
```markdown
| Operation Category | Test | Performance | Throughput |
|-------------------|------|-------------|------------|
| 🔧 Basic Operations | Single Insert | 428 ns | 2.3M ops/sec |
| 📍 Spatial Operations | Point Insert | 315 ns | 3.2M ops/sec |
| 🧵 Concurrent Operations | Concurrent Inserts | 2.1 μs | 46.5M ops/sec |
```

## 🛠️ Technical Implementation

### README Integration
Uses HTML comment markers for automatic content replacement:
```markdown
<!-- BENCHMARK_RESULTS_START -->
... generated content replaces everything here ...
<!-- BENCHMARK_RESULTS_END -->
```

### Benchmark Parsing
- Parses `cargo bench` output using regex patterns
- Supports both standard and Criterion.rs output formats
- Extracts timing data and converts to standardized units
- Groups tests by category for organized presentation

### System Information
Automatically captures:
- CPU model and specifications
- Memory configuration
- Operating system details
- Timestamp and build information

## 🎛️ Customization Guide

### Table Formatting
Edit `scripts/generate_benchmarks.rs`:

```rust
fn format_group_name(name: &str) -> String {
    match name {
        "basic_operations" => "🔧 Basic Operations",
        "your_category" => "🎯 Your Category",
        // Add new categories here
    }
}
```

### Throughput Calculations
```rust
fn calculate_throughput(test_name: &str, time_ns: f64) -> String {
    let ops_per_iter = if test_name.contains("batch_100") {
        100.0  // 100 operations per iteration
    } else {
        1.0    // Single operation per iteration
    };
    // Calculation logic...
}
```

### Category Organization
```rust
let group_order = vec![
    "basic_operations",
    "spatial_operations",
    "your_new_category",  // Add here
];
```

## 🐛 Troubleshooting

| Issue | Symptoms | Solution |
|-------|----------|----------|
| **Benchmarks don't run** | Error during `cargo bench` | Check benchmark code in `benches/` |
| **No output generated** | Empty results files | Use sample generator for testing |
| **README update fails** | Original content preserved | Verify comment markers exist |
| **System info missing** | Generic system description | Platform-specific commands needed |
| **Build failures** | Compilation errors | Check Rust version and dependencies |

### Debug Steps
```bash
# 1. Test basic benchmark functionality
cargo bench

# 2. Test benchmark parsing
cd scripts && cargo run --bin generate_benchmarks

# 3. Test README integration
./scripts/update_readme.sh

# 4. Rollback if needed
mv README.md.backup README.md
```

## 💡 Best Practices

### For Accurate Results
- 🔇 **Quiet System**: Close unnecessary applications
- 🔄 **Multiple Runs**: Verify consistency across runs
- ⚡ **Release Mode**: Always use optimized builds
- 📊 **Stable Environment**: Consistent hardware/software setup

### For Maintenance
- 📅 **Regular Updates**: Run after significant changes
- 💾 **Version Control**: Commit benchmark results with code
- 📈 **Trend Monitoring**: Track performance over time
- 🎯 **Targeted Testing**: Focus on changed components

### For Development
- 🧪 **Sample Data**: Use for workflow testing
- 🔧 **Incremental**: Test individual components
- 📝 **Documentation**: Update help when adding features
- 🔄 **Automation**: Consider git hooks for consistency

## 🚀 Advanced Usage

### Git Integration
```bash
# Pre-commit hook for benchmark updates
echo './scripts/run_benchmarks.sh' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Performance Analysis
```bash
# Compare benchmark results over time
git log --oneline --follow -- BENCHMARK_RESULTS.md

# Track specific performance metrics
grep "Single Insert" BENCHMARK_RESULTS.md
```

### CI/CD Integration
- **Manual Triggers Only**: Avoid automatic benchmark runs in CI
- **Artifact Storage**: Save benchmark results as build artifacts
- **Performance Gates**: Fail builds on significant regressions

## 📊 Performance Expectations

Based on the sample data, SpatioLite achieves:

- **🚀 Basic Operations**: 2-3M ops/sec with sub-microsecond latency
- **📍 Spatial Inserts**: 2M+ ops/sec with automatic indexing
- **🔍 Spatial Queries**: Millisecond-range for complex searches
- **🧵 Concurrency**: Excellent scaling with minimal contention
- **💾 Persistence**: Fast AOF writes with configurable sync policies

## 🔮 Future Enhancements

### Planned Features
- [ ] Performance regression detection
- [ ] Benchmark result comparison tools
- [ ] JSON output format for tooling integration
- [ ] Custom benchmark configuration files
- [ ] Performance trend visualization

### Integration Opportunities
- [ ] Grafana dashboard integration
- [ ] Slack/Discord performance notifications
- [ ] Performance budgets and alerts
- [ ] Automated performance reports

## 📚 Related Documentation

- **`scripts/README.md`** - Detailed script documentation
- **`benches/spatial_benchmarks.rs`** - Actual benchmark implementations
- **`README.md`** - Project documentation with current results
- **`PERFORMANCE.md`** - Static performance analysis (if exists)

## 🎉 Success Metrics

The benchmark system is successful when:
- ✅ Benchmarks run reliably across different systems
- ✅ README stays current with minimal manual effort
- ✅ Performance regressions are caught early
- ✅ Contributors can easily update performance documentation
- ✅ Users have accurate performance expectations

---

**Built with ❤️ for the SpatioLite project**

*For questions or improvements, see the individual script files or run `./scripts/benchmark_help.sh`*

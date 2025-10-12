# Spatio Scripts

This directory contains utility scripts for the Spatio project, focusing on benchmark generation and README maintenance.

## 🚀 Quick Start

```bash
# Complete benchmark workflow (recommended)
./run_benchmarks.sh

# Update README with existing results
./update_readme.sh

# Get help and detailed information
./benchmark_help.sh

# Create sample results for testing
cd scripts && cargo run --bin create_sample_results
```

## 📊 Benchmark System Overview

The Spatio benchmark system provides automated performance testing and documentation generation:

- **🔧 Automated Benchmarking**: Full test suite with Criterion.rs
- **📈 Dynamic Results**: Real-time performance data generation
- **📝 README Integration**: Automatic documentation updates
- **🖥️ System-Aware**: Captures hardware and environment info
- **🎯 Multiple Workflows**: From quick updates to detailed analysis

## 🛠️ Available Tools

### 1. Main Benchmark Runner (`run_benchmarks.sh`)
**Purpose**: Complete benchmark workflow from execution to README generation
**Usage**: `./run_benchmarks.sh`
**Output**:
- `BENCHMARK_RESULTS.md` - Full performance report
- `benchmark_snippet.md` - README-ready content

### 2. README Updater (`update_readme.sh`)
**Purpose**: Update README.md with benchmark results (existing or sample)
**Usage**: `./update_readme.sh`
**Features**:
- Automatic backup creation
- Marker-based content replacement
- Rollback instructions

### 3. Benchmark Generator (`generate_benchmarks.rs`)
**Purpose**: Parse cargo bench output and create formatted tables
**Usage**: `cargo run --bin generate_benchmarks`
**Features**:
- Multiple output formats
- System information capture
- Throughput calculations

### 4. Sample Results Creator (`create_sample_results.rs`)
**Purpose**: Generate realistic sample data for testing and demos
**Usage**: `cargo run --bin create_sample_results`
**Use Cases**:
- Testing documentation workflows
- Demo performance capabilities
- Fallback when real benchmarks fail

### 5. Help System (`benchmark_help.sh`)
**Purpose**: Comprehensive documentation and troubleshooting
**Usage**: `./benchmark_help.sh`
**Content**:
- Detailed workflow explanations
- Troubleshooting guides
- Customization instructions

## 📈 Benchmark Categories

| Category | Icon | Description | Examples |
|----------|------|-------------|----------|
| Basic Operations | 🔧 | Core key-value operations | insert, get, batch |
| Spatial Operations | 📍 | Geospatial indexing and queries | geohash, S2, points |
| Trajectory Operations | 📈 | Time-series spatial data | trajectory insert/query |
| Concurrent Operations | 🧵 | Multi-threaded performance | concurrent inserts |
| High Throughput | ⚡ | Sustained operation rates | bulk operations |
| Large Datasets | 📊 | Big data performance | 1K-100K records |
| Persistence | 💾 | Storage and sync operations | AOF writes |
| Spatial Indexing | 🗂️ | Index performance comparison | indexed vs linear |
| TTL Operations | ⏰ | Time-to-live functionality | expiring data |

## 🔄 Workflows

### Complete Benchmark Update
```bash
./run_benchmarks.sh      # Run benchmarks + generate results
./update_readme.sh       # Update README automatically
git add README.md scripts/
git commit -m "Update benchmark results"
```

### Quick README Update (existing results)
```bash
./update_readme.sh       # Use existing benchmark data
git add README.md
git commit -m "Update performance documentation"
```

### Testing and Development
```bash
cd scripts
cargo run --bin create_sample_results    # Create sample data
cd ..
./update_readme.sh                      # Test README update
mv README.md.backup README.md           # Rollback if needed
```

### Manual Control
```bash
cd scripts && cargo build --release     # Build tools
cd .. && cargo bench                    # Run benchmarks
cd scripts && cargo run --bin generate_benchmarks  # Generate results
cd .. && ./update_readme.sh            # Update README
```

## 📝 README Integration

The system uses HTML comments to mark the benchmark section:

```markdown
<!-- BENCHMARK_RESULTS_START -->
... benchmark content gets inserted here ...
<!-- BENCHMARK_RESULTS_END -->
```

**Benefits**:
- ✅ Preserves all other README content
- ✅ Automatic content replacement
- ✅ Version-controllable benchmark data
- ✅ Easy rollback with backup files

## 🎛️ Customization

### Table Formatting
Edit `generate_benchmarks.rs`:
- `format_group_name()` - Category display names
- `format_test_name()` - Test name formatting
- `calculate_throughput()` - Throughput calculations
- `group_order` - Table organization

### System Information
Modify `get_system_info()` in `generate_benchmarks.rs`:
- Add new system metrics
- Platform-specific commands
- Custom environment details

### README Structure
Update comment markers in `README.md` to change integration points.

## 🐛 Troubleshooting

| Problem | Solution |
|---------|----------|
| Benchmarks fail to run | Check `cargo bench` works manually |
| No output generated | Use `create_sample_results` for testing |
| README update fails | Verify comment markers exist |
| System info missing | Some commands are platform-specific |
| Build errors | Ensure correct directory and Rust version |

## 📋 Requirements

- **Rust**: Latest stable toolchain
- **Platform**: Unix-like environment (macOS, Linux)
- **Dependencies**: No external crates required
- **Benchmarks**: Working `benches/` directory with Criterion.rs

## 💡 Best Practices

- **🔇 Quiet System**: Close other applications during benchmarking
- **🔄 Multiple Runs**: Check consistency across benchmark runs
- **📊 Regular Updates**: Update benchmarks after significant changes
- **💾 Version Control**: Commit benchmark results with code changes
- **📈 Performance Tracking**: Monitor performance trends over time

## 🚀 Advanced Usage

### Git Integration
```bash
# Pre-push hook for automatic benchmark updates
echo './scripts/run_benchmarks.sh && ./scripts/update_readme.sh' > .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

### CI/CD Integration
- Manual benchmark triggers only (not automatic)
- Store results as artifacts
- Compare performance across branches

### Performance Analysis
- Use `BENCHMARK_RESULTS.md` for detailed analysis
- Track performance regressions
- Identify optimization opportunities

---

**📚 For more information, run `./benchmark_help.sh` or check the individual script files.**

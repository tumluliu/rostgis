# Building RostGIS from Source

Complete guide for building RostGIS from source code for development and deployment.

## Quick Build

```bash
# Clone repository
git clone https://github.com/yourusername/rostgis.git
cd rostgis

# Install dependencies
cargo pgrx init

# Build and install
cargo pgrx install
```

## Build Requirements

### System Dependencies

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    postgresql-server-dev-all \
    git \
    curl
```

#### Linux (CentOS/RHEL/Fedora)
```bash
# CentOS/RHEL 7-8
sudo yum install -y \
    gcc \
    pkg-config \
    openssl-devel \
    postgresql-devel \
    git \
    curl

# Fedora/RHEL 9+
sudo dnf install -y \
    gcc \
    pkg-config \
    openssl-devel \
    postgresql-devel \
    git \
    curl
```

#### macOS
```bash
# Using Homebrew
brew install postgresql pkg-config openssl

# Set environment variables
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"
export OPENSSL_ROOT_DIR="/opt/homebrew/opt/openssl"
```

#### Windows
```powershell
# Install PostgreSQL from official installer
# Install Visual Studio Build Tools
# Install pkg-config
choco install pkgconfiglite
```

### Rust Toolchain

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version

# Install required version
rustup install 1.70
rustup default 1.70
```

### pgrx Framework

```bash
# Install specific version compatible with RostGIS
cargo install pgrx --version="=0.11.2"

# Initialize pgrx with PostgreSQL versions
cargo pgrx init

# Or initialize with specific PostgreSQL version
cargo pgrx init --pg15 /usr/bin/pg_config
```

## Building the Extension

### Development Build

```bash
# Clone the repository
git clone https://github.com/yourusername/rostgis.git
cd rostgis

# Check that everything compiles
cargo check

# Run unit tests
cargo test

# Run integration tests
cargo pgrx test

# Build in debug mode (faster compilation)
cargo build
```

### Production Build

```bash
# Build with optimizations
cargo build --release

# Install to PostgreSQL
cargo pgrx install --release

# Or install to specific PostgreSQL version
cargo pgrx install --release --pg-version 15
```

### Cross-Platform Builds

#### Linux to Linux
```bash
# Install cross-compilation targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu

# Build for different architectures
cargo build --target x86_64-unknown-linux-gnu --release
cargo build --target aarch64-unknown-linux-gnu --release
```

#### Building for Docker
```dockerfile
FROM rust:1.70 as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    postgresql-server-dev-all \
    pkg-config \
    libssl-dev

# Install pgrx
RUN cargo install pgrx --version="=0.11.2"

# Copy source code
COPY . /rostgis
WORKDIR /rostgis

# Initialize pgrx
RUN cargo pgrx init

# Build extension
RUN cargo pgrx install --release

# Production image
FROM postgres:15
COPY --from=builder /usr/lib/postgresql/*/lib/rostgis* /usr/lib/postgresql/15/lib/
COPY --from=builder /usr/share/postgresql/*/extension/rostgis* /usr/share/postgresql/15/extension/
```

## Build Configuration

### Environment Variables

```bash
# PostgreSQL configuration
export PG_CONFIG=/usr/bin/pg_config
export PGDATA=/var/lib/postgresql/data

# Rust configuration
export RUSTFLAGS="-C target-cpu=native"  # Optimize for current CPU
export CARGO_TARGET_DIR=/tmp/rostgis-build  # Custom build directory

# SSL configuration (macOS)
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"
export OPENSSL_ROOT_DIR="/opt/homebrew/opt/openssl"

# Performance flags
export CFLAGS="-O3 -march=native"
export CXXFLAGS="-O3 -march=native"
```

### Cargo Configuration

Create `.cargo/config.toml`:
```toml
[build]
# Optimize for current CPU
rustflags = ["-C", "target-cpu=native"]

# Use parallel linking
jobs = 4

[profile.release]
# Maximum optimizations
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"

[profile.dev]
# Faster debug builds
opt-level = 1
debug = true
```

## Development Workflow

### Setting up Development Environment

```bash
# Clone and enter directory
git clone https://github.com/yourusername/rostgis.git
cd rostgis

# Install development tools
cargo install cargo-watch  # Auto-rebuild on changes
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-audit  # Security audit
cargo install cargo-bloat  # Binary size analysis

# Initialize development database
cargo pgrx init --pg15 $(which pg_config)
```

### Incremental Development

```bash
# Watch for changes and rebuild
cargo watch -x check

# Run tests on file changes
cargo watch -x test

# Install on changes (for PostgreSQL testing)
cargo watch -x "pgrx install"
```

### Testing During Development

```bash
# Quick syntax check
cargo check

# Run specific tests
cargo test test_point_creation
cargo test spatial_operators

# Run PostgreSQL integration tests
cargo pgrx test

# Run with specific PostgreSQL version
cargo pgrx test --pg-version 15

# Run benchmarks
cargo bench

# Test SQL compatibility
psql -d test_db -f sql/test_basic_functions.sql
```

## Build Variants

### Debug Build
```bash
# Fast compilation, debug symbols, no optimizations
cargo build

# Characteristics:
# - Faster compilation (seconds vs minutes)
# - Larger binary size
# - Slower runtime performance
# - Full debug information
```

### Release Build
```bash
# Slow compilation, optimized, no debug symbols
cargo build --release

# Characteristics:
# - Slower compilation (minutes)
# - Smaller binary size
# - Maximum runtime performance
# - Optimized for production
```

### Profile-Guided Optimization (PGO)
```bash
# Step 1: Build with instrumentation
RUSTFLAGS="-Cprofile-generate=/tmp/pgo-data" cargo build --release

# Step 2: Run representative workload
cargo pgrx install --release
./run_performance_benchmark.sh

# Step 3: Build with optimization data
RUSTFLAGS="-Cprofile-use=/tmp/pgo-data" cargo build --release
```

## Packaging for Distribution

### Creating Release Artifacts

```bash
# Build release version
cargo build --release

# Create source distribution
git archive --format=tar.gz --prefix=rostgis-0.1.0/ HEAD > rostgis-0.1.0.tar.gz

# Build documentation
cargo doc --release --no-deps

# Create binary package
tar -czf rostgis-0.1.0-linux-x86_64.tar.gz \
    target/release/librostgis.so \
    rostgis.control \
    sql/*.sql \
    docs/
```

### Package Manager Integration

#### Debian Package
```bash
# Install packaging tools
sudo apt install devscripts debhelper

# Create Debian package structure
mkdir -p debian/rostgis/usr/lib/postgresql/15/lib
mkdir -p debian/rostgis/usr/share/postgresql/15/extension

# Copy files
cp target/release/librostgis.so debian/rostgis/usr/lib/postgresql/15/lib/
cp rostgis.control sql/*.sql debian/rostgis/usr/share/postgresql/15/extension/

# Build package
dpkg-deb --build debian/rostgis
```

#### RPM Package
```spec
# rostgis.spec
Name: rostgis
Version: 0.1.0
Release: 1%{?dist}
Summary: PostGIS-compatible spatial extension for PostgreSQL

%description
RostGIS is a high-performance, PostGIS-compatible spatial extension 
for PostgreSQL written in Rust.

%files
/usr/lib64/pgsql/rostgis.so
/usr/share/pgsql/extension/rostgis*
```

## Build Optimization

### Performance Optimization

```bash
# CPU-specific optimizations
export RUSTFLAGS="-C target-cpu=native -C target-feature=+avx2"

# Link-time optimization
export RUSTFLAGS="$RUSTFLAGS -C lto=fat"

# Reduce binary size
export RUSTFLAGS="$RUSTFLAGS -C strip=symbols"

# Build with optimizations
cargo build --release
```

### Memory Usage Optimization

```bash
# Reduce memory usage during compilation
export CARGO_BUILD_JOBS=2  # Limit parallel jobs

# Use custom target directory to save space
export CARGO_TARGET_DIR=/tmp/rostgis-build

# Clean intermediate files
cargo clean
```

### Build Time Optimization

```bash
# Use faster linker (Linux)
sudo apt install lld
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Use shared dependencies
mkdir -p ~/.cargo
echo '[build]' >> ~/.cargo/config.toml
echo 'incremental = true' >> ~/.cargo/config.toml

# Parallel compilation
export CARGO_BUILD_JOBS=$(nproc)
```

## Troubleshooting Build Issues

### Common Build Errors

#### PostgreSQL Headers Not Found
```bash
# Error: "postgres.h: No such file or directory"
# Solution: Install PostgreSQL development headers
sudo apt install postgresql-server-dev-all

# Verify pg_config is available
pg_config --includedir
```

#### Rust Version Issues
```bash
# Error: "rustc version too old"
# Solution: Update Rust toolchain
rustup update stable
rustup default stable
```

#### pgrx Version Mismatch
```bash
# Error: "pgrx version mismatch"
# Solution: Install correct pgrx version
cargo install pgrx --version="=0.11.2" --force
cargo pgrx init
```

#### Link Errors
```bash
# Error: "undefined reference to ..."
# Solution: Check library dependencies
pkg-config --libs openssl
export PKG_CONFIG_PATH="/usr/lib/pkgconfig"
```

### Platform-Specific Issues

#### macOS Issues
```bash
# OpenSSL not found
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"
export OPENSSL_ROOT_DIR="/opt/homebrew/opt/openssl"

# Xcode command line tools
xcode-select --install
```

#### Windows Issues
```powershell
# Visual Studio Build Tools required
# Download from: https://visualstudio.microsoft.com/downloads/

# pkg-config not found
choco install pkgconfiglite

# PostgreSQL paths
set PGSQL_DIR=C:\Program Files\PostgreSQL\15
```

## Continuous Integration

### GitHub Actions
```yaml
# .github/workflows/build.yml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Install PostgreSQL
      run: |
        sudo apt update
        sudo apt install postgresql-15 postgresql-server-dev-15
        
    - name: Install pgrx
      run: cargo install pgrx --version="=0.11.2"
      
    - name: Initialize pgrx
      run: cargo pgrx init
      
    - name: Run tests
      run: |
        cargo test
        cargo pgrx test
        
    - name: Build release
      run: cargo build --release
```

### Docker Build
```dockerfile
# Dockerfile.build
FROM rust:1.70 as builder

RUN apt-get update && apt-get install -y \
    postgresql-server-dev-all \
    pkg-config \
    libssl-dev

RUN cargo install pgrx --version="=0.11.2"

WORKDIR /rostgis
COPY . .

RUN cargo pgrx init
RUN cargo build --release
RUN cargo pgrx install --release

# Test the build
RUN cargo test
RUN cargo pgrx test
```

---

*This build guide covers all aspects of building RostGIS from source. For deployment and packaging, see the [Installation Guide](../user-guide/INSTALLATION.md).* 
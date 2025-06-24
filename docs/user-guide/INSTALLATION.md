# RostGIS Installation Guide

This guide covers installing RostGIS on different platforms and configurations.

## Prerequisites

### System Requirements

- **PostgreSQL**: Version 13, 14, 15, 16, or 17
- **Rust**: Version 1.70 or later
- **Operating System**: Linux, macOS, or Windows
- **Memory**: Minimum 2GB RAM (8GB+ recommended for production)
- **Storage**: 100MB for extension files

### Required Dependencies

#### Rust Toolchain
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### PostgreSQL Development Headers
**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install postgresql-server-dev-all pkg-config libssl-dev
```

**CentOS/RHEL/Fedora:**
```bash
sudo yum install postgresql-devel pkg-config openssl-devel
# OR for newer versions:
sudo dnf install postgresql-devel pkg-config openssl-devel
```

**macOS:**
```bash
# Using Homebrew
brew install postgresql pkg-config openssl

# Using MacPorts
sudo port install postgresql15 +universal
```

**Windows:**
```powershell
# Install PostgreSQL from official installer
# Install Rust from rustup.rs
# Install pkg-config using vcpkg or chocolatey
choco install pkgconfiglite
```

#### pgrx Framework
```bash
# Install the specific version compatible with RostGIS
cargo install pgrx --version="=0.11.2"

# Initialize pgrx (first time only)
cargo pgrx init
```

## Installation Methods

### Method 1: Install from Source (Recommended)

#### 1. Clone the Repository
```bash
git clone https://github.com/yourusername/rostgis.git
cd rostgis
```

#### 2. Build the Extension
```bash
# Verify everything compiles correctly
cargo check

# Run tests to ensure everything works
cargo test

# Run integration tests with PostgreSQL
cargo pgrx test
```

#### 3. Install the Extension
```bash
# Install to the default PostgreSQL installation
cargo pgrx install

# Or install to a specific PostgreSQL version
cargo pgrx install --pg-version 15
```

#### 4. Enable in PostgreSQL
```sql
-- Connect to your database
\c your_database

-- Create the extension
CREATE EXTENSION rostgis;

-- Verify installation
SELECT rostgis_version();
```

### Method 2: Install with Package Manager (Future)

*Package manager installation will be available in future releases for:*
- `apt` (Ubuntu/Debian)
- `yum`/`dnf` (CentOS/RHEL/Fedora)  
- `brew` (macOS)
- `chocolatey` (Windows)

### Method 3: Docker Installation

#### Using Pre-built Image (Future)
```bash
# Pull the official RostGIS image
docker pull rostgis/postgis:latest

# Run with your data directory
docker run -d \
  --name rostgis-db \
  -e POSTGRES_PASSWORD=yourpassword \
  -v /your/data/dir:/var/lib/postgresql/data \
  -p 5432:5432 \
  rostgis/postgis:latest
```

#### Build Your Own Image
```dockerfile
FROM postgres:15

# Install Rust and dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    postgresql-server-dev-15

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install pgrx
RUN cargo install pgrx --version="=0.11.2"
RUN cargo pgrx init --pg15 /usr/lib/postgresql/15/bin/pg_config

# Copy and build RostGIS
COPY . /rostgis
WORKDIR /rostgis
RUN cargo pgrx install --pg-version 15

# Add initialization script
COPY docker-init.sql /docker-entrypoint-initdb.d/
```

## Platform-Specific Instructions

### Ubuntu 22.04 LTS
```bash
# Install dependencies
sudo apt update
sudo apt install -y \
    postgresql-15 \
    postgresql-server-dev-15 \
    pkg-config \
    libssl-dev \
    curl \
    build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install pgrx and build
cargo install pgrx --version="=0.11.2"
cargo pgrx init
git clone https://github.com/yourusername/rostgis.git
cd rostgis
cargo pgrx install
```

### macOS with Homebrew
```bash
# Install dependencies
brew install postgresql@15 pkg-config openssl

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Set environment variables for OpenSSL
export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"
export OPENSSL_ROOT_DIR="/opt/homebrew/opt/openssl"

# Install pgrx and build
cargo install pgrx --version="=0.11.2"
cargo pgrx init --pg15 /opt/homebrew/bin/pg_config
git clone https://github.com/yourusername/rostgis.git
cd rostgis
cargo pgrx install
```

### Windows 10/11
```powershell
# Install PostgreSQL from official installer
# Download from: https://www.postgresql.org/download/windows/

# Install Rust from rustup
# Download from: https://rustup.rs/

# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/

# Install pkg-config
choco install pkgconfiglite

# Open Developer Command Prompt and continue:
cargo install pgrx --version="=0.11.2"
cargo pgrx init
git clone https://github.com/yourusername/rostgis.git
cd rostgis
cargo pgrx install
```

## Post-Installation Setup

### 1. Enable Extension in Database
```sql
-- Connect to PostgreSQL
psql -U postgres -d your_database

-- Create extension
CREATE EXTENSION rostgis;

-- Verify installation
SELECT rostgis_version();
SELECT ST_AsText(ST_MakePoint(1, 2));
```

### 2. Set Up Spatial Indexing
```sql
-- Run the spatial indexing setup script
\i /path/to/rostgis/sql/gist_index_setup.sql

-- Or copy and paste the SQL commands from the file
```

### 3. Test Installation
```sql
-- Create test table
CREATE TABLE test_points (
    id SERIAL PRIMARY KEY,
    name TEXT,
    geom GEOMETRY
);

-- Insert test data
INSERT INTO test_points (name, geom) VALUES
    ('Point A', ST_MakePoint(-122.4194, 37.7749)),
    ('Point B', ST_MakePoint(-74.0060, 40.7128));

-- Create spatial index
CREATE INDEX test_points_geom_idx ON test_points 
USING GIST (geom gist_geometry_ops_simple);

-- Test spatial query
SELECT name, ST_AsText(geom) 
FROM test_points 
WHERE geom && ST_MakePoint(-122.4, 37.7);
```

## Troubleshooting

### Common Issues

#### "pgrx not found" Error
```bash
# Ensure Cargo's bin directory is in PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### "PostgreSQL development headers not found"
```bash
# Ubuntu/Debian
sudo apt install postgresql-server-dev-all

# Verify pg_config is available
pg_config --version
```

#### "Extension not found" Error
```sql
-- Check if extension files are installed
SELECT * FROM pg_available_extensions WHERE name = 'rostgis';

-- If not found, verify installation path
SHOW shared_preload_libraries;
```

#### Permission Errors During Installation
```bash
# Ensure user has write permissions to PostgreSQL directories
sudo chown -R $(whoami) $(pg_config --sharedir)
sudo chown -R $(whoami) $(pg_config --pkglibdir)
```

### Getting Help

If you encounter issues:

1. **Check System Requirements**: Ensure all prerequisites are met
2. **Review Error Messages**: Look for specific error details
3. **Check Logs**: Review PostgreSQL logs for additional context
4. **Search Issues**: Check existing GitHub issues for similar problems
5. **Create Issue**: Report new issues with system details and error messages

### Uninstallation

To remove RostGIS:

```sql
-- Remove extension from databases
DROP EXTENSION IF EXISTS rostgis CASCADE;
```

```bash
# Remove extension files (requires appropriate permissions)
rm -f $(pg_config --pkglibdir)/rostgis*
rm -f $(pg_config --sharedir)/extension/rostgis*
```

## Next Steps

After successful installation:

1. **[Getting Started Guide](GETTING_STARTED.md)** - Learn basic usage
2. **[Spatial Indexing](SPATIAL_INDEXING.md)** - Set up spatial indexes  
3. **[Basic Queries Tutorial](../tutorials/BASIC_QUERIES.md)** - Try your first spatial queries
4. **[Performance Benchmarking](PERFORMANCE_BENCHMARKING.md)** - Test performance in your environment

---

*For the latest installation instructions and platform-specific updates, visit the [GitHub repository](https://github.com/yourusername/rostgis).* 
# Contributing to RostGIS

Thank you for your interest in contributing to RostGIS! This guide will help you get started with development and explain our contribution process.

## 🚀 Quick Start for Contributors

### 1. Development Environment Setup

#### Prerequisites
- Rust 1.70+ (latest stable recommended)
- PostgreSQL 13-17 (with development headers)
- Git
- pgrx framework

#### Setup Steps
```bash
# Clone the repository
git clone https://github.com/yourusername/rostgis.git
cd rostgis

# Install pgrx
cargo install pgrx --version="=0.11.2"
cargo pgrx init

# Build and test
cargo check
cargo test
cargo pgrx test

# Install for local testing
cargo pgrx install
```

### 2. Project Structure

```
rostgis/
├── src/
│   ├── lib.rs              # Extension entry point and PostgreSQL functions
│   ├── geometry.rs         # Core geometry types and operations
│   ├── functions.rs        # Spatial function implementations
│   ├── spatial_index.rs    # GiST indexing support
│   └── utils.rs           # Shared utilities
├── sql/                   # SQL setup scripts
├── tests/                 # Integration tests
├── benches/              # Performance benchmarks
├── docs/                 # Documentation
└── Cargo.toml           # Dependencies and project metadata
```

## 📋 How to Contribute

### Types of Contributions

We welcome various types of contributions:

1. **🐛 Bug Fixes** - Fix existing issues
2. **✨ New Features** - Add spatial functions or capabilities
3. **📚 Documentation** - Improve guides, examples, or API docs
4. **🏗️ Infrastructure** - Testing, CI/CD, tooling improvements
5. **🔧 Performance** - Optimize existing code or algorithms
6. **🧪 Testing** - Add test cases or improve test coverage

### Contribution Workflow

#### Step 1: Choose or Report an Issue

**For Bug Fixes:**
- Check [existing issues](https://github.com/yourusername/rostgis/issues)
- If not found, create a new issue with detailed reproduction steps

**For New Features:**
- Check if the feature aligns with PostGIS compatibility goals
- Create an issue to discuss the feature before implementing
- Wait for maintainer feedback before starting work

#### Step 2: Fork and Branch

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/yourusername/rostgis.git
cd rostgis

# Create a feature branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

#### Step 3: Implement Changes

Follow our [coding guidelines](#coding-guidelines) and ensure:
- Code compiles without warnings
- All tests pass
- New functionality includes tests
- Documentation is updated if needed

#### Step 4: Test Your Changes

```bash
# Run all tests
cargo test
cargo pgrx test

# Run benchmarks if performance-related
cargo bench

# Test with PostgreSQL integration
psql -d test_db -c "CREATE EXTENSION rostgis;"
```

#### Step 5: Submit Pull Request

```bash
# Commit your changes
git add .
git commit -m "Add ST_Buffer function implementation"

# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
```

## 🎯 Coding Guidelines

### Rust Code Style

#### Follow Rust Standards
```bash
# Format code
cargo fmt

# Check linting
cargo clippy

# Ensure no warnings
cargo check
```

#### Code Organization

**Function Implementation Pattern:**
```rust
// In src/lib.rs - PostgreSQL interface
#[pg_extern]
fn st_new_function(input: Geometry) -> Geometry {
    new_function_implementation(input)
}

// In src/functions.rs - Implementation
pub fn new_function_implementation(geom: Geometry) -> Geometry {
    // Actual algorithm here
    geom
}

// Add tests in the same file
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_function() {
        // Test implementation
    }
}
```

**Error Handling:**
```rust
// Use Result types for fallible operations
pub fn parse_geometry(wkt: &str) -> Result<Geometry, GeometryError> {
    // Implementation
}

// Convert to PostgreSQL errors at boundary
#[pg_extern]
fn st_geomfromtext(wkt: &str) -> Result<Geometry, Box<dyn std::error::Error + Send + Sync>> {
    geometry_from_wkt(wkt)
}
```

#### Documentation Standards

**Function Documentation:**
```rust
/// Calculate the area of a polygon geometry.
/// 
/// This function computes the area using the shoelace formula
/// for simple polygons. Multi-polygons are handled by summing
/// the areas of individual polygons.
/// 
/// # Arguments
/// * `geom` - The geometry to calculate area for
/// 
/// # Returns
/// * Area in square units of the geometry's coordinate system
/// * Returns 0.0 for non-polygon geometries
/// 
/// # Examples
/// ```sql
/// SELECT ST_Area(ST_GeomFromText('POLYGON((0 0, 1 0, 1 1, 0 1, 0 0))'));
/// -- Returns: 1.0
/// ```
pub fn geometry_area(geom: Geometry) -> f64 {
    // Implementation
}
```

### SQL Code Style

**Naming Conventions:**
- Use `snake_case` for SQL identifiers
- Function names should match PostGIS exactly (e.g., `ST_MakePoint`)
- Operator class names should be descriptive (`gist_geometry_ops_simple`)

**SQL Formatting:**
```sql
-- Good: Clear formatting and comments
CREATE OPERATOR CLASS gist_geometry_ops_simple
DEFAULT FOR TYPE geometry USING gist AS
    -- Overlap and directional operators
    OPERATOR 1 &&,
    OPERATOR 2 <<,
    OPERATOR 3 >>,
    
    -- Support functions
    FUNCTION 1 geometry_gist_compress(geometry),
    FUNCTION 2 bbox_union(bbox, bbox);
```

## 🧪 Testing Guidelines

### Test Categories

#### 1. Unit Tests
Test individual functions in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        let distance = point_distance(&p1, &p2);
        assert!((distance - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_invalid_wkt() {
        let result = parse_wkt("INVALID WKT");
        assert!(result.is_err());
    }
}
```

#### 2. Integration Tests
Test PostgreSQL integration:

```rust
#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use crate::*;
    use pgrx::prelude::*;

    #[pg_test]
    fn test_st_makepoint_integration() {
        let point = crate::st_makepoint(1.0, 2.0);
        assert_eq!(crate::st_x(point.clone()).unwrap(), 1.0);
        assert_eq!(crate::st_y(point).unwrap(), 2.0);
    }

    #[pg_test]
    fn test_spatial_indexing() {
        // Test index creation and usage
        Spi::run("CREATE TABLE test_points (id INT, geom GEOMETRY);").unwrap();
        Spi::run("INSERT INTO test_points VALUES (1, ST_MakePoint(1, 2));").unwrap();
        Spi::run("CREATE INDEX test_idx ON test_points USING GIST (geom gist_geometry_ops_simple);").unwrap();
        
        let result = Spi::get_one::<i32>("SELECT COUNT(*) FROM test_points WHERE geom && ST_MakePoint(1, 2);").unwrap();
        assert_eq!(result, Some(1));
    }
}
```

#### 3. Performance Tests
Add benchmarks for performance-critical code:

```rust
#[cfg(test)]
mod benches {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    use super::*;

    fn bench_point_creation(c: &mut Criterion) {
        c.bench_function("point creation", |b| {
            b.iter(|| Point::new(black_box(1.0), black_box(2.0)))
        });
    }

    fn bench_wkt_parsing(c: &mut Criterion) {
        c.bench_function("wkt parsing", |b| {
            b.iter(|| parse_wkt(black_box("POINT(1 2)")))
        });
    }

    criterion_group!(benches, bench_point_creation, bench_wkt_parsing);
    criterion_main!(benches);
}
```

### Running Tests

```bash
# Run Rust unit tests
cargo test

# Run PostgreSQL integration tests  
cargo pgrx test

# Run specific test
cargo test test_point_distance

# Run benchmarks
cargo bench

# Test with specific PostgreSQL version
cargo pgrx test --pg-version 15
```

## 📝 Documentation Requirements

### Code Documentation

#### Required for Public Functions
- Purpose and behavior description
- Parameter descriptions with types
- Return value description
- Example usage (SQL when applicable)
- Error conditions

#### Required for Complex Algorithms
- Algorithm explanation
- Complexity analysis
- References to papers or standards

### User Documentation

When adding new features, update:
- Function reference in `docs/api-reference/`
- User guides if the feature changes workflows
- Tutorial examples if applicable

## 🚦 Pull Request Guidelines

### PR Title and Description

**Good PR Title:**
```
Add ST_Buffer function with configurable segments

Fix #123: Incorrect area calculation for complex polygons

Performance: Optimize point-in-polygon testing
```

**PR Description Template:**
```markdown
## Description
Brief description of changes and motivation.

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that causes existing functionality to change)
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated  
- [ ] Benchmarks added/updated (if performance related)
- [ ] Manual testing completed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex areas
- [ ] Documentation updated
- [ ] No breaking changes (or properly documented)

## Related Issues
Fixes #123
Addresses #456
```

### Review Process

1. **Automated Checks**: CI runs tests, linting, formatting
2. **Maintainer Review**: Code quality, design, PostGIS compatibility
3. **Testing**: Reviewers may test changes locally
4. **Approval**: At least one maintainer approval required
5. **Merge**: Squash and merge for clean history

### Review Criteria

**Code Quality:**
- Follows Rust best practices
- Proper error handling
- Adequate test coverage
- Clear documentation

**Design:**
- Fits well with existing architecture
- Maintains PostGIS compatibility
- Performance considerations addressed

**Testing:**
- Comprehensive test coverage
- Edge cases considered
- Performance impact measured

## 🎯 Specific Contribution Areas

### High-Priority Areas

#### 1. PostGIS Function Compatibility
**Goal**: Implement missing PostGIS functions

**Current Priorities:**
- `ST_GeomFromWKB` - Binary format parsing
- `ST_Buffer` - Geometry buffering
- `ST_Union` - Geometry union operations
- `ST_Intersection` - Geometry intersection

**Guidelines:**
- Follow PostGIS function signatures exactly
- Implement the same edge case behavior
- Add comprehensive tests comparing outputs

#### 2. Performance Optimization
**Goal**: Improve spatial operation performance

**Focus Areas:**
- SIMD acceleration for coordinate operations
- Memory allocation optimization
- Algorithmic improvements

**Guidelines:**
- Benchmark before and after changes
- Document performance characteristics
- Consider memory usage impact

#### 3. Spatial Indexing Improvements
**Goal**: Enhanced GiST index support

**Areas for Improvement:**
- Full `consistent` function implementation
- Advanced `picksplit` strategies
- Index-only scan support

#### 4. Error Handling and Robustness
**Goal**: Better error messages and edge case handling

**Focus Areas:**
- Comprehensive WKT/WKB parsing error recovery
- Better geometry validation
- Meaningful error messages

### Documentation Contributions

#### High-Impact Documentation

1. **Tutorial Content**
   - Real-world usage examples
   - Migration guides from PostGIS
   - Performance optimization guides

2. **API Reference**
   - Complete function documentation
   - Usage examples for each function
   - Parameter validation details

3. **Developer Guides**
   - Architecture deep-dives
   - Adding new spatial functions
   - Testing strategies

## 🐛 Bug Report Guidelines

### Bug Report Template

```markdown
**Describe the Bug**
Clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Create table with '...'
2. Insert data '....'
3. Run query '....'
4. See error

**Expected Behavior**
What you expected to happen.

**Actual Behavior**
What actually happened.

**Environment:**
- PostgreSQL version: [e.g., 15.3]
- RostGIS version: [e.g., 0.1.0]
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.70]

**Sample Data**
```sql
-- Minimal example that reproduces the issue
CREATE TABLE test_table (...);
INSERT INTO test_table VALUES (...);
SELECT problematic_function(...);
```

**Additional Context**
Any other context about the problem.
```

## 💡 Feature Request Guidelines

### Feature Request Template

```markdown
**Feature Description**
Clear description of the feature you'd like to see.

**PostGIS Compatibility**
- [ ] This feature exists in PostGIS
- [ ] This is a new feature not in PostGIS
- [ ] This is a RostGIS-specific optimization

**Use Case**
Describe your use case and why this feature would be helpful.

**Proposed Implementation**
If you have ideas about how this could be implemented.

**Examples**
```sql
-- Example of how the feature would be used
SELECT new_function(geom) FROM spatial_table;
```

**Additional Context**
Any other context or references.
```

## 🏆 Recognition

### Contributor Recognition

We recognize contributors through:
- Contributor credits in release notes
- GitHub contributor graphs
- Special recognition for significant contributions

### Types of Recognition

- **Code Contributors**: Implementation of features, bug fixes
- **Documentation Contributors**: Guides, examples, API docs
- **Testing Contributors**: Test cases, issue reproduction
- **Community Contributors**: Issue triage, user support

## 📞 Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests, questions
- **GitHub Discussions**: General questions, brainstorming
- **Documentation**: Check existing docs first

### Response Times

- **Issues**: We aim to respond within 48 hours
- **Pull Requests**: Initial review within 72 hours
- **Security Issues**: Within 24 hours

---

*Thank you for contributing to RostGIS! Every contribution helps make spatial data processing in PostgreSQL better.* 
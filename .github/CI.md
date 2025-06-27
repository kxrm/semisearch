# GitHub Actions CI/CD

This directory contains the GitHub Actions workflows for the semisearch project, following the architecture plan's testing strategy.

## Workflows

### 🔄 CI Pipeline (`.github/workflows/ci.yml`)

**Triggers:** Push to `main`/`develop`, Pull Requests to `main`

**Jobs:**
- **Test Matrix:** Tests on Ubuntu, Windows, macOS with Rust stable/beta/MSRV
- **Feature Tests:** Comprehensive testing of Phase 2 enhanced search features
- **Security Audit:** Dependency vulnerability and license checking
- **Documentation:** Doc generation and link validation
- **Binary Build:** Cross-platform release binary preparation
- **Architecture Compliance:** Validates project follows the architecture plan

**Key Features:**
- ✅ **19 Unit Tests** + **8 Integration Tests**
- ✅ **Cross-platform compatibility** (Linux, Windows, macOS, ARM64)
- ✅ **Code quality** (clippy, formatting, security audit)
- ✅ **Architecture plan compliance** verification
- ✅ **MVP and Phase 2 feature validation**

### 🚀 Release Pipeline (`.github/workflows/release.yml`)

**Triggers:** Git tags matching `v*.*.*`

**Jobs:**
- **GitHub Release:** Automated release creation with changelog
- **Multi-platform Binaries:** Linux x64/ARM64, Windows x64, macOS x64
- **Crates.io Publishing:** Automatic publishing (if token configured)
- **Homebrew Updates:** Formula updates (if configured)

## Test Structure Validation

The CI validates our test structure matches the architecture plan:

```
tests/
├── integration_tests.rs     ✅ 8 comprehensive integration tests
├── run-all.sh              ✅ Master test runner
├── test-search.sh           ✅ Search functionality tests
├── test-performance.sh      ✅ Performance benchmarking
├── test_phase2_features.sh  ✅ Phase 2 feature validation
└── validate-ci.sh           ✅ CI environment validation
```

## Architecture Plan Compliance

The CI automatically verifies:

### ✅ **MVP Features (Phase 1)**
- CLI interface with subcommands (`search`, `index`, `config`)
- File traversal with `.gitignore` respect
- Multiple output formats (plain text, JSON)
- Basic keyword search functionality

### ✅ **Enhanced Search (Phase 2)**
- Fuzzy matching with typo tolerance
- Regular expression pattern matching
- Case-sensitive/insensitive search modes
- Enhanced typo tolerance using edit distance
- Search result scoring and ranking (0.0-1.0)
- Comprehensive test coverage (19 tests total)

### 🔄 **Ready for Phase 3**
- Foundation stable for semantic search implementation
- Test infrastructure supports ML model integration
- Performance benchmarking ready for embedding models

## Running Tests Locally

```bash
# Validate CI environment
bash tests/validate-ci.sh

# Run comprehensive test suite
bash tests/run-all.sh

# Run specific test categories
cargo test --lib                    # Unit tests
cargo test --test integration_tests # Integration tests
bash tests/test_phase2_features.sh  # Phase 2 features

# Performance testing
bash tests/test-performance.sh
```

## CI Environment Requirements

**Required:**
- Rust toolchain (stable, beta, MSRV 1.70.0)
- Cargo with standard tools (clippy, rustfmt)

**Optional (graceful degradation):**
- `jq` for JSON validation
- `timeout` for test time limits
- `cross` for ARM64 builds

## Security & Quality

**Automated Checks:**
- 🔒 **Security audit** via `cargo-audit` and `cargo-deny`
- 📋 **License compliance** checking
- 🧹 **Code quality** via clippy with deny warnings
- 📝 **Formatting** enforcement
- 🔗 **Documentation** link validation

## Performance Targets

**CI Performance Validation:**
- ✅ **Build time:** < 2 minutes
- ✅ **Test execution:** < 5 minutes total
- ✅ **Memory usage:** < 500MB during tests
- ✅ **Cross-platform:** All platforms pass

## Troubleshooting

### Common CI Issues

1. **Test timeouts:** Increase timeout in workflow (currently 300s)
2. **Permission errors:** Scripts auto-fixed by `validate-ci.sh`
3. **Missing dependencies:** CI installs required tools automatically
4. **Platform differences:** Tests adapt to available tools

### Local Development

```bash
# Simulate CI environment
export CI=true
export RUST_BACKTRACE=1
bash tests/validate-ci.sh
bash tests/run-all.sh
```

## Status Badges

Add to your README.md:

```markdown
[![CI](https://github.com/your-username/semisearch/workflows/CI%2FCD%20Pipeline/badge.svg)](https://github.com/your-username/semisearch/actions)
[![Release](https://github.com/your-username/semisearch/workflows/Release/badge.svg)](https://github.com/your-username/semisearch/releases)
```

## Next Steps

**Phase 3 Preparation:**
- CI ready for ML model testing
- Performance benchmarks for embedding comparison
- Multi-platform compatibility verified
- Security and quality gates established

The CI/CD pipeline provides a solid foundation for continued development through Phase 3 (Semantic Search) and beyond.

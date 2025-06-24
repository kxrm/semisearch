# Contributing to Semisearch

Welcome to the Semisearch project! We're excited to have you contribute. This guide will help you understand our development workflow and contribution process.

## 🔒 Branch Protection & Workflow

The `main` branch is **protected** and requires the following for any changes:

### ✅ Required Conditions
- **Pull Request Required**: Direct pushes to `main` are disabled
- **CI/CD Pipeline**: All tests must pass
- **Code Review**: At least 1 approving review required
- **No Force Pushes**: History must be preserved
- **Admin Enforcement**: Even admins must follow these rules

### 🚀 Auto-Merge System
PRs from `feature/*`, `hotfix/*`, or `bugfix/*` branches will be **automatically merged** when:
- ✅ All CI checks pass
- ✅ Required approvals received
- ✅ No pending change requests
- ✅ Branch naming follows conventions

## 🌿 Branch Naming Conventions

Use these prefixes for automatic processing:

```bash
feature/add-semantic-indexing     # New features
bugfix/fix-regex-parsing         # Bug fixes  
hotfix/critical-security-patch   # Urgent fixes
```

## 🔄 Development Workflow

### 1. Create Feature Branch
```bash
git checkout main
git pull origin main
git checkout -b feature/your-feature-name
```

### 2. Make Changes
- Write code following project standards
- Add/update tests for your changes
- Update documentation if needed
- Run local tests before committing

### 3. Local Testing
```bash
# Run full test suite
cargo test

# Check code quality
cargo clippy --all-targets --all-features -- -D warnings

# Verify formatting
cargo fmt --check

# Run performance tests
./tests/test-performance.sh

# Run integration tests
./tests/run-all.sh
```

### 4. Commit & Push
```bash
git add .
git commit -m "feat: add semantic indexing capability"
git push origin feature/your-feature-name
```

### 5. Create Pull Request
- Use the PR template (auto-populated)
- Fill out all sections completely
- Link related issues
- Request review from codeowners

### 6. Auto-Merge Process
Once your PR is approved and CI passes:
1. 🤖 Auto-merge bot will detect eligibility
2. ✅ Squash-merge will be performed automatically
3. 🧹 Feature branch will be deleted
4. 📬 You'll receive a notification

## 🧪 Testing Requirements

### Unit Tests
- All new code must have unit tests
- Existing tests must continue to pass
- Aim for >80% code coverage

### Integration Tests
- Update integration tests for new features
- Ensure end-to-end functionality works
- Test CLI interface changes

### Performance Tests
- Run performance benchmarks for changes affecting speed
- Ensure no regression in search performance
- Document any intentional performance trade-offs

## 📋 Code Standards

### Rust Guidelines
- Follow Rust standard formatting (`cargo fmt`)
- Pass all clippy lints (`cargo clippy`)
- Use meaningful variable and function names
- Add documentation for public APIs

### Commit Messages
Use conventional commit format:
```
type(scope): description

feat: add new search algorithm
fix: resolve regex parsing issue
docs: update API documentation
test: add fuzzy search test cases
perf: optimize file traversal
```

### Documentation
- Update README.md for user-facing changes
- Add inline code documentation
- Update architecture plan for significant changes

## 🔍 Code Review Process

### For Contributors
- Respond to feedback promptly
- Make requested changes in new commits
- Don't force-push after review starts
- Engage constructively with reviewers

### For Reviewers
- Review within 48 hours when possible
- Focus on correctness, maintainability, and performance
- Provide constructive feedback
- Approve when ready for merge

## 🚨 Emergency Procedures

### Hotfix Process
For critical issues requiring immediate attention:

1. Create `hotfix/*` branch from `main`
2. Make minimal necessary changes
3. Expedited review process (can be merged with single approval)
4. Auto-merge will handle the rest

### Rolling Back
If issues are discovered after merge:
1. Create `hotfix/rollback-*` branch
2. Revert problematic changes
3. Follow normal review process

## 🛠️ Development Setup

### Prerequisites
- Rust 1.80.0+ (MSRV)
- Git with GitHub CLI (`gh`)
- Docker (for dev container)

### Local Environment
```bash
# Clone repository
git clone https://github.com/kxrm/semisearch.git
cd semisearch

# Build project
cargo build

# Run tests
cargo test

# Install CLI locally
cargo install --path .
```

### Dev Container
Use the provided dev container for consistent environment:
```bash
# Open in VS Code with dev containers extension
code .
# Select "Reopen in Container"
```

## 📞 Getting Help

- **Issues**: Open GitHub issues for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Code Review**: Tag `@kxrm` for review assistance

## 🎯 Architecture Alignment

All contributions should align with the [Semantic Search Architecture Plan](docs/SEMANTIC_SEARCH_ARCHITECTURE_PLAN.md):

- **Phase 1**: MVP features (✅ Complete)
- **Phase 2**: Enhanced search (✅ Complete)  
- **Phase 3**: Semantic search & ML integration (🔄 In Progress)

## 📜 License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT).

---

Thank you for contributing to Semisearch! 🙏 
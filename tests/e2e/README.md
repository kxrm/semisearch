# SemiSearch End-to-End Tests

This directory contains comprehensive end-to-end tests for the SemiSearch application, implementing Task 3.1.3 (Automated UX Validation) from the UX Remediation Plan.

## Purpose

These tests validate that the UX improvements specified in the remediation plan have been successfully implemented. They test the application from a user's perspective, ensuring that:

1. **Simplicity**: Non-technical users can search without reading documentation
2. **Error Recovery**: When search fails, users know exactly what to try next
3. **Progressive Disclosure**: Advanced features exist but don't overwhelm beginners
4. **Consistency**: Same query gives predictable results across different systems

## Test Structure

The tests are organized into four main categories:

1. **UX Validation Tests** (`ux_validation_tests.rs`): Tests the core user experience, including basic search functionality, result presentation, and command behavior.

2. **Context Detection Tests** (`context_detection_tests.rs`): Tests project type detection and smart defaults based on project context.

3. **Error Handling Tests** (`error_handling_tests.rs`): Tests user-friendly error messages, recovery suggestions, and handling of edge cases.

4. **Advanced Features Tests** (`advanced_features_tests.rs`): Tests that advanced features are hidden by default but accessible via the `--advanced` flag.

## Running the Tests

These tests are included in the standard test suite and will run with:

```bash
cargo test
```

To run only the end-to-end tests:

```bash
cargo test --test e2e_tests
```

To run a specific category of tests:

```bash
cargo test --test e2e_tests ux_validation
```

## Test Data

The tests use the test data in `tests/test-data/` directory, which contains:

1. **Code Projects**: Rust, JavaScript, and Python projects
2. **Documentation Projects**: API docs and user manuals
3. **Mixed Projects**: Web applications with both code and docs
4. **Mixed Documents**: Text files, data files, scripts, and notes

This test data should not be modified without updating the tests, as they serve as stable test fixtures.

## Test Design Philosophy

These tests were designed to:

1. **Be Resilient**: They adapt to different implementations as long as the UX requirements are met
2. **Focus on UX**: They test what users experience, not implementation details
3. **Validate Requirements**: Each test directly maps to requirements in the UX Remediation Plan
4. **Provide Feedback**: Failed tests include helpful messages explaining what UX requirement was not met

## Adding New Tests

When adding new tests:

1. Place them in the appropriate category file
2. Ensure they test user-facing behavior, not implementation details
3. Make them resilient to implementation changes
4. Include clear failure messages that explain what UX requirement was not met 
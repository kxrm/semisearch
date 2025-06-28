# Test Data Directory

## ⚠️ WARNING ⚠️

**DO NOT MODIFY** the structure, content, or naming of files in this directory without running all tests, including end-to-end tests. This directory serves as a stable test fixture for ensuring consistent behavior across versions.

## Purpose

This directory contains various mock projects and file types that represent real-world usage scenarios for SemiSearch. The data is structured to test different search strategies, file type detection, and context-aware features.

The directory is organized as follows:

- **`code-projects/`**: Contains mock programming projects in various languages (Rust, JavaScript, Python, etc.)
- **`docs-projects/`**: Contains documentation-focused projects with various markup formats
- **`mixed-projects/`**: Contains projects that have both code and documentation
- **`mixed-documents/`**: Mimics a typical user's Documents folder with various everyday file types (CSV, TXT, scripts, etc.)

## Usage

This test data is used for:

1. End-to-end testing of search functionality
2. Validating file type detection and context awareness
3. Ensuring consistent behavior across versions
4. Local development and debugging

## Contributing

If you need to modify this directory:

1. Run all existing tests first to understand current behavior
2. Make your changes
3. Update any tests that depend on specific file paths or content
4. Run all tests again to ensure nothing breaks
5. Document your changes in the PR 
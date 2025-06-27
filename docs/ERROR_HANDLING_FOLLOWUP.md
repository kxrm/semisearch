# Error Handling Follow-up Tasks

This document tracks areas that need error handling improvements following Task 1.2.1 implementation.

## Completed in Task 1.2.1
- ✅ User-friendly error types in `src/errors/user_errors.rs`
- ✅ Error translator in `src/errors/translator.rs` 
- ✅ Main error handling in `src/main_new.rs`
- ✅ JSON error format support
- ✅ Proper stderr usage and exit codes
- ✅ Comprehensive test suite

## Areas Needing Follow-up

### 1. Core Module Error Handling
**Files:** `src/core/indexer.rs`, `src/core/embedder.rs`

**Current Issues:**
- Still using `eprintln!` directly instead of user-friendly error system
- Technical error messages exposed to users
- Example from `src/core/indexer.rs:162`:
  ```rust
  eprintln!("Error processing: {path} - {e}", path = entry.path().display(), e = e);
  ```

**Recommendation:**
- Convert these to use `UserError` types
- Route through error translator
- Provide contextual help for indexing failures

### 2. Search Module Error Handling  
**Files:** `src/search/strategy.rs`, `src/lib.rs`

**Current Issues:**
- Basic error handling in search functions
- Limited user guidance for search failures
- Example from `src/lib.rs:149`:
  ```rust
  Err(_) => return Ok(None), // Invalid regex
  ```

**Recommendation:**
- Add specific error types for invalid regex patterns
- Provide suggestions for fixing regex syntax
- Better handling of file permission errors during search

### 3. Storage Module Error Handling
**Files:** `src/storage/database.rs`

**Current Issues:**
- Database errors are technical and not user-friendly
- No guidance for common database issues
- SQLite errors exposed directly

**Recommendation:**
- Create database-specific user error types
- Add suggestions for common database problems
- Handle database corruption gracefully

### 4. Warning vs Error Classification
**Current Issues:**
- Some warnings should be errors and vice versa
- Inconsistent use of stderr for warnings
- Example: Neural embedding fallback should be warning, not error

**Recommendation:**
- Audit all error/warning classifications
- Ensure warnings go to stderr, errors cause exit
- Create clear guidelines for error vs warning

### 5. Contextual Help Integration
**Current Issues:**
- Error messages don't integrate with `help-me` command
- No dynamic help based on recent errors
- Limited cross-referencing between errors and help

**Recommendation:**
- Add error codes for cross-referencing
- Integrate with contextual help system
- Track common user errors for better suggestions

### 6. Configuration Error Handling
**Files:** `src/main_new.rs` (config functions)

**Current Issues:**
- Configuration errors are technical
- No validation of user-provided paths/options
- Poor error messages for invalid configurations

**Recommendation:**
- Add configuration validation
- User-friendly messages for config issues
- Suggestions for fixing common config problems

## Implementation Priority

1. **High Priority:** Core module error handling (affects all operations)
2. **Medium Priority:** Search module improvements (affects user experience)
3. **Medium Priority:** Storage module error handling (affects data integrity)
4. **Low Priority:** Warning classification cleanup
5. **Low Priority:** Contextual help integration

## Testing Requirements

Each follow-up area should include:
- Unit tests for error translation
- Integration tests for user experience
- JSON format validation
- Exit code verification
- Stderr/stdout separation validation

## Compatibility Notes

- All changes must maintain backward compatibility with existing error handling
- JSON format must remain consistent
- Exit codes should follow established conventions
- No breaking changes to public APIs 
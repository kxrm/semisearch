// Main e2e test file that includes all the test modules

// UX validation tests
#[path = "e2e/ux_validation_tests.rs"]
mod ux_validation_tests;

// Context detection tests
#[path = "e2e/context_detection_tests.rs"]
mod context_detection_tests;

// Error handling tests
#[path = "e2e/error_handling_tests.rs"]
mod error_handling_tests;

// Advanced features tests
#[path = "e2e/advanced_features_tests.rs"]
mod advanced_features_tests;

// This ensures the e2e tests are included in the test suite
#[test]
fn dummy_test_to_ensure_e2e_tests_are_included() {
    // This test does nothing but ensures that this file is compiled
    // and the e2e test modules are included
    assert!(true);
}

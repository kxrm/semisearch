use std::fs;
use tempfile::TempDir;

// Import the modules we'll implement
use search::context::project_detector::{ProjectDetector, ProjectType};
use search::context::search_config::ContextAwareConfig;

#[test]
fn test_detect_rust_project() {
    let temp_dir = TempDir::new().unwrap();
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::RustProject));
}

#[test]
fn test_detect_javascript_project() {
    let temp_dir = TempDir::new().unwrap();
    let package_json = temp_dir.path().join("package.json");
    fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::JavaScriptProject));
}

#[test]
fn test_detect_python_project_with_requirements() {
    let temp_dir = TempDir::new().unwrap();
    let requirements = temp_dir.path().join("requirements.txt");
    fs::write(&requirements, "pytest==7.0.0").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::PythonProject));
}

#[test]
fn test_detect_python_project_with_pyproject() {
    let temp_dir = TempDir::new().unwrap();
    let pyproject = temp_dir.path().join("pyproject.toml");
    fs::write(&pyproject, "[tool.poetry]\nname = \"test\"").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::PythonProject));
}

#[test]
fn test_detect_documentation_project() {
    let temp_dir = TempDir::new().unwrap();

    // Create mostly markdown files
    for i in 0..8 {
        let md_file = temp_dir.path().join(format!("doc{i}.md"));
        fs::write(&md_file, "# Documentation").unwrap();
    }

    // Create a few other files
    let txt_file = temp_dir.path().join("notes.txt");
    fs::write(&txt_file, "Some notes").unwrap();

    let rs_file = temp_dir.path().join("script.rs");
    fs::write(&rs_file, "fn main() {}").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::Documentation));
}

#[test]
fn test_detect_mixed_project() {
    let temp_dir = TempDir::new().unwrap();

    // Create equal mix of file types
    for i in 0..3 {
        let md_file = temp_dir.path().join(format!("doc{i}.md"));
        fs::write(&md_file, "# Documentation").unwrap();

        let rs_file = temp_dir.path().join(format!("src{i}.rs"));
        fs::write(&rs_file, "fn main() {}").unwrap();

        let js_file = temp_dir.path().join(format!("app{i}.js"));
        fs::write(&js_file, "console.log('test');").unwrap();
    }

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::Mixed));
}

#[test]
fn test_detect_unknown_project() {
    let temp_dir = TempDir::new().unwrap();

    // Create some random files
    let txt_file = temp_dir.path().join("data.txt");
    fs::write(&txt_file, "Some data").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::Unknown));
}

#[test]
fn test_context_aware_config_for_rust() {
    let config = ContextAwareConfig::from_project_type(ProjectType::RustProject);

    assert!(matches!(config.project_type, ProjectType::RustProject));
    assert_eq!(
        config.search_paths,
        vec!["src/".to_string(), "tests/".to_string()]
    );
    assert_eq!(config.file_patterns, vec!["*.rs".to_string()]);
    assert!(config.ignore_patterns.contains(&"target/".to_string()));
    assert!(config.ignore_patterns.contains(&".git/".to_string()));
    assert!(config.ignore_patterns.contains(&"Cargo.lock".to_string()));
}

#[test]
fn test_context_aware_config_for_javascript() {
    let config = ContextAwareConfig::from_project_type(ProjectType::JavaScriptProject);

    assert!(matches!(
        config.project_type,
        ProjectType::JavaScriptProject
    ));
    assert_eq!(
        config.search_paths,
        vec!["src/".to_string(), "lib/".to_string()]
    );
    assert_eq!(
        config.file_patterns,
        vec!["*.js".to_string(), "*.ts".to_string()]
    );
    assert!(config
        .ignore_patterns
        .contains(&"node_modules/".to_string()));
    assert!(config.ignore_patterns.contains(&"dist/".to_string()));
    assert!(config.ignore_patterns.contains(&".git/".to_string()));
}

#[test]
fn test_context_aware_config_for_python() {
    let config = ContextAwareConfig::from_project_type(ProjectType::PythonProject);

    assert!(matches!(config.project_type, ProjectType::PythonProject));
    assert_eq!(
        config.search_paths,
        vec!["src/".to_string(), "lib/".to_string(), "tests/".to_string()]
    );
    assert_eq!(config.file_patterns, vec!["*.py".to_string()]);
    assert!(config.ignore_patterns.contains(&"__pycache__/".to_string()));
    assert!(config.ignore_patterns.contains(&"venv/".to_string()));
    assert!(config.ignore_patterns.contains(&".git/".to_string()));
}

#[test]
fn test_context_aware_config_for_documentation() {
    let config = ContextAwareConfig::from_project_type(ProjectType::Documentation);

    assert!(matches!(config.project_type, ProjectType::Documentation));
    assert_eq!(config.search_paths, vec!["./".to_string()]);
    assert_eq!(
        config.file_patterns,
        vec!["*.md".to_string(), "*.txt".to_string()]
    );
    assert!(config.ignore_patterns.contains(&".git/".to_string()));
    assert!(!config.ignore_patterns.is_empty());
}

#[test]
fn test_context_aware_config_for_mixed() {
    let config = ContextAwareConfig::from_project_type(ProjectType::Mixed);

    assert!(matches!(config.project_type, ProjectType::Mixed));
    assert_eq!(config.search_paths, vec!["./".to_string()]);
    assert_eq!(config.file_patterns, vec!["*".to_string()]);
    assert!(config.ignore_patterns.contains(&"target/".to_string()));
    assert!(config
        .ignore_patterns
        .contains(&"node_modules/".to_string()));
    assert!(config.ignore_patterns.contains(&".git/".to_string()));
}

#[test]
fn test_context_aware_config_default() {
    let config = ContextAwareConfig::default();

    assert!(matches!(config.project_type, ProjectType::Unknown));
    assert_eq!(config.search_paths, vec!["./".to_string()]);
    assert_eq!(config.file_patterns, vec!["*".to_string()]);
    assert!(config.ignore_patterns.contains(&".git/".to_string()));
    assert!(config.ignore_patterns.contains(&".svn/".to_string()));
    assert!(config.ignore_patterns.contains(&".hg/".to_string()));
}

#[test]
fn test_project_priority_rust_over_others() {
    let temp_dir = TempDir::new().unwrap();

    // Create both Cargo.toml and package.json
    let cargo_toml = temp_dir.path().join("Cargo.toml");
    fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

    let package_json = temp_dir.path().join("package.json");
    fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

    // Rust should take precedence
    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::RustProject));
}

#[test]
fn test_mostly_markdown_calculation() {
    let temp_dir = TempDir::new().unwrap();

    // Create exactly 70% markdown files
    for i in 0..7 {
        let md_file = temp_dir.path().join(format!("doc{i}.md"));
        fs::write(&md_file, "# Documentation").unwrap();
    }

    for i in 0..3 {
        let rs_file = temp_dir.path().join(format!("code{i}.rs"));
        fs::write(&rs_file, "fn main() {}").unwrap();
    }

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::Documentation));
}

#[test]
fn test_empty_directory() {
    let temp_dir = TempDir::new().unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::Unknown));
}

#[test]
fn test_nested_directory_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create nested structure
    let src_dir = temp_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();

    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, "fn main() {}").unwrap();

    let cargo_toml = temp_dir.path().join("Cargo.toml");
    fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

    let project_type = ProjectDetector::detect(temp_dir.path());
    assert!(matches!(project_type, ProjectType::RustProject));
}

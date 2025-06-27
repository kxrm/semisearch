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
    assert_eq!(config.search_paths, vec!["src/", "tests/"]);
    assert_eq!(config.file_patterns, vec!["*.rs"]);
    assert_eq!(config.ignore_patterns, vec!["target/"]);
}

#[test]
fn test_context_aware_config_for_javascript() {
    let config = ContextAwareConfig::from_project_type(ProjectType::JavaScriptProject);

    assert!(matches!(
        config.project_type,
        ProjectType::JavaScriptProject
    ));
    assert_eq!(config.search_paths, vec!["src/", "lib/"]);
    assert_eq!(config.file_patterns, vec!["*.js", "*.ts"]);
    assert_eq!(config.ignore_patterns, vec!["node_modules/", "dist/"]);
}

#[test]
fn test_context_aware_config_for_python() {
    let config = ContextAwareConfig::from_project_type(ProjectType::PythonProject);

    assert!(matches!(config.project_type, ProjectType::PythonProject));
    assert_eq!(config.search_paths, vec!["src/", "lib/", "tests/"]);
    assert_eq!(config.file_patterns, vec!["*.py"]);
    assert_eq!(
        config.ignore_patterns,
        vec!["__pycache__/", "*.pyc", ".pytest_cache/", "venv/", ".venv/"]
    );
}

#[test]
fn test_context_aware_config_for_documentation() {
    let config = ContextAwareConfig::from_project_type(ProjectType::Documentation);

    assert!(matches!(config.project_type, ProjectType::Documentation));
    assert_eq!(config.search_paths, vec!["./"]);
    assert_eq!(config.file_patterns, vec!["*.md", "*.txt"]);
    assert_eq!(config.ignore_patterns, Vec::<String>::new());
}

#[test]
fn test_context_aware_config_for_mixed() {
    let config = ContextAwareConfig::from_project_type(ProjectType::Mixed);

    assert!(matches!(config.project_type, ProjectType::Mixed));
    assert_eq!(config.search_paths, vec!["./"]);
    assert_eq!(config.file_patterns, vec!["*"]);
    assert_eq!(
        config.ignore_patterns,
        vec!["target/", "node_modules/", "__pycache__/", "dist/", ".git/"]
    );
}

#[test]
fn test_context_aware_config_default() {
    let config = ContextAwareConfig::default();

    assert!(matches!(config.project_type, ProjectType::Unknown));
    assert_eq!(config.search_paths, vec!["./"]);
    assert_eq!(config.file_patterns, vec!["*"]);
    assert_eq!(config.ignore_patterns, vec![".git/", ".svn/", ".hg/"]);
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

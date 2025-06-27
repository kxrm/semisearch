use crate::context::project_detector::ProjectType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwareConfig {
    pub project_type: ProjectType,
    pub search_paths: Vec<String>,
    pub file_patterns: Vec<String>,
    pub ignore_patterns: Vec<String>,
}

impl ContextAwareConfig {
    pub fn from_project_type(project_type: ProjectType) -> Self {
        // Common ignore patterns that should be applied to all project types
        let common_ignores = vec![
            ".git/".to_string(),
            ".svn/".to_string(),
            ".hg/".to_string(),
            ".DS_Store".to_string(),
            "Thumbs.db".to_string(),
        ];

        match project_type {
            ProjectType::RustProject => Self {
                project_type,
                search_paths: vec!["src/".to_string(), "tests/".to_string()],
                file_patterns: vec!["*.rs".to_string()],
                ignore_patterns: {
                    let mut ignores = common_ignores.clone();
                    ignores.extend(vec![
                        "target/".to_string(),
                        "Cargo.lock".to_string(),
                        ".cargo/".to_string(),
                    ]);
                    ignores
                },
            },
            ProjectType::JavaScriptProject => Self {
                project_type,
                search_paths: vec!["src/".to_string(), "lib/".to_string()],
                file_patterns: vec!["*.js".to_string(), "*.ts".to_string()],
                ignore_patterns: {
                    let mut ignores = common_ignores.clone();
                    ignores.extend(vec![
                        "node_modules/".to_string(),
                        "dist/".to_string(),
                        "build/".to_string(),
                        "coverage/".to_string(),
                        "package-lock.json".to_string(),
                        "yarn.lock".to_string(),
                    ]);
                    ignores
                },
            },
            ProjectType::PythonProject => Self {
                project_type,
                search_paths: vec!["src/".to_string(), "lib/".to_string(), "tests/".to_string()],
                file_patterns: vec!["*.py".to_string()],
                ignore_patterns: {
                    let mut ignores = common_ignores.clone();
                    ignores.extend(vec![
                        "__pycache__/".to_string(),
                        "*.pyc".to_string(),
                        ".pytest_cache/".to_string(),
                        "venv/".to_string(),
                        ".venv/".to_string(),
                        "env/".to_string(),
                        ".env/".to_string(),
                        "pip-log.txt".to_string(),
                    ]);
                    ignores
                },
            },
            ProjectType::Documentation => Self {
                project_type,
                search_paths: vec!["./".to_string()],
                file_patterns: vec!["*.md".to_string(), "*.txt".to_string()],
                ignore_patterns: common_ignores,
            },
            ProjectType::Mixed => Self {
                project_type,
                search_paths: vec!["./".to_string()],
                file_patterns: vec!["*".to_string()],
                ignore_patterns: {
                    let mut ignores = common_ignores.clone();
                    ignores.extend(vec![
                        "target/".to_string(),
                        "node_modules/".to_string(),
                        "__pycache__/".to_string(),
                        "dist/".to_string(),
                        "build/".to_string(),
                        "coverage/".to_string(),
                    ]);
                    ignores
                },
            },
            ProjectType::Unknown => Self::default(),
        }
    }
}

impl Default for ContextAwareConfig {
    fn default() -> Self {
        Self {
            project_type: ProjectType::Unknown,
            search_paths: vec!["./".to_string()],
            file_patterns: vec!["*".to_string()],
            ignore_patterns: vec![
                ".git/".to_string(),
                ".svn/".to_string(),
                ".hg/".to_string(),
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_project_config() {
        let config = ContextAwareConfig::from_project_type(ProjectType::RustProject);
        assert_eq!(config.project_type, ProjectType::RustProject);
        assert_eq!(
            config.search_paths,
            vec!["src/".to_string(), "tests/".to_string()]
        );
        assert_eq!(config.file_patterns, vec!["*.rs".to_string()]);
        assert!(config.ignore_patterns.contains(&"target/".to_string()));
        assert!(config.ignore_patterns.contains(&".git/".to_string()));
    }

    #[test]
    fn test_javascript_project_config() {
        let config = ContextAwareConfig::from_project_type(ProjectType::JavaScriptProject);
        assert_eq!(config.project_type, ProjectType::JavaScriptProject);
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
        assert!(config.ignore_patterns.contains(&".git/".to_string()));
    }

    #[test]
    fn test_python_project_config() {
        let config = ContextAwareConfig::from_project_type(ProjectType::PythonProject);
        assert_eq!(config.project_type, ProjectType::PythonProject);
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
    fn test_documentation_project_config() {
        let config = ContextAwareConfig::from_project_type(ProjectType::Documentation);
        assert_eq!(config.project_type, ProjectType::Documentation);
        assert_eq!(config.search_paths, vec!["./".to_string()]);
        assert_eq!(
            config.file_patterns,
            vec!["*.md".to_string(), "*.txt".to_string()]
        );
        assert!(config.ignore_patterns.contains(&".git/".to_string()));
        assert!(!config.ignore_patterns.is_empty());
    }

    #[test]
    fn test_mixed_project_config() {
        let config = ContextAwareConfig::from_project_type(ProjectType::Mixed);
        assert_eq!(config.project_type, ProjectType::Mixed);
        assert_eq!(config.search_paths, vec!["./".to_string()]);
        assert_eq!(config.file_patterns, vec!["*".to_string()]);
        assert!(config.ignore_patterns.contains(&"target/".to_string()));
        assert!(config
            .ignore_patterns
            .contains(&"node_modules/".to_string()));
        assert!(config.ignore_patterns.contains(&".git/".to_string()));
    }

    #[test]
    fn test_default_config() {
        let config = ContextAwareConfig::default();
        assert_eq!(config.project_type, ProjectType::Unknown);
        assert_eq!(config.search_paths, vec!["./".to_string()]);
        assert_eq!(config.file_patterns, vec!["*".to_string()]);
        assert!(config.ignore_patterns.contains(&".git/".to_string()));
        assert!(config.ignore_patterns.contains(&".svn/".to_string()));
        assert!(config.ignore_patterns.contains(&".hg/".to_string()));
    }

    #[test]
    fn test_all_configs_have_version_control_ignores() {
        let project_types = vec![
            ProjectType::RustProject,
            ProjectType::JavaScriptProject,
            ProjectType::PythonProject,
            ProjectType::Documentation,
            ProjectType::Mixed,
        ];

        for project_type in project_types {
            let config = ContextAwareConfig::from_project_type(project_type.clone());
            assert!(
                config.ignore_patterns.contains(&".git/".to_string()),
                "Project type {project_type:?} should ignore .git/"
            );
        }
    }
}

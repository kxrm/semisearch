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
        match project_type {
            ProjectType::RustProject => Self {
                project_type,
                search_paths: vec!["src/".to_string(), "tests/".to_string()],
                file_patterns: vec!["*.rs".to_string()],
                ignore_patterns: vec!["target/".to_string()],
            },
            ProjectType::JavaScriptProject => Self {
                project_type,
                search_paths: vec!["src/".to_string(), "lib/".to_string()],
                file_patterns: vec!["*.js".to_string(), "*.ts".to_string()],
                ignore_patterns: vec!["node_modules/".to_string(), "dist/".to_string()],
            },
            ProjectType::PythonProject => Self {
                project_type,
                search_paths: vec!["src/".to_string(), "lib/".to_string(), "tests/".to_string()],
                file_patterns: vec!["*.py".to_string()],
                ignore_patterns: vec![
                    "__pycache__/".to_string(),
                    "*.pyc".to_string(),
                    ".pytest_cache/".to_string(),
                    "venv/".to_string(),
                    ".venv/".to_string(),
                ],
            },
            ProjectType::Documentation => Self {
                project_type,
                search_paths: vec!["./".to_string()],
                file_patterns: vec!["*.md".to_string(), "*.txt".to_string()],
                ignore_patterns: vec![],
            },
            ProjectType::Mixed => Self {
                project_type,
                search_paths: vec!["./".to_string()],
                file_patterns: vec!["*".to_string()],
                ignore_patterns: vec![
                    "target/".to_string(),
                    "node_modules/".to_string(),
                    "__pycache__/".to_string(),
                    "dist/".to_string(),
                    ".git/".to_string(),
                ],
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
            ignore_patterns: vec![".git/".to_string(), ".svn/".to_string(), ".hg/".to_string()],
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
        assert_eq!(config.search_paths, vec!["src/", "tests/"]);
        assert_eq!(config.file_patterns, vec!["*.rs"]);
        assert_eq!(config.ignore_patterns, vec!["target/"]);
    }

    #[test]
    fn test_javascript_project_config() {
        let config = ContextAwareConfig::from_project_type(ProjectType::JavaScriptProject);
        assert_eq!(config.project_type, ProjectType::JavaScriptProject);
        assert_eq!(config.search_paths, vec!["src/", "lib/"]);
        assert_eq!(config.file_patterns, vec!["*.js", "*.ts"]);
        assert_eq!(config.ignore_patterns, vec!["node_modules/", "dist/"]);
    }

    #[test]
    fn test_python_project_config() {
        let config = ContextAwareConfig::from_project_type(ProjectType::PythonProject);
        assert_eq!(config.project_type, ProjectType::PythonProject);
        assert_eq!(config.search_paths, vec!["src/", "lib/", "tests/"]);
        assert_eq!(config.file_patterns, vec!["*.py"]);
        assert_eq!(config.ignore_patterns.len(), 5);
        assert!(config.ignore_patterns.contains(&"__pycache__/".to_string()));
        assert!(config.ignore_patterns.contains(&"venv/".to_string()));
    }

    #[test]
    fn test_documentation_project_config() {
        let config = ContextAwareConfig::from_project_type(ProjectType::Documentation);
        assert_eq!(config.project_type, ProjectType::Documentation);
        assert_eq!(config.search_paths, vec!["./"]);
        assert_eq!(config.file_patterns, vec!["*.md", "*.txt"]);
        assert!(config.ignore_patterns.is_empty());
    }

    #[test]
    fn test_mixed_project_config() {
        let config = ContextAwareConfig::from_project_type(ProjectType::Mixed);
        assert_eq!(config.project_type, ProjectType::Mixed);
        assert_eq!(config.search_paths, vec!["./"]);
        assert_eq!(config.file_patterns, vec!["*"]);
        assert_eq!(config.ignore_patterns.len(), 5);
        assert!(config.ignore_patterns.contains(&"target/".to_string()));
        assert!(config
            .ignore_patterns
            .contains(&"node_modules/".to_string()));
    }

    #[test]
    fn test_default_config() {
        let config = ContextAwareConfig::default();
        assert_eq!(config.project_type, ProjectType::Unknown);
        assert_eq!(config.search_paths, vec!["./"]);
        assert_eq!(config.file_patterns, vec!["*"]);
        assert_eq!(config.ignore_patterns, vec![".git/", ".svn/", ".hg/"]);
    }
}

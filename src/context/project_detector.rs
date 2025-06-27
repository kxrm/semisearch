use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectType {
    RustProject,
    JavaScriptProject,
    PythonProject,
    Documentation,
    Mixed,
    Unknown,
}

pub struct ProjectDetector;

impl ProjectDetector {
    pub fn detect(path: &Path) -> ProjectType {
        // Check for specific project markers in priority order
        if path.join("Cargo.toml").exists() {
            return ProjectType::RustProject;
        }
        if path.join("package.json").exists() {
            return ProjectType::JavaScriptProject;
        }
        if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
            return ProjectType::PythonProject;
        }

        // Check if it's mostly markdown
        if Self::mostly_markdown(path) {
            return ProjectType::Documentation;
        }

        // Check if it's a mixed project
        if Self::is_mixed_project(path) {
            return ProjectType::Mixed;
        }

        ProjectType::Unknown
    }

    fn mostly_markdown(path: &Path) -> bool {
        let file_counts = Self::count_file_types(path);
        let total_files: usize = file_counts.values().sum();

        if total_files == 0 {
            return false;
        }

        let markdown_count = file_counts.get("md").unwrap_or(&0);
        let markdown_percentage = (*markdown_count as f64 / total_files as f64) * 100.0;

        markdown_percentage >= 70.0
    }

    fn is_mixed_project(path: &Path) -> bool {
        let file_counts = Self::count_file_types(path);

        // Consider it mixed if we have multiple significant file types
        let significant_types = file_counts.iter().filter(|(_, &count)| count >= 3).count();

        significant_types >= 2
    }

    fn count_file_types(path: &Path) -> HashMap<String, usize> {
        let mut file_counts = HashMap::new();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(extension) = entry.path().extension() {
                            if let Some(ext_str) = extension.to_str() {
                                *file_counts.entry(ext_str.to_string()).or_insert(0) += 1;
                            }
                        }
                    } else if file_type.is_dir() {
                        // Recursively count files in subdirectories
                        let sub_counts = Self::count_file_types(&entry.path());
                        for (ext, count) in sub_counts {
                            *file_counts.entry(ext).or_insert(0) += count;
                        }
                    }
                }
            }
        }

        file_counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

        let project_type = ProjectDetector::detect(temp_dir.path());
        assert_eq!(project_type, ProjectType::RustProject);
    }

    #[test]
    fn test_detect_javascript_project() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = temp_dir.path().join("package.json");
        fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

        let project_type = ProjectDetector::detect(temp_dir.path());
        assert_eq!(project_type, ProjectType::JavaScriptProject);
    }
}

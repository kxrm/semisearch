use std::collections::HashMap;
use std::collections::VecDeque;
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

    /// Count file types using iterative traversal to avoid stack overflow
    /// Handles symlinks, hidden directories, and case sensitivity
    fn count_file_types(path: &Path) -> HashMap<String, usize> {
        let mut file_counts = HashMap::new();
        let mut dirs_to_visit = VecDeque::new();
        let mut visited_symlinks = std::collections::HashSet::new();

        // Start with the root path
        dirs_to_visit.push_back(path.to_path_buf());

        // Maximum depth to prevent infinite loops
        const MAX_DEPTH: usize = 50;
        let mut current_depth = 0;

        while !dirs_to_visit.is_empty() && current_depth < MAX_DEPTH {
            let current_level_size = dirs_to_visit.len();

            for _ in 0..current_level_size {
                if let Some(current_path) = dirs_to_visit.pop_front() {
                    if let Ok(entries) = fs::read_dir(&current_path) {
                        for entry in entries.flatten() {
                            // Skip hidden files and directories
                            if let Some(file_name) = entry.file_name().to_str() {
                                if file_name.starts_with('.') {
                                    continue;
                                }
                            }

                            if let Ok(file_type) = entry.file_type() {
                                if file_type.is_file() {
                                    // Handle case sensitivity by normalizing to lowercase
                                    if let Some(extension) = entry.path().extension() {
                                        if let Some(ext_str) = extension.to_str() {
                                            let normalized_ext = ext_str.to_lowercase();
                                            *file_counts.entry(normalized_ext).or_insert(0) += 1;
                                        }
                                    }
                                } else if file_type.is_dir() {
                                    // Check if this is a symlink to avoid infinite loops
                                    if let Ok(metadata) = fs::metadata(entry.path()) {
                                        if metadata.file_type().is_symlink() {
                                            // Resolve symlink and check if we've already visited it
                                            if let Ok(resolved_path) = fs::canonicalize(entry.path()) {
                                                if visited_symlinks.contains(&resolved_path) {
                                                    continue; // Skip to avoid infinite loops
                                                }
                                                visited_symlinks.insert(resolved_path);
                                            }
                                        }
                                    }

                                    // Skip common system directories that shouldn't be searched
                                    if let Some(dir_name) = entry.file_name().to_str() {
                                        let skip_dirs = [
                                            "node_modules",
                                            "target",
                                            "__pycache__",
                                            ".git",
                                            "vendor",
                                            "dist",
                                            "build",
                                            ".cargo",
                                            ".rustc",
                                        ];
                                        if skip_dirs.contains(&dir_name) {
                                            continue;
                                        }
                                    }

                                    // Add to queue for next iteration
                                    dirs_to_visit.push_back(entry.path());
                                }
                            }
                        }
                    }
                }
            }

            current_depth += 1;
        }

        file_counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
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

    #[test]
    fn test_case_insensitive_extension_detection() {
        let temp_dir = TempDir::new().unwrap();

        // Create files with different case extensions
        fs::write(temp_dir.path().join("test.MD"), "# Test").unwrap();
        fs::write(temp_dir.path().join("test.md"), "# Test").unwrap();
        fs::write(temp_dir.path().join("test.Md"), "# Test").unwrap();

        let file_counts = ProjectDetector::count_file_types(temp_dir.path());
        assert_eq!(file_counts.get("md"), Some(&3)); // All should be counted as "md"
    }

    #[test]
    fn test_skip_hidden_directories() {
        let temp_dir = TempDir::new().unwrap();

        // Create hidden directory with files
        let hidden_dir = temp_dir.path().join(".hidden");
        fs::create_dir(&hidden_dir).unwrap();
        fs::write(hidden_dir.join("test.md"), "# Hidden").unwrap();

        // Create visible directory with files
        let visible_dir = temp_dir.path().join("visible");
        fs::create_dir(&visible_dir).unwrap();
        fs::write(visible_dir.join("test.md"), "# Visible").unwrap();

        let file_counts = ProjectDetector::count_file_types(temp_dir.path());
        // Should only count the visible file, not the hidden one
        assert_eq!(file_counts.get("md"), Some(&1));
    }

    #[test]
    fn test_skip_system_directories() {
        let temp_dir = TempDir::new().unwrap();

        // Create system directories that should be skipped
        let node_modules = temp_dir.path().join("node_modules");
        fs::create_dir(&node_modules).unwrap();
        fs::write(node_modules.join("test.js"), "console.log('test');").unwrap();

        let target = temp_dir.path().join("target");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("test.rs"), "fn main() {}").unwrap();

        // Create normal directory
        let src = temp_dir.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}").unwrap();

        let file_counts = ProjectDetector::count_file_types(temp_dir.path());
        // Should only count the file in src/, not in node_modules or target
        assert_eq!(file_counts.get("rs"), Some(&1));
        assert_eq!(file_counts.get("js"), None);
    }
}

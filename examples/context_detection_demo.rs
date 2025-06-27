use search::context::{ContextAwareConfig, ProjectDetector, ProjectType};
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 { &args[1] } else { "." };

    println!("=== Context Detection Demo ===");
    println!("Analyzing project at: {path}\n");

    // Detect project type
    let project_type = ProjectDetector::detect(Path::new(path));
    println!("Detected project type: {project_type:?}");

    // Get context-aware configuration
    let config = ContextAwareConfig::from_project_type(project_type.clone());

    println!("\nContext-aware configuration:");
    println!("  Search paths: {:?}", config.search_paths);
    println!("  File patterns: {:?}", config.file_patterns);
    println!("  Ignore patterns: {:?}", config.ignore_patterns);

    // Show how this would affect search behavior
    println!("\nSearch behavior implications:");
    match project_type {
        ProjectType::RustProject => {
            println!("  ✓ Will focus on .rs files in src/ and tests/ directories");
            println!("  ✓ Will automatically ignore target/ directory");
            println!("  ✓ Optimized for Rust code patterns");
        }
        ProjectType::JavaScriptProject => {
            println!("  ✓ Will focus on .js and .ts files in src/ and lib/ directories");
            println!("  ✓ Will automatically ignore node_modules/ and dist/");
            println!("  ✓ Optimized for JavaScript/TypeScript patterns");
        }
        ProjectType::PythonProject => {
            println!("  ✓ Will focus on .py files in src/, lib/, and tests/ directories");
            println!("  ✓ Will automatically ignore __pycache__, venv, and .pytest_cache");
            println!("  ✓ Optimized for Python code patterns");
        }
        ProjectType::Documentation => {
            println!("  ✓ Will focus on .md and .txt files");
            println!("  ✓ Will search all directories (no restrictions)");
            println!("  ✓ Optimized for natural language and documentation");
        }
        ProjectType::Mixed => {
            println!("  ✓ Will search all file types");
            println!("  ✓ Will ignore common build directories from multiple languages");
            println!("  ✓ Balanced search strategy for mixed codebases");
        }
        ProjectType::Unknown => {
            println!("  ✓ Will search all files with minimal restrictions");
            println!("  ✓ Will only ignore version control directories");
            println!("  ✓ Generic search strategy");
        }
    }

    // Example of how to use this in actual search
    println!("\nExample usage in search:");
    println!("  semisearch \"TODO\" # Will automatically use the above configuration");
    println!("  - Searches only relevant files based on project type");
    println!("  - Ignores build artifacts and dependencies");
    println!("  - Uses appropriate search strategies for the content type");
}

use libloading::Library;
use std::path::PathBuf;
use sys_info;

/// Represents the neural embedding capability status
#[derive(Debug, Clone, PartialEq)]
pub enum NeuralCapability {
    Available,
    ModelMissing, // ONNX and resources available, but model needs download
    Unavailable(&'static str),
    Insufficient(&'static str),
}

/// Detects system capabilities for neural embeddings
pub struct CapabilityDetector;

impl CapabilityDetector {
    /// Detect neural embedding capability at runtime
    pub fn detect_neural_capability() -> NeuralCapability {
        // Check 0: Environment variable override (for testing)
        if std::env::var("DISABLE_ONNX").is_ok() {
            return NeuralCapability::Unavailable("ONNX disabled by environment variable");
        }

        // Check 1: ONNX Runtime library available
        if !Self::onnx_runtime_available() {
            return NeuralCapability::Unavailable("ONNX Runtime not found");
        }

        // Check 2: System resources sufficient
        if !Self::system_resources_adequate() {
            return NeuralCapability::Insufficient("Insufficient RAM (< 4GB)");
        }

        // Check 3: Neural model available (but don't require it for capability detection)
        if Self::neural_model_available() {
            NeuralCapability::Available
        } else {
            NeuralCapability::ModelMissing
        }
    }

    /// Check if ONNX Runtime library is available
    fn onnx_runtime_available() -> bool {
        // First check environment variable ORT_DYLIB_PATH
        if let Ok(ort_path) = std::env::var("ORT_DYLIB_PATH") {
            unsafe {
                if Library::new(&ort_path).is_ok() {
                    return true;
                }
            }
        }
        // Check LD_LIBRARY_PATH paths
        if let Ok(ld_library_path) = std::env::var("LD_LIBRARY_PATH") {
            for path in ld_library_path.split(':') {
                let lib_path = PathBuf::from(path).join("libonnxruntime.so");
                unsafe {
                    if Library::new(lib_path).is_ok() {
                        return true;
                    }
                }
            }
        }
        // Check standard paths
        let lib_paths = [
            "libonnxruntime.so.1.16.0",
            "libonnxruntime.so",
            "/usr/lib/libonnxruntime.so",
            "/usr/local/lib/libonnxruntime.so",
        ];

        for path in &lib_paths {
            unsafe {
                if Library::new(path).is_ok() {
                    return true;
                }
            }
        }
        false
    }

    /// Check if system has adequate resources for neural embeddings
    fn system_resources_adequate() -> bool {
        if let Ok(mem_info) = sys_info::mem_info() {
            // mem_info.total is in KB, convert to bytes for comparison
            let total_memory_bytes = mem_info.total * 1024;
            return total_memory_bytes >= 4 * 1024 * 1024 * 1024; // 4GB
        }
        false
    }

    /// Check if neural model is available
    fn neural_model_available() -> bool {
        let model_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".semisearch")
            .join("models")
            .join("model.onnx");

        model_path.exists()
    }

    /// Get detailed capability information for doctor command
    pub fn get_capability_details() -> CapabilityDetails {
        let onnx_available = Self::onnx_runtime_available();
        let resources_adequate = Self::system_resources_adequate();
        let model_available = Self::neural_model_available();

        let memory_info = sys_info::mem_info().ok();
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);

        CapabilityDetails {
            onnx_available,
            resources_adequate,
            model_available,
            memory_info,
            cpu_count,
        }
    }
}

/// Detailed capability information for diagnostics
#[derive(Debug)]
pub struct CapabilityDetails {
    pub onnx_available: bool,
    pub resources_adequate: bool,
    pub model_available: bool,
    pub memory_info: Option<sys_info::MemInfo>,
    pub cpu_count: usize,
}

impl CapabilityDetails {
    /// Get human-readable capability status
    pub fn get_status(&self) -> &'static str {
        if self.onnx_available && self.resources_adequate && self.model_available {
            "Full"
        } else if self.resources_adequate {
            "TfIdf"
        } else {
            "None"
        }
    }

    /// Get recommendations for improving capabilities
    pub fn get_recommendations(&self) -> Vec<&'static str> {
        let mut recommendations = Vec::new();

        if !self.onnx_available {
            recommendations.push("Install ONNX Runtime for neural embeddings");
        }

        if !self.resources_adequate {
            recommendations.push("Upgrade to 4GB+ RAM for neural embeddings");
        }

        if !self.model_available {
            recommendations.push("Run 'semisearch index --semantic' to download neural model");
        }

        recommendations
    }
}

use libloading::Library;
use std::path::PathBuf;
use sys_info;

/// Represents the neural embedding capability status
#[derive(Debug, Clone, PartialEq)]
pub enum NeuralCapability {
    Available,
    Unavailable(&'static str),
    Insufficient(&'static str),
    NoModel(&'static str),
}

/// Detects system capabilities for neural embeddings
pub struct CapabilityDetector;

impl CapabilityDetector {
    /// Detect neural embedding capability at runtime
    pub fn detect_neural_capability() -> NeuralCapability {
        // Check 1: ONNX Runtime library available
        if !Self::onnx_runtime_available() {
            return NeuralCapability::Unavailable("ONNX Runtime not found");
        }

        // Check 2: System resources sufficient
        if !Self::system_resources_adequate() {
            return NeuralCapability::Insufficient("Insufficient RAM (< 4GB)");
        }

        // Check 3: Neural model available
        if !Self::neural_model_available() {
            return NeuralCapability::NoModel("Neural model not downloaded");
        }

        NeuralCapability::Available
    }

    /// Check if ONNX Runtime library is available
    fn onnx_runtime_available() -> bool {
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

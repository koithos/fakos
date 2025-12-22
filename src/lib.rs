//! fakos - A Kubernetes CLI tool
//!
//! This crate provides functionality for managing and inspecting container images
//! in Kubernetes clusters.

// Public API
pub use cli::Args;
pub use k8s::K8sClient;

// Internal modules
mod cli;
mod k8s;
mod utils;

// Re-export commonly used items
pub use cli::{Commands, GetResources, LogFormat, OutputFormat};
pub use k8s::{FarosNode, FarosPod, K8sError};
pub use utils::logging;
pub use utils::{display_nodes, display_pods};

/// Result type for fakos operations
pub type FakosResult<T> = anyhow::Result<T>;

/// Error type for fakos operations
pub type FakosError = anyhow::Error;

use anyhow::{Context, Result};
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use thiserror::Error;
use tracing::{debug, error, info, instrument};

/// Represents a running Kubernetes pod
#[derive(Debug, Clone)]
pub struct FarosPod {
    /// Name of the pod
    pub name: String,
    /// Kubernetes namespace of the pod
    pub namespace: String,
}

/// Errors that can occur when interacting with Kubernetes
#[derive(Debug, Error)]
pub enum K8sError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    ConfigError(String),
    /// Connection-related errors
    #[error("Connection error: {0}")]
    ConnectionError(String),
    /// API-related errors
    #[error("API error: {0}")]
    ApiError(String),
    /// Resource not found errors
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
}

/// Client for interacting with Kubernetes clusters
pub struct K8sClient {
    /// The underlying Kubernetes client
    client: Client,
}

impl K8sClient {
    /// Create a new Kubernetes client
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - A new K8sClient instance or an error if initialization fails
    #[instrument(skip_all)]
    pub async fn new() -> Result<Self> {
        debug!("Initializing Kubernetes client");

        let kubeconfig_path = Self::get_kubeconfig_path()?;
        debug!(path = %kubeconfig_path, "Using kubeconfig path");

        let client = Client::try_default()
            .await
            .context("Failed to create Kubernetes client")?;

        let k8s_client = Self { client };

        // Verify cluster accessibility
        if !k8s_client.is_accessible().await? {
            return Err(
                K8sError::ConnectionError("Kubernetes cluster is not accessible".into()).into(),
            );
        }

        info!("Successfully initialized Kubernetes client");
        Ok(k8s_client)
    }

    /// Get the path to the kubeconfig file
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The path to the kubeconfig file or an error if not found
    fn get_kubeconfig_path() -> Result<String> {
        if let Ok(path) = std::env::var("KUBECONFIG") {
            info!("Using kubeconfig from KUBECONFIG environment variable");
            return Ok(path);
        }

        debug!("KUBECONFIG not set, checking default location");
        let home_dir = std::env::var("HOME").context("Failed to get HOME directory")?;
        let default_kubeconfig = format!("{}/.kube/config", home_dir);

        if !std::path::Path::new(&default_kubeconfig).exists() {
            return Err(
                K8sError::ConfigError("No kubeconfig found at default location".into()).into(),
            );
        }

        info!("Using default kubeconfig location");
        Ok(default_kubeconfig)
    }

    /// Check if the Kubernetes cluster is accessible
    ///
    /// # Returns
    ///
    /// * `Result<bool>` - True if the cluster is accessible, false otherwise
    #[instrument(skip(self))]
    pub async fn is_accessible(&self) -> Result<bool> {
        debug!("Checking cluster accessibility");
        let api: Api<Pod> = Api::namespaced(self.client.clone(), "default");

        match api.list(&Default::default()).await {
            Ok(_) => {
                debug!("Successfully connected to cluster");
                Ok(true)
            }
            Err(e) => match e {
                kube::Error::Api(api_err) => {
                    error!("Kubernetes API error occurred");
                    Err(
                        K8sError::ApiError(format!("{} ({})", api_err.message, api_err.reason))
                            .into(),
                    )
                }
                _ => {
                    error!("Failed to connect to Kubernetes cluster");
                    Err(
                        K8sError::ConnectionError("Failed to connect to Kubernetes cluster".into())
                            .into(),
                    )
                }
            },
        }
    }

    /// Get the pods API for the specified namespace
    pub fn get_pods_api(
        &self,
        namespace: &str,
        all_namespaces: bool,
        _node_name: Option<&str>,
    ) -> Result<Api<Pod>> {
        let api = if all_namespaces {
            Api::all(self.client.clone())
        } else {
            Api::namespaced(self.client.clone(), namespace)
        };

        Ok(api)
    }
}

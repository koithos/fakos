use anyhow::{Context, Result};
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::api::ListParams;
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
    /// Node name where the pod is running
    pub node: Option<String>,
    /// Labels attached to the pod
    pub labels: std::collections::BTreeMap<String, String>,
    /// Annotations attached to the pod
    pub annotations: std::collections::BTreeMap<String, String>,
}

/// Represents a Kubernetes node
#[derive(Debug, Clone)]
pub struct FarosNode {
    /// Name of the node
    pub name: String,
    /// Labels attached to the node
    pub labels: std::collections::BTreeMap<String, String>,
    /// Annotations attached to the node
    pub annotations: std::collections::BTreeMap<String, String>,
    /// Status of the node (Ready, NotReady, etc.)
    pub status: String,
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

    /// Get pods that match the specified filters
    ///
    /// # Arguments
    ///
    /// * `namespace` - The namespace to query (ignored if `all_namespaces` is true)
    /// * `all_namespaces` - If true, query pods across all namespaces
    /// * `node_name` - Optional filter by node name
    /// * `pod_name` - Optional filter by pod name
    ///
    /// # Returns
    ///
    /// * `Result<Vec<FarosPod>>` - A list of pods matching the filters
    #[instrument(skip(self), level = "debug")]
    pub async fn get_pods(
        &self,
        namespace: &str,
        all_namespaces: bool,
        node_name: Option<&str>,
        pod_name: Option<&str>,
    ) -> Result<Vec<FarosPod>> {
        let api = if all_namespaces {
            Api::all(self.client.clone())
        } else {
            Api::namespaced(self.client.clone(), namespace)
        };

        // Build field selector for better performance with large datasets
        let mut list_params = ListParams::default();
        if let Some(node) = node_name {
            list_params = list_params.fields(&format!("spec.nodeName={}", node));
        }

        let pod_list = api
            .list(&list_params)
            .await
            .context("Failed to list pods from Kubernetes API")?;

        let pods: Vec<FarosPod> = pod_list
            .items
            .into_iter()
            .filter_map(|pod: Pod| {
                // Filter by pod name if specified (field selector doesn't support pod name)
                if let Some(name) = pod_name
                    && pod.metadata.name.as_deref() != Some(name)
                {
                    return None;
                }

                // Extract pod name
                let name = pod.metadata.name.as_deref().unwrap_or_default().to_string();

                // Extract namespace
                let pod_namespace = pod
                    .metadata
                    .namespace
                    .as_deref()
                    .unwrap_or_default()
                    .to_string();

                // Extract node name
                let node = pod
                    .spec
                    .as_ref()
                    .and_then(|spec| spec.node_name.as_ref())
                    .cloned();

                // Extract labels
                let labels = pod.metadata.labels.unwrap_or_default();
                let annotations = pod.metadata.annotations.unwrap_or_default();

                Some(FarosPod {
                    name,
                    namespace: pod_namespace,
                    node,
                    labels,
                    annotations,
                })
            })
            .collect();

        Ok(pods)
    }

    /// Get nodes that match the specified filters
    ///
    /// # Arguments
    ///
    /// * `node_name` - Optional filter by node name
    ///
    /// # Returns
    ///
    /// * `Result<Vec<FarosNode>>` - A list of nodes matching the filters
    #[instrument(skip(self), level = "debug")]
    pub async fn get_nodes(&self, node_name: Option<&str>) -> Result<Vec<FarosNode>> {
        let api: Api<Node> = Api::all(self.client.clone());
        let list_params = ListParams::default();

        let node_list = api
            .list(&list_params)
            .await
            .context("Failed to list nodes from Kubernetes API")?;

        let nodes: Vec<FarosNode> = node_list
            .items
            .into_iter()
            .filter_map(|node: Node| {
                // Filter by node name if specified
                if let Some(name) = node_name
                    && node.metadata.name.as_deref() != Some(name)
                {
                    return None;
                }

                // Extract node name
                let name = node
                    .metadata
                    .name
                    .as_deref()
                    .unwrap_or_default()
                    .to_string();

                // Extract labels
                let labels = node.metadata.labels.unwrap_or_default();
                let annotations = node.metadata.annotations.unwrap_or_default();

                // Extract status
                let status = node
                    .status
                    .as_ref()
                    .and_then(|s| s.conditions.as_ref())
                    .and_then(|conditions| {
                        conditions.iter().find(|c| c.type_ == "Ready").map(|c| {
                            if c.status == "True" {
                                "Ready".to_string()
                            } else {
                                "NotReady".to_string()
                            }
                        })
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                Some(FarosNode {
                    name,
                    labels,
                    annotations,
                    status,
                })
            })
            .collect();

        Ok(nodes)
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
}

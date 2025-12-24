use crate::cli::formats::OutputFormat;
use clap::Subcommand;
use std::path::PathBuf;

/// CLI command structure for Kimspect
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get information about Kubernetes resources
    Get {
        /// The resource type to query
        #[command(subcommand)]
        resource: GetResources,
    },
}

/// Resource types that can be queried in the Kubernetes cluster
#[derive(Subcommand, Debug)]
pub enum GetResources {
    /// List pods
    Pods {
        /// Pod name to filter by (if not specified, all pods in the namespace are shown)
        #[arg(value_name = "POD")]
        pod_name: Option<String>,

        /// Kubernetes namespace to query (defaults to "default", ignored when --node is specified)
        #[arg(
            short,
            long,
            default_value = "default",
            conflicts_with = "all_namespaces"
        )]
        namespace: String,

        /// Filter pods by node name
        #[arg(short = 'N', long = "node", conflicts_with = "all_namespaces")]
        node: Option<String>,

        /// Query pods across all namespaces
        #[arg(short = 'A', long = "all-namespaces", conflicts_with = "namespace")]
        all_namespaces: bool,

        /// Output format (default: normal, wide: shows additional columns)
        #[arg(short = 'o', long = "output", default_value = "normal")]
        output: OutputFormat,

        /// Display only labels attached to the pods
        #[arg(long = "labels")]
        labels: bool,

        /// Display annotations attached to the pods
        #[arg(long = "annotations")]
        annotations: bool,

        /// Display environment variables for each container in the pods
        /// Optionally accepts a regex pattern to filter containers (e.g. --env-vars ".*-app")
        #[arg(long = "env-vars", num_args(0..=1), default_missing_value = ".*")]
        env_vars: Option<crate::EnvVarsFilter>,

        /// Path to kubeconfig file (default: ~/.kube/config)
        #[arg(long = "kubeconfig")]
        kubeconfig: Option<PathBuf>,
    },

    /// List nodes
    Nodes {
        /// Node name to filter by (if not specified, all nodes are shown)
        #[arg(value_name = "NODE")]
        node_name: Option<String>,

        /// Output format (default: normal, wide: shows additional columns)
        #[arg(short = 'o', long = "output", default_value = "normal")]
        output: OutputFormat,

        /// Display only labels attached to the nodes
        #[arg(long = "labels")]
        labels: bool,

        /// Display annotations attached to the nodes
        #[arg(long = "annotations")]
        annotations: bool,

        /// Path to kubeconfig file (default: ~/.kube/config)
        #[arg(long = "kubeconfig")]
        kubeconfig: Option<PathBuf>,
    },
}

impl GetResources {
    /// Get the kubeconfig path for this command
    ///
    /// # Returns
    ///
    /// * `Option<PathBuf>` - The path to the kubeconfig file if specified
    pub fn get_kubeconfig_path(&self) -> Option<PathBuf> {
        match self {
            GetResources::Pods { kubeconfig, .. } => kubeconfig.clone(),
            GetResources::Nodes { kubeconfig, .. } => kubeconfig.clone(),
        }
    }

    /// Get the namespace for this command
    ///
    /// # Returns
    ///
    /// * `&str` - The namespace to query
    pub fn get_namespace(&self) -> &str {
        match self {
            GetResources::Pods { namespace, .. } => namespace,
            GetResources::Nodes { .. } => "default",
        }
    }

    /// Check if this command should query all namespaces
    ///
    /// # Returns
    ///
    /// * `bool` - True if all namespaces should be queried
    pub fn is_all_namespaces(&self) -> bool {
        match self {
            GetResources::Pods { all_namespaces, .. } => *all_namespaces,
            GetResources::Nodes { .. } => false,
        }
    }
}

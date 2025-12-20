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
        resource: GetPods,
    },
}

/// Resource types that can be queried in the Kubernetes cluster
#[derive(Subcommand, Debug)]
pub enum GetPods {
    /// List pods
    Pods {
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

        /// Filter pods by pod name
        #[arg(short, long)]
        pod: Option<String>,

        /// Query pods across all namespaces
        #[arg(short = 'A', long = "all-namespaces", conflicts_with = "namespace")]
        all_namespaces: bool,

        /// Output format (default: normal, wide: shows additional columns)
        #[arg(short = 'o', long = "output", default_value = "normal")]
        output: OutputFormat,

        /// Display only labels attached to the pods
        #[arg(long = "labels")]
        labels: bool,

        /// Path to kubeconfig file (default: ~/.kube/config)
        #[arg(long = "kubeconfig")]
        kubeconfig: Option<PathBuf>,
    },
}

impl GetPods {
    /// Get the kubeconfig path for this command
    ///
    /// # Returns
    ///
    /// * `Option<PathBuf>` - The path to the kubeconfig file if specified
    pub fn get_kubeconfig_path(&self) -> Option<PathBuf> {
        match self {
            GetPods::Pods { kubeconfig, .. } => kubeconfig.clone(),
        }
    }

    /// Get the namespace for this command
    ///
    /// # Returns
    ///
    /// * `&str` - The namespace to query
    pub fn get_namespace(&self) -> &str {
        match self {
            GetPods::Pods { namespace, .. } => namespace,
        }
    }

    /// Check if this command should query all namespaces
    ///
    /// # Returns
    ///
    /// * `bool` - True if all namespaces should be queried
    pub fn is_all_namespaces(&self) -> bool {
        match self {
            GetPods::Pods { all_namespaces, .. } => *all_namespaces,
        }
    }
}

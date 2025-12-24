use anyhow::Context;
use clap::Parser;
use fakos::{
    Args, Commands, FakosResult, GetResources, K8sClient, display_nodes, display_pods, logging,
};
use tracing::{debug, info, instrument, warn};

/// Main entry point for the fakos application
#[tokio::main]
async fn main() -> FakosResult<()> {
    let args = Args::parse();

    // Initialize rustls crypto provider
    // Ignore error if already installed (e.g. by other dependencies)
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Initialize logging with the specified format
    logging::init_logging(logging::configure_logging(args.verbose), args.log_format)
        .context("Failed to initialize logging")?;

    debug!("Application started with args: {:?}", args);

    // Create the client with improved error context
    let client = K8sClient::new()
        .await
        .context("Failed to create Kubernetes client")?;

    info!("Successfully connected to Kubernetes cluster");

    process_commands(args, client).await?;

    debug!("Application completed successfully");
    Ok(())
}

/// Process the command line arguments and execute the corresponding command
#[instrument(skip(client), level = "debug")]
async fn process_commands(args: Args, client: K8sClient) -> FakosResult<()> {
    match args.command {
        Commands::Get { resource } => match resource {
            GetResources::Pods {
                namespace,
                node,
                pod_name,
                all_namespaces,
                output,
                labels,
                annotations,
                env_vars,
                ..
            } => {
                if let Some(ref pod) = pod_name
                    && all_namespaces
                {
                    warn!(
                        pod = %pod,
                        "Warning: Pod name specified with --all-namespaces flag. Pod names are unique within a namespace, so searching across all namespaces may be inefficient."
                    );
                }

                debug!(
                    namespace = %namespace,
                    node = ?node,
                    pod = ?pod_name,
                    all_namespaces = %all_namespaces,
                    output = ?output,
                    labels = %labels,
                    annotations = %annotations,
                    "Processing..."
                );

                let pods = client
                    .get_pods(
                        &namespace,
                        all_namespaces,
                        node.as_deref(),
                        pod_name.as_deref(),
                    )
                    .await
                    .context("Failed to get pods")?;

                display_pods(
                    &pods,
                    &output,
                    labels,
                    annotations,
                    all_namespaces,
                    env_vars,
                )?;
            }
            GetResources::Nodes {
                node_name,
                output,
                labels,
                annotations,
                ..
            } => {
                debug!(
                    node = ?node_name,
                    output = ?output,
                    labels = %labels,
                    annotations = %annotations,
                    "Processing..."
                );

                let nodes = client
                    .get_nodes(node_name.as_deref())
                    .await
                    .context("Failed to get nodes")?;

                display_nodes(&nodes, &output, labels, annotations)?;
            }
        },
    }
    Ok(())
}

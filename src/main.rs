use anyhow::Context;
use clap::Parser;
use fakos::{Args, Commands, FakosResult, GetPods, K8sClient, display_pods, logging};
use tracing::{debug, info, instrument};

/// Main entry point for the fakos application
#[tokio::main]
async fn main() -> FakosResult<()> {
    let args = Args::parse();

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
            GetPods::Pods {
                namespace,
                node,
                pod_name,
                all_namespaces,
                output,
                labels,
                annotations,
                ..
            } => {
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

                display_pods(&pods, &output, labels, annotations)?;
            }
        },
    }
    Ok(())
}

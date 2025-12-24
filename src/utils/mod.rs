use crate::{
    OutputFormat,
    k8s::{FarosNode, FarosPod},
};
use anyhow::Result;
use prettytable::{Cell, Row, Table, format::FormatBuilder};
use regex::Regex;
use tracing::warn;

pub mod logging;

/// Filter configuration for environment variables
#[derive(Debug, Clone)]
pub struct EnvVarsFilter {
    pub regex: Regex,
    pub invert: bool,
}

impl EnvVarsFilter {
    pub fn new(regex: Regex, invert: bool) -> Self {
        Self { regex, invert }
    }

    pub fn matches(&self, text: &str) -> bool {
        let matches = self.regex.is_match(text);
        if self.invert { !matches } else { matches }
    }
}

impl std::str::FromStr for EnvVarsFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (pattern, invert) = if let Some(stripped) = s.strip_prefix('!') {
            (stripped, true)
        } else {
            (s, false)
        };

        let regex = Regex::new(pattern).map_err(|e| e.to_string())?;
        Ok(Self::new(regex, invert))
    }
}

/// Error type for table display operations
#[derive(Debug)]
pub struct TableDisplayError {
    /// Error message describing what went wrong
    message: String,
}

impl std::error::Error for TableDisplayError {}

impl std::fmt::Display for TableDisplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Table display error: {}", self.message)
    }
}

impl TableDisplayError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

/// Display pods in a formatted table
///
/// # Arguments
///
/// * `pods` - List of pods to display
/// * `output_format` - Format to use for displaying the pods
/// * `show_labels` - Whether to include labels in the output
/// * `show_annotations` - Whether to include annotations in the output
/// * `all_namespaces` - Whether to show namespace column (only when querying all namespaces)
///
/// # Returns
///
/// * `Result<()>` - Success or error
pub fn display_pods(
    pods: &[FarosPod],
    output_format: &OutputFormat,
    show_labels: bool,
    show_annotations: bool,
    all_namespaces: bool,
    env_vars_filter: Option<&EnvVarsFilter>,
) -> Result<(), TableDisplayError> {
    if pods.is_empty() {
        warn!("No pods found matching criteria");
        return Ok(());
    }

    let mut table = create_table()?;
    let mut header_cells = Vec::new();

    if all_namespaces {
        header_cells.push(Cell::new("NAMESPACE"));
    }
    header_cells.push(Cell::new("POD"));

    if env_vars_filter.is_some() {
        header_cells.push(Cell::new("CONTAINERS"));
        header_cells.push(Cell::new("ENV VARS"));
    }

    if show_labels {
        header_cells.push(Cell::new("LABELS"));
    }

    if show_annotations {
        header_cells.push(Cell::new("ANNOTATIONS"));
    }

    if matches!(output_format, OutputFormat::Wide) {
        header_cells.push(Cell::new("NODE"));
    }

    let header_row = Row::new(header_cells);
    table.add_row(header_row);

    for pod in pods {
        let mut row_cells = Vec::new();

        if all_namespaces {
            row_cells.push(Cell::new(&pod.namespace));
        }
        row_cells.push(Cell::new(&pod.name));

        if let Some(filter) = env_vars_filter {
            let (containers, env_vars) =
                format_container_and_env_vars(&pod.container_env_vars, filter);
            row_cells.push(Cell::new(&containers));
            row_cells.push(Cell::new(&env_vars));
        }

        if show_labels {
            row_cells.push(Cell::new(&format_metadata(&pod.labels)));
        }

        if show_annotations {
            row_cells.push(Cell::new(&format_metadata(&pod.annotations)));
        }

        if matches!(output_format, OutputFormat::Wide) {
            let node_display = pod.node.as_deref().unwrap_or("<none>");
            row_cells.push(Cell::new(node_display));
        }

        table.add_row(Row::new(row_cells));
    }

    table.printstd();
    Ok(())
}

/// Create a new table with default formatting
///
/// # Returns
///
/// * `Result<Table>` - A new table instance or error
fn create_table() -> Result<Table, TableDisplayError> {
    let format = FormatBuilder::new()
        .column_separator(' ')
        .separator(
            prettytable::format::LinePosition::Title,
            prettytable::format::LineSeparator::new('-', '-', '-', '-'),
        )
        .padding(0, 1)
        .build();

    let mut table = Table::new();
    table.set_format(format);
    Ok(table)
}

fn format_metadata(map: &std::collections::BTreeMap<String, String>) -> String {
    if map.is_empty() {
        "<none>".to_string()
    } else {
        // Use fold to build string efficiently without intermediate Vec allocation
        map.iter().fold(String::new(), |mut acc, (k, v)| {
            if !acc.is_empty() {
                acc.push('\n');
            }
            acc.push_str(k);
            acc.push('=');
            acc.push_str(v);
            acc
        })
    }
}

fn format_container_and_env_vars(
    container_env_vars: &std::collections::BTreeMap<
        String,
        std::collections::BTreeMap<String, String>,
    >,
    filter: &EnvVarsFilter,
) -> (String, String) {
    if container_env_vars.is_empty() {
        return ("<none>".to_string(), "<none>".to_string());
    }

    let mut containers_str = String::new();
    let mut env_vars_str = String::new();
    let mut first = true;

    for (container_name, env_vars) in container_env_vars {
        // Apply filter to container name
        if !filter.matches(container_name) {
            continue;
        }

        if !first {
            containers_str.push('\n');
            env_vars_str.push('\n');
        }
        first = false;

        // Add container name
        containers_str.push_str(container_name);

        if env_vars.is_empty() {
            env_vars_str.push_str("<none>");
        } else {
            // Add env vars
            let mut env_first = true;
            for (key, value) in env_vars {
                if !env_first {
                    // For subsequent env vars, we need to add newlines to the container string to keep alignment
                    containers_str.push('\n');
                    env_vars_str.push('\n');
                }
                env_first = false;

                let entry = format!("{}={}", key, value);
                env_vars_str.push_str(&entry);

                // Add padding to container string for each newline in the environment variable value
                // This ensures that the next environment variable or container starts at the correct vertical position
                let newline_count = entry.chars().filter(|&c| c == '\n').count();
                for _ in 0..newline_count {
                    containers_str.push('\n');
                }
            }
        }
    }
    (containers_str, env_vars_str)
}

/// Display nodes in a formatted table
///
/// # Arguments
///
/// * `nodes` - List of nodes to display
/// * `output_format` - Format to use for displaying the nodes
/// * `show_labels` - Whether to include labels in the output
/// * `show_annotations` - Whether to include annotations in the output
///
/// # Returns
///
/// * `Result<()>` - Success or error
pub fn display_nodes(
    nodes: &[FarosNode],
    _output_format: &OutputFormat,
    show_labels: bool,
    show_annotations: bool,
) -> Result<(), TableDisplayError> {
    if nodes.is_empty() {
        warn!("No nodes found matching criteria");
        return Ok(());
    }

    let mut table = create_table()?;
    let mut header_cells = Vec::new();

    header_cells.push(Cell::new("NAME"));
    header_cells.push(Cell::new("STATUS"));

    if show_labels {
        header_cells.push(Cell::new("LABELS"));
    }

    if show_annotations {
        header_cells.push(Cell::new("ANNOTATIONS"));
    }

    let header_row = Row::new(header_cells);
    table.add_row(header_row);

    for node in nodes {
        let mut row_cells = Vec::new();

        row_cells.push(Cell::new(&node.name));
        row_cells.push(Cell::new(&node.status));

        if show_labels {
            row_cells.push(Cell::new(&format_metadata(&node.labels)));
        }

        if show_annotations {
            row_cells.push(Cell::new(&format_metadata(&node.annotations)));
        }

        table.add_row(Row::new(row_cells));
    }

    table.printstd();
    Ok(())
}

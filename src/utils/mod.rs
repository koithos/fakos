use crate::{OutputFormat, k8s::FarosPod};
use anyhow::Result;
use prettytable::{Cell, Row, Table, format::FormatBuilder};
use tracing::warn;

pub mod logging;

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
        map.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

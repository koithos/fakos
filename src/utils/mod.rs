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
///
/// # Returns
///
/// * `Result<()>` - Success or error
pub fn display_pods(
    pods: &[FarosPod],
    output_format: &OutputFormat,
) -> Result<(), TableDisplayError> {
    if pods.is_empty() {
        warn!("No pods found matching criteria");
        return Ok(());
    }

    let mut table = create_table()?;
    let header_row = create_header_row(output_format);
    table.add_row(header_row);

    for pod in pods {
        let row =
            create_pod_row(pod, output_format).map_err(|e| TableDisplayError::new(&e.message))?;
        table.add_row(row);
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

/// Create a header row for the table based on output format for pods
///
/// # Arguments
///
/// * `output_format` - Format to use for displaying the pods
///
/// # Returns
///
/// * `Row` - A row containing the table headers
fn create_header_row(output_format: &OutputFormat) -> Row {
    let mut header_cells = vec![Cell::new("POD"), Cell::new("NAMESPACE")];

    if matches!(output_format, OutputFormat::Wide) {
        header_cells.extend_from_slice(&[Cell::new("NODE")]);
    }

    Row::new(header_cells)
}

/// Create a row for a single pod
///
/// # Arguments
///
/// * `pod` - The pod to create a row for
/// * `output_format` - Format to use for displaying the pod
///
/// # Returns
///
/// * `Result<Row>` - A row containing the pod information or error
fn create_pod_row(pod: &FarosPod, output_format: &OutputFormat) -> Result<Row, TableDisplayError> {
    let cells = vec![Cell::new(&pod.name), Cell::new(&pod.namespace)];

    Ok(Row::new(cells))
}

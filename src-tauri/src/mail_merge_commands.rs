//! Tauri IPC commands for mail merge operations

use crate::state::MailMergeState;
use mail_merge::{
    ColumnDef, CsvConfig, CsvParser, DataSource, DataType, JsonConfig, JsonParser, Value,
};
use serde::{Deserialize, Serialize};
use tauri::State;

// =============================================================================
// DTOs for Mail Merge Operations
// =============================================================================

/// Column definition DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnDefDto {
    /// Column name (used for field mapping)
    pub name: String,
    /// Data type of the column
    pub data_type: String,
    /// Optional display name
    pub display_name: Option<String>,
    /// Optional description
    pub description: Option<String>,
}

impl From<&ColumnDef> for ColumnDefDto {
    fn from(col: &ColumnDef) -> Self {
        Self {
            name: col.name.clone(),
            data_type: col.data_type.as_str().to_string(),
            display_name: col.display_name.clone(),
            description: col.description.clone(),
        }
    }
}

/// Value DTO for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueDto {
    /// The type of value
    pub value_type: String,
    /// String representation of the value
    pub value: String,
}

impl From<&Value> for ValueDto {
    fn from(val: &Value) -> Self {
        let (value_type, value) = match val {
            Value::Text(s) => ("text", s.clone()),
            Value::Number(n) => ("number", n.to_string()),
            Value::Date(d) => ("date", d.format("%Y-%m-%d").to_string()),
            Value::Boolean(b) => ("boolean", b.to_string()),
            Value::Null => ("null", String::new()),
        };
        Self {
            value_type: value_type.to_string(),
            value,
        }
    }
}

/// Record DTO for frontend (column name -> value)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordDto {
    /// Record data as key-value pairs
    pub data: std::collections::HashMap<String, ValueDto>,
}

/// Data source summary DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSourceSummaryDto {
    /// Data source ID
    pub id: String,
    /// Source type (csv, json, inline)
    pub source_type: String,
    /// Number of columns
    pub column_count: usize,
    /// Number of records
    pub record_count: usize,
    /// Column names
    pub column_names: Vec<String>,
}

impl From<&DataSource> for DataSourceSummaryDto {
    fn from(ds: &DataSource) -> Self {
        let source_type = match &ds.source_type {
            mail_merge::DataSourceType::Csv { .. } => "csv",
            mail_merge::DataSourceType::Json { .. } => "json",
            mail_merge::DataSourceType::Inline { .. } => "inline",
        };
        Self {
            id: ds.id.clone(),
            source_type: source_type.to_string(),
            column_count: ds.column_count(),
            record_count: ds.record_count(),
            column_names: ds.column_names().into_iter().map(String::from).collect(),
        }
    }
}

/// Preview data DTO with columns and sample records
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataPreviewDto {
    /// Column definitions
    pub columns: Vec<ColumnDefDto>,
    /// Preview records
    pub records: Vec<RecordDto>,
    /// Total record count
    pub total_records: usize,
}

// =============================================================================
// Mail Merge Commands
// =============================================================================

/// Load a CSV data source
#[tauri::command]
pub fn load_csv_data_source(
    path: String,
    delimiter: Option<char>,
    has_header: Option<bool>,
    state: State<'_, MailMergeState>,
) -> Result<DataSourceSummaryDto, String> {
    let config = CsvConfig::default()
        .with_delimiter(delimiter.unwrap_or(','))
        .with_header(has_header.unwrap_or(true));

    let parser = CsvParser::with_config(config);
    let data_source = parser.parse_file(&path).map_err(|e| e.to_string())?;

    let summary = DataSourceSummaryDto::from(&data_source);
    let id = data_source.id.clone();

    // Store the data source
    let mut sources = state.sources.lock().map_err(|e| e.to_string())?;
    sources.insert(id, data_source);

    Ok(summary)
}

/// Load a JSON data source
#[tauri::command]
pub fn load_json_data_source(
    path: String,
    root_path: Option<String>,
    state: State<'_, MailMergeState>,
) -> Result<DataSourceSummaryDto, String> {
    let mut config = JsonConfig::new();
    if let Some(rp) = root_path {
        config = config.with_root_path(rp);
    }

    let parser = JsonParser::with_config(config);
    let data_source = parser.parse_file(&path).map_err(|e| e.to_string())?;

    let summary = DataSourceSummaryDto::from(&data_source);
    let id = data_source.id.clone();

    // Store the data source
    let mut sources = state.sources.lock().map_err(|e| e.to_string())?;
    sources.insert(id, data_source);

    Ok(summary)
}

/// Load a data source from a string (for inline data)
#[tauri::command]
pub fn load_csv_from_string(
    data: String,
    id: String,
    delimiter: Option<char>,
    has_header: Option<bool>,
    state: State<'_, MailMergeState>,
) -> Result<DataSourceSummaryDto, String> {
    let config = CsvConfig::default()
        .with_delimiter(delimiter.unwrap_or(','))
        .with_header(has_header.unwrap_or(true));

    let parser = CsvParser::with_config(config);
    let data_source = parser.parse_string(&data, &id).map_err(|e| e.to_string())?;

    let summary = DataSourceSummaryDto::from(&data_source);

    let mut sources = state.sources.lock().map_err(|e| e.to_string())?;
    sources.insert(id, data_source);

    Ok(summary)
}

/// Load a JSON data source from a string
#[tauri::command]
pub fn load_json_from_string(
    data: String,
    id: String,
    root_path: Option<String>,
    state: State<'_, MailMergeState>,
) -> Result<DataSourceSummaryDto, String> {
    let mut config = JsonConfig::new();
    if let Some(rp) = root_path {
        config = config.with_root_path(rp);
    }

    let parser = JsonParser::with_config(config);
    let data_source = parser.parse_string(&data, &id).map_err(|e| e.to_string())?;

    let summary = DataSourceSummaryDto::from(&data_source);

    let mut sources = state.sources.lock().map_err(|e| e.to_string())?;
    sources.insert(id, data_source);

    Ok(summary)
}

/// Get columns for a data source
#[tauri::command]
pub fn get_data_source_columns(
    id: String,
    state: State<'_, MailMergeState>,
) -> Result<Vec<ColumnDefDto>, String> {
    let sources = state.sources.lock().map_err(|e| e.to_string())?;
    let data_source = sources
        .get(&id)
        .ok_or_else(|| format!("Data source '{}' not found", id))?;

    Ok(data_source.columns.iter().map(ColumnDefDto::from).collect())
}

/// Get a preview of records from a data source
#[tauri::command]
pub fn get_data_source_preview(
    id: String,
    limit: Option<usize>,
    state: State<'_, MailMergeState>,
) -> Result<DataPreviewDto, String> {
    let sources = state.sources.lock().map_err(|e| e.to_string())?;
    let data_source = sources
        .get(&id)
        .ok_or_else(|| format!("Data source '{}' not found", id))?;

    let limit = limit.unwrap_or(10);
    let preview_records = data_source.preview(limit);

    let records: Vec<RecordDto> = preview_records
        .into_iter()
        .map(|record| {
            let data: std::collections::HashMap<String, ValueDto> = record
                .iter()
                .map(|(k, v)| (k.clone(), ValueDto::from(v)))
                .collect();
            RecordDto { data }
        })
        .collect();

    Ok(DataPreviewDto {
        columns: data_source.columns.iter().map(ColumnDefDto::from).collect(),
        records,
        total_records: data_source.record_count(),
    })
}

/// Get a specific record from a data source
#[tauri::command]
pub fn get_data_source_record(
    id: String,
    index: usize,
    state: State<'_, MailMergeState>,
) -> Result<RecordDto, String> {
    let sources = state.sources.lock().map_err(|e| e.to_string())?;
    let data_source = sources
        .get(&id)
        .ok_or_else(|| format!("Data source '{}' not found", id))?;

    let record = data_source
        .get_record(index)
        .ok_or_else(|| format!("Record at index {} not found", index))?;

    let data: std::collections::HashMap<String, ValueDto> = record
        .iter()
        .map(|(k, v)| (k.clone(), ValueDto::from(v)))
        .collect();

    Ok(RecordDto { data })
}

/// Get all records from a data source
#[tauri::command]
pub fn get_data_source_records(
    id: String,
    offset: Option<usize>,
    limit: Option<usize>,
    state: State<'_, MailMergeState>,
) -> Result<Vec<RecordDto>, String> {
    let sources = state.sources.lock().map_err(|e| e.to_string())?;
    let data_source = sources
        .get(&id)
        .ok_or_else(|| format!("Data source '{}' not found", id))?;

    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(data_source.record_count());

    let records: Vec<RecordDto> = data_source
        .records
        .iter()
        .skip(offset)
        .take(limit)
        .map(|record| {
            let data: std::collections::HashMap<String, ValueDto> = record
                .iter()
                .map(|(k, v)| (k.clone(), ValueDto::from(v)))
                .collect();
            RecordDto { data }
        })
        .collect();

    Ok(records)
}

/// Get a value from a specific record and column
#[tauri::command]
pub fn get_data_source_value(
    id: String,
    record_index: usize,
    column_name: String,
    state: State<'_, MailMergeState>,
) -> Result<ValueDto, String> {
    let sources = state.sources.lock().map_err(|e| e.to_string())?;
    let data_source = sources
        .get(&id)
        .ok_or_else(|| format!("Data source '{}' not found", id))?;

    let value = data_source
        .get_value(record_index, &column_name)
        .ok_or_else(|| {
            format!(
                "Value not found at record {} column '{}'",
                record_index, column_name
            )
        })?;

    Ok(ValueDto::from(value))
}

/// List all loaded data sources
#[tauri::command]
pub fn list_data_sources(
    state: State<'_, MailMergeState>,
) -> Result<Vec<DataSourceSummaryDto>, String> {
    let sources = state.sources.lock().map_err(|e| e.to_string())?;
    Ok(sources.values().map(DataSourceSummaryDto::from).collect())
}

/// Remove a data source
#[tauri::command]
pub fn remove_data_source(
    id: String,
    state: State<'_, MailMergeState>,
) -> Result<bool, String> {
    let mut sources = state.sources.lock().map_err(|e| e.to_string())?;
    Ok(sources.remove(&id).is_some())
}

/// Clear all data sources
#[tauri::command]
pub fn clear_data_sources(state: State<'_, MailMergeState>) -> Result<(), String> {
    let mut sources = state.sources.lock().map_err(|e| e.to_string())?;
    sources.clear();
    Ok(())
}

/// Detect delimiter in a CSV string
#[tauri::command]
pub fn detect_csv_delimiter(content: String) -> char {
    mail_merge::detect_delimiter(&content)
}

/// Detect if CSV has header row
#[tauri::command]
pub fn detect_csv_has_header(content: String, delimiter: Option<char>) -> bool {
    let delim = delimiter.unwrap_or(',');
    mail_merge::detect_has_header(&content, delim)
}

/// Get data source info by ID
#[tauri::command]
pub fn get_data_source_info(
    id: String,
    state: State<'_, MailMergeState>,
) -> Result<DataSourceSummaryDto, String> {
    let sources = state.sources.lock().map_err(|e| e.to_string())?;
    let data_source = sources
        .get(&id)
        .ok_or_else(|| format!("Data source '{}' not found", id))?;

    Ok(DataSourceSummaryDto::from(data_source))
}

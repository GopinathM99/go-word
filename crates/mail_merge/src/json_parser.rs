//! JSON parser for mail merge data sources

use std::collections::HashSet;
use std::path::Path;

use serde_json::{Map, Value as JsonValue};

use crate::data_source::{ColumnDef, DataSource, DataSourceType, DataType, Record, Value};
use crate::error::{MailMergeError, Result};

/// JSON parser configuration
#[derive(Debug, Clone, Default)]
pub struct JsonConfig {
    /// Root path to the data array (e.g., "data.customers")
    pub root_path: Option<String>,
    /// Whether to flatten nested objects with dot notation
    pub flatten_nested: bool,
    /// Maximum nesting depth for flattening (default: 3)
    pub max_depth: usize,
}

impl JsonConfig {
    /// Create a new JSON config with default settings
    pub fn new() -> Self {
        Self {
            root_path: None,
            flatten_nested: true,
            max_depth: 3,
        }
    }

    /// Set the root path for the data array
    pub fn with_root_path(mut self, path: impl Into<String>) -> Self {
        self.root_path = Some(path.into());
        self
    }

    /// Set whether to flatten nested objects
    pub fn with_flatten(mut self, flatten: bool) -> Self {
        self.flatten_nested = flatten;
        self
    }

    /// Set maximum depth for flattening
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }
}

/// JSON parser for creating data sources from JSON files or strings
pub struct JsonParser {
    config: JsonConfig,
}

impl JsonParser {
    /// Create a new JSON parser with default configuration
    pub fn new() -> Self {
        Self {
            config: JsonConfig::new(),
        }
    }

    /// Create a new JSON parser with custom configuration
    pub fn with_config(config: JsonConfig) -> Self {
        Self { config }
    }

    /// Parse a JSON file and return a DataSource
    pub fn parse_file(&self, path: impl AsRef<Path>) -> Result<DataSource> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(MailMergeError::FileNotFound(
                path.display().to_string()
            ));
        }

        let content = std::fs::read_to_string(path)?;
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("json_source")
            .to_string();

        let source_type = DataSourceType::Json {
            path: path.display().to_string(),
            root_path: self.config.root_path.clone(),
        };

        self.parse_string_with_source_type(&content, id, source_type)
    }

    /// Parse JSON from a string and return a DataSource
    pub fn parse_string(&self, data: &str, id: impl Into<String>) -> Result<DataSource> {
        let source_type = DataSourceType::Inline {
            data: Vec::new(),
        };
        self.parse_string_with_source_type(data, id.into(), source_type)
    }

    /// Parse JSON string with a specific source type
    fn parse_string_with_source_type(
        &self,
        data: &str,
        id: String,
        source_type: DataSourceType,
    ) -> Result<DataSource> {
        let json: JsonValue = serde_json::from_str(data)?;
        self.parse_json_value(&json, id, source_type)
    }

    /// Parse a JSON value into a DataSource
    fn parse_json_value(
        &self,
        json: &JsonValue,
        id: String,
        source_type: DataSourceType,
    ) -> Result<DataSource> {
        // Navigate to root path if specified
        let data = if let Some(ref path) = self.config.root_path {
            navigate_to_path(json, path)?
        } else {
            json
        };

        // Expect an array of objects
        let array = match data {
            JsonValue::Array(arr) => arr,
            JsonValue::Object(_) => {
                // Single object - wrap in array
                return self.parse_single_object(data, id, source_type);
            }
            _ => {
                return Err(MailMergeError::InvalidDataSource(
                    "Expected JSON array or object".to_string()
                ));
            }
        };

        if array.is_empty() {
            return Err(MailMergeError::EmptyDataSource(
                "JSON array is empty".to_string()
            ));
        }

        let mut data_source = DataSource::new(id, source_type);

        // Collect all unique column names from all objects
        let columns = self.collect_columns(array);

        // Add column definitions
        for (name, data_type) in &columns {
            data_source.add_column(ColumnDef::new(name.clone(), *data_type));
        }

        // Convert each object to a record
        for item in array {
            if let JsonValue::Object(obj) = item {
                let record = self.object_to_record(obj, "", 0);
                data_source.add_record(record);
            }
        }

        Ok(data_source)
    }

    /// Parse a single JSON object as a data source with one record
    fn parse_single_object(
        &self,
        json: &JsonValue,
        id: String,
        source_type: DataSourceType,
    ) -> Result<DataSource> {
        let obj = match json {
            JsonValue::Object(obj) => obj,
            _ => {
                return Err(MailMergeError::InvalidDataSource(
                    "Expected JSON object".to_string()
                ));
            }
        };

        let mut data_source = DataSource::new(id, source_type);
        let record = self.object_to_record(obj, "", 0);

        // Infer columns from the record
        for (key, value) in &record {
            let data_type = value.data_type().unwrap_or(DataType::Text);
            data_source.add_column(ColumnDef::new(key.clone(), data_type));
        }

        data_source.add_record(record);
        Ok(data_source)
    }

    /// Collect all unique column names and their types from an array of objects
    fn collect_columns(&self, array: &[JsonValue]) -> Vec<(String, DataType)> {
        let mut column_types: std::collections::HashMap<String, Vec<DataType>> = std::collections::HashMap::new();
        let mut column_order: Vec<String> = Vec::new();

        for item in array {
            if let JsonValue::Object(obj) = item {
                self.collect_columns_from_object(obj, "", 0, &mut column_types, &mut column_order);
            }
        }

        // Determine final type for each column
        let empty_vec = vec![];
        column_order
            .into_iter()
            .map(|name| {
                let types = column_types.get(&name).unwrap_or(&empty_vec);
                let data_type = infer_column_type(types);
                (name, data_type)
            })
            .collect()
    }

    /// Recursively collect column names from a JSON object
    fn collect_columns_from_object(
        &self,
        obj: &Map<String, JsonValue>,
        prefix: &str,
        depth: usize,
        column_types: &mut std::collections::HashMap<String, Vec<DataType>>,
        column_order: &mut Vec<String>,
    ) {
        for (key, value) in obj {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            match value {
                JsonValue::Object(nested) if self.config.flatten_nested && depth < self.config.max_depth => {
                    self.collect_columns_from_object(nested, &full_key, depth + 1, column_types, column_order);
                }
                _ => {
                    let data_type = json_value_to_data_type(value);

                    // Track column order
                    if !column_types.contains_key(&full_key) {
                        column_order.push(full_key.clone());
                    }

                    // Track types for this column
                    column_types
                        .entry(full_key)
                        .or_default()
                        .push(data_type);
                }
            }
        }
    }

    /// Convert a JSON object to a Record
    fn object_to_record(&self, obj: &Map<String, JsonValue>, prefix: &str, depth: usize) -> Record {
        let mut record = Record::new();

        for (key, value) in obj {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            match value {
                JsonValue::Object(nested) if self.config.flatten_nested && depth < self.config.max_depth => {
                    let nested_record = self.object_to_record(nested, &full_key, depth + 1);
                    for (k, v) in nested_record {
                        record.insert(k, v);
                    }
                }
                _ => {
                    let converted = json_value_to_value(value);
                    record.insert(full_key, converted);
                }
            }
        }

        record
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Navigate to a nested path in a JSON value using dot notation
fn navigate_to_path<'a>(json: &'a JsonValue, path: &str) -> Result<&'a JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = json;

    for part in parts {
        match current {
            JsonValue::Object(obj) => {
                current = obj.get(part).ok_or_else(|| {
                    MailMergeError::InvalidPath(format!("Path '{}' not found", path))
                })?;
            }
            JsonValue::Array(arr) => {
                // Support numeric indices for arrays
                let index: usize = part.parse().map_err(|_| {
                    MailMergeError::InvalidPath(format!(
                        "Expected numeric index for array access, got '{}'",
                        part
                    ))
                })?;
                current = arr.get(index).ok_or_else(|| {
                    MailMergeError::InvalidPath(format!("Array index {} out of bounds", index))
                })?;
            }
            _ => {
                return Err(MailMergeError::InvalidPath(format!(
                    "Cannot access '{}' on non-object/array value",
                    part
                )));
            }
        }
    }

    Ok(current)
}

/// Convert a JSON value to our Value type
fn json_value_to_value(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else if let Some(i) = n.as_i64() {
                Value::Number(i as f64)
            } else if let Some(u) = n.as_u64() {
                Value::Number(u as f64)
            } else {
                Value::Null
            }
        }
        JsonValue::String(s) => {
            // Try to detect if it's a date
            Value::parse_auto(s)
        }
        JsonValue::Array(arr) => {
            // Convert array to a JSON string representation
            Value::Text(serde_json::to_string(arr).unwrap_or_default())
        }
        JsonValue::Object(obj) => {
            // Convert object to a JSON string representation
            Value::Text(serde_json::to_string(obj).unwrap_or_default())
        }
    }
}

/// Determine the data type from a JSON value
fn json_value_to_data_type(json: &JsonValue) -> DataType {
    match json {
        JsonValue::Null => DataType::Text, // Default for null
        JsonValue::Bool(_) => DataType::Boolean,
        JsonValue::Number(_) => DataType::Number,
        JsonValue::String(s) => {
            // Check if it's a date
            match Value::parse_auto(s) {
                Value::Date(_) => DataType::Date,
                Value::Number(_) => DataType::Number,
                Value::Boolean(_) => DataType::Boolean,
                _ => DataType::Text,
            }
        }
        JsonValue::Array(_) | JsonValue::Object(_) => DataType::Text,
    }
}

/// Infer the best column type from a list of observed types
fn infer_column_type(types: &[DataType]) -> DataType {
    if types.is_empty() {
        return DataType::Text;
    }

    let unique_types: HashSet<DataType> = types.iter().copied().collect();

    // If all types are the same, use that type
    if unique_types.len() == 1 {
        return types[0];
    }

    // If there's any text, use text
    if unique_types.contains(&DataType::Text) {
        return DataType::Text;
    }

    // If mixed numeric types, use number
    if unique_types.contains(&DataType::Number) {
        return DataType::Number;
    }

    // Default to text for mixed types
    DataType::Text
}

/// Extract a value from a JSON object using dot notation path
pub fn get_nested_value<'a>(json: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
    navigate_to_path(json, path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_array() {
        let json = r#"[
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]"#;

        let parser = JsonParser::new();
        let ds = parser.parse_string(json, "test").unwrap();

        assert_eq!(ds.column_count(), 2);
        assert_eq!(ds.record_count(), 2);
        assert!(ds.has_column("name"));
        assert!(ds.has_column("age"));
    }

    #[test]
    fn test_parse_with_root_path() {
        let json = r#"{
            "status": "ok",
            "data": {
                "customers": [
                    {"name": "Alice", "email": "alice@example.com"},
                    {"name": "Bob", "email": "bob@example.com"}
                ]
            }
        }"#;

        let parser = JsonParser::with_config(
            JsonConfig::new().with_root_path("data.customers")
        );
        let ds = parser.parse_string(json, "test").unwrap();

        assert_eq!(ds.column_count(), 2);
        assert_eq!(ds.record_count(), 2);
        assert!(ds.has_column("name"));
        assert!(ds.has_column("email"));
    }

    #[test]
    fn test_parse_nested_objects() {
        let json = r#"[
            {
                "name": "Alice",
                "address": {
                    "city": "New York",
                    "zip": "10001"
                }
            },
            {
                "name": "Bob",
                "address": {
                    "city": "Los Angeles",
                    "zip": "90001"
                }
            }
        ]"#;

        let parser = JsonParser::with_config(JsonConfig::new().with_flatten(true));
        let ds = parser.parse_string(json, "test").unwrap();

        assert!(ds.has_column("name"));
        assert!(ds.has_column("address.city"));
        assert!(ds.has_column("address.zip"));

        let record = ds.get_record(0).unwrap();
        assert_eq!(record.get("address.city").unwrap().to_string_value(), "New York");
    }

    #[test]
    fn test_parse_no_flatten() {
        let json = r#"[
            {
                "name": "Alice",
                "address": {
                    "city": "New York"
                }
            }
        ]"#;

        let parser = JsonParser::with_config(JsonConfig::new().with_flatten(false));
        let ds = parser.parse_string(json, "test").unwrap();

        assert!(ds.has_column("name"));
        assert!(ds.has_column("address"));
        assert!(!ds.has_column("address.city"));

        // Address should be a JSON string
        let record = ds.get_record(0).unwrap();
        let addr = record.get("address").unwrap().to_string_value();
        assert!(addr.contains("city"));
    }

    #[test]
    fn test_type_detection() {
        let json = r#"[
            {
                "name": "Alice",
                "age": 30,
                "active": true,
                "joined": "2024-01-15"
            }
        ]"#;

        let parser = JsonParser::new();
        let ds = parser.parse_string(json, "test").unwrap();

        assert_eq!(ds.get_column("name").unwrap().data_type, DataType::Text);
        assert_eq!(ds.get_column("age").unwrap().data_type, DataType::Number);
        assert_eq!(ds.get_column("active").unwrap().data_type, DataType::Boolean);
        assert_eq!(ds.get_column("joined").unwrap().data_type, DataType::Date);
    }

    #[test]
    fn test_null_values() {
        let json = r#"[
            {"name": "Alice", "value": null},
            {"name": "Bob", "value": 42}
        ]"#;

        let parser = JsonParser::new();
        let ds = parser.parse_string(json, "test").unwrap();

        let r1 = ds.get_record(0).unwrap();
        assert!(r1.get("value").unwrap().is_null());

        let r2 = ds.get_record(1).unwrap();
        assert!(!r2.get("value").unwrap().is_null());
    }

    #[test]
    fn test_single_object() {
        let json = r#"{"name": "Alice", "age": 30}"#;

        let parser = JsonParser::new();
        let ds = parser.parse_string(json, "test").unwrap();

        assert_eq!(ds.column_count(), 2);
        assert_eq!(ds.record_count(), 1);
    }

    #[test]
    fn test_array_in_root_path() {
        let json = r#"{
            "items": [
                {"id": 1, "name": "Item1"},
                {"id": 2, "name": "Item2"}
            ]
        }"#;

        let parser = JsonParser::with_config(JsonConfig::new().with_root_path("items"));
        let ds = parser.parse_string(json, "test").unwrap();

        assert_eq!(ds.record_count(), 2);
    }

    #[test]
    fn test_invalid_root_path() {
        let json = r#"{"data": {"items": []}}"#;

        let parser = JsonParser::with_config(
            JsonConfig::new().with_root_path("nonexistent.path")
        );
        let result = parser.parse_string(json, "test");

        assert!(matches!(result, Err(MailMergeError::InvalidPath(_))));
    }

    #[test]
    fn test_empty_array() {
        let json = r#"[]"#;

        let parser = JsonParser::new();
        let result = parser.parse_string(json, "test");

        assert!(matches!(result, Err(MailMergeError::EmptyDataSource(_))));
    }

    #[test]
    fn test_mixed_types_fallback_to_text() {
        let json = r#"[
            {"value": "text"},
            {"value": 42},
            {"value": true}
        ]"#;

        let parser = JsonParser::new();
        let ds = parser.parse_string(json, "test").unwrap();

        // Mixed types should fall back to Text
        assert_eq!(ds.get_column("value").unwrap().data_type, DataType::Text);
    }

    #[test]
    fn test_navigate_to_path() {
        let json: JsonValue = serde_json::from_str(r#"{
            "a": {
                "b": {
                    "c": [1, 2, 3]
                }
            }
        }"#).unwrap();

        let result = navigate_to_path(&json, "a.b.c").unwrap();
        assert!(result.is_array());

        let result = navigate_to_path(&json, "a.b.c.0").unwrap();
        assert_eq!(result.as_i64(), Some(1));
    }

    #[test]
    fn test_sparse_columns() {
        // Not all records have all columns
        let json = r#"[
            {"name": "Alice", "age": 30},
            {"name": "Bob", "city": "LA"},
            {"name": "Charlie", "age": 25, "city": "NYC"}
        ]"#;

        let parser = JsonParser::new();
        let ds = parser.parse_string(json, "test").unwrap();

        assert_eq!(ds.column_count(), 3);
        assert!(ds.has_column("name"));
        assert!(ds.has_column("age"));
        assert!(ds.has_column("city"));
    }

    #[test]
    fn test_deep_nesting_limit() {
        let json = r#"[
            {
                "a": {
                    "b": {
                        "c": {
                            "d": {
                                "e": "deep"
                            }
                        }
                    }
                }
            }
        ]"#;

        // With max depth of 2, we should stop flattening at a.b.c
        let parser = JsonParser::with_config(
            JsonConfig::new().with_max_depth(2)
        );
        let ds = parser.parse_string(json, "test").unwrap();

        assert!(ds.has_column("a.b.c"));
        assert!(!ds.has_column("a.b.c.d"));
    }
}

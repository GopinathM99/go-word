//! Mail Merge Execution Engine
//!
//! Orchestrates the merge process: iterating records, resolving fields,
//! generating output documents or previews.

use crate::data_source::{DataSource, DataSourceType, Record, Value};
use crate::merge_field::{ComparisonOperator, ConditionalField, MergeField, MergeFieldInstruction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeOutputType { SingleDocument, IndividualDocuments, Preview, Email }

impl Default for MergeOutputType { fn default() -> Self { MergeOutputType::SingleDocument } }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordRange {
    All,
    Range { start: usize, end: usize },
    Current(usize),
    Filter { field: String, operator: String, value: String },
}
impl Default for RecordRange { fn default() -> Self { RecordRange::All } }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeOptions {
    pub output_type: MergeOutputType,
    pub record_range: RecordRange,
    pub page_break_between_records: bool,
    pub trim_values: bool,
    pub remove_empty_paragraphs: bool,
    pub max_records: usize,
    pub output_name_pattern: String,
    pub output_directory: Option<String>,
}

impl Default for MergeOptions {
    fn default() -> Self {
        Self { output_type: MergeOutputType::SingleDocument, record_range: RecordRange::All,
               page_break_between_records: true, trim_values: true, remove_empty_paragraphs: false,
               max_records: 0, output_name_pattern: "merged_{index}.docx".to_string(), output_directory: None }
    }
}

impl MergeOptions {
    pub fn single_document() -> Self { Self { output_type: MergeOutputType::SingleDocument, ..Default::default() } }
    pub fn individual_documents() -> Self { Self { output_type: MergeOutputType::IndividualDocuments, ..Default::default() } }
    pub fn preview() -> Self { Self { output_type: MergeOutputType::Preview, max_records: 10, ..Default::default() } }
    pub fn with_range(mut self, range: RecordRange) -> Self { self.record_range = range; self }
    pub fn with_max_records(mut self, max: usize) -> Self { self.max_records = max; self }
    pub fn with_output_pattern(mut self, pattern: impl Into<String>) -> Self { self.output_name_pattern = pattern.into(); self }
    pub fn with_output_directory(mut self, dir: impl Into<String>) -> Self { self.output_directory = Some(dir.into()); self }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedRecord {
    pub record_index: usize,
    pub field_values: HashMap<String, String>,
    pub skipped: bool,
    pub skip_reason: Option<String>,
    pub output_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStatus { Pending, InProgress, Completed, Failed, Cancelled }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeResult {
    pub status: MergeStatus, pub total_records: usize, pub processed_count: usize,
    pub skipped_count: usize, pub error_count: usize, pub merged_records: Vec<MergedRecord>,
    pub errors: Vec<MergeError>, pub output_paths: Vec<String>, pub summary: String,
}

impl MergeResult {
    pub fn new(total_records: usize) -> Self {
        Self { status: MergeStatus::Pending, total_records, processed_count: 0, skipped_count: 0,
               error_count: 0, merged_records: Vec::new(), errors: Vec::new(), output_paths: Vec::new(), summary: String::new() }
    }
    pub fn is_success(&self) -> bool { self.status == MergeStatus::Completed && self.error_count == 0 }
    pub fn success_rate(&self) -> f64 {
        if self.processed_count == 0 { return 0.0; }
        ((self.processed_count - self.error_count) as f64 / self.processed_count as f64) * 100.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeError { pub record_index: usize, pub message: String, pub field_name: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeProgress { pub current_record: usize, pub total_records: usize, pub status: MergeStatus, pub percent: f64 }

impl MergeProgress {
    pub fn at(current: usize, total: usize) -> Self {
        let percent = if total > 0 { (current as f64 / total as f64) * 100.0 } else { 0.0 };
        Self { current_record: current, total_records: total, status: MergeStatus::InProgress, percent }
    }
}

pub struct MergeEngine {
    data_source: DataSource,
    fields: Vec<MergeFieldInstruction>,
    options: MergeOptions,
}

impl MergeEngine {
    pub fn new(data_source: DataSource, fields: Vec<MergeFieldInstruction>, options: MergeOptions) -> Self {
        Self { data_source, fields, options }
    }

    pub fn execute(&self) -> MergeResult {
        let record_indices = self.resolve_record_range();
        let mut result = MergeResult::new(self.data_source.record_count());
        result.status = MergeStatus::InProgress;
        for (i, &record_idx) in record_indices.iter().enumerate() {
            if self.options.max_records > 0 && i >= self.options.max_records { break; }
            match self.process_record(record_idx) {
                Ok(merged) => {
                    result.processed_count += 1;
                    if merged.skipped { result.skipped_count += 1; }
                    if let Some(ref name) = merged.output_name { result.output_paths.push(name.clone()); }
                    result.merged_records.push(merged);
                }
                Err(err) => { result.processed_count += 1; result.error_count += 1; result.errors.push(err); }
            }
        }
        result.status = if result.error_count == 0 { MergeStatus::Completed }
            else if result.processed_count == result.error_count { MergeStatus::Failed }
            else { MergeStatus::Completed };
        result.summary = format!("Processed {} of {} records ({} skipped, {} errors)",
            result.processed_count, result.total_records, result.skipped_count, result.error_count);
        result
    }

    pub fn execute_with_progress<F>(&self, mut on_progress: F) -> MergeResult where F: FnMut(MergeProgress) {
        let ri = self.resolve_record_range();
        let total = ri.len();
        let mut result = MergeResult::new(self.data_source.record_count());
        result.status = MergeStatus::InProgress;
        for (i, &idx) in ri.iter().enumerate() {
            if self.options.max_records > 0 && i >= self.options.max_records { break; }
            on_progress(MergeProgress::at(i + 1, total));
            match self.process_record(idx) {
                Ok(m) => { result.processed_count += 1; if m.skipped { result.skipped_count += 1; } if let Some(ref n) = m.output_name { result.output_paths.push(n.clone()); } result.merged_records.push(m); }
                Err(e) => { result.processed_count += 1; result.error_count += 1; result.errors.push(e); }
            }
        }
        result.status = MergeStatus::Completed;
        result.summary = format!("Processed {} of {} records ({} skipped, {} errors)",
            result.processed_count, result.total_records, result.skipped_count, result.error_count);
        result
    }

    pub fn preview(&self, count: usize) -> MergeResult {
        let ri = self.resolve_record_range();
        let pc = count.min(ri.len());
        let mut result = MergeResult::new(self.data_source.record_count());
        result.status = MergeStatus::InProgress;
        for &idx in ri.iter().take(pc) {
            match self.process_record(idx) {
                Ok(m) => { result.processed_count += 1; if m.skipped { result.skipped_count += 1; } result.merged_records.push(m); }
                Err(e) => { result.processed_count += 1; result.error_count += 1; result.errors.push(e); }
            }
        }
        result.status = MergeStatus::Completed;
        result.summary = format!("Preview: {} records shown", result.processed_count);
        result
    }

    fn process_record(&self, record_index: usize) -> Result<MergedRecord, MergeError> {
        let record = self.data_source.get_record(record_index).ok_or(MergeError {
            record_index, message: format!("Record at index {} not found", record_index), field_name: None,
        })?;
        let mut field_values = HashMap::new();
        let mut skipped = false;
        let mut skip_reason = None;
        for instruction in &self.fields {
            match instruction {
                MergeFieldInstruction::Field(field) => {
                    let value = field.resolve(record);
                    let value = if self.options.trim_values { value.trim().to_string() } else { value };
                    field_values.insert(field.field_name.clone(), value);
                }
                MergeFieldInstruction::SkipIf(condition) => {
                    if condition.evaluate(record) {
                        skipped = true;
                        skip_reason = Some(format!("SKIPIF: {} {} {}", condition.field_name, condition.operator.as_str(), condition.compare_value));
                    }
                }
                MergeFieldInstruction::Next | MergeFieldInstruction::NextIf(_) => {}
            }
        }
        let output_name = if self.options.output_type == MergeOutputType::IndividualDocuments {
            Some(self.resolve_output_name(&field_values, record_index))
        } else { None };
        Ok(MergedRecord { record_index, field_values, skipped, skip_reason, output_name })
    }

    fn resolve_record_range(&self) -> Vec<usize> {
        let total = self.data_source.record_count();
        match &self.options.record_range {
            RecordRange::All => (0..total).collect(),
            RecordRange::Range { start, end } => { let s = (*start).min(total); let e = (*end + 1).min(total); (s..e).collect() }
            RecordRange::Current(idx) => { if *idx < total { vec![*idx] } else { Vec::new() } }
            RecordRange::Filter { field, operator, value } => {
                let op = ComparisonOperator::from_str(operator).unwrap_or(ComparisonOperator::Equal);
                (0..total).filter(|&i| {
                    if let Some(record) = self.data_source.get_record(i) {
                        let fv = record.get(field).map(|v| v.to_string_value()).unwrap_or_default();
                        op.evaluate(&fv, value)
                    } else { false }
                }).collect()
            }
        }
    }

    fn resolve_output_name(&self, field_values: &HashMap<String, String>, index: usize) -> String {
        let mut name = self.options.output_name_pattern.clone();
        name = name.replace("{index}", &(index + 1).to_string());
        for (field, value) in field_values {
            let placeholder = format!("{{{}}}", field);
            let safe_value: String = value.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
            name = name.replace(&placeholder, &safe_value);
        }
        name
    }

    pub fn data_source(&self) -> &DataSource { &self.data_source }
    pub fn options(&self) -> &MergeOptions { &self.options }
    pub fn fields(&self) -> &[MergeFieldInstruction] { &self.fields }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_source::{ColumnDef, DataSource, DataSourceType, DataType};
    use crate::merge_field::{ComparisonOperator, ConditionalField, MergeField};

    fn sample_data_source() -> DataSource {
        let mut ds = DataSource::inline("test");
        ds.add_column(ColumnDef::new("first_name", DataType::Text));
        ds.add_column(ColumnDef::new("last_name", DataType::Text));
        ds.add_column(ColumnDef::new("amount", DataType::Number));
        ds.add_column(ColumnDef::new("city", DataType::Text));
        let mut r1 = Record::new(); r1.insert("first_name".into(), Value::Text("John".into())); r1.insert("last_name".into(), Value::Text("Doe".into())); r1.insert("amount".into(), Value::Number(100.0)); r1.insert("city".into(), Value::Text("Boston".into())); ds.add_record(r1);
        let mut r2 = Record::new(); r2.insert("first_name".into(), Value::Text("Jane".into())); r2.insert("last_name".into(), Value::Text("Smith".into())); r2.insert("amount".into(), Value::Number(250.0)); r2.insert("city".into(), Value::Text("Chicago".into())); ds.add_record(r2);
        let mut r3 = Record::new(); r3.insert("first_name".into(), Value::Text("Bob".into())); r3.insert("last_name".into(), Value::Text("Jones".into())); r3.insert("amount".into(), Value::Number(50.0)); r3.insert("city".into(), Value::Text("Denver".into())); ds.add_record(r3);
        ds
    }

    fn sample_fields() -> Vec<MergeFieldInstruction> {
        vec![MergeFieldInstruction::Field(MergeField::new("first_name")), MergeFieldInstruction::Field(MergeField::new("last_name")),
             MergeFieldInstruction::Field(MergeField::new("amount")), MergeFieldInstruction::Field(MergeField::new("city"))]
    }

    #[test] fn test_merge_all_records() {
        let result = MergeEngine::new(sample_data_source(), sample_fields(), MergeOptions::single_document()).execute();
        assert_eq!(result.status, MergeStatus::Completed); assert_eq!(result.processed_count, 3);
        assert!(result.is_success()); assert_eq!(result.merged_records[0].field_values.get("first_name").unwrap(), "John");
    }

    #[test] fn test_merge_with_range() {
        let r = MergeEngine::new(sample_data_source(), sample_fields(), MergeOptions::single_document().with_range(RecordRange::Range { start: 0, end: 1 })).execute();
        assert_eq!(r.processed_count, 2);
    }

    #[test] fn test_merge_current_record() {
        let r = MergeEngine::new(sample_data_source(), sample_fields(), MergeOptions::single_document().with_range(RecordRange::Current(1))).execute();
        assert_eq!(r.processed_count, 1); assert_eq!(r.merged_records[0].field_values.get("first_name").unwrap(), "Jane");
    }

    #[test] fn test_merge_with_filter() {
        let r = MergeEngine::new(sample_data_source(), sample_fields(),
            MergeOptions::single_document().with_range(RecordRange::Filter { field: "amount".into(), operator: ">=".into(), value: "100".into() })).execute();
        assert_eq!(r.processed_count, 2);
    }

    #[test] fn test_merge_max_records() {
        assert_eq!(MergeEngine::new(sample_data_source(), sample_fields(), MergeOptions::single_document().with_max_records(2)).execute().processed_count, 2);
    }

    #[test] fn test_merge_with_skipif() {
        let fields = vec![MergeFieldInstruction::Field(MergeField::new("first_name")),
            MergeFieldInstruction::SkipIf(ConditionalField::new("amount", ComparisonOperator::LessThan, "100"))];
        let r = MergeEngine::new(sample_data_source(), fields, MergeOptions::single_document()).execute();
        assert_eq!(r.processed_count, 3); assert_eq!(r.skipped_count, 1); assert!(r.merged_records[2].skipped);
    }

    #[test] fn test_merge_preview() {
        let r = MergeEngine::new(sample_data_source(), sample_fields(), MergeOptions::preview()).preview(2);
        assert_eq!(r.processed_count, 2); assert_eq!(r.summary, "Preview: 2 records shown");
    }

    #[test] fn test_merge_individual_documents() {
        let r = MergeEngine::new(sample_data_source(), sample_fields(), MergeOptions::individual_documents().with_output_pattern("Letter_{last_name}.docx")).execute();
        assert_eq!(r.output_paths.len(), 3);
        assert_eq!(r.merged_records[0].output_name.as_ref().unwrap(), "Letter_Doe.docx");
    }

    #[test] fn test_merge_with_progress() {
        let mut updates = Vec::new();
        let r = MergeEngine::new(sample_data_source(), sample_fields(), MergeOptions::single_document())
            .execute_with_progress(|p| { updates.push(p); });
        assert_eq!(updates.len(), 3); assert!(r.is_success());
    }

    #[test] fn test_merge_success_rate() {
        let mut r = MergeResult::new(10); r.processed_count = 10; r.error_count = 2;
        assert!((r.success_rate() - 80.0).abs() < 0.01);
    }

    #[test] fn test_merge_options_defaults() {
        let opts = MergeOptions::default();
        assert_eq!(opts.output_type, MergeOutputType::SingleDocument); assert!(opts.trim_values);
    }

    #[test] fn test_merge_status_serialization() {
        assert_eq!(serde_json::to_string(&MergeStatus::Completed).unwrap(), "\"completed\"");
    }

    #[test] fn test_empty_data_source_merge() {
        let r = MergeEngine::new(DataSource::inline("empty"), sample_fields(), MergeOptions::single_document()).execute();
        assert_eq!(r.processed_count, 0); assert_eq!(r.status, MergeStatus::Completed);
    }
}

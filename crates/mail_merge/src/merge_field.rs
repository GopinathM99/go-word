//! Merge field types for mail merge operations

use crate::data_source::{Record, Value};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeField {
    pub field_name: String,
    pub format: Option<String>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub default_value: Option<String>,
}

impl MergeField {
    pub fn new(name: impl Into<String>) -> Self {
        Self { field_name: name.into(), format: None, prefix: None, suffix: None, default_value: None }
    }
    pub fn with_format(mut self, format: impl Into<String>) -> Self { self.format = Some(format.into()); self }
    pub fn with_default(mut self, default: impl Into<String>) -> Self { self.default_value = Some(default.into()); self }
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self { self.prefix = Some(prefix.into()); self }
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self { self.suffix = Some(suffix.into()); self }

    pub fn resolve(&self, record: &Record) -> String {
        let raw_value = record.get(&self.field_name).map(|v| v.to_string_value()).unwrap_or_default();
        let value = if raw_value.is_empty() {
            self.default_value.clone().unwrap_or_default()
        } else {
            self.apply_format(&raw_value)
        };
        let mut result = String::new();
        if let Some(ref prefix) = self.prefix { result.push_str(prefix); }
        result.push_str(&value);
        if let Some(ref suffix) = self.suffix { result.push_str(suffix); }
        result
    }

    fn apply_format(&self, value: &str) -> String {
        match self.format.as_deref() {
            Some("upper") | Some("UPPER") => value.to_uppercase(),
            Some("lower") | Some("LOWER") => value.to_lowercase(),
            Some("title") | Some("TITLE") => title_case(value),
            Some("trim") | Some("TRIM") => value.trim().to_string(),
            _ => value.to_string(),
        }
    }
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => { let u: String = c.to_uppercase().collect(); format!("{}{}", u, chars.as_str().to_lowercase()) }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonOperator {
    Equal, NotEqual, LessThan, LessThanOrEqual, GreaterThan, GreaterThanOrEqual,
    Contains, NotContains, StartsWith, EndsWith, IsEmpty, IsNotEmpty,
}

impl ComparisonOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Equal => "==", Self::NotEqual => "!=", Self::LessThan => "<",
            Self::LessThanOrEqual => "<=", Self::GreaterThan => ">", Self::GreaterThanOrEqual => ">=",
            Self::Contains => "contains", Self::NotContains => "not_contains",
            Self::StartsWith => "starts_with", Self::EndsWith => "ends_with",
            Self::IsEmpty => "is_empty", Self::IsNotEmpty => "is_not_empty",
        }
    }

    pub fn from_str(s: &str) -> Option<ComparisonOperator> {
        match s.trim() {
            "==" | "=" | "equal" => Some(Self::Equal),
            "!=" | "<>" | "not_equal" => Some(Self::NotEqual),
            "<" | "less_than" => Some(Self::LessThan),
            "<=" | "less_than_or_equal" => Some(Self::LessThanOrEqual),
            ">" | "greater_than" => Some(Self::GreaterThan),
            ">=" | "greater_than_or_equal" => Some(Self::GreaterThanOrEqual),
            "contains" => Some(Self::Contains), "not_contains" => Some(Self::NotContains),
            "starts_with" => Some(Self::StartsWith), "ends_with" => Some(Self::EndsWith),
            "is_empty" => Some(Self::IsEmpty), "is_not_empty" => Some(Self::IsNotEmpty),
            _ => None,
        }
    }

    pub fn evaluate(&self, left: &str, right: &str) -> bool {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
            Self::LessThan => compare_values(left, right) == std::cmp::Ordering::Less,
            Self::LessThanOrEqual => { let o = compare_values(left, right); o == std::cmp::Ordering::Less || o == std::cmp::Ordering::Equal }
            Self::GreaterThan => compare_values(left, right) == std::cmp::Ordering::Greater,
            Self::GreaterThanOrEqual => { let o = compare_values(left, right); o == std::cmp::Ordering::Greater || o == std::cmp::Ordering::Equal }
            Self::Contains => left.contains(right),
            Self::NotContains => !left.contains(right),
            Self::StartsWith => left.starts_with(right),
            Self::EndsWith => left.ends_with(right),
            Self::IsEmpty => left.is_empty(),
            Self::IsNotEmpty => !left.is_empty(),
        }
    }
}

fn compare_values(left: &str, right: &str) -> std::cmp::Ordering {
    if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
        return l.partial_cmp(&r).unwrap_or(std::cmp::Ordering::Equal);
    }
    left.cmp(right)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalField {
    pub field_name: String,
    pub operator: ComparisonOperator,
    pub compare_value: String,
    pub true_text: Option<String>,
    pub false_text: Option<String>,
}

impl ConditionalField {
    pub fn new(field_name: impl Into<String>, operator: ComparisonOperator, compare_value: impl Into<String>) -> Self {
        Self { field_name: field_name.into(), operator, compare_value: compare_value.into(), true_text: None, false_text: None }
    }
    pub fn with_true_text(mut self, text: impl Into<String>) -> Self { self.true_text = Some(text.into()); self }
    pub fn with_false_text(mut self, text: impl Into<String>) -> Self { self.false_text = Some(text.into()); self }
    pub fn evaluate(&self, record: &Record) -> bool {
        let fv = record.get(&self.field_name).map(|v| v.to_string_value()).unwrap_or_default();
        self.operator.evaluate(&fv, &self.compare_value)
    }
    pub fn resolve(&self, record: &Record) -> String {
        if self.evaluate(record) { self.true_text.clone().unwrap_or_default() }
        else { self.false_text.clone().unwrap_or_default() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MergeFieldInstruction {
    Field(MergeField),
    SkipIf(ConditionalField),
    Next,
    NextIf(ConditionalField),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_source::Value;

    fn sample_record() -> Record {
        let mut r = Record::new();
        r.insert("first_name".into(), Value::Text("John".into()));
        r.insert("last_name".into(), Value::Text("Doe".into()));
        r.insert("amount".into(), Value::Number(100.0));
        r.insert("city".into(), Value::Text("Boston".into()));
        r
    }

    #[test] fn test_basic() { assert_eq!(MergeField::new("first_name").resolve(&sample_record()), "John"); }
    #[test] fn test_missing() { assert_eq!(MergeField::new("email").resolve(&sample_record()), ""); }
    #[test] fn test_default() { assert_eq!(MergeField::new("email").with_default("N/A").resolve(&sample_record()), "N/A"); }
    #[test] fn test_cmp_eq() { assert!(ComparisonOperator::Equal.evaluate("a", "a")); }
    #[test] fn test_cmp_num() { assert!(ComparisonOperator::GreaterThan.evaluate("100", "50")); }
    #[test] fn test_cond_eval() { assert!(ConditionalField::new("city", ComparisonOperator::Equal, "Boston").evaluate(&sample_record())); }
    #[test] fn test_cond_resolve() {
        let c = ConditionalField::new("amount", ComparisonOperator::GreaterThan, "50").with_true_text("High").with_false_text("Low");
        assert_eq!(c.resolve(&sample_record()), "High");
    }
}

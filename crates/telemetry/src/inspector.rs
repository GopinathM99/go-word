//! Document Inspector Module
//!
//! Provides document inspection capabilities for debugging and diagnostics.
//! Generates tree structures showing document hierarchy, properties, and CRDT state.
//!
//! # Example
//!
//! ```rust
//! use telemetry::inspector::{DocumentInspector, InspectorNode};
//!
//! // Create inspector and examine document structure
//! let inspector = DocumentInspector::new();
//!
//! // Build a node tree from document data
//! let mut root = InspectorNode::new("document", "Document");
//! root.set_property("version", "1.0");
//! root.add_child(InspectorNode::new("paragraph", "Paragraph"));
//!
//! let tree = inspector.get_node_tree(&root);
//! println!("{}", tree);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// =============================================================================
// Inspector Node
// =============================================================================

/// A node in the document inspection tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorNode {
    /// Unique identifier for this node
    pub id: String,
    /// Type of node (e.g., "paragraph", "run", "table")
    pub node_type: String,
    /// Display name/label
    pub name: String,
    /// Node properties
    pub properties: HashMap<String, String>,
    /// Child nodes
    pub children: Vec<InspectorNode>,
    /// CRDT state information (if applicable)
    pub crdt_state: Option<CrdtState>,
    /// Whether this node is expanded in UI
    pub expanded: bool,
    /// Whether this node is selected in UI
    pub selected: bool,
}

impl InspectorNode {
    /// Create a new inspector node.
    pub fn new(node_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            node_type: node_type.into(),
            name: name.into(),
            properties: HashMap::new(),
            children: Vec::new(),
            crdt_state: None,
            expanded: false,
            selected: false,
        }
    }

    /// Create a node with a specific ID.
    pub fn with_id(id: impl Into<String>, node_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            node_type: node_type.into(),
            name: name.into(),
            properties: HashMap::new(),
            children: Vec::new(),
            crdt_state: None,
            expanded: false,
            selected: false,
        }
    }

    /// Set a property on this node.
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    /// Get a property value.
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: InspectorNode) {
        self.children.push(child);
    }

    /// Add multiple children.
    pub fn add_children(&mut self, children: Vec<InspectorNode>) {
        self.children.extend(children);
    }

    /// Set CRDT state.
    pub fn set_crdt_state(&mut self, state: CrdtState) {
        self.crdt_state = Some(state);
    }

    /// Get total node count including children.
    pub fn total_nodes(&self) -> usize {
        1 + self.children.iter().map(|c| c.total_nodes()).sum::<usize>()
    }

    /// Get depth of this subtree.
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Find a node by ID.
    pub fn find_by_id(&self, id: &str) -> Option<&InspectorNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &self.children {
            if let Some(found) = child.find_by_id(id) {
                return Some(found);
            }
        }
        None
    }

    /// Find a node by ID (mutable).
    pub fn find_by_id_mut(&mut self, id: &str) -> Option<&mut InspectorNode> {
        if self.id == id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_by_id_mut(id) {
                return Some(found);
            }
        }
        None
    }

    /// Find nodes by type.
    pub fn find_by_type(&self, node_type: &str) -> Vec<&InspectorNode> {
        let mut results = Vec::new();
        if self.node_type == node_type {
            results.push(self);
        }
        for child in &self.children {
            results.extend(child.find_by_type(node_type));
        }
        results
    }

    /// Get path to a node by ID.
    pub fn path_to(&self, id: &str) -> Option<Vec<String>> {
        if self.id == id {
            return Some(vec![self.id.clone()]);
        }
        for child in &self.children {
            if let Some(mut path) = child.path_to(id) {
                path.insert(0, self.id.clone());
                return Some(path);
            }
        }
        None
    }

    /// Builder: with property.
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.set_property(key, value);
        self
    }

    /// Builder: with child.
    pub fn with_child(mut self, child: InspectorNode) -> Self {
        self.add_child(child);
        self
    }

    /// Builder: with CRDT state.
    pub fn with_crdt_state(mut self, state: CrdtState) -> Self {
        self.crdt_state = Some(state);
        self
    }

    /// Builder: set expanded.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

impl fmt::Display for InspectorNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.node_type)
    }
}

// =============================================================================
// CRDT State
// =============================================================================

/// CRDT state information for collaborative editing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtState {
    /// Lamport timestamp
    pub lamport_clock: u64,
    /// Site/replica identifier
    pub site_id: String,
    /// Version vector entries
    pub version_vector: HashMap<String, u64>,
    /// Whether there are pending operations
    pub pending_ops: usize,
    /// Last sync timestamp
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    /// Conflict state
    pub has_conflicts: bool,
}

impl CrdtState {
    /// Create new CRDT state.
    pub fn new(site_id: impl Into<String>) -> Self {
        Self {
            lamport_clock: 0,
            site_id: site_id.into(),
            version_vector: HashMap::new(),
            pending_ops: 0,
            last_sync: None,
            has_conflicts: false,
        }
    }

    /// Update the version for a site.
    pub fn update_version(&mut self, site_id: impl Into<String>, version: u64) {
        let site = site_id.into();
        self.version_vector.insert(site, version);
        self.lamport_clock = self.lamport_clock.max(version) + 1;
    }

    /// Mark as synced now.
    pub fn mark_synced(&mut self) {
        self.last_sync = Some(chrono::Utc::now());
        self.pending_ops = 0;
    }
}

// =============================================================================
// Document Inspector
// =============================================================================

/// Document inspector for examining document structure and state.
#[derive(Debug, Clone)]
pub struct DocumentInspector {
    /// Filter configuration
    filter: InspectorFilter,
    /// Whether to include CRDT state
    include_crdt: bool,
    /// Maximum depth to inspect
    max_depth: usize,
}

impl Default for DocumentInspector {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentInspector {
    /// Create a new document inspector.
    pub fn new() -> Self {
        Self {
            filter: InspectorFilter::default(),
            include_crdt: true,
            max_depth: 100,
        }
    }

    /// Create an inspector with custom settings.
    pub fn with_settings(include_crdt: bool, max_depth: usize) -> Self {
        Self {
            include_crdt,
            max_depth,
            ..Self::new()
        }
    }

    /// Set the filter.
    pub fn set_filter(&mut self, filter: InspectorFilter) {
        self.filter = filter;
    }

    /// Get the current filter.
    pub fn filter(&self) -> &InspectorFilter {
        &self.filter
    }

    /// Generate a tree representation of a node hierarchy.
    pub fn get_node_tree(&self, root: &InspectorNode) -> String {
        let mut output = String::new();
        self.format_tree_node(root, "", true, &mut output, 0);
        output
    }

    fn format_tree_node(
        &self,
        node: &InspectorNode,
        prefix: &str,
        is_last: bool,
        output: &mut String,
        depth: usize,
    ) {
        if depth > self.max_depth {
            return;
        }

        // Apply filter
        if !self.filter.should_include(node) {
            return;
        }

        let connector = if is_last { "+-- " } else { "|-- " };
        let child_prefix = if is_last { "    " } else { "|   " };

        output.push_str(prefix);
        output.push_str(connector);
        output.push_str(&format!("{} [{}]", node.name, node.node_type));

        // Add property summary
        if !node.properties.is_empty() && self.filter.include_properties {
            let props: Vec<String> = node
                .properties
                .iter()
                .take(3)
                .map(|(k, v)| format!("{}={}", k, truncate(v, 20)))
                .collect();
            output.push_str(&format!(" {{{}}}", props.join(", ")));
        }

        // Add CRDT indicator
        if self.include_crdt {
            if let Some(ref crdt) = node.crdt_state {
                output.push_str(&format!(" (clock: {}", crdt.lamport_clock));
                if crdt.has_conflicts {
                    output.push_str(", CONFLICT");
                }
                output.push(')');
            }
        }

        output.push('\n');

        // Process children
        let filtered_children: Vec<&InspectorNode> = node
            .children
            .iter()
            .filter(|c| self.filter.should_include(c))
            .collect();

        for (i, child) in filtered_children.iter().enumerate() {
            let is_last_child = i == filtered_children.len() - 1;
            let new_prefix = format!("{}{}", prefix, child_prefix);
            self.format_tree_node(child, &new_prefix, is_last_child, output, depth + 1);
        }
    }

    /// Get properties for a specific node.
    pub fn get_node_properties(&self, node: &InspectorNode) -> NodeProperties {
        NodeProperties {
            id: node.id.clone(),
            node_type: node.node_type.clone(),
            name: node.name.clone(),
            properties: node.properties.clone(),
            crdt_state: if self.include_crdt {
                node.crdt_state.clone()
            } else {
                None
            },
            child_count: node.children.len(),
            total_descendants: node.total_nodes() - 1,
            depth: node.depth(),
        }
    }

    /// Inspect a document and return the root node.
    ///
    /// This is a placeholder that should be integrated with the actual document model.
    pub fn inspect_document(&self, document_data: &DocumentData) -> InspectorNode {
        let mut root = InspectorNode::new("document", &document_data.title);
        root.set_property("format", &document_data.format);
        root.set_property("page_count", &document_data.page_count.to_string());

        if document_data.is_collaborative {
            root.set_crdt_state(CrdtState::new("local"));
        }

        // Add body content
        let mut body = InspectorNode::new("body", "Body");
        for (i, section) in document_data.sections.iter().enumerate() {
            let section_node = InspectorNode::new("section", &format!("Section {}", i + 1))
                .with_property("content_preview", truncate(section, 50));
            body.add_child(section_node);
        }
        root.add_child(body);

        // Add styles
        if !document_data.styles.is_empty() {
            let mut styles = InspectorNode::new("styles", "Styles");
            for style in &document_data.styles {
                styles.add_child(InspectorNode::new("style", style));
            }
            root.add_child(styles);
        }

        root
    }

    /// Generate a summary of the document structure.
    pub fn summarize(&self, root: &InspectorNode) -> DocumentSummary {
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        count_types(root, &mut type_counts);

        let crdt_nodes = count_crdt_nodes(root);
        let conflict_nodes = count_conflict_nodes(root);

        DocumentSummary {
            total_nodes: root.total_nodes(),
            max_depth: root.depth(),
            type_counts,
            has_crdt: crdt_nodes > 0,
            crdt_node_count: crdt_nodes,
            conflict_count: conflict_nodes,
        }
    }

    /// Export the node tree to JSON.
    pub fn export_to_json(&self, root: &InspectorNode) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(root)
    }

    /// Create a filtered view of the node tree.
    pub fn filter_tree(&self, root: &InspectorNode) -> Option<InspectorNode> {
        self.filter_node_recursive(root, 0)
    }

    fn filter_node_recursive(&self, node: &InspectorNode, depth: usize) -> Option<InspectorNode> {
        if depth > self.max_depth || !self.filter.should_include(node) {
            return None;
        }

        let mut filtered = node.clone();
        filtered.children = node
            .children
            .iter()
            .filter_map(|c| self.filter_node_recursive(c, depth + 1))
            .collect();

        Some(filtered)
    }
}

fn count_types(node: &InspectorNode, counts: &mut HashMap<String, usize>) {
    *counts.entry(node.node_type.clone()).or_insert(0) += 1;
    for child in &node.children {
        count_types(child, counts);
    }
}

fn count_crdt_nodes(node: &InspectorNode) -> usize {
    let self_count = if node.crdt_state.is_some() { 1 } else { 0 };
    self_count + node.children.iter().map(count_crdt_nodes).sum::<usize>()
}

fn count_conflict_nodes(node: &InspectorNode) -> usize {
    let self_count = if node.crdt_state.as_ref().map_or(false, |c| c.has_conflicts) {
        1
    } else {
        0
    };
    self_count + node.children.iter().map(count_conflict_nodes).sum::<usize>()
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

// =============================================================================
// Inspector Filter
// =============================================================================

/// Filter configuration for document inspection.
#[derive(Debug, Clone, Default)]
pub struct InspectorFilter {
    /// Only include these node types (empty = all)
    pub include_types: Vec<String>,
    /// Exclude these node types
    pub exclude_types: Vec<String>,
    /// Include properties in output
    pub include_properties: bool,
    /// Only show nodes with CRDT state
    pub crdt_only: bool,
    /// Only show nodes with conflicts
    pub conflicts_only: bool,
    /// Text search filter
    pub search_text: Option<String>,
}

impl InspectorFilter {
    /// Create a new filter.
    pub fn new() -> Self {
        Self {
            include_properties: true,
            ..Default::default()
        }
    }

    /// Filter to specific node types.
    pub fn with_types(mut self, types: Vec<String>) -> Self {
        self.include_types = types;
        self
    }

    /// Exclude specific node types.
    pub fn excluding_types(mut self, types: Vec<String>) -> Self {
        self.exclude_types = types;
        self
    }

    /// Only show CRDT nodes.
    pub fn crdt_only(mut self) -> Self {
        self.crdt_only = true;
        self
    }

    /// Only show conflict nodes.
    pub fn conflicts_only(mut self) -> Self {
        self.conflicts_only = true;
        self
    }

    /// Set search text filter.
    pub fn with_search(mut self, text: impl Into<String>) -> Self {
        self.search_text = Some(text.into());
        self
    }

    /// Check if a node should be included.
    pub fn should_include(&self, node: &InspectorNode) -> bool {
        // Check type filters
        if !self.include_types.is_empty() && !self.include_types.contains(&node.node_type) {
            return false;
        }
        if self.exclude_types.contains(&node.node_type) {
            return false;
        }

        // Check CRDT filter
        if self.crdt_only && node.crdt_state.is_none() {
            return false;
        }

        // Check conflicts filter
        if self.conflicts_only {
            if node.crdt_state.as_ref().map_or(true, |c| !c.has_conflicts) {
                return false;
            }
        }

        // Check text search
        if let Some(ref search) = self.search_text {
            let search_lower = search.to_lowercase();
            let matches_name = node.name.to_lowercase().contains(&search_lower);
            let matches_type = node.node_type.to_lowercase().contains(&search_lower);
            let matches_props = node
                .properties
                .values()
                .any(|v| v.to_lowercase().contains(&search_lower));
            if !matches_name && !matches_type && !matches_props {
                return false;
            }
        }

        true
    }
}

// =============================================================================
// Supporting Types
// =============================================================================

/// Properties of an inspected node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeProperties {
    /// Node ID
    pub id: String,
    /// Node type
    pub node_type: String,
    /// Node name
    pub name: String,
    /// All properties
    pub properties: HashMap<String, String>,
    /// CRDT state
    pub crdt_state: Option<CrdtState>,
    /// Number of direct children
    pub child_count: usize,
    /// Total descendants
    pub total_descendants: usize,
    /// Subtree depth
    pub depth: usize,
}

/// Summary of document structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Maximum tree depth
    pub max_depth: usize,
    /// Count of each node type
    pub type_counts: HashMap<String, usize>,
    /// Whether document has CRDT state
    pub has_crdt: bool,
    /// Number of nodes with CRDT state
    pub crdt_node_count: usize,
    /// Number of nodes with conflicts
    pub conflict_count: usize,
}

/// Placeholder document data for inspection.
#[derive(Debug, Clone)]
pub struct DocumentData {
    /// Document title
    pub title: String,
    /// File format
    pub format: String,
    /// Number of pages
    pub page_count: u32,
    /// Whether collaborative editing is enabled
    pub is_collaborative: bool,
    /// Section content (simplified)
    pub sections: Vec<String>,
    /// Style names
    pub styles: Vec<String>,
}

impl DocumentData {
    /// Create sample document data for testing.
    pub fn sample() -> Self {
        Self {
            title: "Sample Document".to_string(),
            format: "docx".to_string(),
            page_count: 5,
            is_collaborative: true,
            sections: vec![
                "Introduction section content...".to_string(),
                "Main body content with detailed text...".to_string(),
                "Conclusion summarizing the document...".to_string(),
            ],
            styles: vec![
                "Normal".to_string(),
                "Heading 1".to_string(),
                "Heading 2".to_string(),
            ],
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_tree() -> InspectorNode {
        InspectorNode::new("document", "Test Document")
            .with_property("version", "1.0")
            .with_child(
                InspectorNode::new("paragraph", "Paragraph 1")
                    .with_property("style", "Normal")
                    .with_child(InspectorNode::new("run", "Run 1"))
                    .with_child(InspectorNode::new("run", "Run 2")),
            )
            .with_child(
                InspectorNode::new("paragraph", "Paragraph 2")
                    .with_child(InspectorNode::new("run", "Run 3")),
            )
    }

    #[test]
    fn test_inspector_node_new() {
        let node = InspectorNode::new("paragraph", "Test Paragraph");
        assert_eq!(node.node_type, "paragraph");
        assert_eq!(node.name, "Test Paragraph");
        assert!(!node.id.is_empty());
    }

    #[test]
    fn test_inspector_node_with_id() {
        let node = InspectorNode::with_id("custom-id", "paragraph", "Test");
        assert_eq!(node.id, "custom-id");
    }

    #[test]
    fn test_inspector_node_properties() {
        let mut node = InspectorNode::new("test", "Test");
        node.set_property("key1", "value1");
        node.set_property("key2", "value2");

        assert_eq!(node.get_property("key1"), Some(&"value1".to_string()));
        assert_eq!(node.get_property("key2"), Some(&"value2".to_string()));
        assert_eq!(node.get_property("key3"), None);
    }

    #[test]
    fn test_inspector_node_children() {
        let mut parent = InspectorNode::new("parent", "Parent");
        parent.add_child(InspectorNode::new("child", "Child 1"));
        parent.add_child(InspectorNode::new("child", "Child 2"));

        assert_eq!(parent.children.len(), 2);
    }

    #[test]
    fn test_inspector_node_total_nodes() {
        let tree = make_test_tree();
        // document + 2 paragraphs + 3 runs = 6
        assert_eq!(tree.total_nodes(), 6);
    }

    #[test]
    fn test_inspector_node_depth() {
        let tree = make_test_tree();
        // document -> paragraph -> run = depth 3
        assert_eq!(tree.depth(), 3);
    }

    #[test]
    fn test_inspector_node_find_by_id() {
        let tree = make_test_tree();
        let target_id = tree.children[0].children[0].id.clone();

        let found = tree.find_by_id(&target_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Run 1");
    }

    #[test]
    fn test_inspector_node_find_by_type() {
        let tree = make_test_tree();
        let runs = tree.find_by_type("run");
        assert_eq!(runs.len(), 3);
    }

    #[test]
    fn test_inspector_node_path_to() {
        let tree = make_test_tree();
        let target_id = tree.children[0].children[1].id.clone();

        let path = tree.path_to(&target_id).unwrap();
        assert_eq!(path.len(), 3);
    }

    #[test]
    fn test_crdt_state_new() {
        let state = CrdtState::new("site-1");
        assert_eq!(state.site_id, "site-1");
        assert_eq!(state.lamport_clock, 0);
        assert!(!state.has_conflicts);
    }

    #[test]
    fn test_crdt_state_update_version() {
        let mut state = CrdtState::new("site-1");
        state.update_version("site-2", 5);
        state.update_version("site-3", 10);

        assert!(state.lamport_clock > 10);
        assert_eq!(state.version_vector.get("site-2"), Some(&5));
        assert_eq!(state.version_vector.get("site-3"), Some(&10));
    }

    #[test]
    fn test_document_inspector_new() {
        let inspector = DocumentInspector::new();
        assert!(inspector.include_crdt);
        assert_eq!(inspector.max_depth, 100);
    }

    #[test]
    fn test_document_inspector_get_node_tree() {
        let tree = make_test_tree();
        let inspector = DocumentInspector::new();

        let output = inspector.get_node_tree(&tree);
        assert!(output.contains("Test Document"));
        assert!(output.contains("Paragraph 1"));
        assert!(output.contains("Run 1"));
    }

    #[test]
    fn test_document_inspector_get_node_properties() {
        let tree = make_test_tree();
        let inspector = DocumentInspector::new();

        let props = inspector.get_node_properties(&tree);
        assert_eq!(props.node_type, "document");
        assert_eq!(props.child_count, 2);
        assert_eq!(props.total_descendants, 5);
    }

    #[test]
    fn test_document_inspector_summarize() {
        let tree = make_test_tree();
        let inspector = DocumentInspector::new();

        let summary = inspector.summarize(&tree);
        assert_eq!(summary.total_nodes, 6);
        assert_eq!(summary.max_depth, 3);
        assert_eq!(summary.type_counts.get("run"), Some(&3));
    }

    #[test]
    fn test_document_inspector_inspect_document() {
        let data = DocumentData::sample();
        let inspector = DocumentInspector::new();

        let root = inspector.inspect_document(&data);
        assert_eq!(root.node_type, "document");
        assert_eq!(root.name, "Sample Document");
        assert!(root.crdt_state.is_some());
    }

    #[test]
    fn test_document_inspector_export_json() {
        let tree = make_test_tree();
        let inspector = DocumentInspector::new();

        let json = inspector.export_to_json(&tree).unwrap();
        assert!(json.contains("Test Document"));
        assert!(json.contains("paragraph"));
    }

    #[test]
    fn test_inspector_filter_include_types() {
        let tree = make_test_tree();
        let filter = InspectorFilter::new().with_types(vec!["paragraph".to_string()]);

        assert!(filter.should_include(&tree.children[0])); // paragraph
        assert!(!filter.should_include(&tree.children[0].children[0])); // run
    }

    #[test]
    fn test_inspector_filter_exclude_types() {
        let tree = make_test_tree();
        let filter = InspectorFilter::new().excluding_types(vec!["run".to_string()]);

        assert!(filter.should_include(&tree.children[0])); // paragraph
        assert!(!filter.should_include(&tree.children[0].children[0])); // run
    }

    #[test]
    fn test_inspector_filter_search() {
        let mut tree = make_test_tree();
        tree.children[0].set_property("content", "hello world");

        let filter = InspectorFilter::new().with_search("hello");
        assert!(filter.should_include(&tree.children[0]));
        assert!(!filter.should_include(&tree.children[1]));
    }

    #[test]
    fn test_inspector_filter_crdt_only() {
        let mut node_with_crdt = InspectorNode::new("test", "With CRDT");
        node_with_crdt.set_crdt_state(CrdtState::new("site-1"));

        let node_without_crdt = InspectorNode::new("test", "Without CRDT");

        let filter = InspectorFilter::new().crdt_only();
        assert!(filter.should_include(&node_with_crdt));
        assert!(!filter.should_include(&node_without_crdt));
    }

    #[test]
    fn test_inspector_filter_conflicts_only() {
        let mut node_with_conflict = InspectorNode::new("test", "With Conflict");
        let mut crdt = CrdtState::new("site-1");
        crdt.has_conflicts = true;
        node_with_conflict.set_crdt_state(crdt);

        let mut node_no_conflict = InspectorNode::new("test", "No Conflict");
        node_no_conflict.set_crdt_state(CrdtState::new("site-2"));

        let filter = InspectorFilter::new().conflicts_only();
        assert!(filter.should_include(&node_with_conflict));
        assert!(!filter.should_include(&node_no_conflict));
    }

    #[test]
    fn test_document_inspector_filter_tree() {
        let tree = make_test_tree();
        let mut inspector = DocumentInspector::new();
        inspector.set_filter(InspectorFilter::new().excluding_types(vec!["run".to_string()]));

        let filtered = inspector.filter_tree(&tree).unwrap();
        assert_eq!(filtered.total_nodes(), 3); // document + 2 paragraphs
    }

    #[test]
    fn test_inspector_node_serialization() {
        let node = InspectorNode::new("paragraph", "Test")
            .with_property("key", "value");

        let json = serde_json::to_string(&node).unwrap();
        let deserialized: InspectorNode = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.node_type, "paragraph");
        assert_eq!(deserialized.name, "Test");
    }

    #[test]
    fn test_document_summary_serialization() {
        let tree = make_test_tree();
        let inspector = DocumentInspector::new();
        let summary = inspector.summarize(&tree);

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: DocumentSummary = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_nodes, 6);
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
    }
}

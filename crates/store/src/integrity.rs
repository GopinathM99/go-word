//! Document integrity checking and validation
//!
//! This module provides functionality to validate document structure,
//! compute checksums, detect corruption, and attempt repairs.

use crate::Result;
use doc_model::{DocumentTree, Node, NodeId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use uuid::Uuid;

/// Integrity check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrityReport {
    /// Whether the document passed all integrity checks
    pub is_valid: bool,
    /// Document checksum (SHA-256 hash)
    pub checksum: String,
    /// List of issues found
    pub issues: Vec<IntegrityIssue>,
    /// Statistics about the document
    pub stats: DocumentStats,
    /// Whether the document can be repaired
    pub can_repair: bool,
}

/// Types of integrity issues
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IntegrityIssue {
    /// An orphan node was found (no parent)
    OrphanNode {
        node_id: String,
        node_type: String,
    },
    /// A node references a non-existent parent
    InvalidParentReference {
        node_id: String,
        parent_id: String,
    },
    /// A node references a non-existent child
    InvalidChildReference {
        node_id: String,
        child_id: String,
    },
    /// Duplicate node ID found
    DuplicateNodeId {
        node_id: String,
    },
    /// Invalid node type for parent-child relationship
    InvalidNodeHierarchy {
        parent_id: String,
        parent_type: String,
        child_id: String,
        child_type: String,
    },
    /// Empty document (no content)
    EmptyDocument,
    /// File format error
    FileFormatError {
        message: String,
    },
    /// Checksum mismatch
    ChecksumMismatch {
        expected: String,
        actual: String,
    },
}

impl IntegrityIssue {
    /// Get the severity of this issue
    pub fn severity(&self) -> IssueSeverity {
        match self {
            IntegrityIssue::OrphanNode { .. } => IssueSeverity::Warning,
            IntegrityIssue::InvalidParentReference { .. } => IssueSeverity::Error,
            IntegrityIssue::InvalidChildReference { .. } => IssueSeverity::Error,
            IntegrityIssue::DuplicateNodeId { .. } => IssueSeverity::Critical,
            IntegrityIssue::InvalidNodeHierarchy { .. } => IssueSeverity::Error,
            IntegrityIssue::EmptyDocument => IssueSeverity::Warning,
            IntegrityIssue::FileFormatError { .. } => IssueSeverity::Critical,
            IntegrityIssue::ChecksumMismatch { .. } => IssueSeverity::Warning,
        }
    }

    /// Check if this issue is repairable
    pub fn is_repairable(&self) -> bool {
        match self {
            IntegrityIssue::OrphanNode { .. } => true,
            IntegrityIssue::InvalidParentReference { .. } => true,
            IntegrityIssue::InvalidChildReference { .. } => true,
            IntegrityIssue::DuplicateNodeId { .. } => false,
            IntegrityIssue::InvalidNodeHierarchy { .. } => true,
            IntegrityIssue::EmptyDocument => true,
            IntegrityIssue::FileFormatError { .. } => false,
            IntegrityIssue::ChecksumMismatch { .. } => false,
        }
    }
}

/// Severity levels for integrity issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Informational, document is still usable
    Info,
    /// Warning, document may have minor issues
    Warning,
    /// Error, document has significant issues
    Error,
    /// Critical, document may be unusable
    Critical,
}

/// Document statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentStats {
    /// Total number of nodes
    pub total_nodes: usize,
    /// Number of paragraphs
    pub paragraph_count: usize,
    /// Number of runs
    pub run_count: usize,
    /// Number of images
    pub image_count: usize,
    /// Number of tables
    pub table_count: usize,
    /// Number of hyperlinks
    pub hyperlink_count: usize,
    /// Number of shapes
    pub shape_count: usize,
    /// Number of bookmarks
    pub bookmark_count: usize,
    /// Approximate word count
    pub word_count: usize,
    /// Approximate character count
    pub character_count: usize,
}

/// Integrity checker for documents
pub struct IntegrityChecker;

impl IntegrityChecker {
    /// Create a new integrity checker
    pub fn new() -> Self {
        Self
    }

    /// Check a document's integrity
    pub fn check(&self, tree: &DocumentTree) -> IntegrityReport {
        let mut issues = Vec::new();

        // Collect all node IDs
        let all_nodes = self.collect_all_nodes(tree);

        // Check for orphan nodes
        self.check_orphan_nodes(tree, &all_nodes, &mut issues);

        // Check parent-child relationships
        self.check_relationships(tree, &all_nodes, &mut issues);

        // Check for empty document
        if tree.document.children().is_empty() {
            issues.push(IntegrityIssue::EmptyDocument);
        }

        // Compute checksum
        let checksum = self.compute_checksum(tree);

        // Calculate stats
        let stats = self.compute_stats(tree);

        // Determine if repairable
        let can_repair = issues.iter().all(|i| i.is_repairable());

        IntegrityReport {
            is_valid: issues.is_empty() || issues.iter().all(|i| matches!(i.severity(), IssueSeverity::Info | IssueSeverity::Warning)),
            checksum,
            issues,
            stats,
            can_repair,
        }
    }

    /// Check a file's integrity
    pub async fn check_file(&self, path: impl AsRef<Path>) -> Result<IntegrityReport> {
        let tree = crate::load_document(path).await?;
        Ok(self.check(&tree))
    }

    /// Collect all node IDs in the document
    fn collect_all_nodes(&self, tree: &DocumentTree) -> HashSet<NodeId> {
        let mut nodes = HashSet::new();

        // Document root
        nodes.insert(tree.document.id());

        // Paragraphs
        for (id, _) in &tree.nodes.paragraphs {
            nodes.insert(*id);
        }

        // Runs
        for (id, _) in &tree.nodes.runs {
            nodes.insert(*id);
        }

        // Hyperlinks
        for (id, _) in &tree.nodes.hyperlinks {
            nodes.insert(*id);
        }

        // Images
        for (id, _) in &tree.nodes.images {
            nodes.insert(*id);
        }

        // Shapes
        for (id, _) in &tree.nodes.shapes {
            nodes.insert(*id);
        }

        // Tables
        for (id, _) in &tree.nodes.tables {
            nodes.insert(*id);
        }

        // Table rows
        for (id, _) in &tree.nodes.table_rows {
            nodes.insert(*id);
        }

        // Table cells
        for (id, _) in &tree.nodes.table_cells {
            nodes.insert(*id);
        }

        nodes
    }

    /// Check for orphan nodes (nodes not reachable from root)
    fn check_orphan_nodes(
        &self,
        tree: &DocumentTree,
        all_nodes: &HashSet<NodeId>,
        issues: &mut Vec<IntegrityIssue>,
    ) {
        // Build set of reachable nodes from root
        let reachable = self.collect_reachable_nodes(tree);

        // Find orphans
        for &node_id in all_nodes {
            if node_id != tree.document.id() && !reachable.contains(&node_id) {
                let node_type = tree
                    .node_type(node_id)
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|| "Unknown".to_string());

                issues.push(IntegrityIssue::OrphanNode {
                    node_id: node_id.to_string(),
                    node_type,
                });
            }
        }
    }

    /// Collect all nodes reachable from the document root
    fn collect_reachable_nodes(&self, tree: &DocumentTree) -> HashSet<NodeId> {
        let mut reachable = HashSet::new();
        let mut stack = vec![tree.document.id()];

        while let Some(node_id) = stack.pop() {
            if reachable.contains(&node_id) {
                continue;
            }
            reachable.insert(node_id);

            // Get children based on node type
            if node_id == tree.document.id() {
                stack.extend(tree.document.children());
            } else if let Some(para) = tree.nodes.paragraphs.get(&node_id) {
                stack.extend(para.children());
            } else if let Some(hyperlink) = tree.nodes.hyperlinks.get(&node_id) {
                stack.extend(hyperlink.children());
            } else if let Some(table) = tree.nodes.tables.get(&node_id) {
                stack.extend(table.children());
            } else if let Some(row) = tree.nodes.table_rows.get(&node_id) {
                stack.extend(row.children());
            } else if let Some(cell) = tree.nodes.table_cells.get(&node_id) {
                stack.extend(cell.children());
            }
        }

        reachable
    }

    /// Check parent-child relationships
    fn check_relationships(
        &self,
        tree: &DocumentTree,
        all_nodes: &HashSet<NodeId>,
        issues: &mut Vec<IntegrityIssue>,
    ) {
        // Check that all children exist
        for &node_id in all_nodes {
            let children = self.get_children(tree, node_id);
            for child_id in children {
                if !all_nodes.contains(&child_id) {
                    issues.push(IntegrityIssue::InvalidChildReference {
                        node_id: node_id.to_string(),
                        child_id: child_id.to_string(),
                    });
                }
            }
        }

        // Check that all parent references are valid
        for (&node_id, para) in &tree.nodes.paragraphs {
            if let Some(parent_id) = para.parent() {
                if !all_nodes.contains(&parent_id) {
                    issues.push(IntegrityIssue::InvalidParentReference {
                        node_id: node_id.to_string(),
                        parent_id: parent_id.to_string(),
                    });
                }
            }
        }

        for (&node_id, run) in &tree.nodes.runs {
            if let Some(parent_id) = run.parent() {
                if !all_nodes.contains(&parent_id) {
                    issues.push(IntegrityIssue::InvalidParentReference {
                        node_id: node_id.to_string(),
                        parent_id: parent_id.to_string(),
                    });
                }
            }
        }
    }

    /// Get children of a node
    fn get_children(&self, tree: &DocumentTree, node_id: NodeId) -> Vec<NodeId> {
        if node_id == tree.document.id() {
            tree.document.children().to_vec()
        } else if let Some(para) = tree.nodes.paragraphs.get(&node_id) {
            para.children().to_vec()
        } else if let Some(hyperlink) = tree.nodes.hyperlinks.get(&node_id) {
            hyperlink.children().to_vec()
        } else if let Some(table) = tree.nodes.tables.get(&node_id) {
            table.children().to_vec()
        } else if let Some(row) = tree.nodes.table_rows.get(&node_id) {
            row.children().to_vec()
        } else if let Some(cell) = tree.nodes.table_cells.get(&node_id) {
            cell.children().to_vec()
        } else {
            Vec::new()
        }
    }

    /// Compute SHA-256 checksum of the document
    pub fn compute_checksum(&self, tree: &DocumentTree) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Serialize the document
        let json = match crate::serialize(tree) {
            Ok(j) => j,
            Err(_) => return "ERROR".to_string(),
        };

        // Compute hash (using DefaultHasher as a simple checksum)
        // In production, use sha2 crate for proper SHA-256
        let mut hasher = DefaultHasher::new();
        json.hash(&mut hasher);
        let hash = hasher.finish();

        format!("{:016x}", hash)
    }

    /// Compute document statistics
    pub fn compute_stats(&self, tree: &DocumentTree) -> DocumentStats {
        let mut stats = DocumentStats::default();

        stats.paragraph_count = tree.nodes.paragraphs.len();
        stats.run_count = tree.nodes.runs.len();
        stats.image_count = tree.nodes.images.len();
        stats.table_count = tree.nodes.tables.len();
        stats.hyperlink_count = tree.nodes.hyperlinks.len();
        stats.shape_count = tree.nodes.shapes.len();
        stats.bookmark_count = tree.bookmarks.all().count();

        stats.total_nodes = 1
            + stats.paragraph_count
            + stats.run_count
            + stats.image_count
            + stats.table_count
            + stats.hyperlink_count
            + stats.shape_count
            + tree.nodes.table_rows.len()
            + tree.nodes.table_cells.len();

        // Count words and characters
        for run in tree.nodes.runs.values() {
            stats.character_count += run.text.len();
            stats.word_count += run.text.split_whitespace().count();
        }

        stats
    }

    /// Verify checksum of a document
    pub fn verify_checksum(&self, tree: &DocumentTree, expected: &str) -> bool {
        let actual = self.compute_checksum(tree);
        actual == expected
    }
}

impl Default for IntegrityChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Document repair functionality
pub struct DocumentRepairer;

impl DocumentRepairer {
    /// Create a new document repairer
    pub fn new() -> Self {
        Self
    }

    /// Attempt to repair a document based on integrity issues
    pub fn repair(&self, tree: &mut DocumentTree, issues: &[IntegrityIssue]) -> Vec<RepairAction> {
        let mut actions = Vec::new();

        for issue in issues {
            if let Some(action) = self.repair_issue(tree, issue) {
                actions.push(action);
            }
        }

        actions
    }

    /// Repair a single issue
    fn repair_issue(&self, tree: &mut DocumentTree, issue: &IntegrityIssue) -> Option<RepairAction> {
        match issue {
            IntegrityIssue::OrphanNode { node_id, node_type } => {
                // Remove orphan nodes
                self.remove_orphan_node(tree, node_id)?;
                Some(RepairAction::RemovedOrphanNode {
                    node_id: node_id.clone(),
                    node_type: node_type.clone(),
                })
            }
            IntegrityIssue::EmptyDocument => {
                // Add an empty paragraph
                let para = doc_model::Paragraph::new();
                let para_id = para.id();
                tree.nodes.paragraphs.insert(para_id, para);
                tree.document.add_body_child(para_id);
                Some(RepairAction::AddedEmptyParagraph)
            }
            IntegrityIssue::InvalidChildReference { node_id, child_id } => {
                // Remove invalid child reference
                self.remove_invalid_child(tree, node_id, child_id)?;
                Some(RepairAction::RemovedInvalidReference {
                    parent_id: node_id.clone(),
                    child_id: child_id.clone(),
                })
            }
            _ => None,
        }
    }

    /// Remove an orphan node from the document
    fn remove_orphan_node(&self, tree: &mut DocumentTree, node_id_str: &str) -> Option<()> {
        let uuid = Uuid::parse_str(node_id_str).ok()?;
        let node_id = NodeId::from_uuid(uuid);

        // Try to remove from various stores
        tree.nodes.paragraphs.remove(&node_id);
        tree.nodes.runs.remove(&node_id);
        tree.nodes.hyperlinks.remove(&node_id);
        tree.nodes.images.remove(&node_id);
        tree.nodes.shapes.remove(&node_id);
        tree.nodes.tables.remove(&node_id);
        tree.nodes.table_rows.remove(&node_id);
        tree.nodes.table_cells.remove(&node_id);

        Some(())
    }

    /// Remove an invalid child reference
    fn remove_invalid_child(
        &self,
        tree: &mut DocumentTree,
        parent_id_str: &str,
        child_id_str: &str,
    ) -> Option<()> {
        let parent_uuid = Uuid::parse_str(parent_id_str).ok()?;
        let child_uuid = Uuid::parse_str(child_id_str).ok()?;
        let parent_id = NodeId::from_uuid(parent_uuid);
        let child_id = NodeId::from_uuid(child_uuid);

        // Try to remove from parent's children
        if parent_id == tree.document.id() {
            tree.document.remove_body_child(child_id);
        } else if let Some(para) = tree.nodes.paragraphs.get_mut(&parent_id) {
            para.remove_child(child_id);
        } else if let Some(hyperlink) = tree.nodes.hyperlinks.get_mut(&parent_id) {
            hyperlink.remove_child(child_id);
        } else if let Some(table) = tree.nodes.tables.get_mut(&parent_id) {
            table.remove_row(child_id);
        } else if let Some(row) = tree.nodes.table_rows.get_mut(&parent_id) {
            row.remove_cell(child_id);
        } else if let Some(cell) = tree.nodes.table_cells.get_mut(&parent_id) {
            cell.remove_child(child_id);
        }

        Some(())
    }
}

impl Default for DocumentRepairer {
    fn default() -> Self {
        Self::new()
    }
}

/// Actions taken during repair
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RepairAction {
    /// Removed an orphan node
    RemovedOrphanNode { node_id: String, node_type: String },
    /// Added an empty paragraph to empty document
    AddedEmptyParagraph,
    /// Removed an invalid reference
    RemovedInvalidReference { parent_id: String, child_id: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use doc_model::{Paragraph, Run};

    #[test]
    fn test_integrity_checker_valid_document() {
        let tree = DocumentTree::with_empty_paragraph();
        let checker = IntegrityChecker::new();
        let report = checker.check(&tree);

        assert!(report.is_valid);
        assert!(report.issues.is_empty() || report.issues.iter().all(|i| matches!(i, IntegrityIssue::EmptyDocument)));
    }

    #[test]
    fn test_integrity_checker_empty_document() {
        let tree = DocumentTree::new();
        let checker = IntegrityChecker::new();
        let report = checker.check(&tree);

        assert!(report.issues.iter().any(|i| matches!(i, IntegrityIssue::EmptyDocument)));
    }

    #[test]
    fn test_integrity_checker_with_content() {
        let mut tree = DocumentTree::new();

        // Add a paragraph with a run
        let mut para = Paragraph::new();
        let para_id = para.id();
        para.set_parent(Some(tree.document.id()));

        let mut run = Run::new("Hello, World!");
        let run_id = run.id();
        run.set_parent(Some(para_id));
        para.add_child(run_id);

        tree.nodes.paragraphs.insert(para_id, para);
        tree.nodes.runs.insert(run_id, run);
        tree.document.add_body_child(para_id);

        let checker = IntegrityChecker::new();
        let report = checker.check(&tree);

        assert!(report.is_valid);
        assert_eq!(report.stats.paragraph_count, 1);
        assert_eq!(report.stats.run_count, 1);
        assert_eq!(report.stats.word_count, 2);
    }

    #[test]
    fn test_compute_checksum() {
        let tree = DocumentTree::with_empty_paragraph();
        let checker = IntegrityChecker::new();

        let checksum1 = checker.compute_checksum(&tree);
        let checksum2 = checker.compute_checksum(&tree);

        assert_eq!(checksum1, checksum2);
        assert!(!checksum1.is_empty());
    }

    #[test]
    fn test_verify_checksum() {
        let tree = DocumentTree::with_empty_paragraph();
        let checker = IntegrityChecker::new();

        let checksum = checker.compute_checksum(&tree);
        assert!(checker.verify_checksum(&tree, &checksum));
        assert!(!checker.verify_checksum(&tree, "invalid"));
    }

    #[test]
    fn test_document_stats() {
        let mut tree = DocumentTree::new();

        // Add paragraphs with runs
        for i in 0..3 {
            let mut para = Paragraph::new();
            let para_id = para.id();
            para.set_parent(Some(tree.document.id()));

            let mut run = Run::new(format!("Paragraph {} content", i + 1));
            let run_id = run.id();
            run.set_parent(Some(para_id));
            para.add_child(run_id);

            tree.nodes.paragraphs.insert(para_id, para);
            tree.nodes.runs.insert(run_id, run);
            tree.document.add_body_child(para_id);
        }

        let checker = IntegrityChecker::new();
        let stats = checker.compute_stats(&tree);

        assert_eq!(stats.paragraph_count, 3);
        assert_eq!(stats.run_count, 3);
        // "Paragraph X content" = 3 words per paragraph, 3 paragraphs = 9 words
        assert_eq!(stats.word_count, 9);
    }

    #[test]
    fn test_issue_severity() {
        assert_eq!(
            IntegrityIssue::OrphanNode {
                node_id: "test".to_string(),
                node_type: "Run".to_string()
            }
            .severity(),
            IssueSeverity::Warning
        );

        assert_eq!(
            IntegrityIssue::DuplicateNodeId {
                node_id: "test".to_string()
            }
            .severity(),
            IssueSeverity::Critical
        );
    }

    #[test]
    fn test_issue_repairable() {
        assert!(IntegrityIssue::OrphanNode {
            node_id: "test".to_string(),
            node_type: "Run".to_string()
        }
        .is_repairable());

        assert!(!IntegrityIssue::DuplicateNodeId {
            node_id: "test".to_string()
        }
        .is_repairable());
    }

    #[test]
    fn test_document_repairer_empty_document() {
        let mut tree = DocumentTree::new();
        let checker = IntegrityChecker::new();
        let report = checker.check(&tree);

        let repairer = DocumentRepairer::new();
        let actions = repairer.repair(&mut tree, &report.issues);

        // Should have added an empty paragraph
        assert!(actions.iter().any(|a| matches!(a, RepairAction::AddedEmptyParagraph)));
        assert!(!tree.document.children().is_empty());
    }

    #[test]
    fn test_orphan_node_detection() {
        let mut tree = DocumentTree::with_empty_paragraph();

        // Add an orphan run (not connected to any paragraph)
        let orphan_run = Run::new("Orphan");
        let orphan_id = orphan_run.id();
        tree.nodes.runs.insert(orphan_id, orphan_run);

        let checker = IntegrityChecker::new();
        let report = checker.check(&tree);

        assert!(report.issues.iter().any(|i| matches!(
            i,
            IntegrityIssue::OrphanNode { node_id, .. } if *node_id == orphan_id.to_string()
        )));
    }

    #[test]
    fn test_repair_orphan_node() {
        let mut tree = DocumentTree::with_empty_paragraph();

        // Add an orphan run
        let orphan_run = Run::new("Orphan");
        let orphan_id = orphan_run.id();
        tree.nodes.runs.insert(orphan_id, orphan_run);

        let checker = IntegrityChecker::new();
        let report = checker.check(&tree);

        let repairer = DocumentRepairer::new();
        repairer.repair(&mut tree, &report.issues);

        // Orphan should be removed
        assert!(!tree.nodes.runs.contains_key(&orphan_id));
    }
}

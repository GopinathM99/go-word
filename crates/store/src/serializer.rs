//! Document serialization

use crate::{DocumentFile, Result};
use doc_model::DocumentTree;

/// Serialize a document tree to JSON
pub fn serialize(tree: &DocumentTree) -> Result<String> {
    let file = DocumentFile::new(tree.clone());
    let json = serde_json::to_string_pretty(&file)?;
    Ok(json)
}

/// Deserialize a document tree from JSON
pub fn deserialize(json: &str) -> Result<DocumentTree> {
    let file: DocumentFile = serde_json::from_str(json)?;

    if !file.header.is_valid() {
        return Err(crate::StoreError::InvalidFormat(
            format!("Invalid or unsupported format version: {}", file.header.version)
        ));
    }

    Ok(file.document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let tree = DocumentTree::with_empty_paragraph();
        let json = serialize(&tree).unwrap();
        let loaded = deserialize(&json).unwrap();

        assert_eq!(tree.root_id(), loaded.root_id());
    }
}

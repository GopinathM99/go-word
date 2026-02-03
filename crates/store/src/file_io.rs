//! File I/O operations

use crate::{Result, StoreError};
use doc_model::DocumentTree;
use std::path::Path;

/// Save a document to a file
pub async fn save_document(tree: &DocumentTree, path: impl AsRef<Path>) -> Result<()> {
    let json = crate::serialize(tree)?;
    tokio::fs::write(path, json).await?;
    Ok(())
}

/// Load a document from a file
pub async fn load_document(path: impl AsRef<Path>) -> Result<DocumentTree> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(StoreError::FileNotFound(
            path.display().to_string()
        ));
    }

    let json = tokio::fs::read_to_string(path).await?;
    crate::deserialize(&json)
}

/// Save a document synchronously (for use in Tauri commands)
pub fn save_document_sync(tree: &DocumentTree, path: impl AsRef<Path>) -> Result<()> {
    let json = crate::serialize(tree)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load a document synchronously
pub fn load_document_sync(path: impl AsRef<Path>) -> Result<DocumentTree> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(StoreError::FileNotFound(
            path.display().to_string()
        ));
    }

    let json = std::fs::read_to_string(path)?;
    crate::deserialize(&json)
}

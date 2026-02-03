//! Image commands for inserting, resizing, and modifying images

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{
    Dimension, DocumentTree, ImageNode, ImagePosition, ImageProperties, Node, NodeId, NodeType,
    Paragraph, Position, ResourceId, Selection, WrapType,
};
use serde::{Deserialize, Serialize};

/// Insert an image at the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertImage {
    /// Resource ID of the stored image
    pub resource_id: ResourceId,
    /// Original image width in pixels
    pub original_width: u32,
    /// Original image height in pixels
    pub original_height: u32,
    /// Image properties (size, wrap type, etc.)
    pub properties: ImageProperties,
    /// Alternative text
    pub alt_text: Option<String>,
    /// Image title (tooltip)
    pub title: Option<String>,
}

impl InsertImage {
    /// Create a new insert image command for an inline image
    pub fn inline(
        resource_id: ResourceId,
        original_width: u32,
        original_height: u32,
        width: f32,
        height: f32,
    ) -> Self {
        Self {
            resource_id,
            original_width,
            original_height,
            properties: ImageProperties::inline(width, height),
            alt_text: None,
            title: None,
        }
    }

    /// Create a new insert image command with auto dimensions
    pub fn auto(resource_id: ResourceId, original_width: u32, original_height: u32) -> Self {
        Self {
            resource_id,
            original_width,
            original_height,
            properties: ImageProperties::default(),
            alt_text: None,
            title: None,
        }
    }

    /// Set the alt text
    pub fn with_alt_text(mut self, alt_text: impl Into<String>) -> Self {
        self.alt_text = Some(alt_text.into());
        self
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the properties
    pub fn with_properties(mut self, properties: ImageProperties) -> Self {
        self.properties = properties;
        self
    }
}

impl Command for InsertImage {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the paragraph containing the selection
        let para_id = get_paragraph_for_position(&new_tree, &selection.anchor)?;

        // Create the image node
        let mut image = ImageNode::with_size(
            self.resource_id.clone(),
            self.original_width,
            self.original_height,
            self.properties.width.resolve(468.0).unwrap_or(self.original_width as f32),
            self.properties.height.resolve(648.0).unwrap_or(self.original_height as f32),
        );
        image.set_properties(self.properties.clone());

        if let Some(ref alt) = self.alt_text {
            image.set_alt_text(alt.clone());
        }
        if let Some(ref title) = self.title {
            image.set_title(title.clone());
        }

        let image_id = image.id();

        // Find the insertion index in the paragraph
        let insert_index = find_insertion_index(&new_tree, para_id, &selection.start())?;

        // Insert the image
        new_tree
            .insert_image(image, para_id, Some(insert_index))
            .map_err(|e| EditError::DocModel(e))?;

        // New selection is after the image (treat image as a single character)
        let new_selection =
            Selection::collapsed(Position::new(para_id, selection.start().offset + 1));

        // Create the inverse command
        let inverse = Box::new(DeleteImage { image_id });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // This will be replaced by the proper inverse in apply()
        Box::new(DeleteImage {
            image_id: NodeId::new(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        // Images take up one character position
        if selection.anchor.offset >= selection.focus.offset {
            Selection::new(
                Position::new(selection.anchor.node_id, selection.anchor.offset + 1),
                selection.focus,
            )
        } else {
            Selection::new(
                selection.anchor,
                Position::new(selection.focus.node_id, selection.focus.offset + 1),
            )
        }
    }

    fn display_name(&self) -> &str {
        "Insert Image"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Delete an image by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteImage {
    /// The image node ID to delete
    pub image_id: NodeId,
}

impl DeleteImage {
    pub fn new(image_id: NodeId) -> Self {
        Self { image_id }
    }
}

impl Command for DeleteImage {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the image before removing
        let image = new_tree
            .get_image(self.image_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Image not found: {:?}", self.image_id)))?
            .clone();

        let para_id = image
            .parent()
            .ok_or_else(|| EditError::InvalidCommand("Image has no parent".to_string()))?;

        // Store image data for undo
        let resource_id = image.resource_id.clone();
        let original_width = image.original_width;
        let original_height = image.original_height;
        let properties = image.properties.clone();
        let alt_text = image.alt_text.clone();
        let title = image.title.clone();

        // Remove the image
        new_tree
            .remove_image(self.image_id)
            .map_err(|e| EditError::DocModel(e))?;

        // Create the inverse command
        let mut inverse = InsertImage {
            resource_id,
            original_width,
            original_height,
            properties,
            alt_text,
            title,
        };

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse: Box::new(inverse),
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(image) = tree.get_image(self.image_id) {
            Box::new(InsertImage {
                resource_id: image.resource_id.clone(),
                original_width: image.original_width,
                original_height: image.original_height,
                properties: image.properties.clone(),
                alt_text: image.alt_text.clone(),
                title: image.title.clone(),
            })
        } else {
            Box::new(InsertImage::auto(ResourceId::new(""), 0, 0))
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Image"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Resize an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeImage {
    /// The image node ID
    pub image_id: NodeId,
    /// New width
    pub width: Dimension,
    /// New height
    pub height: Dimension,
}

impl ResizeImage {
    pub fn new(image_id: NodeId, width: Dimension, height: Dimension) -> Self {
        Self {
            image_id,
            width,
            height,
        }
    }

    /// Create a resize command with point dimensions
    pub fn points(image_id: NodeId, width: f32, height: f32) -> Self {
        Self {
            image_id,
            width: Dimension::points(width),
            height: Dimension::points(height),
        }
    }
}

impl Command for ResizeImage {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current dimensions for undo
        let image = new_tree
            .get_image(self.image_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Image not found: {:?}", self.image_id)))?;

        let old_width = image.properties.width;
        let old_height = image.properties.height;

        // Apply the resize
        let image = new_tree
            .get_image_mut(self.image_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Image not found: {:?}", self.image_id)))?;

        image.properties.width = self.width;
        image.properties.height = self.height;

        // Create the inverse command
        let inverse = Box::new(ResizeImage {
            image_id: self.image_id,
            width: old_width,
            height: old_height,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(image) = tree.get_image(self.image_id) {
            Box::new(ResizeImage {
                image_id: self.image_id,
                width: image.properties.width,
                height: image.properties.height,
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Resize Image"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Set image wrap type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetImageWrap {
    /// The image node ID
    pub image_id: NodeId,
    /// New wrap type
    pub wrap_type: WrapType,
}

impl SetImageWrap {
    pub fn new(image_id: NodeId, wrap_type: WrapType) -> Self {
        Self { image_id, wrap_type }
    }
}

impl Command for SetImageWrap {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current wrap type for undo
        let image = new_tree
            .get_image(self.image_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Image not found: {:?}", self.image_id)))?;

        let old_wrap_type = image.properties.wrap_type;

        // Apply the new wrap type
        let image = new_tree
            .get_image_mut(self.image_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Image not found: {:?}", self.image_id)))?;

        image.properties.wrap_type = self.wrap_type;

        // Update position based on wrap type
        if self.wrap_type == WrapType::Inline {
            image.properties.position = ImagePosition::Inline;
        } else if matches!(image.properties.position, ImagePosition::Inline) {
            // Convert to anchor position for non-inline wrap
            image.properties.position = ImagePosition::Anchor(Default::default());
        }

        // Create the inverse command
        let inverse = Box::new(SetImageWrap {
            image_id: self.image_id,
            wrap_type: old_wrap_type,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(image) = tree.get_image(self.image_id) {
            Box::new(SetImageWrap {
                image_id: self.image_id,
                wrap_type: image.properties.wrap_type,
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Image Wrap"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Set image position (for floating images)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetImagePosition {
    /// The image node ID
    pub image_id: NodeId,
    /// New position
    pub position: ImagePosition,
}

impl SetImagePosition {
    pub fn new(image_id: NodeId, position: ImagePosition) -> Self {
        Self { image_id, position }
    }
}

impl Command for SetImagePosition {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current position for undo
        let image = new_tree
            .get_image(self.image_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Image not found: {:?}", self.image_id)))?;

        let old_position = image.properties.position;

        // Apply the new position
        let image = new_tree
            .get_image_mut(self.image_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Image not found: {:?}", self.image_id)))?;

        image.properties.position = self.position;

        // Create the inverse command
        let inverse = Box::new(SetImagePosition {
            image_id: self.image_id,
            position: old_position,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(image) = tree.get_image(self.image_id) {
            Box::new(SetImagePosition {
                image_id: self.image_id,
                position: image.properties.position,
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Image Position"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Update image properties (alt text, title, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateImageProperties {
    /// The image node ID
    pub image_id: NodeId,
    /// New alt text (if Some, updates; if None, no change)
    pub alt_text: Option<Option<String>>,
    /// New title (if Some, updates; if None, no change)
    pub title: Option<Option<String>>,
    /// New rotation (if Some, updates)
    pub rotation: Option<f32>,
    /// Lock aspect ratio setting
    pub lock_aspect_ratio: Option<bool>,
}

impl UpdateImageProperties {
    pub fn new(image_id: NodeId) -> Self {
        Self {
            image_id,
            alt_text: None,
            title: None,
            rotation: None,
            lock_aspect_ratio: None,
        }
    }

    pub fn with_alt_text(mut self, alt_text: Option<String>) -> Self {
        self.alt_text = Some(alt_text);
        self
    }

    pub fn with_title(mut self, title: Option<String>) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = Some(rotation);
        self
    }

    pub fn with_lock_aspect_ratio(mut self, lock: bool) -> Self {
        self.lock_aspect_ratio = Some(lock);
        self
    }
}

impl Command for UpdateImageProperties {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current values for undo
        let image = new_tree
            .get_image(self.image_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Image not found: {:?}", self.image_id)))?;

        let old_alt_text = image.alt_text.clone();
        let old_title = image.title.clone();
        let old_rotation = image.properties.rotation;
        let old_lock_aspect_ratio = image.properties.lock_aspect_ratio;

        // Apply updates
        let image = new_tree
            .get_image_mut(self.image_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Image not found: {:?}", self.image_id)))?;

        if let Some(ref alt) = self.alt_text {
            image.alt_text = alt.clone();
        }
        if let Some(ref t) = self.title {
            image.title = t.clone();
        }
        if let Some(rot) = self.rotation {
            image.properties.rotation = rot;
        }
        if let Some(lock) = self.lock_aspect_ratio {
            image.properties.lock_aspect_ratio = lock;
        }

        // Create the inverse command
        let mut inverse = UpdateImageProperties::new(self.image_id);
        if self.alt_text.is_some() {
            inverse.alt_text = Some(old_alt_text);
        }
        if self.title.is_some() {
            inverse.title = Some(old_title);
        }
        if self.rotation.is_some() {
            inverse.rotation = Some(old_rotation);
        }
        if self.lock_aspect_ratio.is_some() {
            inverse.lock_aspect_ratio = Some(old_lock_aspect_ratio);
        }

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse: Box::new(inverse),
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(image) = tree.get_image(self.image_id) {
            let mut inverse = UpdateImageProperties::new(self.image_id);
            if self.alt_text.is_some() {
                inverse.alt_text = Some(image.alt_text.clone());
            }
            if self.title.is_some() {
                inverse.title = Some(image.title.clone());
            }
            if self.rotation.is_some() {
                inverse.rotation = Some(image.properties.rotation);
            }
            if self.lock_aspect_ratio.is_some() {
                inverse.lock_aspect_ratio = Some(image.properties.lock_aspect_ratio);
            }
            Box::new(inverse)
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Update Image Properties"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Get the paragraph containing a position
fn get_paragraph_for_position(tree: &DocumentTree, position: &Position) -> Result<NodeId> {
    let node_type = tree.node_type(position.node_id).ok_or_else(|| {
        EditError::InvalidCommand(format!("Node not found: {:?}", position.node_id))
    })?;

    match node_type {
        NodeType::Paragraph => Ok(position.node_id),
        NodeType::Run => {
            let run = tree.get_run(position.node_id).ok_or_else(|| {
                EditError::InvalidCommand(format!("Run not found: {:?}", position.node_id))
            })?;

            let parent_id = run
                .parent()
                .ok_or_else(|| EditError::InvalidCommand("Run has no parent".to_string()))?;

            if tree.get_paragraph(parent_id).is_some() {
                return Ok(parent_id);
            }

            // Parent might be a hyperlink
            if let Some(hyperlink) = tree.get_hyperlink(parent_id) {
                return hyperlink.parent().ok_or_else(|| {
                    EditError::InvalidCommand("Hyperlink has no parent".to_string())
                });
            }

            Err(EditError::InvalidCommand(
                "Cannot determine paragraph".to_string(),
            ))
        }
        NodeType::Hyperlink => {
            let hyperlink = tree.get_hyperlink(position.node_id).ok_or_else(|| {
                EditError::InvalidCommand(format!("Hyperlink not found: {:?}", position.node_id))
            })?;

            hyperlink
                .parent()
                .ok_or_else(|| EditError::InvalidCommand("Hyperlink has no parent".to_string()))
        }
        NodeType::Image => {
            let image = tree.get_image(position.node_id).ok_or_else(|| {
                EditError::InvalidCommand(format!("Image not found: {:?}", position.node_id))
            })?;

            image
                .parent()
                .ok_or_else(|| EditError::InvalidCommand("Image has no parent".to_string()))
        }
        _ => Err(EditError::InvalidCommand(format!(
            "Invalid node type for position: {:?}",
            node_type
        ))),
    }
}

/// Find the insertion index in a paragraph for a given position
fn find_insertion_index(tree: &DocumentTree, para_id: NodeId, position: &Position) -> Result<usize> {
    let para = tree.get_paragraph(para_id).ok_or_else(|| {
        EditError::InvalidCommand(format!("Paragraph not found: {:?}", para_id))
    })?;

    let mut offset = 0;

    for (index, &child_id) in para.children().iter().enumerate() {
        // Check for run
        if let Some(run) = tree.get_run(child_id) {
            let run_len = run.text.chars().count();
            if offset + run_len >= position.offset {
                return Ok(index);
            }
            offset += run_len;
        }
        // Check for hyperlink
        else if let Some(hyperlink) = tree.get_hyperlink(child_id) {
            let mut hyperlink_len = 0;
            for &run_id in hyperlink.children() {
                if let Some(run) = tree.get_run(run_id) {
                    hyperlink_len += run.text.chars().count();
                }
            }
            if offset + hyperlink_len >= position.offset {
                return Ok(index);
            }
            offset += hyperlink_len;
        }
        // Check for image (counts as 1 character)
        else if tree.get_image(child_id).is_some() {
            if offset + 1 >= position.offset {
                return Ok(index);
            }
            offset += 1;
        }
    }

    Ok(para.children().len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.insert_paragraph(para, tree.root_id(), None).unwrap();
        (tree, para_id)
    }

    #[test]
    fn test_insert_image() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = InsertImage::inline(ResourceId::new("test-image"), 800, 600, 400.0, 300.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        // Should have one image
        assert_eq!(result.tree.images().count(), 1);

        let image = result.tree.images().next().unwrap();
        assert_eq!(image.original_width, 800);
        assert_eq!(image.original_height, 600);
    }

    #[test]
    fn test_delete_image() {
        let (mut tree, para_id) = create_test_tree();

        // Insert an image first
        let image = ImageNode::with_size(ResourceId::new("test"), 800, 600, 400.0, 300.0);
        let image_id = tree.insert_image(image, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Delete the image
        let cmd = DeleteImage::new(image_id);
        let result = cmd.apply(&tree, &selection).unwrap();

        assert_eq!(result.tree.images().count(), 0);
    }

    #[test]
    fn test_resize_image() {
        let (mut tree, para_id) = create_test_tree();

        // Insert an image first
        let image = ImageNode::with_size(ResourceId::new("test"), 800, 600, 400.0, 300.0);
        let image_id = tree.insert_image(image, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Resize the image
        let cmd = ResizeImage::points(image_id, 200.0, 150.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        let resized = result.tree.get_image(image_id).unwrap();
        assert_eq!(resized.properties.width, Dimension::points(200.0));
        assert_eq!(resized.properties.height, Dimension::points(150.0));
    }

    #[test]
    fn test_set_wrap_type() {
        let (mut tree, para_id) = create_test_tree();

        // Insert an image first
        let image = ImageNode::with_size(ResourceId::new("test"), 800, 600, 400.0, 300.0);
        let image_id = tree.insert_image(image, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Change wrap type
        let cmd = SetImageWrap::new(image_id, WrapType::Square);
        let result = cmd.apply(&tree, &selection).unwrap();

        let updated = result.tree.get_image(image_id).unwrap();
        assert_eq!(updated.properties.wrap_type, WrapType::Square);
    }
}

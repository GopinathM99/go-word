//! Shape commands for inserting, resizing, and modifying shapes

use crate::{Command, CommandResult, EditError, Result};
use doc_model::{
    Dimension, DocumentTree, ImagePosition, Node, NodeId, NodeType, Position, Selection,
    ShapeColor, ShapeEffects, ShapeFill, ShapeNode, ShapeProperties, ShapeStroke, ShapeType,
    WrapType,
};
use serde::{Deserialize, Serialize};

/// Insert a shape at the current selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertShape {
    /// Type of shape to insert
    pub shape_type: ShapeType,
    /// Shape properties (size, wrap type, etc.)
    pub properties: ShapeProperties,
    /// Optional name for the shape
    pub name: Option<String>,
    /// Alternative text for accessibility
    pub alt_text: Option<String>,
}

impl InsertShape {
    /// Create a new insert shape command for an inline rectangle
    pub fn rectangle(width: f32, height: f32) -> Self {
        Self {
            shape_type: ShapeType::Rectangle,
            properties: ShapeProperties::inline(width, height),
            name: None,
            alt_text: None,
        }
    }

    /// Create a new insert shape command for a rounded rectangle
    pub fn rounded_rectangle(width: f32, height: f32, corner_radius: f32) -> Self {
        Self {
            shape_type: ShapeType::RoundedRectangle { corner_radius },
            properties: ShapeProperties::inline(width, height),
            name: None,
            alt_text: None,
        }
    }

    /// Create a new insert shape command for an oval
    pub fn oval(width: f32, height: f32) -> Self {
        Self {
            shape_type: ShapeType::Oval,
            properties: ShapeProperties::inline(width, height),
            name: None,
            alt_text: None,
        }
    }

    /// Create a new insert shape command for a line
    pub fn line(width: f32, height: f32) -> Self {
        Self {
            shape_type: ShapeType::Line,
            properties: ShapeProperties::line(width, height),
            name: None,
            alt_text: None,
        }
    }

    /// Create a new insert shape command for an arrow
    pub fn arrow(width: f32, height: f32) -> Self {
        Self {
            shape_type: ShapeType::Arrow,
            properties: ShapeProperties::line(width, height),
            name: None,
            alt_text: None,
        }
    }

    /// Create a new insert shape command for a triangle
    pub fn triangle(width: f32, height: f32) -> Self {
        Self {
            shape_type: ShapeType::Triangle,
            properties: ShapeProperties::inline(width, height),
            name: None,
            alt_text: None,
        }
    }

    /// Create a new insert shape command for a star
    pub fn star(width: f32, height: f32, points: u8) -> Self {
        Self {
            shape_type: ShapeType::Star {
                points: points.clamp(5, 12),
                inner_radius_ratio: 0.4,
            },
            properties: ShapeProperties::inline(width, height),
            name: None,
            alt_text: None,
        }
    }

    /// Create a new insert shape command for a text box
    pub fn text_box(width: f32, height: f32) -> Self {
        Self {
            shape_type: ShapeType::TextBox,
            properties: ShapeProperties::text_box(width, height),
            name: None,
            alt_text: None,
        }
    }

    /// Create a new insert shape command for a callout
    pub fn callout(width: f32, height: f32) -> Self {
        Self {
            shape_type: ShapeType::Callout {
                tail_position: (0.5, 1.2),
                tail_width: 20.0,
            },
            properties: ShapeProperties::inline(width, height),
            name: None,
            alt_text: None,
        }
    }

    /// Set the name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the alt text
    pub fn with_alt_text(mut self, alt_text: impl Into<String>) -> Self {
        self.alt_text = Some(alt_text.into());
        self
    }

    /// Set the properties
    pub fn with_properties(mut self, properties: ShapeProperties) -> Self {
        self.properties = properties;
        self
    }
}

impl Command for InsertShape {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the paragraph containing the selection
        let para_id = get_paragraph_for_position(&new_tree, &selection.anchor)?;

        // Create the shape node
        let mut shape = ShapeNode::new(self.shape_type.clone());
        shape.set_properties(self.properties.clone());

        if let Some(ref name) = self.name {
            shape.set_name(name.clone());
        }
        if let Some(ref alt) = self.alt_text {
            shape.set_alt_text(alt.clone());
        }

        let shape_id = shape.id();

        // Find the insertion index in the paragraph
        let insert_index = find_insertion_index(&new_tree, para_id, &selection.start())?;

        // Insert the shape
        new_tree
            .insert_shape(shape, para_id, Some(insert_index))
            .map_err(|e| EditError::DocModel(e))?;

        // New selection is after the shape (treat shape as a single character)
        let new_selection =
            Selection::collapsed(Position::new(para_id, selection.start().offset + 1));

        // Create the inverse command
        let inverse = Box::new(DeleteShape { shape_id });

        Ok(CommandResult {
            tree: new_tree,
            selection: new_selection,
            inverse,
        })
    }

    fn invert(&self, _tree: &DocumentTree) -> Box<dyn Command> {
        // This will be replaced by the proper inverse in apply()
        Box::new(DeleteShape {
            shape_id: NodeId::new(),
        })
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        // Shapes take up one character position
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
        "Insert Shape"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Delete a shape by ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteShape {
    /// The shape node ID to delete
    pub shape_id: NodeId,
}

impl DeleteShape {
    pub fn new(shape_id: NodeId) -> Self {
        Self { shape_id }
    }
}

impl Command for DeleteShape {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the shape before removing
        let shape = new_tree
            .get_shape(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?
            .clone();

        // Store shape data for undo
        let shape_type = shape.shape_type.clone();
        let properties = shape.properties.clone();
        let name = shape.name.clone();
        let alt_text = shape.alt_text.clone();

        // Remove the shape
        new_tree
            .remove_shape(self.shape_id)
            .map_err(|e| EditError::DocModel(e))?;

        // Create the inverse command
        let inverse = InsertShape {
            shape_type,
            properties,
            name,
            alt_text,
        };

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse: Box::new(inverse),
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(shape) = tree.get_shape(self.shape_id) {
            Box::new(InsertShape {
                shape_type: shape.shape_type.clone(),
                properties: shape.properties.clone(),
                name: shape.name.clone(),
                alt_text: shape.alt_text.clone(),
            })
        } else {
            Box::new(InsertShape::rectangle(100.0, 100.0))
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Delete Shape"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Resize a shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeShape {
    /// The shape node ID
    pub shape_id: NodeId,
    /// New width
    pub width: Dimension,
    /// New height
    pub height: Dimension,
}

impl ResizeShape {
    pub fn new(shape_id: NodeId, width: Dimension, height: Dimension) -> Self {
        Self {
            shape_id,
            width,
            height,
        }
    }

    /// Create a resize command with point dimensions
    pub fn points(shape_id: NodeId, width: f32, height: f32) -> Self {
        Self {
            shape_id,
            width: Dimension::points(width),
            height: Dimension::points(height),
        }
    }
}

impl Command for ResizeShape {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current dimensions for undo
        let shape = new_tree
            .get_shape(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        let old_width = shape.properties.width;
        let old_height = shape.properties.height;

        // Apply the resize
        let shape = new_tree
            .get_shape_mut(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        shape.properties.width = self.width;
        shape.properties.height = self.height;

        // Create the inverse command
        let inverse = Box::new(ResizeShape {
            shape_id: self.shape_id,
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
        if let Some(shape) = tree.get_shape(self.shape_id) {
            Box::new(ResizeShape {
                shape_id: self.shape_id,
                width: shape.properties.width,
                height: shape.properties.height,
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Resize Shape"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Move a shape (for floating shapes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveShape {
    /// The shape node ID
    pub shape_id: NodeId,
    /// New position
    pub position: ImagePosition,
}

impl MoveShape {
    pub fn new(shape_id: NodeId, position: ImagePosition) -> Self {
        Self { shape_id, position }
    }
}

impl Command for MoveShape {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current position for undo
        let shape = new_tree
            .get_shape(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        let old_position = shape.properties.position;

        // Apply the move
        let shape = new_tree
            .get_shape_mut(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        shape.properties.position = self.position;

        // Create the inverse command
        let inverse = Box::new(MoveShape {
            shape_id: self.shape_id,
            position: old_position,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(shape) = tree.get_shape(self.shape_id) {
            Box::new(MoveShape {
                shape_id: self.shape_id,
                position: shape.properties.position,
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Move Shape"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Set shape fill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetShapeFill {
    /// The shape node ID
    pub shape_id: NodeId,
    /// New fill style
    pub fill: Option<ShapeFill>,
}

impl SetShapeFill {
    pub fn new(shape_id: NodeId, fill: Option<ShapeFill>) -> Self {
        Self { shape_id, fill }
    }

    /// Create a solid fill command
    pub fn solid(shape_id: NodeId, color: ShapeColor) -> Self {
        Self {
            shape_id,
            fill: Some(ShapeFill::Solid(color)),
        }
    }

    /// Create a no-fill command
    pub fn none(shape_id: NodeId) -> Self {
        Self {
            shape_id,
            fill: Some(ShapeFill::None),
        }
    }
}

impl Command for SetShapeFill {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current fill for undo
        let shape = new_tree
            .get_shape(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        let old_fill = shape.properties.fill.clone();

        // Apply the fill
        let shape = new_tree
            .get_shape_mut(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        shape.properties.fill = self.fill.clone();

        // Create the inverse command
        let inverse = Box::new(SetShapeFill {
            shape_id: self.shape_id,
            fill: old_fill,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(shape) = tree.get_shape(self.shape_id) {
            Box::new(SetShapeFill {
                shape_id: self.shape_id,
                fill: shape.properties.fill.clone(),
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Shape Fill"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Set shape stroke
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetShapeStroke {
    /// The shape node ID
    pub shape_id: NodeId,
    /// New stroke style
    pub stroke: Option<ShapeStroke>,
}

impl SetShapeStroke {
    pub fn new(shape_id: NodeId, stroke: Option<ShapeStroke>) -> Self {
        Self { shape_id, stroke }
    }

    /// Create a solid stroke command
    pub fn solid(shape_id: NodeId, color: ShapeColor, width: f32) -> Self {
        Self {
            shape_id,
            stroke: Some(ShapeStroke::solid(color, width)),
        }
    }

    /// Create a no-stroke command
    pub fn none(shape_id: NodeId) -> Self {
        Self {
            shape_id,
            stroke: None,
        }
    }
}

impl Command for SetShapeStroke {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current stroke for undo
        let shape = new_tree
            .get_shape(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        let old_stroke = shape.properties.stroke.clone();

        // Apply the stroke
        let shape = new_tree
            .get_shape_mut(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        shape.properties.stroke = self.stroke.clone();

        // Create the inverse command
        let inverse = Box::new(SetShapeStroke {
            shape_id: self.shape_id,
            stroke: old_stroke,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(shape) = tree.get_shape(self.shape_id) {
            Box::new(SetShapeStroke {
                shape_id: self.shape_id,
                stroke: shape.properties.stroke.clone(),
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Shape Stroke"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Rotate a shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateShape {
    /// The shape node ID
    pub shape_id: NodeId,
    /// Rotation angle in degrees (clockwise)
    pub angle: f32,
}

impl RotateShape {
    pub fn new(shape_id: NodeId, angle: f32) -> Self {
        Self { shape_id, angle }
    }
}

impl Command for RotateShape {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current rotation for undo
        let shape = new_tree
            .get_shape(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        let old_angle = shape.properties.rotation;

        // Apply the rotation
        let shape = new_tree
            .get_shape_mut(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        shape.properties.rotation = self.angle;

        // Create the inverse command
        let inverse = Box::new(RotateShape {
            shape_id: self.shape_id,
            angle: old_angle,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(shape) = tree.get_shape(self.shape_id) {
            Box::new(RotateShape {
                shape_id: self.shape_id,
                angle: shape.properties.rotation,
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Rotate Shape"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Set shape wrap type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetShapeWrap {
    /// The shape node ID
    pub shape_id: NodeId,
    /// New wrap type
    pub wrap_type: WrapType,
}

impl SetShapeWrap {
    pub fn new(shape_id: NodeId, wrap_type: WrapType) -> Self {
        Self { shape_id, wrap_type }
    }
}

impl Command for SetShapeWrap {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current wrap type for undo
        let shape = new_tree
            .get_shape(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        let old_wrap_type = shape.properties.wrap_type;

        // Apply the new wrap type
        let shape = new_tree
            .get_shape_mut(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        shape.properties.wrap_type = self.wrap_type;

        // Update position based on wrap type
        if self.wrap_type == WrapType::Inline {
            shape.properties.position = ImagePosition::Inline;
        } else if matches!(shape.properties.position, ImagePosition::Inline) {
            // Convert to anchor position for non-inline wrap
            shape.properties.position = ImagePosition::Anchor(Default::default());
        }

        // Create the inverse command
        let inverse = Box::new(SetShapeWrap {
            shape_id: self.shape_id,
            wrap_type: old_wrap_type,
        });

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse,
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(shape) = tree.get_shape(self.shape_id) {
            Box::new(SetShapeWrap {
                shape_id: self.shape_id,
                wrap_type: shape.properties.wrap_type,
            })
        } else {
            Box::new(self.clone())
        }
    }

    fn transform_selection(&self, selection: &Selection) -> Selection {
        *selection
    }

    fn display_name(&self) -> &str {
        "Set Shape Wrap"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

/// Update shape properties (generic updates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateShapeProperties {
    /// The shape node ID
    pub shape_id: NodeId,
    /// New name (if Some, updates)
    pub name: Option<Option<String>>,
    /// New alt text (if Some, updates)
    pub alt_text: Option<Option<String>>,
    /// New effects (if Some, updates)
    pub effects: Option<ShapeEffects>,
    /// Flip horizontal (if Some, updates)
    pub flip_horizontal: Option<bool>,
    /// Flip vertical (if Some, updates)
    pub flip_vertical: Option<bool>,
}

impl UpdateShapeProperties {
    pub fn new(shape_id: NodeId) -> Self {
        Self {
            shape_id,
            name: None,
            alt_text: None,
            effects: None,
            flip_horizontal: None,
            flip_vertical: None,
        }
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_alt_text(mut self, alt_text: Option<String>) -> Self {
        self.alt_text = Some(alt_text);
        self
    }

    pub fn with_effects(mut self, effects: ShapeEffects) -> Self {
        self.effects = Some(effects);
        self
    }

    pub fn with_flip_horizontal(mut self, flip: bool) -> Self {
        self.flip_horizontal = Some(flip);
        self
    }

    pub fn with_flip_vertical(mut self, flip: bool) -> Self {
        self.flip_vertical = Some(flip);
        self
    }
}

impl Command for UpdateShapeProperties {
    fn apply(&self, tree: &DocumentTree, selection: &Selection) -> Result<CommandResult> {
        let mut new_tree = tree.clone();

        // Get the current values for undo
        let shape = new_tree
            .get_shape(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        let old_name = shape.name.clone();
        let old_alt_text = shape.alt_text.clone();
        let old_effects = shape.properties.effects.clone();
        let old_flip_horizontal = shape.properties.flip_horizontal;
        let old_flip_vertical = shape.properties.flip_vertical;

        // Apply updates
        let shape = new_tree
            .get_shape_mut(self.shape_id)
            .ok_or_else(|| EditError::InvalidCommand(format!("Shape not found: {:?}", self.shape_id)))?;

        if let Some(ref name) = self.name {
            shape.name = name.clone();
        }
        if let Some(ref alt) = self.alt_text {
            shape.alt_text = alt.clone();
        }
        if let Some(ref effects) = self.effects {
            shape.properties.effects = effects.clone();
        }
        if let Some(flip) = self.flip_horizontal {
            shape.properties.flip_horizontal = flip;
        }
        if let Some(flip) = self.flip_vertical {
            shape.properties.flip_vertical = flip;
        }

        // Create the inverse command
        let mut inverse = UpdateShapeProperties::new(self.shape_id);
        if self.name.is_some() {
            inverse.name = Some(old_name);
        }
        if self.alt_text.is_some() {
            inverse.alt_text = Some(old_alt_text);
        }
        if self.effects.is_some() {
            inverse.effects = Some(old_effects);
        }
        if self.flip_horizontal.is_some() {
            inverse.flip_horizontal = Some(old_flip_horizontal);
        }
        if self.flip_vertical.is_some() {
            inverse.flip_vertical = Some(old_flip_vertical);
        }

        Ok(CommandResult {
            tree: new_tree,
            selection: *selection,
            inverse: Box::new(inverse),
        })
    }

    fn invert(&self, tree: &DocumentTree) -> Box<dyn Command> {
        if let Some(shape) = tree.get_shape(self.shape_id) {
            let mut inverse = UpdateShapeProperties::new(self.shape_id);
            if self.name.is_some() {
                inverse.name = Some(shape.name.clone());
            }
            if self.alt_text.is_some() {
                inverse.alt_text = Some(shape.alt_text.clone());
            }
            if self.effects.is_some() {
                inverse.effects = Some(shape.properties.effects.clone());
            }
            if self.flip_horizontal.is_some() {
                inverse.flip_horizontal = Some(shape.properties.flip_horizontal);
            }
            if self.flip_vertical.is_some() {
                inverse.flip_vertical = Some(shape.properties.flip_vertical);
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
        "Update Shape Properties"
    }

    fn clone_box(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
}

// ============================================================================
// Helper functions (reused from image_commands)
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
        NodeType::Shape => {
            let shape = tree.get_shape(position.node_id).ok_or_else(|| {
                EditError::InvalidCommand(format!("Shape not found: {:?}", position.node_id))
            })?;

            shape
                .parent()
                .ok_or_else(|| EditError::InvalidCommand("Shape has no parent".to_string()))
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
        // Check for shape (counts as 1 character)
        else if tree.get_shape(child_id).is_some() {
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
    use doc_model::Paragraph;

    fn create_test_tree() -> (DocumentTree, NodeId) {
        let mut tree = DocumentTree::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.insert_paragraph(para, tree.root_id(), None).unwrap();
        (tree, para_id)
    }

    #[test]
    fn test_insert_shape() {
        let (tree, para_id) = create_test_tree();
        let selection = Selection::collapsed(Position::new(para_id, 0));

        let cmd = InsertShape::rectangle(100.0, 50.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        // Should have one shape
        assert_eq!(result.tree.shapes().count(), 1);

        let shape = result.tree.shapes().next().unwrap();
        assert!(matches!(shape.shape_type, ShapeType::Rectangle));
    }

    #[test]
    fn test_delete_shape() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a shape first
        let shape = ShapeNode::rectangle(100.0, 50.0);
        let shape_id = tree.insert_shape(shape, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Delete the shape
        let cmd = DeleteShape::new(shape_id);
        let result = cmd.apply(&tree, &selection).unwrap();

        assert_eq!(result.tree.shapes().count(), 0);
    }

    #[test]
    fn test_resize_shape() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a shape first
        let shape = ShapeNode::rectangle(100.0, 50.0);
        let shape_id = tree.insert_shape(shape, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Resize the shape
        let cmd = ResizeShape::points(shape_id, 200.0, 100.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        let resized = result.tree.get_shape(shape_id).unwrap();
        assert_eq!(resized.properties.width, Dimension::points(200.0));
        assert_eq!(resized.properties.height, Dimension::points(100.0));
    }

    #[test]
    fn test_set_shape_fill() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a shape first
        let shape = ShapeNode::rectangle(100.0, 50.0);
        let shape_id = tree.insert_shape(shape, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Change fill
        let cmd = SetShapeFill::solid(shape_id, ShapeColor::RED);
        let result = cmd.apply(&tree, &selection).unwrap();

        let updated = result.tree.get_shape(shape_id).unwrap();
        assert!(matches!(
            updated.properties.fill,
            Some(ShapeFill::Solid(ShapeColor { r: 192, g: 0, b: 0, .. }))
        ));
    }

    #[test]
    fn test_rotate_shape() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a shape first
        let shape = ShapeNode::rectangle(100.0, 50.0);
        let shape_id = tree.insert_shape(shape, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Rotate the shape
        let cmd = RotateShape::new(shape_id, 45.0);
        let result = cmd.apply(&tree, &selection).unwrap();

        let rotated = result.tree.get_shape(shape_id).unwrap();
        assert_eq!(rotated.properties.rotation, 45.0);
    }

    #[test]
    fn test_set_shape_wrap() {
        let (mut tree, para_id) = create_test_tree();

        // Insert a shape first
        let shape = ShapeNode::rectangle(100.0, 50.0);
        let shape_id = tree.insert_shape(shape, para_id, None).unwrap();

        let selection = Selection::collapsed(Position::new(para_id, 0));

        // Change wrap type
        let cmd = SetShapeWrap::new(shape_id, WrapType::Square);
        let result = cmd.apply(&tree, &selection).unwrap();

        let updated = result.tree.get_shape(shape_id).unwrap();
        assert_eq!(updated.properties.wrap_type, WrapType::Square);
    }
}

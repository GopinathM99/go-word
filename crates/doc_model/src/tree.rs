//! Document tree operations and storage

use crate::{
    Bookmark, BookmarkRange, BookmarkRegistry, BookmarkValidationError, CellPadding,
    CharacterProperties, Comment, CommentId, CommentReply, CommentStore, CommentValidationError,
    ComputedCharacterProperties, ComputedParagraphProperties, Document, DocModelError,
    EndnoteProperties, FootnoteProperties, Hyperlink, ImageNode, Node, NodeId, NodeType, Note,
    NoteId, NoteRef, NoteStore, NoteType, NumberingRegistry, Paragraph, ParagraphProperties,
    Position, ReplyId, Result, Run, Selection, ShapeNode, StyleId, StyleRegistry, Table, TableCell,
    TableRow, TextBox,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Storage for different node types
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeStorage {
    pub paragraphs: HashMap<NodeId, Paragraph>,
    pub runs: HashMap<NodeId, Run>,
    pub hyperlinks: HashMap<NodeId, Hyperlink>,
    pub images: HashMap<NodeId, ImageNode>,
    pub shapes: HashMap<NodeId, ShapeNode>,
    pub textboxes: HashMap<NodeId, TextBox>,
    pub tables: HashMap<NodeId, Table>,
    pub table_rows: HashMap<NodeId, TableRow>,
    pub table_cells: HashMap<NodeId, TableCell>,
}

/// The complete document tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTree {
    /// The root document
    pub document: Document,
    /// Storage for all nodes
    pub nodes: NodeStorage,
    /// Style registry for this document
    #[serde(default)]
    pub styles: StyleRegistry,
    /// Bookmark registry for this document
    #[serde(default)]
    pub bookmarks: BookmarkRegistry,
    /// Numbering registry for lists
    #[serde(default)]
    pub numbering: NumberingRegistry,
    /// Comment store for this document
    #[serde(default)]
    pub comments: CommentStore,
    /// Footnote and endnote store for this document
    #[serde(default)]
    pub notes: NoteStore,
}

impl DocumentTree {
    /// Create a new empty document tree
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            nodes: NodeStorage::default(),
            styles: StyleRegistry::default(),
            comments: CommentStore::default(),
            bookmarks: BookmarkRegistry::default(),
            numbering: NumberingRegistry::default(),
            notes: NoteStore::default(),
        }
    }

    /// Create a document tree with a single empty paragraph
    pub fn with_empty_paragraph() -> Self {
        let mut tree = Self::new();
        let para = Paragraph::new();
        let para_id = para.id();
        tree.nodes.paragraphs.insert(para_id, para);
        tree.document.add_body_child(para_id);
        tree
    }

    /// Get the numbering registry
    pub fn numbering_registry(&self) -> &NumberingRegistry {
        &self.numbering
    }

    /// Get a mutable reference to the numbering registry
    pub fn numbering_registry_mut(&mut self) -> &mut NumberingRegistry {
        &mut self.numbering
    }

    /// Get the style registry
    pub fn style_registry(&self) -> &StyleRegistry {
        &self.styles
    }

    /// Get a mutable reference to the style registry
    pub fn style_registry_mut(&mut self) -> &mut StyleRegistry {
        &mut self.styles
    }

    /// Apply a paragraph style to a paragraph
    pub fn apply_paragraph_style(&mut self, para_id: NodeId, style_id: StyleId) -> Result<()> {
        let para = self.nodes.paragraphs.get_mut(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;
        para.set_paragraph_style(Some(style_id));
        Ok(())
    }

    /// Apply a character style to a run
    pub fn apply_character_style(&mut self, run_id: NodeId, style_id: StyleId) -> Result<()> {
        let run = self.nodes.runs.get_mut(&run_id)
            .ok_or(DocModelError::NodeNotFound(run_id.as_uuid()))?;
        run.set_character_style(Some(style_id));
        Ok(())
    }

    /// Apply direct formatting to a paragraph
    pub fn apply_paragraph_direct_formatting(
        &mut self,
        para_id: NodeId,
        formatting: ParagraphProperties,
    ) -> Result<()> {
        let para = self.nodes.paragraphs.get_mut(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;
        para.apply_direct_formatting(formatting);
        Ok(())
    }

    /// Apply direct formatting to a run
    pub fn apply_run_direct_formatting(
        &mut self,
        run_id: NodeId,
        formatting: CharacterProperties,
    ) -> Result<()> {
        let run = self.nodes.runs.get_mut(&run_id)
            .ok_or(DocModelError::NodeNotFound(run_id.as_uuid()))?;
        run.apply_direct_formatting(formatting);
        Ok(())
    }

    /// Clear direct formatting from a paragraph
    pub fn clear_paragraph_direct_formatting(&mut self, para_id: NodeId) -> Result<()> {
        let para = self.nodes.paragraphs.get_mut(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;
        para.clear_direct_formatting();
        Ok(())
    }

    /// Clear direct formatting from a run
    pub fn clear_run_direct_formatting(&mut self, run_id: NodeId) -> Result<()> {
        let run = self.nodes.runs.get_mut(&run_id)
            .ok_or(DocModelError::NodeNotFound(run_id.as_uuid()))?;
        run.clear_direct_formatting();
        Ok(())
    }

    /// Compute the resolved paragraph properties for a paragraph
    pub fn compute_paragraph_properties(&self, para_id: NodeId) -> Option<ParagraphProperties> {
        let para = self.nodes.paragraphs.get(&para_id)?;
        Some(self.styles.resolve_paragraph_props(
            para.paragraph_style_id.as_ref(),
            &para.direct_formatting,
        ))
    }

    /// Compute the resolved character properties for a run
    pub fn compute_character_properties(&self, run_id: NodeId) -> Option<CharacterProperties> {
        let run = self.nodes.runs.get(&run_id)?;
        Some(self.styles.resolve_character_props(
            run.character_style_id.as_ref(),
            &run.direct_formatting,
        ))
    }

    /// Compute paragraph properties with source tracking for the inspector
    pub fn compute_paragraph_properties_with_sources(
        &self,
        para_id: NodeId,
    ) -> Option<ComputedParagraphProperties> {
        let para = self.nodes.paragraphs.get(&para_id)?;
        Some(self.compute_paragraph_props_with_sources_internal(
            para.paragraph_style_id.as_ref(),
            &para.direct_formatting,
        ))
    }

    /// Compute character properties with source tracking for the inspector
    pub fn compute_character_properties_with_sources(
        &self,
        run_id: NodeId,
    ) -> Option<ComputedCharacterProperties> {
        let run = self.nodes.runs.get(&run_id)?;
        Some(self.styles.compute_character_props_with_sources(
            run.character_style_id.as_ref(),
            &run.direct_formatting,
        ))
    }

    /// Internal helper to compute paragraph properties with sources
    fn compute_paragraph_props_with_sources_internal(
        &self,
        style_id: Option<&StyleId>,
        direct_formatting: &ParagraphProperties,
    ) -> ComputedParagraphProperties {
        use crate::PropertySource;

        let mut result = ComputedParagraphProperties::default();
        let style_source = style_id
            .map(|id| PropertySource::Style(id.clone()))
            .unwrap_or(PropertySource::Default);

        // Apply style properties
        if let Some(id) = style_id {
            if let Some(resolved) = self.styles.resolve(id) {
                if let Some(v) = resolved.paragraph_props.alignment {
                    result.alignment.value = v;
                    result.alignment.source = style_source.clone();
                }
                if let Some(v) = resolved.paragraph_props.indent_left {
                    result.indent_left.value = v;
                    result.indent_left.source = style_source.clone();
                }
                if let Some(v) = resolved.paragraph_props.indent_right {
                    result.indent_right.value = v;
                    result.indent_right.source = style_source.clone();
                }
                if let Some(v) = resolved.paragraph_props.indent_first_line {
                    result.indent_first_line.value = v;
                    result.indent_first_line.source = style_source.clone();
                }
                if let Some(v) = resolved.paragraph_props.space_before {
                    result.space_before.value = v;
                    result.space_before.source = style_source.clone();
                }
                if let Some(v) = resolved.paragraph_props.space_after {
                    result.space_after.value = v;
                    result.space_after.source = style_source.clone();
                }
                if let Some(v) = resolved.paragraph_props.line_spacing {
                    result.line_spacing.value = v;
                    result.line_spacing.source = style_source.clone();
                }
            }
        }

        // Apply direct formatting (always wins)
        if let Some(v) = direct_formatting.alignment {
            result.alignment.value = v;
            result.alignment.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.indent_left {
            result.indent_left.value = v;
            result.indent_left.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.indent_right {
            result.indent_right.value = v;
            result.indent_right.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.indent_first_line {
            result.indent_first_line.value = v;
            result.indent_first_line.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.space_before {
            result.space_before.value = v;
            result.space_before.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.space_after {
            result.space_after.value = v;
            result.space_after.source = PropertySource::DirectFormatting;
        }
        if let Some(v) = direct_formatting.line_spacing {
            result.line_spacing.value = v;
            result.line_spacing.source = PropertySource::DirectFormatting;
        }

        result
    }

    /// Get the document root ID
    pub fn root_id(&self) -> NodeId {
        self.document.id()
    }

    /// Get a paragraph by ID
    pub fn get_paragraph(&self, id: NodeId) -> Option<&Paragraph> {
        self.nodes.paragraphs.get(&id)
    }

    /// Get a mutable paragraph by ID
    pub fn get_paragraph_mut(&mut self, id: NodeId) -> Option<&mut Paragraph> {
        self.nodes.paragraphs.get_mut(&id)
    }

    /// Get a run by ID
    pub fn get_run(&self, id: NodeId) -> Option<&Run> {
        self.nodes.runs.get(&id)
    }

    /// Get a mutable run by ID
    pub fn get_run_mut(&mut self, id: NodeId) -> Option<&mut Run> {
        self.nodes.runs.get_mut(&id)
    }

    /// Get the node type for a given ID
    pub fn node_type(&self, id: NodeId) -> Option<NodeType> {
        if id == self.document.id() {
            return Some(NodeType::Document);
        }
        if self.nodes.paragraphs.contains_key(&id) {
            return Some(NodeType::Paragraph);
        }
        if self.nodes.runs.contains_key(&id) {
            return Some(NodeType::Run);
        }
        if self.nodes.hyperlinks.contains_key(&id) {
            return Some(NodeType::Hyperlink);
        }
        if self.nodes.images.contains_key(&id) {
            return Some(NodeType::Image);
        }
        if self.nodes.shapes.contains_key(&id) {
            return Some(NodeType::Shape);
        }
        if self.nodes.textboxes.contains_key(&id) {
            return Some(NodeType::TextBox);
        }
        if self.nodes.tables.contains_key(&id) {
            return Some(NodeType::Table);
        }
        if self.nodes.table_rows.contains_key(&id) {
            return Some(NodeType::TableRow);
        }
        if self.nodes.table_cells.contains_key(&id) {
            return Some(NodeType::TableCell);
        }
        None
    }

    /// Get a hyperlink by ID
    pub fn get_hyperlink(&self, id: NodeId) -> Option<&Hyperlink> {
        self.nodes.hyperlinks.get(&id)
    }

    /// Get a mutable hyperlink by ID
    pub fn get_hyperlink_mut(&mut self, id: NodeId) -> Option<&mut Hyperlink> {
        self.nodes.hyperlinks.get_mut(&id)
    }

    /// Insert a hyperlink into the tree under a paragraph
    pub fn insert_hyperlink(&mut self, mut hyperlink: Hyperlink, para_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let hyperlink_id = hyperlink.id();
        hyperlink.set_parent(Some(para_id));

        let para = self.nodes.paragraphs.get_mut(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;

        match index {
            Some(idx) => para.insert_child(idx, hyperlink_id),
            None => para.add_child(hyperlink_id),
        }

        self.nodes.hyperlinks.insert(hyperlink_id, hyperlink);
        Ok(hyperlink_id)
    }

    /// Remove a hyperlink from the tree
    pub fn remove_hyperlink(&mut self, hyperlink_id: NodeId) -> Result<Hyperlink> {
        let hyperlink = self.nodes.hyperlinks.remove(&hyperlink_id)
            .ok_or(DocModelError::NodeNotFound(hyperlink_id.as_uuid()))?;

        // Remove all child runs
        for &child_id in hyperlink.children() {
            self.nodes.runs.remove(&child_id);
        }

        // Remove from parent paragraph
        if let Some(parent_id) = hyperlink.parent() {
            if let Some(para) = self.nodes.paragraphs.get_mut(&parent_id) {
                para.remove_child(hyperlink_id);
            }
        }

        Ok(hyperlink)
    }

    /// Insert a run into a hyperlink
    pub fn insert_run_into_hyperlink(&mut self, mut run: Run, hyperlink_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let run_id = run.id();
        run.set_parent(Some(hyperlink_id));

        let hyperlink = self.nodes.hyperlinks.get_mut(&hyperlink_id)
            .ok_or(DocModelError::NodeNotFound(hyperlink_id.as_uuid()))?;

        match index {
            Some(idx) => hyperlink.insert_child(idx, run_id),
            None => hyperlink.add_child(run_id),
        }

        self.nodes.runs.insert(run_id, run);
        Ok(run_id)
    }

    /// Find the hyperlink containing a run (if any)
    pub fn find_hyperlink_for_run(&self, run_id: NodeId) -> Option<NodeId> {
        if let Some(run) = self.nodes.runs.get(&run_id) {
            if let Some(parent_id) = run.parent() {
                if self.nodes.hyperlinks.contains_key(&parent_id) {
                    return Some(parent_id);
                }
            }
        }
        None
    }

    /// Get all hyperlinks in the document
    pub fn hyperlinks(&self) -> impl Iterator<Item = &Hyperlink> {
        self.nodes.hyperlinks.values()
    }

    /// Get an image by ID
    pub fn get_image(&self, id: NodeId) -> Option<&ImageNode> {
        self.nodes.images.get(&id)
    }

    /// Get a mutable image by ID
    pub fn get_image_mut(&mut self, id: NodeId) -> Option<&mut ImageNode> {
        self.nodes.images.get_mut(&id)
    }

    /// Insert an image into a paragraph (inline image)
    pub fn insert_image(&mut self, mut image: ImageNode, para_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let image_id = image.id();
        image.set_parent(Some(para_id));

        let para = self.nodes.paragraphs.get_mut(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;

        match index {
            Some(idx) => para.insert_child(idx, image_id),
            None => para.add_child(image_id),
        }

        self.nodes.images.insert(image_id, image);
        Ok(image_id)
    }

    /// Remove an image from the tree
    pub fn remove_image(&mut self, image_id: NodeId) -> Result<ImageNode> {
        let image = self.nodes.images.remove(&image_id)
            .ok_or(DocModelError::NodeNotFound(image_id.as_uuid()))?;

        // Remove from parent paragraph
        if let Some(parent_id) = image.parent() {
            if let Some(para) = self.nodes.paragraphs.get_mut(&parent_id) {
                para.remove_child(image_id);
            }
        }

        Ok(image)
    }

    /// Get all images in the document
    pub fn images(&self) -> impl Iterator<Item = &ImageNode> {
        self.nodes.images.values()
    }

    /// Find inline images in a paragraph
    pub fn inline_images_in_paragraph(&self, para_id: NodeId) -> Vec<&ImageNode> {
        let Some(para) = self.nodes.paragraphs.get(&para_id) else {
            return Vec::new();
        };

        para.children()
            .iter()
            .filter_map(|id| self.nodes.images.get(id))
            .filter(|img| img.is_inline())
            .collect()
    }

    /// Find all floating images (images with wrap != Inline)
    pub fn floating_images(&self) -> impl Iterator<Item = &ImageNode> {
        self.nodes.images.values().filter(|img| img.is_floating())
    }

    // =========================================================================
    // Shape Methods
    // =========================================================================

    /// Get a shape by ID
    pub fn get_shape(&self, id: NodeId) -> Option<&ShapeNode> {
        self.nodes.shapes.get(&id)
    }

    /// Get a mutable shape by ID
    pub fn get_shape_mut(&mut self, id: NodeId) -> Option<&mut ShapeNode> {
        self.nodes.shapes.get_mut(&id)
    }

    /// Insert a shape into a paragraph (inline shape)
    pub fn insert_shape(&mut self, mut shape: ShapeNode, para_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let shape_id = shape.id();
        shape.set_parent(Some(para_id));

        let para = self.nodes.paragraphs.get_mut(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;

        match index {
            Some(idx) => para.insert_child(idx, shape_id),
            None => para.add_child(shape_id),
        }

        self.nodes.shapes.insert(shape_id, shape);
        Ok(shape_id)
    }

    /// Remove a shape from the tree
    pub fn remove_shape(&mut self, shape_id: NodeId) -> Result<ShapeNode> {
        let shape = self.nodes.shapes.remove(&shape_id)
            .ok_or(DocModelError::NodeNotFound(shape_id.as_uuid()))?;

        // Remove from parent paragraph
        if let Some(parent_id) = shape.parent() {
            if let Some(para) = self.nodes.paragraphs.get_mut(&parent_id) {
                para.remove_child(shape_id);
            }
        }

        Ok(shape)
    }

    /// Get all shapes in the document
    pub fn shapes(&self) -> impl Iterator<Item = &ShapeNode> {
        self.nodes.shapes.values()
    }

    /// Find inline shapes in a paragraph
    pub fn inline_shapes_in_paragraph(&self, para_id: NodeId) -> Vec<&ShapeNode> {
        let Some(para) = self.nodes.paragraphs.get(&para_id) else {
            return Vec::new();
        };

        para.children()
            .iter()
            .filter_map(|id| self.nodes.shapes.get(id))
            .filter(|shape| shape.is_inline())
            .collect()
    }

    /// Find all floating shapes (shapes with wrap != Inline)
    pub fn floating_shapes(&self) -> impl Iterator<Item = &ShapeNode> {
        self.nodes.shapes.values().filter(|shape| shape.is_floating())
    }

    // =========================================================================
    // TextBox Methods
    // =========================================================================

    /// Get a text box by ID
    pub fn get_textbox(&self, id: NodeId) -> Option<&TextBox> {
        self.nodes.textboxes.get(&id)
    }

    /// Get a mutable text box by ID
    pub fn get_textbox_mut(&mut self, id: NodeId) -> Option<&mut TextBox> {
        self.nodes.textboxes.get_mut(&id)
    }

    /// Insert a text box into a paragraph (for anchoring)
    pub fn insert_textbox(&mut self, mut textbox: TextBox, para_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let textbox_id = textbox.id();
        textbox.set_parent(Some(para_id));

        let para = self.nodes.paragraphs.get_mut(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;

        match index {
            Some(idx) => para.insert_child(idx, textbox_id),
            None => para.add_child(textbox_id),
        }

        self.nodes.textboxes.insert(textbox_id, textbox);
        Ok(textbox_id)
    }

    /// Remove a text box from the tree
    pub fn remove_textbox(&mut self, textbox_id: NodeId) -> Result<TextBox> {
        let textbox = self.nodes.textboxes.remove(&textbox_id)
            .ok_or(DocModelError::NodeNotFound(textbox_id.as_uuid()))?;

        // Remove content paragraphs
        for &para_id in &textbox.content {
            // Remove runs from the paragraph
            if let Some(para) = self.nodes.paragraphs.remove(&para_id) {
                for &child_id in para.children() {
                    self.nodes.runs.remove(&child_id);
                }
            }
        }

        // Remove from parent paragraph
        if let Some(parent_id) = textbox.parent() {
            if let Some(para) = self.nodes.paragraphs.get_mut(&parent_id) {
                para.remove_child(textbox_id);
            }
        }

        Ok(textbox)
    }

    /// Get all text boxes in the document
    pub fn textboxes(&self) -> impl Iterator<Item = &TextBox> {
        self.nodes.textboxes.values()
    }

    /// Find inline text boxes in a paragraph
    pub fn inline_textboxes_in_paragraph(&self, para_id: NodeId) -> Vec<&TextBox> {
        let Some(para) = self.nodes.paragraphs.get(&para_id) else {
            return Vec::new();
        };

        para.children()
            .iter()
            .filter_map(|id| self.nodes.textboxes.get(id))
            .filter(|tb| tb.is_inline())
            .collect()
    }

    /// Find all floating text boxes
    pub fn floating_textboxes(&self) -> impl Iterator<Item = &TextBox> {
        self.nodes.textboxes.values().filter(|tb| tb.is_floating())
    }

    /// Insert a paragraph into a text box
    pub fn insert_paragraph_into_textbox(&mut self, mut para: Paragraph, textbox_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let para_id = para.id();
        para.set_parent(Some(textbox_id));

        let textbox = self.nodes.textboxes.get_mut(&textbox_id)
            .ok_or(DocModelError::NodeNotFound(textbox_id.as_uuid()))?;

        match index {
            Some(idx) => textbox.insert_content(idx, para_id),
            None => textbox.add_content(para_id),
        }

        self.nodes.paragraphs.insert(para_id, para);
        Ok(para_id)
    }

    /// Remove a paragraph from a text box
    pub fn remove_paragraph_from_textbox(&mut self, para_id: NodeId, textbox_id: NodeId) -> Result<Paragraph> {
        let para = self.nodes.paragraphs.remove(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;

        // Remove all runs from the paragraph
        for &child_id in para.children() {
            self.nodes.runs.remove(&child_id);
        }

        // Remove from text box
        if let Some(textbox) = self.nodes.textboxes.get_mut(&textbox_id) {
            textbox.remove_content(para_id);
        }

        Ok(para)
    }

    /// Get content paragraphs from a text box
    pub fn textbox_paragraphs(&self, textbox_id: NodeId) -> Vec<&Paragraph> {
        let Some(textbox) = self.nodes.textboxes.get(&textbox_id) else {
            return Vec::new();
        };

        textbox.content
            .iter()
            .filter_map(|id| self.nodes.paragraphs.get(id))
            .collect()
    }

    /// Insert a paragraph into the tree
    pub fn insert_paragraph(&mut self, mut para: Paragraph, parent_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let para_id = para.id();
        para.set_parent(Some(parent_id));

        if parent_id == self.document.id() {
            match index {
                Some(idx) => self.document.insert_body_child(idx, para_id),
                None => self.document.add_body_child(para_id),
            }
        } else {
            return Err(DocModelError::InvalidOperation(
                "Paragraphs can only be children of the document".into(),
            ));
        }

        self.nodes.paragraphs.insert(para_id, para);
        Ok(para_id)
    }

    /// Insert a run into a paragraph
    pub fn insert_run(&mut self, mut run: Run, para_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let run_id = run.id();
        run.set_parent(Some(para_id));

        let para = self.nodes.paragraphs.get_mut(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;

        match index {
            Some(idx) => para.insert_child(idx, run_id),
            None => para.add_child(run_id),
        }

        self.nodes.runs.insert(run_id, run);
        Ok(run_id)
    }

    /// Remove a run from the tree
    pub fn remove_run(&mut self, run_id: NodeId) -> Result<Run> {
        let run = self.nodes.runs.remove(&run_id)
            .ok_or(DocModelError::NodeNotFound(run_id.as_uuid()))?;

        if let Some(parent_id) = run.parent() {
            if let Some(para) = self.nodes.paragraphs.get_mut(&parent_id) {
                para.remove_child(run_id);
            }
        }

        Ok(run)
    }

    /// Remove a paragraph from the tree
    pub fn remove_paragraph(&mut self, para_id: NodeId) -> Result<Paragraph> {
        let para = self.nodes.paragraphs.remove(&para_id)
            .ok_or(DocModelError::NodeNotFound(para_id.as_uuid()))?;

        // Remove all child runs
        for &child_id in para.children() {
            self.nodes.runs.remove(&child_id);
        }

        // Remove from document
        self.document.remove_body_child(para_id);

        Ok(para)
    }

    /// Find the path from root to a node
    pub fn path_to_node(&self, target_id: NodeId) -> Option<Vec<NodeId>> {
        if target_id == self.document.id() {
            return Some(vec![target_id]);
        }

        // Check if it's a paragraph
        if let Some(para) = self.nodes.paragraphs.get(&target_id) {
            if para.parent() == Some(self.document.id()) {
                return Some(vec![self.document.id(), target_id]);
            }
        }

        // Check if it's a hyperlink
        if let Some(hyperlink) = self.nodes.hyperlinks.get(&target_id) {
            if let Some(para_id) = hyperlink.parent() {
                return Some(vec![self.document.id(), para_id, target_id]);
            }
        }

        // Check if it's an image
        if let Some(image) = self.nodes.images.get(&target_id) {
            if let Some(para_id) = image.parent() {
                return Some(vec![self.document.id(), para_id, target_id]);
            }
        }

        // Check if it's a shape
        if let Some(shape) = self.nodes.shapes.get(&target_id) {
            if let Some(para_id) = shape.parent() {
                return Some(vec![self.document.id(), para_id, target_id]);
            }
        }

        // Check if it's a text box
        if let Some(textbox) = self.nodes.textboxes.get(&target_id) {
            if let Some(para_id) = textbox.parent() {
                return Some(vec![self.document.id(), para_id, target_id]);
            }
        }

        // Check if it's a run
        if let Some(run) = self.nodes.runs.get(&target_id) {
            if let Some(parent_id) = run.parent() {
                // Parent could be a paragraph or a hyperlink
                if self.nodes.paragraphs.contains_key(&parent_id) {
                    return Some(vec![self.document.id(), parent_id, target_id]);
                }
                // Parent is a hyperlink
                if let Some(hyperlink) = self.nodes.hyperlinks.get(&parent_id) {
                    if let Some(para_id) = hyperlink.parent() {
                        return Some(vec![self.document.id(), para_id, parent_id, target_id]);
                    }
                }
            }
        }

        None
    }

    /// Iterate over all paragraphs in document order
    pub fn paragraphs(&self) -> impl Iterator<Item = &Paragraph> {
        self.document.children()
            .iter()
            .filter_map(|id| self.nodes.paragraphs.get(id))
    }

    /// Get the total text content of the document
    pub fn text_content(&self) -> String {
        let mut result = String::new();
        for para in self.paragraphs() {
            for &run_id in para.children() {
                if let Some(run) = self.nodes.runs.get(&run_id) {
                    result.push_str(&run.text);
                }
            }
            result.push('\n');
        }
        result
    }

    // =========================================================================
    // Table Methods
    // =========================================================================

    /// Get a table by ID
    pub fn get_table(&self, id: NodeId) -> Option<&Table> {
        self.nodes.tables.get(&id)
    }

    /// Get a mutable table by ID
    pub fn get_table_mut(&mut self, id: NodeId) -> Option<&mut Table> {
        self.nodes.tables.get_mut(&id)
    }

    /// Get a table row by ID
    pub fn get_table_row(&self, id: NodeId) -> Option<&TableRow> {
        self.nodes.table_rows.get(&id)
    }

    /// Get a mutable table row by ID
    pub fn get_table_row_mut(&mut self, id: NodeId) -> Option<&mut TableRow> {
        self.nodes.table_rows.get_mut(&id)
    }

    /// Get a table cell by ID
    pub fn get_table_cell(&self, id: NodeId) -> Option<&TableCell> {
        self.nodes.table_cells.get(&id)
    }

    /// Get a mutable table cell by ID
    pub fn get_table_cell_mut(&mut self, id: NodeId) -> Option<&mut TableCell> {
        self.nodes.table_cells.get_mut(&id)
    }

    /// Insert a table into the document body
    pub fn insert_table(&mut self, mut table: Table, index: Option<usize>) -> Result<NodeId> {
        let table_id = table.id();
        table.set_parent(Some(self.document.id()));

        match index {
            Some(idx) => self.document.insert_body_child(idx, table_id),
            None => self.document.add_body_child(table_id),
        }

        self.nodes.tables.insert(table_id, table);
        Ok(table_id)
    }

    /// Insert a row into a table
    pub fn insert_table_row(&mut self, mut row: TableRow, table_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let row_id = row.id();
        row.set_parent(Some(table_id));

        let table = self.nodes.tables.get_mut(&table_id)
            .ok_or(DocModelError::NodeNotFound(table_id.as_uuid()))?;

        match index {
            Some(idx) => table.insert_row(idx, row_id),
            None => table.add_row(row_id),
        }

        self.nodes.table_rows.insert(row_id, row);
        Ok(row_id)
    }

    /// Insert a cell into a row
    pub fn insert_table_cell(&mut self, mut cell: TableCell, row_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let cell_id = cell.id();
        cell.set_parent(Some(row_id));

        let row = self.nodes.table_rows.get_mut(&row_id)
            .ok_or(DocModelError::NodeNotFound(row_id.as_uuid()))?;

        match index {
            Some(idx) => row.insert_cell(idx, cell_id),
            None => row.add_cell(cell_id),
        }

        self.nodes.table_cells.insert(cell_id, cell);
        Ok(cell_id)
    }

    /// Insert a paragraph into a table cell
    pub fn insert_paragraph_into_cell(&mut self, mut para: Paragraph, cell_id: NodeId, index: Option<usize>) -> Result<NodeId> {
        let para_id = para.id();
        para.set_parent(Some(cell_id));

        let cell = self.nodes.table_cells.get_mut(&cell_id)
            .ok_or(DocModelError::NodeNotFound(cell_id.as_uuid()))?;

        match index {
            Some(idx) => cell.insert_child(idx, para_id),
            None => cell.add_child(para_id),
        }

        self.nodes.paragraphs.insert(para_id, para);
        Ok(para_id)
    }

    /// Remove a table from the document
    pub fn remove_table(&mut self, table_id: NodeId) -> Result<Table> {
        let table = self.nodes.tables.remove(&table_id)
            .ok_or(DocModelError::NodeNotFound(table_id.as_uuid()))?;

        // Remove all rows and their contents
        for &row_id in table.children() {
            self.remove_table_row_contents(row_id);
        }

        // Remove from document body
        self.document.remove_body_child(table_id);

        Ok(table)
    }

    /// Remove a row from a table
    pub fn remove_table_row(&mut self, row_id: NodeId) -> Result<TableRow> {
        let row = self.nodes.table_rows.remove(&row_id)
            .ok_or(DocModelError::NodeNotFound(row_id.as_uuid()))?;

        // Remove all cells and their contents
        for &cell_id in row.children() {
            self.remove_table_cell_contents(cell_id);
        }

        // Remove from parent table
        if let Some(parent_id) = row.parent() {
            if let Some(table) = self.nodes.tables.get_mut(&parent_id) {
                table.remove_row(row_id);
            }
        }

        Ok(row)
    }

    /// Remove a cell from a row
    pub fn remove_table_cell(&mut self, cell_id: NodeId) -> Result<TableCell> {
        let cell = self.nodes.table_cells.remove(&cell_id)
            .ok_or(DocModelError::NodeNotFound(cell_id.as_uuid()))?;

        // Remove all child content (paragraphs, etc.)
        for &child_id in cell.children() {
            // Try to remove as paragraph
            if self.nodes.paragraphs.contains_key(&child_id) {
                let _ = self.remove_paragraph(child_id);
            }
        }

        // Remove from parent row
        if let Some(parent_id) = cell.parent() {
            if let Some(row) = self.nodes.table_rows.get_mut(&parent_id) {
                row.remove_cell(cell_id);
            }
        }

        Ok(cell)
    }

    /// Internal helper to remove row contents
    fn remove_table_row_contents(&mut self, row_id: NodeId) {
        if let Some(row) = self.nodes.table_rows.remove(&row_id) {
            for &cell_id in row.children() {
                self.remove_table_cell_contents(cell_id);
            }
        }
    }

    /// Internal helper to remove cell contents
    fn remove_table_cell_contents(&mut self, cell_id: NodeId) {
        if let Some(cell) = self.nodes.table_cells.remove(&cell_id) {
            for &child_id in cell.children() {
                // Remove paragraphs and their runs
                if let Some(para) = self.nodes.paragraphs.remove(&child_id) {
                    for &run_id in para.children() {
                        self.nodes.runs.remove(&run_id);
                    }
                }
            }
        }
    }

    /// Get all tables in the document
    pub fn tables(&self) -> impl Iterator<Item = &Table> {
        self.document.children()
            .iter()
            .filter_map(|id| self.nodes.tables.get(id))
    }

    /// Find the table containing a node
    pub fn find_table_for_node(&self, node_id: NodeId) -> Option<NodeId> {
        // Check if node is a table
        if self.nodes.tables.contains_key(&node_id) {
            return Some(node_id);
        }

        // Check if node is a row
        if let Some(row) = self.nodes.table_rows.get(&node_id) {
            return row.parent();
        }

        // Check if node is a cell
        if let Some(cell) = self.nodes.table_cells.get(&node_id) {
            if let Some(row_id) = cell.parent() {
                if let Some(row) = self.nodes.table_rows.get(&row_id) {
                    return row.parent();
                }
            }
        }

        // Check if node is a paragraph in a cell
        if let Some(para) = self.nodes.paragraphs.get(&node_id) {
            if let Some(cell_id) = para.parent() {
                if let Some(cell) = self.nodes.table_cells.get(&cell_id) {
                    if let Some(row_id) = cell.parent() {
                        if let Some(row) = self.nodes.table_rows.get(&row_id) {
                            return row.parent();
                        }
                    }
                }
            }
        }

        // Check if node is a run in a paragraph in a cell
        if let Some(run) = self.nodes.runs.get(&node_id) {
            if let Some(para_id) = run.parent() {
                return self.find_table_for_node(para_id);
            }
        }

        None
    }

    /// Find the cell containing a node
    pub fn find_cell_for_node(&self, node_id: NodeId) -> Option<NodeId> {
        // Check if node is a cell
        if self.nodes.table_cells.contains_key(&node_id) {
            return Some(node_id);
        }

        // Check if node is a paragraph in a cell
        if let Some(para) = self.nodes.paragraphs.get(&node_id) {
            if let Some(cell_id) = para.parent() {
                if self.nodes.table_cells.contains_key(&cell_id) {
                    return Some(cell_id);
                }
            }
        }

        // Check if node is a run in a paragraph in a cell
        if let Some(run) = self.nodes.runs.get(&node_id) {
            if let Some(para_id) = run.parent() {
                return self.find_cell_for_node(para_id);
            }
        }

        None
    }

    /// Get cell position in table (row index, column index)
    pub fn get_cell_position(&self, cell_id: NodeId) -> Option<(usize, usize)> {
        let cell = self.nodes.table_cells.get(&cell_id)?;
        let row_id = cell.parent()?;
        let row = self.nodes.table_rows.get(&row_id)?;
        let table_id = row.parent()?;
        let table = self.nodes.tables.get(&table_id)?;

        // Find row index
        let row_index = table.children().iter().position(|&id| id == row_id)?;

        // Find column index
        let col_index = row.children().iter().position(|&id| id == cell_id)?;

        Some((row_index, col_index))
    }

    /// Get the cell at a specific position in a table
    pub fn get_cell_at(&self, table_id: NodeId, row_index: usize, col_index: usize) -> Option<NodeId> {
        let table = self.nodes.tables.get(&table_id)?;
        let row_id = table.children().get(row_index)?;
        let row = self.nodes.table_rows.get(row_id)?;
        row.children().get(col_index).copied()
    }

    /// Get the effective cell padding for a cell
    pub fn get_cell_padding(&self, cell_id: NodeId) -> CellPadding {
        // Try cell-level padding first
        if let Some(cell) = self.nodes.table_cells.get(&cell_id) {
            if let Some(padding) = &cell.properties.padding {
                return *padding;
            }

            // Try table-level default padding
            if let Some(row_id) = cell.parent() {
                if let Some(row) = self.nodes.table_rows.get(&row_id) {
                    if let Some(table_id) = row.parent() {
                        if let Some(table) = self.nodes.tables.get(&table_id) {
                            if let Some(padding) = &table.properties.default_cell_padding {
                                return *padding;
                            }
                        }
                    }
                }
            }
        }

        // Return default
        CellPadding::default()
    }

    // =========================================================================
    // Bookmark Methods
    // =========================================================================

    /// Get the bookmark registry
    pub fn bookmark_registry(&self) -> &BookmarkRegistry {
        &self.bookmarks
    }

    /// Get a mutable reference to the bookmark registry
    pub fn bookmark_registry_mut(&mut self) -> &mut BookmarkRegistry {
        &mut self.bookmarks
    }

    /// Insert a bookmark at the current selection
    pub fn insert_bookmark(
        &mut self,
        name: impl Into<String>,
        selection: &Selection,
    ) -> std::result::Result<NodeId, BookmarkValidationError> {
        let bookmark = Bookmark::from_selection(name, selection.anchor, selection.focus);
        self.bookmarks.insert(bookmark)
    }

    /// Insert a point bookmark at a specific position
    pub fn insert_point_bookmark(
        &mut self,
        name: impl Into<String>,
        position: Position,
    ) -> std::result::Result<NodeId, BookmarkValidationError> {
        let bookmark = Bookmark::new_point(name, position);
        self.bookmarks.insert(bookmark)
    }

    /// Insert a range bookmark
    pub fn insert_range_bookmark(
        &mut self,
        name: impl Into<String>,
        start: Position,
        end: Position,
    ) -> std::result::Result<NodeId, BookmarkValidationError> {
        let bookmark = Bookmark::new_range(name, start, end);
        self.bookmarks.insert(bookmark)
    }

    /// Get a bookmark by ID
    pub fn get_bookmark(&self, id: NodeId) -> Option<&Bookmark> {
        self.bookmarks.get(id)
    }

    /// Get a bookmark by name
    pub fn get_bookmark_by_name(&self, name: &str) -> Option<&Bookmark> {
        self.bookmarks.get_by_name(name)
    }

    /// Remove a bookmark by ID
    pub fn remove_bookmark(&mut self, id: NodeId) -> Option<Bookmark> {
        self.bookmarks.remove(id)
    }

    /// Remove a bookmark by name
    pub fn remove_bookmark_by_name(&mut self, name: &str) -> Option<Bookmark> {
        self.bookmarks.remove_by_name(name)
    }

    /// Rename a bookmark
    pub fn rename_bookmark(
        &mut self,
        id: NodeId,
        new_name: impl Into<String>,
    ) -> std::result::Result<(), BookmarkValidationError> {
        self.bookmarks.rename(id, new_name)
    }

    /// Get all bookmarks
    pub fn all_bookmarks(&self) -> impl Iterator<Item = &Bookmark> {
        self.bookmarks.all()
    }

    /// Get bookmark names sorted alphabetically
    pub fn bookmark_names_sorted(&self) -> Vec<&str> {
        self.bookmarks.names_sorted()
    }

    /// Check if a bookmark with the given name exists
    pub fn has_bookmark(&self, name: &str) -> bool {
        self.bookmarks.contains_name(name)
    }

    /// Find bookmarks at a position
    pub fn bookmarks_at_position(&self, position: &Position) -> Vec<&Bookmark> {
        self.bookmarks.find_at_position(position)
    }

    /// Get the selection that would navigate to a bookmark
    pub fn selection_for_bookmark(&self, name: &str) -> Option<Selection> {
        let bookmark = self.bookmarks.get_by_name(name)?;
        match bookmark.range() {
            BookmarkRange::Point(pos) => Some(Selection::collapsed(*pos)),
            BookmarkRange::Range { start, end } => Some(Selection::new(*start, *end)),
        }
    }

    // =========================================================================
    // Comment Methods
    // =========================================================================

    /// Get the comment store
    pub fn comment_store(&self) -> &CommentStore {
        &self.comments
    }

    /// Get a mutable reference to the comment store
    pub fn comment_store_mut(&mut self) -> &mut CommentStore {
        &mut self.comments
    }

    /// Add a comment to the document
    pub fn add_comment(
        &mut self,
        start: Position,
        end: Position,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> std::result::Result<CommentId, CommentValidationError> {
        let author = author.into();
        let content = content.into();

        crate::validate_comment_author(&author)?;
        crate::validate_comment_content(&content)?;

        let comment = Comment::from_selection(start, end, author, content);
        Ok(self.comments.insert(comment))
    }

    /// Add a comment from a selection
    pub fn add_comment_at_selection(
        &mut self,
        selection: &Selection,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> std::result::Result<CommentId, CommentValidationError> {
        self.add_comment(selection.anchor, selection.focus, author, content)
    }

    /// Get a comment by ID
    pub fn get_comment(&self, id: CommentId) -> Option<&Comment> {
        self.comments.get(id)
    }

    /// Get a mutable comment by ID
    pub fn get_comment_mut(&mut self, id: CommentId) -> Option<&mut Comment> {
        self.comments.get_mut(id)
    }

    /// Remove a comment
    pub fn remove_comment(&mut self, id: CommentId) -> Option<Comment> {
        self.comments.remove(id)
    }

    /// Edit a comment's content
    pub fn edit_comment(
        &mut self,
        id: CommentId,
        content: impl Into<String>,
    ) -> std::result::Result<(), CommentValidationError> {
        let content = content.into();
        crate::validate_comment_content(&content)?;

        let comment = self.comments.get_mut(id)
            .ok_or(CommentValidationError::NotFound)?;
        comment.set_content(content);
        Ok(())
    }

    /// Add a reply to a comment
    pub fn add_reply(
        &mut self,
        comment_id: CommentId,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> std::result::Result<ReplyId, CommentValidationError> {
        let author = author.into();
        let content = content.into();

        crate::validate_comment_author(&author)?;
        crate::validate_comment_content(&content)?;

        let comment = self.comments.get_mut(comment_id)
            .ok_or(CommentValidationError::NotFound)?;

        let reply = CommentReply::new(author, content);
        let reply_id = reply.id();
        comment.add_reply(reply);
        Ok(reply_id)
    }

    /// Edit a reply's content
    pub fn edit_reply(
        &mut self,
        comment_id: CommentId,
        reply_id: ReplyId,
        content: impl Into<String>,
    ) -> std::result::Result<(), CommentValidationError> {
        let content = content.into();
        crate::validate_comment_content(&content)?;

        let comment = self.comments.get_mut(comment_id)
            .ok_or(CommentValidationError::NotFound)?;

        let reply = comment.get_reply_mut(reply_id)
            .ok_or(CommentValidationError::ReplyNotFound)?;

        reply.set_content(content);
        Ok(())
    }

    /// Delete a reply
    pub fn delete_reply(
        &mut self,
        comment_id: CommentId,
        reply_id: ReplyId,
    ) -> std::result::Result<CommentReply, CommentValidationError> {
        let comment = self.comments.get_mut(comment_id)
            .ok_or(CommentValidationError::NotFound)?;

        comment.remove_reply(reply_id)
            .ok_or(CommentValidationError::ReplyNotFound)
    }

    /// Resolve a comment
    pub fn resolve_comment(
        &mut self,
        id: CommentId,
        resolved_by: impl Into<String>,
    ) -> std::result::Result<(), CommentValidationError> {
        let comment = self.comments.get_mut(id)
            .ok_or(CommentValidationError::NotFound)?;
        comment.resolve(resolved_by);
        Ok(())
    }

    /// Reopen a resolved comment
    pub fn reopen_comment(&mut self, id: CommentId) -> std::result::Result<(), CommentValidationError> {
        let comment = self.comments.get_mut(id)
            .ok_or(CommentValidationError::NotFound)?;
        comment.reopen();
        Ok(())
    }

    /// Get all comments
    pub fn all_comments(&self) -> impl Iterator<Item = &Comment> {
        self.comments.all()
    }

    /// Find comments at a position
    pub fn comments_at_position(&self, position: &Position) -> Vec<&Comment> {
        self.comments.find_at_position(position)
    }

    /// Find comments in a range
    pub fn comments_in_range(&self, start: &Position, end: &Position) -> Vec<&Comment> {
        self.comments.find_in_range(start, end)
    }

    /// Find comments by author
    pub fn comments_by_author(&self, author: &str) -> Vec<&Comment> {
        self.comments.filter_by_author(author)
    }

    /// Get unresolved comments
    pub fn unresolved_comments(&self) -> Vec<&Comment> {
        self.comments.unresolved()
    }

    /// Get resolved comments
    pub fn resolved_comments(&self) -> Vec<&Comment> {
        self.comments.resolved()
    }

    /// Get comments sorted by date
    pub fn comments_sorted_by_date(&self) -> Vec<&Comment> {
        self.comments.sorted_by_date()
    }

    /// Get comments sorted by position
    pub fn comments_sorted_by_position(&self) -> Vec<&Comment> {
        self.comments.sorted_by_position()
    }

    /// Get the selection that would navigate to a comment
    pub fn selection_for_comment(&self, id: CommentId) -> Option<Selection> {
        let comment = self.comments.get(id)?;
        let anchor = comment.anchor();
        Some(Selection::new(anchor.start, anchor.end))
    }

    /// Get comment count
    pub fn comment_count(&self) -> usize {
        self.comments.len()
    }

    // =========================================================================
    // Footnote and Endnote Methods
    // =========================================================================

    /// Get the note store
    pub fn note_store(&self) -> &NoteStore {
        &self.notes
    }

    /// Get a mutable reference to the note store
    pub fn note_store_mut(&mut self) -> &mut NoteStore {
        &mut self.notes
    }

    /// Insert a footnote at a position
    ///
    /// Creates both the note and a reference at the given position.
    /// Returns the (note_id, reference_node_id) tuple.
    pub fn insert_footnote(
        &mut self,
        position: Position,
        section_id: Option<NodeId>,
    ) -> (NoteId, NodeId) {
        let mut note = Note::footnote();
        note.set_reference_position(position);
        if let Some(sid) = section_id {
            note.set_section(sid);
        }

        let note_id = self.notes.insert_footnote(note);
        let reference = NoteRef::footnote(note_id);
        let ref_id = self.notes.insert_reference(reference);

        // Renumber footnotes
        self.notes.renumber_footnotes();

        (note_id, ref_id)
    }

    /// Insert an endnote at a position
    ///
    /// Creates both the note and a reference at the given position.
    /// Returns the (note_id, reference_node_id) tuple.
    pub fn insert_endnote(
        &mut self,
        position: Position,
        section_id: Option<NodeId>,
    ) -> (NoteId, NodeId) {
        let mut note = Note::endnote();
        note.set_reference_position(position);
        if let Some(sid) = section_id {
            note.set_section(sid);
        }

        let note_id = self.notes.insert_endnote(note);
        let reference = NoteRef::endnote(note_id);
        let ref_id = self.notes.insert_reference(reference);

        // Renumber endnotes
        self.notes.renumber_endnotes();

        (note_id, ref_id)
    }

    /// Get a footnote by ID
    pub fn get_footnote(&self, id: NoteId) -> Option<&Note> {
        self.notes.get_footnote(id)
    }

    /// Get a mutable footnote by ID
    pub fn get_footnote_mut(&mut self, id: NoteId) -> Option<&mut Note> {
        self.notes.get_footnote_mut(id)
    }

    /// Get an endnote by ID
    pub fn get_endnote(&self, id: NoteId) -> Option<&Note> {
        self.notes.get_endnote(id)
    }

    /// Get a mutable endnote by ID
    pub fn get_endnote_mut(&mut self, id: NoteId) -> Option<&mut Note> {
        self.notes.get_endnote_mut(id)
    }

    /// Get a note reference by node ID
    pub fn get_note_reference(&self, ref_id: NodeId) -> Option<&NoteRef> {
        self.notes.get_reference(ref_id)
    }

    /// Delete a footnote and its reference
    pub fn delete_footnote(&mut self, note_id: NoteId) -> Option<Note> {
        // Remove the reference first
        if let Some(ref_id) = self.notes.find_reference_id_for_note(note_id) {
            self.notes.remove_reference(ref_id);
        }

        let note = self.notes.remove_footnote(note_id);

        // Renumber remaining footnotes
        self.notes.renumber_footnotes();

        note
    }

    /// Delete an endnote and its reference
    pub fn delete_endnote(&mut self, note_id: NoteId) -> Option<Note> {
        // Remove the reference first
        if let Some(ref_id) = self.notes.find_reference_id_for_note(note_id) {
            self.notes.remove_reference(ref_id);
        }

        let note = self.notes.remove_endnote(note_id);

        // Renumber remaining endnotes
        self.notes.renumber_endnotes();

        note
    }

    /// Delete a note (footnote or endnote) by ID
    pub fn delete_note(&mut self, note_id: NoteId, note_type: NoteType) -> Option<Note> {
        match note_type {
            NoteType::Footnote => self.delete_footnote(note_id),
            NoteType::Endnote => self.delete_endnote(note_id),
        }
    }

    /// Add content paragraph to a footnote
    pub fn add_footnote_content(&mut self, note_id: NoteId, para_id: NodeId) -> bool {
        if let Some(note) = self.notes.get_footnote_mut(note_id) {
            note.add_content(para_id);
            true
        } else {
            false
        }
    }

    /// Add content paragraph to an endnote
    pub fn add_endnote_content(&mut self, note_id: NoteId, para_id: NodeId) -> bool {
        if let Some(note) = self.notes.get_endnote_mut(note_id) {
            note.add_content(para_id);
            true
        } else {
            false
        }
    }

    /// Convert a footnote to an endnote
    pub fn convert_footnote_to_endnote(&mut self, note_id: NoteId) -> Option<NoteId> {
        let new_id = self.notes.convert_note(note_id)?;

        // Renumber both types
        self.notes.renumber_footnotes();
        self.notes.renumber_endnotes();

        Some(new_id)
    }

    /// Convert an endnote to a footnote
    pub fn convert_endnote_to_footnote(&mut self, note_id: NoteId) -> Option<NoteId> {
        // Same operation, just different source
        self.convert_footnote_to_endnote(note_id)
    }

    /// Get footnote properties for a section (or default)
    pub fn get_footnote_properties(&self, section_id: Option<NodeId>) -> &FootnoteProperties {
        self.notes.get_footnote_props(section_id)
    }

    /// Set footnote properties for a section (or default)
    pub fn set_footnote_properties(
        &mut self,
        section_id: Option<NodeId>,
        props: FootnoteProperties,
    ) {
        self.notes.set_footnote_props(section_id, props);
        self.notes.renumber_footnotes();
    }

    /// Get endnote properties for a section (or default)
    pub fn get_endnote_properties(&self, section_id: Option<NodeId>) -> &EndnoteProperties {
        self.notes.get_endnote_props(section_id)
    }

    /// Set endnote properties for a section (or default)
    pub fn set_endnote_properties(&mut self, section_id: Option<NodeId>, props: EndnoteProperties) {
        self.notes.set_endnote_props(section_id, props);
        self.notes.renumber_endnotes();
    }

    /// Get all footnotes sorted by document order for a page
    pub fn get_footnotes_on_page(&self, page: usize) -> Vec<&Note> {
        self.notes.get_footnotes_sorted(page)
    }

    /// Get all endnotes sorted by document order
    pub fn get_endnotes_sorted(&self, section_id: Option<NodeId>) -> Vec<&Note> {
        self.notes.get_endnotes_sorted(section_id)
    }

    /// Find the note ID for a reference node ID
    pub fn find_note_for_reference(&self, ref_id: NodeId) -> Option<(NoteId, NoteType)> {
        let reference = self.notes.get_reference(ref_id)?;
        Some((reference.note_id(), reference.note_type))
    }

    /// Find the reference node ID for a note ID
    pub fn find_reference_for_note(&self, note_id: NoteId) -> Option<NodeId> {
        self.notes.find_reference_id_for_note(note_id)
    }

    /// Get the position to navigate to for a note (the reference position)
    pub fn get_note_reference_position(&self, note_id: NoteId, note_type: NoteType) -> Option<Position> {
        let note = self.notes.get_note(note_id, note_type)?;
        note.reference_position
    }

    /// Check if the document has any footnotes
    pub fn has_footnotes(&self) -> bool {
        self.notes.has_footnotes()
    }

    /// Check if the document has any endnotes
    pub fn has_endnotes(&self) -> bool {
        self.notes.has_endnotes()
    }

    /// Get footnote count
    pub fn footnote_count(&self) -> usize {
        self.notes.footnote_count()
    }

    /// Get endnote count
    pub fn endnote_count(&self) -> usize {
        self.notes.endnote_count()
    }
}

impl Default for DocumentTree {
    fn default() -> Self {
        Self::with_empty_paragraph()
    }
}

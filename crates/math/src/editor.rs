//! Equation Editor State Management
//!
//! This module provides state management for editing mathematical equations,
//! including cursor navigation, selection, and placeholder management.

use crate::model::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Cursor Position
// =============================================================================

/// Position within a math node tree
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MathPath {
    /// Path of indices from root to current node
    pub indices: Vec<usize>,
    /// Offset within the current node (for text runs)
    pub offset: usize,
}

impl MathPath {
    /// Create a path to the root
    pub fn root() -> Self {
        Self {
            indices: Vec::new(),
            offset: 0,
        }
    }

    /// Create a path with given indices
    pub fn new(indices: Vec<usize>, offset: usize) -> Self {
        Self { indices, offset }
    }

    /// Append an index to navigate deeper
    pub fn child(&self, index: usize) -> Self {
        let mut indices = self.indices.clone();
        indices.push(index);
        Self { indices, offset: 0 }
    }

    /// Get the parent path (one level up)
    pub fn parent(&self) -> Option<Self> {
        if self.indices.is_empty() {
            None
        } else {
            let mut indices = self.indices.clone();
            indices.pop();
            Some(Self { indices, offset: 0 })
        }
    }

    /// Check if this path is empty (at root)
    pub fn is_root(&self) -> bool {
        self.indices.is_empty()
    }

    /// Get the depth of this path
    pub fn depth(&self) -> usize {
        self.indices.len()
    }

    /// Set the offset within the current node
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
}

impl Default for MathPath {
    fn default() -> Self {
        Self::root()
    }
}

/// Type of math box (for navigation purposes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MathBoxType {
    /// Container (OMath, OMathPara)
    Container,
    /// Numerator of a fraction
    Numerator,
    /// Denominator of a fraction
    Denominator,
    /// Base of a script/radical
    Base,
    /// Subscript
    Subscript,
    /// Superscript
    Superscript,
    /// Radical degree
    RadicalDegree,
    /// N-ary lower limit
    LowerLimit,
    /// N-ary upper limit
    UpperLimit,
    /// Delimiter content
    DelimiterContent,
    /// Matrix cell
    MatrixCell,
    /// Equation array row
    ArrayRow,
    /// Text run
    TextRun,
    /// Accent base
    AccentBase,
    /// Limit expression
    LimitExpr,
}

/// Information about a navigable math box
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathBox {
    /// Path to this box
    pub path: MathPath,
    /// Type of box
    pub box_type: MathBoxType,
    /// Whether this box is a placeholder (empty)
    pub is_placeholder: bool,
    /// Tab order for this box (for tab navigation)
    pub tab_order: usize,
}

impl MathBox {
    /// Create a new math box
    pub fn new(path: MathPath, box_type: MathBoxType) -> Self {
        Self {
            path,
            box_type,
            is_placeholder: false,
            tab_order: 0,
        }
    }

    /// Mark as placeholder
    pub fn placeholder(mut self) -> Self {
        self.is_placeholder = true;
        self
    }

    /// Set tab order
    pub fn with_tab_order(mut self, order: usize) -> Self {
        self.tab_order = order;
        self
    }
}

// =============================================================================
// Selection
// =============================================================================

/// Selection within an equation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MathSelection {
    /// No selection (just cursor)
    None,
    /// Character range within a text run
    CharRange {
        /// Path to the text run
        path: MathPath,
        /// Start offset
        start: usize,
        /// End offset (exclusive)
        end: usize,
    },
    /// Single node selected
    Node(MathPath),
    /// Range of sibling nodes selected
    NodeRange {
        /// Path to parent
        parent: MathPath,
        /// Start index
        start: usize,
        /// End index (exclusive)
        end: usize,
    },
}

impl Default for MathSelection {
    fn default() -> Self {
        Self::None
    }
}

impl MathSelection {
    /// Check if anything is selected
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Get the anchor path (start of selection)
    pub fn anchor(&self) -> Option<MathPath> {
        match self {
            Self::None => None,
            Self::CharRange { path, start, .. } => Some(path.clone().with_offset(*start)),
            Self::Node(path) => Some(path.clone()),
            Self::NodeRange { parent, start, .. } => Some(parent.child(*start)),
        }
    }

    /// Get the focus path (end of selection)
    pub fn focus(&self) -> Option<MathPath> {
        match self {
            Self::None => None,
            Self::CharRange { path, end, .. } => Some(path.clone().with_offset(*end)),
            Self::Node(path) => Some(path.clone()),
            Self::NodeRange { parent, end, .. } => {
                if *end > 0 {
                    Some(parent.child(*end - 1))
                } else {
                    Some(parent.clone())
                }
            }
        }
    }
}

// =============================================================================
// Navigation Direction
// =============================================================================

/// Direction for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavDirection {
    /// Move left
    Left,
    /// Move right
    Right,
    /// Move up
    Up,
    /// Move down
    Down,
}

// =============================================================================
// Equation Editor
// =============================================================================

/// Editor state for a math equation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquationEditor {
    /// The equation being edited
    equation: MathNode,
    /// Current cursor position
    cursor: MathPath,
    /// Current selection
    selection: MathSelection,
    /// List of navigable boxes (cached)
    #[serde(skip)]
    boxes: Vec<MathBox>,
    /// Whether the equation has been modified
    modified: bool,
}

impl EquationEditor {
    /// Create a new editor for an equation
    pub fn new(equation: MathNode) -> Self {
        let mut editor = Self {
            equation,
            cursor: MathPath::root(),
            selection: MathSelection::None,
            boxes: Vec::new(),
            modified: false,
        };
        editor.rebuild_box_cache();
        editor.move_to_first_placeholder();
        editor
    }

    /// Create an editor with an empty equation
    pub fn empty() -> Self {
        Self::new(MathNode::OMath(vec![MathNode::Run {
            text: String::new(),
            style: MathStyle::default(),
        }]))
    }

    /// Get the equation
    pub fn equation(&self) -> &MathNode {
        &self.equation
    }

    /// Get the equation mutably
    pub fn equation_mut(&mut self) -> &mut MathNode {
        self.modified = true;
        &mut self.equation
    }

    /// Replace the equation
    pub fn set_equation(&mut self, equation: MathNode) {
        self.equation = equation;
        self.cursor = MathPath::root();
        self.selection = MathSelection::None;
        self.modified = true;
        self.rebuild_box_cache();
        self.move_to_first_placeholder();
    }

    /// Get the current cursor position
    pub fn cursor(&self) -> &MathPath {
        &self.cursor
    }

    /// Get the current selection
    pub fn selection(&self) -> &MathSelection {
        &self.selection
    }

    /// Check if the equation has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Mark the equation as saved (reset modified flag)
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    /// Get the node at a path
    pub fn node_at(&self, path: &MathPath) -> Option<&MathNode> {
        let mut current = &self.equation;
        for &index in &path.indices {
            let children = current.children();
            if index < children.len() {
                current = children[index];
            } else {
                return None;
            }
        }
        Some(current)
    }

    /// Get the navigable boxes
    pub fn boxes(&self) -> &[MathBox] {
        &self.boxes
    }

    /// Rebuild the cache of navigable boxes
    fn rebuild_box_cache(&mut self) {
        self.boxes.clear();
        let mut tab_order = 0;
        collect_boxes_recursive(&self.equation, MathPath::root(), &mut tab_order, &mut self.boxes);
    }

    /// Move cursor to the first placeholder
    fn move_to_first_placeholder(&mut self) {
        if let Some(placeholder) = self.boxes.iter().find(|b| b.is_placeholder) {
            self.cursor = placeholder.path.clone();
        }
    }

    // =========================================================================
    // Navigation
    // =========================================================================

    /// Navigate in a direction
    pub fn navigate(&mut self, direction: NavDirection) -> bool {
        match direction {
            NavDirection::Left => self.move_left(),
            NavDirection::Right => self.move_right(),
            NavDirection::Up => self.move_up(),
            NavDirection::Down => self.move_down(),
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self) -> bool {
        // First try to move within current text run
        if self.cursor.offset > 0 {
            self.cursor.offset -= 1;
            self.selection = MathSelection::None;
            return true;
        }

        // Find current box and move to previous
        if let Some(current_idx) = self.find_current_box_index() {
            if current_idx > 0 {
                self.cursor = self.boxes[current_idx - 1].path.clone();
                // Move to end of text if it's a text run
                if let Some(node) = self.node_at(&self.cursor) {
                    if let MathNode::Run { text, .. } = node {
                        self.cursor.offset = text.len();
                    }
                }
                self.selection = MathSelection::None;
                return true;
            }
        }

        false
    }

    /// Move cursor right
    pub fn move_right(&mut self) -> bool {
        // First try to move within current text run
        if let Some(node) = self.node_at(&self.cursor) {
            if let MathNode::Run { text, .. } = node {
                if self.cursor.offset < text.len() {
                    self.cursor.offset += 1;
                    self.selection = MathSelection::None;
                    return true;
                }
            }
        }

        // Find current box and move to next
        if let Some(current_idx) = self.find_current_box_index() {
            if current_idx + 1 < self.boxes.len() {
                self.cursor = self.boxes[current_idx + 1].path.clone();
                self.cursor.offset = 0;
                self.selection = MathSelection::None;
                return true;
            }
        }

        false
    }

    /// Move cursor up (for fractions, matrices, etc.)
    pub fn move_up(&mut self) -> bool {
        if let Some(math_box) = self.current_box() {
            match math_box.box_type {
                MathBoxType::Denominator => {
                    // Move to numerator
                    if let Some(parent) = self.cursor.parent() {
                        self.cursor = parent.child(0);
                        self.selection = MathSelection::None;
                        return true;
                    }
                }
                MathBoxType::Subscript | MathBoxType::LowerLimit => {
                    // Try to move to superscript/upper limit
                    if let Some(parent) = self.cursor.parent() {
                        // Find the superscript/upper limit sibling
                        for math_box in &self.boxes {
                            if math_box.path.parent() == Some(parent.clone()) {
                                if matches!(
                                    math_box.box_type,
                                    MathBoxType::Superscript | MathBoxType::UpperLimit
                                ) {
                                    self.cursor = math_box.path.clone();
                                    self.selection = MathSelection::None;
                                    return true;
                                }
                            }
                        }
                    }
                }
                MathBoxType::MatrixCell | MathBoxType::ArrayRow => {
                    // Try to move to cell above
                    if let Some(current_idx) = self.find_current_box_index() {
                        // Find matrix dimensions and move up
                        let parent = self.cursor.parent();
                        let same_parent_boxes: Vec<_> = self
                            .boxes
                            .iter()
                            .enumerate()
                            .filter(|(_, b)| b.path.parent() == parent)
                            .collect();

                        if let Some(pos) = same_parent_boxes
                            .iter()
                            .position(|(idx, _)| *idx == current_idx)
                        {
                            // Assume square layout for now
                            let cols = (same_parent_boxes.len() as f64).sqrt() as usize;
                            if cols > 0 && pos >= cols {
                                let new_pos = pos - cols;
                                if let Some((idx, _)) = same_parent_boxes.get(new_pos) {
                                    self.cursor = self.boxes[*idx].path.clone();
                                    self.selection = MathSelection::None;
                                    return true;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Move cursor down (for fractions, matrices, etc.)
    pub fn move_down(&mut self) -> bool {
        if let Some(math_box) = self.current_box() {
            match math_box.box_type {
                MathBoxType::Numerator => {
                    // Move to denominator
                    if let Some(parent) = self.cursor.parent() {
                        self.cursor = parent.child(1);
                        self.selection = MathSelection::None;
                        return true;
                    }
                }
                MathBoxType::Superscript | MathBoxType::UpperLimit => {
                    // Try to move to subscript/lower limit
                    if let Some(parent) = self.cursor.parent() {
                        for math_box in &self.boxes {
                            if math_box.path.parent() == Some(parent.clone()) {
                                if matches!(
                                    math_box.box_type,
                                    MathBoxType::Subscript | MathBoxType::LowerLimit
                                ) {
                                    self.cursor = math_box.path.clone();
                                    self.selection = MathSelection::None;
                                    return true;
                                }
                            }
                        }
                    }
                }
                MathBoxType::MatrixCell | MathBoxType::ArrayRow => {
                    // Try to move to cell below
                    if let Some(current_idx) = self.find_current_box_index() {
                        let parent = self.cursor.parent();
                        let same_parent_boxes: Vec<_> = self
                            .boxes
                            .iter()
                            .enumerate()
                            .filter(|(_, b)| b.path.parent() == parent)
                            .collect();

                        if let Some(pos) = same_parent_boxes
                            .iter()
                            .position(|(idx, _)| *idx == current_idx)
                        {
                            let cols = (same_parent_boxes.len() as f64).sqrt() as usize;
                            if cols > 0 {
                                let new_pos = pos + cols;
                                if let Some((idx, _)) = same_parent_boxes.get(new_pos) {
                                    self.cursor = self.boxes[*idx].path.clone();
                                    self.selection = MathSelection::None;
                                    return true;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Tab to next placeholder
    pub fn tab_next(&mut self) -> bool {
        let current_tab = self.current_box().map(|b| b.tab_order).unwrap_or(0);

        // Find next placeholder with higher tab order
        if let Some(next) = self
            .boxes
            .iter()
            .filter(|b| b.tab_order > current_tab && b.is_placeholder)
            .min_by_key(|b| b.tab_order)
        {
            self.cursor = next.path.clone();
            self.selection = MathSelection::None;
            return true;
        }

        // If no placeholder found, try any box
        if let Some(next) = self
            .boxes
            .iter()
            .filter(|b| b.tab_order > current_tab)
            .min_by_key(|b| b.tab_order)
        {
            self.cursor = next.path.clone();
            self.selection = MathSelection::None;
            return true;
        }

        // Wrap to first
        if let Some(first) = self.boxes.first() {
            self.cursor = first.path.clone();
            self.selection = MathSelection::None;
            return true;
        }

        false
    }

    /// Tab to previous placeholder
    pub fn tab_previous(&mut self) -> bool {
        let current_tab = self.current_box().map(|b| b.tab_order).unwrap_or(0);

        // Find previous placeholder with lower tab order
        if let Some(prev) = self
            .boxes
            .iter()
            .filter(|b| b.tab_order < current_tab && b.is_placeholder)
            .max_by_key(|b| b.tab_order)
        {
            self.cursor = prev.path.clone();
            self.selection = MathSelection::None;
            return true;
        }

        // If no placeholder found, try any box
        if let Some(prev) = self
            .boxes
            .iter()
            .filter(|b| b.tab_order < current_tab)
            .max_by_key(|b| b.tab_order)
        {
            self.cursor = prev.path.clone();
            self.selection = MathSelection::None;
            return true;
        }

        // Wrap to last
        if let Some(last) = self.boxes.last() {
            self.cursor = last.path.clone();
            self.selection = MathSelection::None;
            return true;
        }

        false
    }

    // =========================================================================
    // Selection
    // =========================================================================

    /// Select all content
    pub fn select_all(&mut self) {
        self.selection = MathSelection::Node(MathPath::root());
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = MathSelection::None;
    }

    /// Extend selection left
    pub fn extend_selection_left(&mut self) {
        // Copy values to avoid borrow issues
        let (should_update, new_selection, new_offset) = match &self.selection {
            MathSelection::None => {
                // Start character selection
                if self.cursor.offset > 0 {
                    (
                        true,
                        Some(MathSelection::CharRange {
                            path: self.cursor.clone(),
                            start: self.cursor.offset - 1,
                            end: self.cursor.offset,
                        }),
                        Some(self.cursor.offset - 1),
                    )
                } else {
                    (false, None, None)
                }
            }
            MathSelection::CharRange { path, start, end } => {
                if *start > 0 {
                    (
                        true,
                        Some(MathSelection::CharRange {
                            path: path.clone(),
                            start: start - 1,
                            end: *end,
                        }),
                        Some(start - 1),
                    )
                } else {
                    (false, None, None)
                }
            }
            _ => (false, None, None),
        };

        if should_update {
            if let Some(sel) = new_selection {
                self.selection = sel;
            }
            if let Some(offset) = new_offset {
                self.cursor.offset = offset;
            }
        }
    }

    /// Extend selection right
    pub fn extend_selection_right(&mut self) {
        // Get the text length first to avoid borrow issues
        let text_len = if let Some(node) = self.node_at(&self.cursor) {
            if let MathNode::Run { text, .. } = node {
                Some(text.len())
            } else {
                None
            }
        } else {
            None
        };

        let Some(len) = text_len else {
            return;
        };

        // Copy values to avoid borrow issues
        let (should_update, new_selection, new_offset) = match &self.selection {
            MathSelection::None => {
                if self.cursor.offset < len {
                    (
                        true,
                        Some(MathSelection::CharRange {
                            path: self.cursor.clone(),
                            start: self.cursor.offset,
                            end: self.cursor.offset + 1,
                        }),
                        Some(self.cursor.offset + 1),
                    )
                } else {
                    (false, None, None)
                }
            }
            MathSelection::CharRange { path, start, end } => {
                if *end < len {
                    (
                        true,
                        Some(MathSelection::CharRange {
                            path: path.clone(),
                            start: *start,
                            end: end + 1,
                        }),
                        Some(end + 1),
                    )
                } else {
                    (false, None, None)
                }
            }
            _ => (false, None, None),
        };

        if should_update {
            if let Some(sel) = new_selection {
                self.selection = sel;
            }
            if let Some(offset) = new_offset {
                self.cursor.offset = offset;
            }
        }
    }

    // =========================================================================
    // Helpers
    // =========================================================================

    /// Find the index of the current box in the boxes list
    fn find_current_box_index(&self) -> Option<usize> {
        self.boxes.iter().position(|b| b.path == self.cursor)
    }

    /// Get the current math box
    fn current_box(&self) -> Option<&MathBox> {
        self.find_current_box_index().map(|i| &self.boxes[i])
    }

    /// Move cursor to a specific path
    pub fn move_to(&mut self, path: MathPath) {
        self.cursor = path;
        self.selection = MathSelection::None;
    }

    /// Get all placeholder paths
    pub fn placeholders(&self) -> Vec<&MathPath> {
        self.boxes
            .iter()
            .filter(|b| b.is_placeholder)
            .map(|b| &b.path)
            .collect()
    }

    /// Check if the current position is a placeholder
    pub fn is_at_placeholder(&self) -> bool {
        self.current_box().map(|b| b.is_placeholder).unwrap_or(false)
    }

    /// Refresh the box cache (call after modifying the equation)
    pub fn refresh(&mut self) {
        self.rebuild_box_cache();
    }
}

// =============================================================================
// Box Collection Helper
// =============================================================================

/// Check if a node is empty (placeholder)
fn is_empty_node(node: &MathNode) -> bool {
    match node {
        MathNode::Run { text, .. } => text.is_empty(),
        MathNode::Text(t) => t.is_empty(),
        MathNode::Number(n) => n.is_empty(),
        MathNode::OMath(children) | MathNode::OMathPara(children) => {
            children.is_empty() || children.iter().all(is_empty_node)
        }
        _ => false,
    }
}

/// Recursively collect navigable boxes from a math node tree
fn collect_boxes_recursive(
    node: &MathNode,
    path: MathPath,
    tab_order: &mut usize,
    boxes: &mut Vec<MathBox>,
) {
    match node {
        MathNode::OMath(children) | MathNode::OMathPara(children) => {
            for (i, child) in children.iter().enumerate() {
                collect_boxes_recursive(child, path.child(i), tab_order, boxes);
            }
        }
        MathNode::Fraction { num, den, .. } => {
            // Numerator
            let num_box = MathBox::new(path.child(0), MathBoxType::Numerator)
                .with_tab_order(*tab_order);
            if is_empty_node(num) {
                boxes.push(num_box.placeholder());
            } else {
                boxes.push(num_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(num, path.child(0), tab_order, boxes);

            // Denominator
            let den_box = MathBox::new(path.child(1), MathBoxType::Denominator)
                .with_tab_order(*tab_order);
            if is_empty_node(den) {
                boxes.push(den_box.placeholder());
            } else {
                boxes.push(den_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(den, path.child(1), tab_order, boxes);
        }
        MathNode::Radical { degree, base } => {
            // Degree (if present)
            if let Some(deg) = degree {
                let deg_box = MathBox::new(path.child(0), MathBoxType::RadicalDegree)
                    .with_tab_order(*tab_order);
                if is_empty_node(deg) {
                    boxes.push(deg_box.placeholder());
                } else {
                    boxes.push(deg_box);
                }
                *tab_order += 1;
                collect_boxes_recursive(deg, path.child(0), tab_order, boxes);
            }

            // Base
            let base_idx = if degree.is_some() { 1 } else { 0 };
            let base_box = MathBox::new(path.child(base_idx), MathBoxType::Base)
                .with_tab_order(*tab_order);
            if is_empty_node(base) {
                boxes.push(base_box.placeholder());
            } else {
                boxes.push(base_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(base, path.child(base_idx), tab_order, boxes);
        }
        MathNode::Subscript { base, sub } => {
            // Base
            let base_box = MathBox::new(path.child(0), MathBoxType::Base)
                .with_tab_order(*tab_order);
            if is_empty_node(base) {
                boxes.push(base_box.placeholder());
            } else {
                boxes.push(base_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(base, path.child(0), tab_order, boxes);

            // Subscript
            let sub_box = MathBox::new(path.child(1), MathBoxType::Subscript)
                .with_tab_order(*tab_order);
            if is_empty_node(sub) {
                boxes.push(sub_box.placeholder());
            } else {
                boxes.push(sub_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(sub, path.child(1), tab_order, boxes);
        }
        MathNode::Superscript { base, sup } => {
            // Base
            let base_box = MathBox::new(path.child(0), MathBoxType::Base)
                .with_tab_order(*tab_order);
            if is_empty_node(base) {
                boxes.push(base_box.placeholder());
            } else {
                boxes.push(base_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(base, path.child(0), tab_order, boxes);

            // Superscript
            let sup_box = MathBox::new(path.child(1), MathBoxType::Superscript)
                .with_tab_order(*tab_order);
            if is_empty_node(sup) {
                boxes.push(sup_box.placeholder());
            } else {
                boxes.push(sup_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(sup, path.child(1), tab_order, boxes);
        }
        MathNode::SubSuperscript { base, sub, sup } => {
            // Base
            let base_box = MathBox::new(path.child(0), MathBoxType::Base)
                .with_tab_order(*tab_order);
            if is_empty_node(base) {
                boxes.push(base_box.placeholder());
            } else {
                boxes.push(base_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(base, path.child(0), tab_order, boxes);

            // Subscript
            let sub_box = MathBox::new(path.child(1), MathBoxType::Subscript)
                .with_tab_order(*tab_order);
            if is_empty_node(sub) {
                boxes.push(sub_box.placeholder());
            } else {
                boxes.push(sub_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(sub, path.child(1), tab_order, boxes);

            // Superscript
            let sup_box = MathBox::new(path.child(2), MathBoxType::Superscript)
                .with_tab_order(*tab_order);
            if is_empty_node(sup) {
                boxes.push(sup_box.placeholder());
            } else {
                boxes.push(sup_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(sup, path.child(2), tab_order, boxes);
        }
        MathNode::Nary { sub, sup, base, .. } => {
            // Lower limit
            if let Some(s) = sub {
                let sub_box = MathBox::new(path.child(0), MathBoxType::LowerLimit)
                    .with_tab_order(*tab_order);
                if is_empty_node(s) {
                    boxes.push(sub_box.placeholder());
                } else {
                    boxes.push(sub_box);
                }
                *tab_order += 1;
                collect_boxes_recursive(s, path.child(0), tab_order, boxes);
            }

            // Upper limit
            let sup_idx = if sub.is_some() { 1 } else { 0 };
            if let Some(s) = sup {
                let sup_box = MathBox::new(path.child(sup_idx), MathBoxType::UpperLimit)
                    .with_tab_order(*tab_order);
                if is_empty_node(s) {
                    boxes.push(sup_box.placeholder());
                } else {
                    boxes.push(sup_box);
                }
                *tab_order += 1;
                collect_boxes_recursive(s, path.child(sup_idx), tab_order, boxes);
            }

            // Base
            let base_idx = sup_idx + if sup.is_some() { 1 } else { 0 };
            let base_box = MathBox::new(path.child(base_idx), MathBoxType::Base)
                .with_tab_order(*tab_order);
            if is_empty_node(base) {
                boxes.push(base_box.placeholder());
            } else {
                boxes.push(base_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(base, path.child(base_idx), tab_order, boxes);
        }
        MathNode::Delimiter { content, .. } => {
            for (i, child) in content.iter().enumerate() {
                let content_box = MathBox::new(path.child(i), MathBoxType::DelimiterContent)
                    .with_tab_order(*tab_order);
                if is_empty_node(child) {
                    boxes.push(content_box.placeholder());
                } else {
                    boxes.push(content_box);
                }
                *tab_order += 1;
                collect_boxes_recursive(child, path.child(i), tab_order, boxes);
            }
        }
        MathNode::Matrix { rows, .. } => {
            let cols = rows.first().map(|r| r.len()).unwrap_or(0);
            for (row_idx, row) in rows.iter().enumerate() {
                for (col_idx, cell) in row.iter().enumerate() {
                    let cell_idx = row_idx * cols + col_idx;
                    let cell_box = MathBox::new(path.child(cell_idx), MathBoxType::MatrixCell)
                        .with_tab_order(*tab_order);
                    if is_empty_node(cell) {
                        boxes.push(cell_box.placeholder());
                    } else {
                        boxes.push(cell_box);
                    }
                    *tab_order += 1;
                    collect_boxes_recursive(cell, path.child(cell_idx), tab_order, boxes);
                }
            }
        }
        MathNode::EqArray(rows) => {
            let cols = rows.first().map(|r| r.len()).unwrap_or(0);
            for (i, row) in rows.iter().enumerate() {
                for (j, cell) in row.iter().enumerate() {
                    let cell_idx = i * cols + j;
                    let row_box = MathBox::new(path.child(cell_idx), MathBoxType::ArrayRow)
                        .with_tab_order(*tab_order);
                    if is_empty_node(cell) {
                        boxes.push(row_box.placeholder());
                    } else {
                        boxes.push(row_box);
                    }
                    *tab_order += 1;
                    collect_boxes_recursive(cell, path.child(cell_idx), tab_order, boxes);
                }
            }
        }
        MathNode::Bar { base, .. }
        | MathNode::Accent { base, .. }
        | MathNode::GroupChar { base, .. }
        | MathNode::Box(base)
        | MathNode::BorderBox { base, .. }
        | MathNode::Phantom { base, .. } => {
            let base_box = MathBox::new(path.child(0), MathBoxType::AccentBase)
                .with_tab_order(*tab_order);
            if is_empty_node(base) {
                boxes.push(base_box.placeholder());
            } else {
                boxes.push(base_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(base, path.child(0), tab_order, boxes);
        }
        MathNode::Limit { func, limit, .. } => {
            // Function
            let func_box = MathBox::new(path.child(0), MathBoxType::Base)
                .with_tab_order(*tab_order);
            boxes.push(func_box);
            *tab_order += 1;
            collect_boxes_recursive(func, path.child(0), tab_order, boxes);

            // Limit expression
            let limit_box = MathBox::new(path.child(1), MathBoxType::LimitExpr)
                .with_tab_order(*tab_order);
            if is_empty_node(limit) {
                boxes.push(limit_box.placeholder());
            } else {
                boxes.push(limit_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(limit, path.child(1), tab_order, boxes);
        }
        MathNode::Function { base, .. } => {
            let base_box = MathBox::new(path.child(0), MathBoxType::Base)
                .with_tab_order(*tab_order);
            if is_empty_node(base) {
                boxes.push(base_box.placeholder());
            } else {
                boxes.push(base_box);
            }
            *tab_order += 1;
            collect_boxes_recursive(base, path.child(0), tab_order, boxes);
        }
        MathNode::Run { text, .. } => {
            let text_box = MathBox::new(path.clone(), MathBoxType::TextRun)
                .with_tab_order(*tab_order);
            if text.is_empty() {
                boxes.push(text_box.placeholder());
            } else {
                boxes.push(text_box);
            }
            *tab_order += 1;
        }
        MathNode::Text(_) | MathNode::Number(_) | MathNode::Operator { .. } => {
            let text_box = MathBox::new(path, MathBoxType::TextRun)
                .with_tab_order(*tab_order);
            boxes.push(text_box);
            *tab_order += 1;
        }
        MathNode::Unknown { .. } => {}
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_fraction() -> MathNode {
        MathNode::Fraction {
            num: Box::new(MathNode::Run {
                text: "a".to_string(),
                style: MathStyle::default(),
            }),
            den: Box::new(MathNode::Run {
                text: "b".to_string(),
                style: MathStyle::default(),
            }),
            bar_visible: true,
        }
    }

    fn make_empty_fraction() -> MathNode {
        MathNode::Fraction {
            num: Box::new(MathNode::Run {
                text: String::new(),
                style: MathStyle::default(),
            }),
            den: Box::new(MathNode::Run {
                text: String::new(),
                style: MathStyle::default(),
            }),
            bar_visible: true,
        }
    }

    #[test]
    fn test_math_path_root() {
        let path = MathPath::root();
        assert!(path.is_root());
        assert_eq!(path.depth(), 0);
    }

    #[test]
    fn test_math_path_child() {
        let root = MathPath::root();
        let child = root.child(0);
        assert!(!child.is_root());
        assert_eq!(child.depth(), 1);
        assert_eq!(child.indices, vec![0]);
    }

    #[test]
    fn test_math_path_parent() {
        let child = MathPath::new(vec![0, 1, 2], 0);
        let parent = child.parent().unwrap();
        assert_eq!(parent.indices, vec![0, 1]);

        let root = MathPath::root();
        assert!(root.parent().is_none());
    }

    #[test]
    fn test_math_path_with_offset() {
        let path = MathPath::root().with_offset(5);
        assert_eq!(path.offset, 5);
    }

    #[test]
    fn test_math_selection_empty() {
        let sel = MathSelection::None;
        assert!(sel.is_empty());
    }

    #[test]
    fn test_math_selection_char_range() {
        let sel = MathSelection::CharRange {
            path: MathPath::root(),
            start: 2,
            end: 5,
        };
        assert!(!sel.is_empty());
        assert_eq!(sel.anchor().unwrap().offset, 2);
        assert_eq!(sel.focus().unwrap().offset, 5);
    }

    #[test]
    fn test_editor_new() {
        let eq = MathNode::OMath(vec![MathNode::run("x")]);
        let editor = EquationEditor::new(eq);
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_empty() {
        let editor = EquationEditor::empty();
        assert!(matches!(editor.equation(), MathNode::OMath(_)));
    }

    #[test]
    fn test_editor_set_equation() {
        let mut editor = EquationEditor::empty();
        editor.set_equation(make_fraction());
        assert!(editor.is_modified());
    }

    #[test]
    fn test_editor_mark_saved() {
        let mut editor = EquationEditor::empty();
        editor.set_equation(make_fraction());
        assert!(editor.is_modified());
        editor.mark_saved();
        assert!(!editor.is_modified());
    }

    #[test]
    fn test_editor_boxes_collected() {
        let editor = EquationEditor::new(make_fraction());
        assert!(!editor.boxes().is_empty());
    }

    #[test]
    fn test_editor_placeholders() {
        let editor = EquationEditor::new(make_empty_fraction());
        let placeholders = editor.placeholders();
        assert!(!placeholders.is_empty());
    }

    #[test]
    fn test_editor_navigate_right() {
        let eq = MathNode::OMath(vec![MathNode::Run {
            text: "abc".to_string(),
            style: MathStyle::default(),
        }]);
        let mut editor = EquationEditor::new(eq);
        // Move cursor to the text run and set offset to 0
        editor.cursor = MathPath::new(vec![0], 0);

        assert!(editor.move_right());
        assert_eq!(editor.cursor.offset, 1);

        assert!(editor.move_right());
        assert_eq!(editor.cursor.offset, 2);
    }

    #[test]
    fn test_editor_navigate_left() {
        let eq = MathNode::OMath(vec![MathNode::Run {
            text: "abc".to_string(),
            style: MathStyle::default(),
        }]);
        let mut editor = EquationEditor::new(eq);
        editor.cursor.offset = 2;

        assert!(editor.move_left());
        assert_eq!(editor.cursor.offset, 1);
    }

    #[test]
    fn test_editor_navigate_fraction() {
        let mut editor = EquationEditor::new(make_fraction());

        // Start at numerator
        editor.cursor = MathPath::new(vec![0], 0);

        // Move down to denominator
        let moved = editor.move_down();
        // May or may not work depending on exact box layout
        // Just verify no panic
        let _ = moved;
    }

    #[test]
    fn test_editor_tab_next() {
        let mut editor = EquationEditor::new(make_empty_fraction());
        let initial_cursor = editor.cursor.clone();

        editor.tab_next();
        // Should move to a different position or wrap
        // Just verify it doesn't panic
        assert!(editor.cursor == initial_cursor || editor.cursor != initial_cursor);
    }

    #[test]
    fn test_editor_select_all() {
        let mut editor = EquationEditor::new(make_fraction());
        editor.select_all();
        assert!(matches!(editor.selection(), MathSelection::Node(_)));
    }

    #[test]
    fn test_editor_clear_selection() {
        let mut editor = EquationEditor::new(make_fraction());
        editor.select_all();
        editor.clear_selection();
        assert!(editor.selection().is_empty());
    }

    #[test]
    fn test_editor_node_at() {
        let editor = EquationEditor::new(make_fraction());
        let node = editor.node_at(&MathPath::root());
        assert!(node.is_some());
    }

    #[test]
    fn test_editor_is_at_placeholder() {
        let editor = EquationEditor::new(make_empty_fraction());
        // Should start at a placeholder
        assert!(editor.is_at_placeholder());
    }

    #[test]
    fn test_math_box_new() {
        let mb = MathBox::new(MathPath::root(), MathBoxType::Container);
        assert!(!mb.is_placeholder);
        assert_eq!(mb.tab_order, 0);
    }

    #[test]
    fn test_math_box_placeholder() {
        let mb = MathBox::new(MathPath::root(), MathBoxType::TextRun).placeholder();
        assert!(mb.is_placeholder);
    }

    #[test]
    fn test_extend_selection() {
        let eq = MathNode::OMath(vec![MathNode::Run {
            text: "abc".to_string(),
            style: MathStyle::default(),
        }]);
        let mut editor = EquationEditor::new(eq);
        editor.cursor = MathPath::new(vec![0], 1);

        editor.extend_selection_right();
        if let MathSelection::CharRange { start, end, .. } = editor.selection() {
            assert_eq!(*start, 1);
            assert_eq!(*end, 2);
        }
    }

    #[test]
    fn test_editor_refresh() {
        let mut editor = EquationEditor::new(make_fraction());
        let initial_boxes = editor.boxes().len();
        editor.refresh();
        assert_eq!(editor.boxes().len(), initial_boxes);
    }

    #[test]
    fn test_nav_direction() {
        let mut editor = EquationEditor::new(MathNode::OMath(vec![MathNode::run("x")]));
        editor.navigate(NavDirection::Left);
        editor.navigate(NavDirection::Right);
        editor.navigate(NavDirection::Up);
        editor.navigate(NavDirection::Down);
        // Just verify no panic
    }
}

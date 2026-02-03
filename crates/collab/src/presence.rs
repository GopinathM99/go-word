//! Presence system for showing remote cursors and selections.
//!
//! This module provides types and utilities for tracking user presence
//! in a collaborative document editing session, including cursor positions,
//! selections, and typing indicators.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A position in the document
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// The node ID in the document tree
    pub node_id: String,
    /// Character offset within the node
    pub offset: usize,
}

impl Position {
    /// Create a new position
    pub fn new(node_id: impl Into<String>, offset: usize) -> Self {
        Self {
            node_id: node_id.into(),
            offset,
        }
    }
}

/// A selection range in the document
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionRange {
    /// Start position of the selection
    pub start: Position,
    /// End position of the selection
    pub end: Position,
}

impl SelectionRange {
    /// Create a new selection range
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Check if this is a collapsed selection (cursor position)
    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }
}

/// User's presence state in a document
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresenceState {
    /// User ID
    pub user_id: String,
    /// Display name
    pub display_name: String,
    /// Assigned color (hex)
    pub color: String,
    /// Current cursor position
    pub cursor: Option<Position>,
    /// Current selection
    pub selection: Option<SelectionRange>,
    /// Whether user is currently typing
    pub is_typing: bool,
    /// Last activity timestamp (ms since epoch)
    pub last_active: u64,
    /// User's view scroll position (for "follow" feature)
    pub scroll_position: Option<f64>,
}

impl PresenceState {
    /// Create a new presence state for a user
    pub fn new(user_id: String, display_name: String, color: String) -> Self {
        Self {
            user_id,
            display_name,
            color,
            cursor: None,
            selection: None,
            is_typing: false,
            last_active: current_timestamp_ms(),
            scroll_position: None,
        }
    }

    /// Update cursor position
    pub fn set_cursor(&mut self, position: Option<Position>) {
        self.cursor = position;
        self.touch();
    }

    /// Update selection
    pub fn set_selection(&mut self, selection: Option<SelectionRange>) {
        self.selection = selection;
        self.touch();
    }

    /// Set typing indicator
    pub fn set_typing(&mut self, is_typing: bool) {
        self.is_typing = is_typing;
        self.touch();
    }

    /// Set scroll position
    pub fn set_scroll_position(&mut self, position: Option<f64>) {
        self.scroll_position = position;
        self.touch();
    }

    /// Touch last active timestamp
    pub fn touch(&mut self) {
        self.last_active = current_timestamp_ms();
    }

    /// Check if user is idle (no activity for N milliseconds)
    pub fn is_idle(&self, idle_threshold_ms: u64) -> bool {
        let now = current_timestamp_ms();
        now.saturating_sub(self.last_active) > idle_threshold_ms
    }
}

/// Remote cursor for rendering
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteCursor {
    /// User ID
    pub user_id: String,
    /// Display name
    pub display_name: String,
    /// Color (hex)
    pub color: String,
    /// Cursor position
    pub position: Position,
    /// Whether user is typing
    pub is_typing: bool,
}

/// Remote selection for rendering
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteSelection {
    /// User ID
    pub user_id: String,
    /// Color (hex)
    pub color: String,
    /// Selection range
    pub selection: SelectionRange,
}

/// Presence manager for a document
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresenceManager {
    /// All users' presence states
    users: HashMap<String, PresenceState>,
    /// Color assignments
    color_assignments: HashMap<String, String>,
    /// Available colors for new users
    available_colors: Vec<String>,
    /// Idle threshold in milliseconds
    idle_threshold_ms: u64,
    /// Next color index for round-robin assignment
    next_color_index: usize,
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PresenceManager {
    /// Create a new presence manager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            color_assignments: HashMap::new(),
            available_colors: default_colors(),
            idle_threshold_ms: 60_000, // 1 minute
            next_color_index: 0,
        }
    }

    /// Create a presence manager with custom idle threshold
    pub fn with_idle_threshold(idle_threshold_ms: u64) -> Self {
        Self {
            users: HashMap::new(),
            color_assignments: HashMap::new(),
            available_colors: default_colors(),
            idle_threshold_ms,
            next_color_index: 0,
        }
    }

    /// Add or update a user's presence
    pub fn update_user(&mut self, state: PresenceState) {
        let user_id = state.user_id.clone();
        // Ensure color is assigned
        if !self.color_assignments.contains_key(&user_id) {
            let color = state.color.clone();
            self.color_assignments.insert(user_id.clone(), color);
        }
        self.users.insert(user_id, state);
    }

    /// Remove a user
    pub fn remove_user(&mut self, user_id: &str) {
        self.users.remove(user_id);
        // Keep color assignment for consistency if user rejoins
    }

    /// Get a user's presence
    pub fn get_user(&self, user_id: &str) -> Option<&PresenceState> {
        self.users.get(user_id)
    }

    /// Get a mutable reference to a user's presence
    pub fn get_user_mut(&mut self, user_id: &str) -> Option<&mut PresenceState> {
        self.users.get_mut(user_id)
    }

    /// Get all active users (not idle)
    pub fn active_users(&self) -> Vec<&PresenceState> {
        self.users
            .values()
            .filter(|state| !state.is_idle(self.idle_threshold_ms))
            .collect()
    }

    /// Get all users (including idle)
    pub fn all_users(&self) -> Vec<&PresenceState> {
        self.users.values().collect()
    }

    /// Get the number of users
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Assign a color to a new user
    pub fn assign_color(&mut self, user_id: &str) -> String {
        // Return existing color if already assigned
        if let Some(color) = self.color_assignments.get(user_id) {
            return color.clone();
        }

        // Assign next color in round-robin fashion
        let color = self.available_colors[self.next_color_index].clone();
        self.next_color_index = (self.next_color_index + 1) % self.available_colors.len();

        self.color_assignments.insert(user_id.to_string(), color.clone());
        color
    }

    /// Update cursor for a user
    pub fn update_cursor(&mut self, user_id: &str, position: Option<Position>) {
        if let Some(state) = self.users.get_mut(user_id) {
            state.set_cursor(position);
        }
    }

    /// Update selection for a user
    pub fn update_selection(&mut self, user_id: &str, selection: Option<SelectionRange>) {
        if let Some(state) = self.users.get_mut(user_id) {
            state.set_selection(selection);
        }
    }

    /// Set typing indicator for a user
    pub fn set_typing(&mut self, user_id: &str, is_typing: bool) {
        if let Some(state) = self.users.get_mut(user_id) {
            state.set_typing(is_typing);
        }
    }

    /// Update scroll position for a user
    pub fn update_scroll_position(&mut self, user_id: &str, position: Option<f64>) {
        if let Some(state) = self.users.get_mut(user_id) {
            state.set_scroll_position(position);
        }
    }

    /// Clean up idle users and return their IDs
    pub fn cleanup_idle(&mut self) -> Vec<String> {
        let idle_users: Vec<String> = self
            .users
            .iter()
            .filter(|(_, state)| state.is_idle(self.idle_threshold_ms))
            .map(|(id, _)| id.clone())
            .collect();

        for id in &idle_users {
            self.users.remove(id);
        }

        idle_users
    }

    /// Get cursors for rendering (excludes specified user)
    pub fn get_remote_cursors(&self, exclude_user_id: &str) -> Vec<RemoteCursor> {
        self.users
            .values()
            .filter(|state| state.user_id != exclude_user_id && state.cursor.is_some())
            .map(|state| RemoteCursor {
                user_id: state.user_id.clone(),
                display_name: state.display_name.clone(),
                color: state.color.clone(),
                position: state.cursor.clone().unwrap(),
                is_typing: state.is_typing,
            })
            .collect()
    }

    /// Get selections for rendering (excludes specified user)
    pub fn get_remote_selections(&self, exclude_user_id: &str) -> Vec<RemoteSelection> {
        self.users
            .values()
            .filter(|state| state.user_id != exclude_user_id && state.selection.is_some())
            .map(|state| RemoteSelection {
                user_id: state.user_id.clone(),
                color: state.color.clone(),
                selection: state.selection.clone().unwrap(),
            })
            .collect()
    }

    /// Set the idle threshold
    pub fn set_idle_threshold(&mut self, threshold_ms: u64) {
        self.idle_threshold_ms = threshold_ms;
    }

    /// Get the idle threshold
    pub fn idle_threshold(&self) -> u64 {
        self.idle_threshold_ms
    }

    /// Set custom colors
    pub fn set_colors(&mut self, colors: Vec<String>) {
        self.available_colors = colors;
        self.next_color_index = 0;
    }
}

/// Default color palette for user cursors
pub fn default_colors() -> Vec<String> {
    vec![
        "#E91E63".into(), // Pink
        "#9C27B0".into(), // Purple
        "#3F51B5".into(), // Indigo
        "#2196F3".into(), // Blue
        "#00BCD4".into(), // Cyan
        "#4CAF50".into(), // Green
        "#FF9800".into(), // Orange
        "#795548".into(), // Brown
    ]
}

/// Get the current timestamp in milliseconds since epoch
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new("node-1", 5);
        assert_eq!(pos.node_id, "node-1");
        assert_eq!(pos.offset, 5);
    }

    #[test]
    fn test_selection_range_collapsed() {
        let start = Position::new("node-1", 5);
        let end = Position::new("node-1", 5);
        let range = SelectionRange::new(start, end);
        assert!(range.is_collapsed());

        let start2 = Position::new("node-1", 5);
        let end2 = Position::new("node-1", 10);
        let range2 = SelectionRange::new(start2, end2);
        assert!(!range2.is_collapsed());
    }

    #[test]
    fn test_presence_state_new() {
        let state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );

        assert_eq!(state.user_id, "user-1");
        assert_eq!(state.display_name, "Alice");
        assert_eq!(state.color, "#E91E63");
        assert!(state.cursor.is_none());
        assert!(state.selection.is_none());
        assert!(!state.is_typing);
        assert!(state.last_active > 0);
    }

    #[test]
    fn test_presence_state_set_cursor() {
        let mut state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );

        let pos = Position::new("node-1", 10);
        state.set_cursor(Some(pos.clone()));

        assert_eq!(state.cursor, Some(pos));
    }

    #[test]
    fn test_presence_state_set_selection() {
        let mut state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );

        let start = Position::new("node-1", 5);
        let end = Position::new("node-1", 15);
        let selection = SelectionRange::new(start, end);
        state.set_selection(Some(selection.clone()));

        assert_eq!(state.selection, Some(selection));
    }

    #[test]
    fn test_presence_state_typing() {
        let mut state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );

        assert!(!state.is_typing);
        state.set_typing(true);
        assert!(state.is_typing);
        state.set_typing(false);
        assert!(!state.is_typing);
    }

    #[test]
    fn test_presence_state_is_idle() {
        let mut state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );

        // Freshly created state should not be idle
        assert!(!state.is_idle(60_000));

        // Simulate old activity by manually setting last_active
        state.last_active = current_timestamp_ms().saturating_sub(120_000);
        assert!(state.is_idle(60_000));
    }

    #[test]
    fn test_presence_manager_new() {
        let manager = PresenceManager::new();
        assert_eq!(manager.user_count(), 0);
        assert_eq!(manager.idle_threshold(), 60_000);
    }

    #[test]
    fn test_presence_manager_update_user() {
        let mut manager = PresenceManager::new();

        let state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );

        manager.update_user(state);

        assert_eq!(manager.user_count(), 1);
        assert!(manager.get_user("user-1").is_some());
    }

    #[test]
    fn test_presence_manager_remove_user() {
        let mut manager = PresenceManager::new();

        let state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );

        manager.update_user(state);
        assert_eq!(manager.user_count(), 1);

        manager.remove_user("user-1");
        assert_eq!(manager.user_count(), 0);
        assert!(manager.get_user("user-1").is_none());
    }

    #[test]
    fn test_presence_manager_assign_color() {
        let mut manager = PresenceManager::new();

        let color1 = manager.assign_color("user-1");
        let color2 = manager.assign_color("user-2");
        let color3 = manager.assign_color("user-1"); // Same user

        // Different users get different colors
        assert_ne!(color1, color2);

        // Same user gets same color
        assert_eq!(color1, color3);
    }

    #[test]
    fn test_presence_manager_update_cursor() {
        let mut manager = PresenceManager::new();

        let state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );
        manager.update_user(state);

        let pos = Position::new("node-1", 10);
        manager.update_cursor("user-1", Some(pos.clone()));

        let user = manager.get_user("user-1").unwrap();
        assert_eq!(user.cursor, Some(pos));
    }

    #[test]
    fn test_presence_manager_update_selection() {
        let mut manager = PresenceManager::new();

        let state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );
        manager.update_user(state);

        let start = Position::new("node-1", 5);
        let end = Position::new("node-1", 15);
        let selection = SelectionRange::new(start, end);
        manager.update_selection("user-1", Some(selection.clone()));

        let user = manager.get_user("user-1").unwrap();
        assert_eq!(user.selection, Some(selection));
    }

    #[test]
    fn test_presence_manager_set_typing() {
        let mut manager = PresenceManager::new();

        let state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );
        manager.update_user(state);

        manager.set_typing("user-1", true);

        let user = manager.get_user("user-1").unwrap();
        assert!(user.is_typing);
    }

    #[test]
    fn test_presence_manager_get_remote_cursors() {
        let mut manager = PresenceManager::new();

        // Add user 1 with cursor
        let mut state1 = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );
        state1.set_cursor(Some(Position::new("node-1", 10)));
        manager.update_user(state1);

        // Add user 2 with cursor
        let mut state2 = PresenceState::new(
            "user-2".into(),
            "Bob".into(),
            "#9C27B0".into(),
        );
        state2.set_cursor(Some(Position::new("node-2", 20)));
        manager.update_user(state2);

        // Add user 3 without cursor
        let state3 = PresenceState::new(
            "user-3".into(),
            "Charlie".into(),
            "#3F51B5".into(),
        );
        manager.update_user(state3);

        // Get remote cursors excluding user-1
        let cursors = manager.get_remote_cursors("user-1");

        // Should only get user-2's cursor (user-3 has no cursor)
        assert_eq!(cursors.len(), 1);
        assert_eq!(cursors[0].user_id, "user-2");
        assert_eq!(cursors[0].display_name, "Bob");
    }

    #[test]
    fn test_presence_manager_get_remote_selections() {
        let mut manager = PresenceManager::new();

        // Add user 1 with selection
        let mut state1 = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );
        state1.set_selection(Some(SelectionRange::new(
            Position::new("node-1", 0),
            Position::new("node-1", 10),
        )));
        manager.update_user(state1);

        // Add user 2 with selection
        let mut state2 = PresenceState::new(
            "user-2".into(),
            "Bob".into(),
            "#9C27B0".into(),
        );
        state2.set_selection(Some(SelectionRange::new(
            Position::new("node-2", 5),
            Position::new("node-2", 15),
        )));
        manager.update_user(state2);

        // Get remote selections excluding user-1
        let selections = manager.get_remote_selections("user-1");

        assert_eq!(selections.len(), 1);
        assert_eq!(selections[0].user_id, "user-2");
    }

    #[test]
    fn test_presence_manager_cleanup_idle() {
        let mut manager = PresenceManager::with_idle_threshold(1000); // 1 second

        // Add active user
        let state1 = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );
        manager.update_user(state1);

        // Add "idle" user by manually setting old timestamp
        let mut state2 = PresenceState::new(
            "user-2".into(),
            "Bob".into(),
            "#9C27B0".into(),
        );
        state2.last_active = current_timestamp_ms().saturating_sub(5000);
        manager.update_user(state2);

        let removed = manager.cleanup_idle();

        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], "user-2");
        assert_eq!(manager.user_count(), 1);
        assert!(manager.get_user("user-1").is_some());
        assert!(manager.get_user("user-2").is_none());
    }

    #[test]
    fn test_presence_manager_active_users() {
        let mut manager = PresenceManager::with_idle_threshold(1000);

        // Add active user
        let state1 = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );
        manager.update_user(state1);

        // Add "idle" user
        let mut state2 = PresenceState::new(
            "user-2".into(),
            "Bob".into(),
            "#9C27B0".into(),
        );
        state2.last_active = current_timestamp_ms().saturating_sub(5000);
        manager.update_user(state2);

        let active = manager.active_users();
        let all = manager.all_users();

        assert_eq!(active.len(), 1);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_default_colors() {
        let colors = default_colors();
        assert!(!colors.is_empty());
        assert_eq!(colors.len(), 8);

        // All colors should be hex format
        for color in &colors {
            assert!(color.starts_with('#'));
            assert_eq!(color.len(), 7);
        }
    }

    #[test]
    fn test_color_round_robin() {
        let mut manager = PresenceManager::new();
        let _colors = default_colors();

        // Assign colors to more users than available colors
        let mut assigned: Vec<String> = Vec::new();
        for i in 0..12 {
            let color = manager.assign_color(&format!("user-{}", i));
            assigned.push(color);
        }

        // First 8 users should get unique colors
        let first_8: Vec<_> = assigned[..8].to_vec();
        let unique_first_8: std::collections::HashSet<_> = first_8.iter().collect();
        assert_eq!(unique_first_8.len(), 8);

        // Colors should wrap around
        assert_eq!(assigned[0], assigned[8]);
        assert_eq!(assigned[1], assigned[9]);
    }

    #[test]
    fn test_presence_manager_set_custom_colors() {
        let mut manager = PresenceManager::new();

        let custom_colors = vec![
            "#FF0000".into(),
            "#00FF00".into(),
            "#0000FF".into(),
        ];
        manager.set_colors(custom_colors);

        let color1 = manager.assign_color("user-1");
        let color2 = manager.assign_color("user-2");
        let color3 = manager.assign_color("user-3");
        let color4 = manager.assign_color("user-4");

        assert_eq!(color1, "#FF0000");
        assert_eq!(color2, "#00FF00");
        assert_eq!(color3, "#0000FF");
        assert_eq!(color4, "#FF0000"); // Wraps around
    }

    #[test]
    fn test_presence_state_scroll_position() {
        let mut state = PresenceState::new(
            "user-1".into(),
            "Alice".into(),
            "#E91E63".into(),
        );

        assert!(state.scroll_position.is_none());

        state.set_scroll_position(Some(150.5));
        assert_eq!(state.scroll_position, Some(150.5));

        state.set_scroll_position(None);
        assert!(state.scroll_position.is_none());
    }
}

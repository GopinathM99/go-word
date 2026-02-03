//! Last-Writer-Wins (LWW) Register CRDT for formatting attributes.
//!
//! This module provides CRDT data structures for managing concurrent updates
//! to values where the most recent write wins. This is particularly useful
//! for text formatting attributes where users expect their most recent
//! formatting choice to be applied.
//!
//! # Overview
//!
//! The LWW Register uses timestamps to determine which write "wins" when
//! concurrent updates occur. The ordering is:
//! 1. Higher physical timestamp wins
//! 2. Higher logical counter breaks ties
//! 3. Higher client ID is the final tie-breaker (for determinism)
//!
//! # Example
//!
//! ```
//! use collab::lww_register::{LwwRegister, LwwMap};
//! use collab::clock::Timestamp;
//! use collab::op_id::ClientId;
//!
//! // Create a register for a boolean value
//! let mut bold = LwwRegister::new(false, Timestamp::epoch(ClientId::new(1)), ClientId::new(1));
//! 
//! // Update with a newer timestamp
//! let newer = Timestamp::new(100, 0, ClientId::new(1));
//! bold.set(true, newer, ClientId::new(1));
//! 
//! assert_eq!(*bold.get(), true);
//! ```

use crate::clock::Timestamp;
use crate::op_id::ClientId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;

/// A Last-Writer-Wins Register for a single value.
///
/// This CRDT always converges to the value with the highest timestamp.
/// When timestamps are equal, the higher client ID wins.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LwwRegister<T> {
    /// The current value
    value: T,
    /// Timestamp of the last update
    timestamp: Timestamp,
    /// Client that made the last update
    client_id: ClientId,
}

impl<T: Clone> LwwRegister<T> {
    /// Create a new register with an initial value.
    ///
    /// # Arguments
    ///
    /// * `value` - The initial value
    /// * `timestamp` - The timestamp for this initial value
    /// * `client_id` - The client creating this register
    pub fn new(value: T, timestamp: Timestamp, client_id: ClientId) -> Self {
        Self {
            value,
            timestamp,
            client_id,
        }
    }

    /// Get the current value.
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Set a new value if the timestamp is newer.
    ///
    /// Returns `true` if the value was updated, `false` otherwise.
    ///
    /// # Arguments
    ///
    /// * `value` - The new value to set
    /// * `timestamp` - The timestamp for this update
    /// * `client_id` - The client making this update
    pub fn set(&mut self, value: T, timestamp: Timestamp, client_id: ClientId) -> bool {
        if self.should_update(&timestamp, client_id) {
            self.value = value;
            self.timestamp = timestamp;
            self.client_id = client_id;
            true
        } else {
            false
        }
    }

    /// Apply a remote update.
    ///
    /// This is semantically equivalent to `set`, but the name makes the
    /// intent clearer when applying updates from remote peers.
    ///
    /// Returns `true` if the value was updated, `false` otherwise.
    pub fn apply(&mut self, value: T, timestamp: Timestamp, client_id: ClientId) -> bool {
        self.set(value, timestamp, client_id)
    }

    /// Get the timestamp of the last update.
    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Get the client who made the last update.
    pub fn last_writer(&self) -> ClientId {
        self.client_id
    }

    /// Check if an update with the given timestamp should win.
    ///
    /// The update wins if:
    /// 1. Its timestamp is greater (physical > logical > client_id)
    /// 2. Or timestamps are equal but client_id is higher (tie-breaker)
    fn should_update(&self, new_timestamp: &Timestamp, new_client_id: ClientId) -> bool {
        // Compare timestamps first
        match new_timestamp.cmp(&self.timestamp) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal => {
                // Timestamps are equal, use client_id as tie-breaker
                // Note: The timestamp already contains client_id, but we use the
                // explicit client_id parameter for the actual writer
                new_client_id > self.client_id
            }
        }
    }
}

impl<T: Clone + Default> Default for LwwRegister<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            timestamp: Timestamp::default(),
            client_id: ClientId::new(0),
        }
    }
}

impl<T: Clone + PartialEq> PartialEq for LwwRegister<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
            && self.timestamp == other.timestamp
            && self.client_id == other.client_id
    }
}

impl<T: Clone + Eq> Eq for LwwRegister<T> {}

/// Operations on LWW structures for synchronization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LwwOperation<K, V> {
    /// Set a key to a value (or None to remove)
    Set {
        key: K,
        value: Option<V>,
        timestamp: Timestamp,
        client_id: ClientId,
    },
}

impl<K, V> LwwOperation<K, V> {
    /// Create a Set operation.
    pub fn set(key: K, value: Option<V>, timestamp: Timestamp, client_id: ClientId) -> Self {
        Self::Set {
            key,
            value,
            timestamp,
            client_id,
        }
    }

    /// Get the timestamp of this operation.
    pub fn timestamp(&self) -> Timestamp {
        match self {
            Self::Set { timestamp, .. } => *timestamp,
        }
    }

    /// Get the client ID of this operation.
    pub fn client_id(&self) -> ClientId {
        match self {
            Self::Set { client_id, .. } => *client_id,
        }
    }
}

/// A map of LWW registers for multiple attributes.
///
/// This is useful for tracking multiple independent values that can be
/// updated concurrently, such as text formatting attributes (bold, italic,
/// font size, etc.).
///
/// Values are wrapped in `Option<V>` internally to support removal (tombstones).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(
    serialize = "K: std::cmp::Eq + std::hash::Hash + Clone + Serialize, V: Clone + Serialize",
    deserialize = "K: std::cmp::Eq + std::hash::Hash + Clone + for<'a> Deserialize<'a>, V: Clone + for<'a> Deserialize<'a>"
))]
pub struct LwwMap<K, V> {
    /// Map of keys to LWW registers holding optional values
    registers: HashMap<K, LwwRegister<Option<V>>>,
    /// Client ID for local operations
    client_id: ClientId,
}

impl<K: Eq + Hash + Clone, V: Clone> LwwMap<K, V> {
    /// Create a new empty LWW map.
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID for local operations
    pub fn new(client_id: ClientId) -> Self {
        Self {
            registers: HashMap::new(),
            client_id,
        }
    }

    /// Get a value by key.
    ///
    /// Returns `None` if the key doesn't exist or has been removed.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.registers
            .get(key)
            .and_then(|reg| reg.get().as_ref())
    }

    /// Set a value for a key.
    ///
    /// Creates a new register if the key doesn't exist.
    /// Returns `true` if the value was updated.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set
    /// * `value` - The value to set
    /// * `timestamp` - The timestamp for this update
    pub fn set(&mut self, key: K, value: V, timestamp: Timestamp) -> bool {
        let client_id = self.client_id;
        self.apply(key, Some(value), timestamp, client_id)
    }

    /// Remove a key by setting it to None.
    ///
    /// This creates a tombstone - the key still exists in the map but
    /// with a None value, which allows proper conflict resolution with
    /// concurrent updates.
    ///
    /// Returns `true` if the removal was applied.
    pub fn remove(&mut self, key: K, timestamp: Timestamp) -> bool {
        let client_id = self.client_id;
        self.apply(key, None, timestamp, client_id)
    }

    /// Apply a remote update.
    ///
    /// This handles both setting and removing values from remote peers.
    /// Returns `true` if the update was applied.
    pub fn apply(
        &mut self,
        key: K,
        value: Option<V>,
        timestamp: Timestamp,
        client_id: ClientId,
    ) -> bool {
        match self.registers.get_mut(&key) {
            Some(register) => register.apply(value, timestamp, client_id),
            None => {
                // Create new register for this key
                self.registers.insert(
                    key,
                    LwwRegister::new(value, timestamp, client_id),
                );
                true
            }
        }
    }

    /// Get all keys that currently have values (not removed).
    pub fn keys(&self) -> Vec<&K> {
        self.registers
            .iter()
            .filter_map(|(k, v)| if v.get().is_some() { Some(k) } else { None })
            .collect()
    }

    /// Iterate over all key-value pairs (excluding removed keys).
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.registers
            .iter()
            .filter_map(|(k, v)| v.get().as_ref().map(|val| (k, val)))
    }

    /// Merge with another LwwMap.
    ///
    /// For each key, the value with the higher timestamp wins.
    pub fn merge(&mut self, other: &LwwMap<K, V>) {
        for (key, other_register) in &other.registers {
            let value = other_register.get().clone();
            let timestamp = other_register.timestamp();
            let client_id = other_register.last_writer();

            self.apply(key.clone(), value, timestamp, client_id);
        }
    }

    /// Get all operations for synchronization.
    ///
    /// Returns a list of Set operations representing the current state,
    /// which can be sent to other peers for syncing.
    pub fn all_ops(&self) -> Vec<LwwOperation<K, V>>
    where
        K: Clone,
        V: Clone,
    {
        self.registers
            .iter()
            .map(|(key, register)| {
                LwwOperation::Set {
                    key: key.clone(),
                    value: register.get().clone(),
                    timestamp: register.timestamp(),
                    client_id: register.last_writer(),
                }
            })
            .collect()
    }

    /// Get the client ID for this map.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Check if a key exists (even if removed/tombstoned).
    pub fn contains_key(&self, key: &K) -> bool {
        self.registers.contains_key(key)
    }

    /// Check if a key exists and has a value (not removed).
    pub fn has_value(&self, key: &K) -> bool {
        self.registers
            .get(key)
            .map(|r| r.get().is_some())
            .unwrap_or(false)
    }

    /// Get the number of keys with values (excluding removed).
    pub fn len(&self) -> usize {
        self.registers
            .values()
            .filter(|r| r.get().is_some())
            .count()
    }

    /// Check if the map is empty (no keys with values).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the total number of keys (including tombstones).
    pub fn total_keys(&self) -> usize {
        self.registers.len()
    }
}

impl<K: Eq + Hash + Clone, V: Clone + Default> Default for LwwMap<K, V> {
    fn default() -> Self {
        Self::new(ClientId::new(0))
    }
}

/// Formatting attributes that use LWW.
///
/// This struct represents common text formatting options.
/// Each field is optional to support partial updates.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FormattingAttributes {
    /// Bold formatting
    pub bold: Option<bool>,
    /// Italic formatting
    pub italic: Option<bool>,
    /// Underline formatting
    pub underline: Option<bool>,
    /// Strikethrough formatting
    pub strikethrough: Option<bool>,
    /// Subscript formatting
    pub subscript: Option<bool>,
    /// Superscript formatting
    pub superscript: Option<bool>,
    /// Font family name
    pub font_family: Option<String>,
    /// Font size in points
    pub font_size: Option<f32>,
    /// Text color (hex string like "#FF0000")
    pub color: Option<String>,
    /// Background/highlight color
    pub background_color: Option<String>,
}

impl FormattingAttributes {
    /// Create new empty formatting attributes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create formatting with bold set.
    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    /// Create formatting with italic set.
    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = Some(italic);
        self
    }

    /// Create formatting with underline set.
    pub fn with_underline(mut self, underline: bool) -> Self {
        self.underline = Some(underline);
        self
    }

    /// Create formatting with font family set.
    pub fn with_font_family(mut self, font: impl Into<String>) -> Self {
        self.font_family = Some(font.into());
        self
    }

    /// Create formatting with font size set.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = Some(size);
        self
    }

    /// Create formatting with color set.
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Merge another formatting into this one (other takes precedence for set values).
    pub fn merge(&mut self, other: &FormattingAttributes) {
        if other.bold.is_some() {
            self.bold = other.bold;
        }
        if other.italic.is_some() {
            self.italic = other.italic;
        }
        if other.underline.is_some() {
            self.underline = other.underline;
        }
        if other.strikethrough.is_some() {
            self.strikethrough = other.strikethrough;
        }
        if other.subscript.is_some() {
            self.subscript = other.subscript;
        }
        if other.superscript.is_some() {
            self.superscript = other.superscript;
        }
        if other.font_family.is_some() {
            self.font_family = other.font_family.clone();
        }
        if other.font_size.is_some() {
            self.font_size = other.font_size;
        }
        if other.color.is_some() {
            self.color = other.color.clone();
        }
        if other.background_color.is_some() {
            self.background_color = other.background_color.clone();
        }
    }
}

/// LWW map specifically for text formatting.
///
/// Uses String keys to allow flexible attribute names and serde_json::Value
/// for flexible value types.
pub type FormattingMap = LwwMap<String, serde_json::Value>;

#[cfg(test)]
mod tests {
    use super::*;

    fn ts(physical: u64, logical: u64, client: u64) -> Timestamp {
        Timestamp::new(physical, logical, ClientId::new(client))
    }

    // ========== LwwRegister Tests ==========

    #[test]
    fn test_register_new_and_get() {
        let reg = LwwRegister::new(42, ts(100, 0, 1), ClientId::new(1));
        assert_eq!(*reg.get(), 42);
        assert_eq!(reg.timestamp(), ts(100, 0, 1));
        assert_eq!(reg.last_writer(), ClientId::new(1));
    }

    #[test]
    fn test_register_set_newer_timestamp_wins() {
        let mut reg = LwwRegister::new(1, ts(100, 0, 1), ClientId::new(1));

        // Newer timestamp should win
        assert!(reg.set(2, ts(200, 0, 1), ClientId::new(1)));
        assert_eq!(*reg.get(), 2);

        // Older timestamp should lose
        assert!(!reg.set(3, ts(150, 0, 1), ClientId::new(1)));
        assert_eq!(*reg.get(), 2);
    }

    #[test]
    fn test_register_concurrent_updates_higher_timestamp_wins() {
        let mut reg = LwwRegister::new(1, ts(100, 0, 1), ClientId::new(1));

        // Client 2 writes with higher physical time - should win
        assert!(reg.set(2, ts(200, 0, 2), ClientId::new(2)));
        assert_eq!(*reg.get(), 2);
        assert_eq!(reg.last_writer(), ClientId::new(2));
    }

    #[test]
    fn test_register_same_timestamp_client_id_breaks_tie() {
        let mut reg = LwwRegister::new(1, ts(100, 0, 1), ClientId::new(1));

        // Same timestamp but higher client_id should win
        assert!(reg.set(2, ts(100, 0, 1), ClientId::new(2)));
        assert_eq!(*reg.get(), 2);
        assert_eq!(reg.last_writer(), ClientId::new(2));

        // Lower client_id with same timestamp should lose
        assert!(!reg.set(3, ts(100, 0, 1), ClientId::new(1)));
        assert_eq!(*reg.get(), 2);
    }

    #[test]
    fn test_register_logical_timestamp_breaks_tie() {
        let mut reg = LwwRegister::new(1, ts(100, 0, 1), ClientId::new(1));

        // Same physical time but higher logical should win
        assert!(reg.set(2, ts(100, 1, 1), ClientId::new(1)));
        assert_eq!(*reg.get(), 2);

        // Lower logical should lose
        assert!(!reg.set(3, ts(100, 0, 1), ClientId::new(1)));
        assert_eq!(*reg.get(), 2);
    }

    #[test]
    fn test_register_apply_is_same_as_set() {
        let mut reg1 = LwwRegister::new(1, ts(100, 0, 1), ClientId::new(1));
        let mut reg2 = LwwRegister::new(1, ts(100, 0, 1), ClientId::new(1));

        reg1.set(2, ts(200, 0, 1), ClientId::new(1));
        reg2.apply(2, ts(200, 0, 1), ClientId::new(1));

        assert_eq!(reg1, reg2);
    }

    // ========== LwwMap Tests ==========

    #[test]
    fn test_map_new_and_get() {
        let map: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));
        assert!(map.is_empty());
        assert_eq!(map.get(&"key".to_string()), None);
    }

    #[test]
    fn test_map_set_and_get() {
        let mut map: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));

        map.set("key".to_string(), 42, ts(100, 0, 1));

        assert_eq!(map.get(&"key".to_string()), Some(&42));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_map_remove() {
        let mut map: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));

        map.set("key".to_string(), 42, ts(100, 0, 1));
        assert_eq!(map.get(&"key".to_string()), Some(&42));

        map.remove("key".to_string(), ts(200, 0, 1));
        assert_eq!(map.get(&"key".to_string()), None);

        // Key still exists (tombstone) but has no value
        assert!(map.contains_key(&"key".to_string()));
        assert!(!map.has_value(&"key".to_string()));
    }

    #[test]
    fn test_map_remove_then_set() {
        let mut map: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));

        map.set("key".to_string(), 42, ts(100, 0, 1));
        map.remove("key".to_string(), ts(200, 0, 1));

        // Re-setting with newer timestamp should work
        map.set("key".to_string(), 99, ts(300, 0, 1));
        assert_eq!(map.get(&"key".to_string()), Some(&99));
    }

    #[test]
    fn test_map_concurrent_updates() {
        let mut map: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));

        // Client 1 sets value at t=100
        map.apply("key".to_string(), Some(1), ts(100, 0, 1), ClientId::new(1));

        // Client 2 sets value at t=200 - should win
        map.apply("key".to_string(), Some(2), ts(200, 0, 2), ClientId::new(2));
        assert_eq!(map.get(&"key".to_string()), Some(&2));

        // Client 1 tries to set at t=150 - should lose
        map.apply("key".to_string(), Some(3), ts(150, 0, 1), ClientId::new(1));
        assert_eq!(map.get(&"key".to_string()), Some(&2));
    }

    #[test]
    fn test_map_concurrent_set_and_remove() {
        let mut map: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));

        // Client 1 sets at t=100
        map.apply("key".to_string(), Some(42), ts(100, 0, 1), ClientId::new(1));

        // Client 2 removes at t=200 - should win
        map.apply("key".to_string(), None, ts(200, 0, 2), ClientId::new(2));
        assert_eq!(map.get(&"key".to_string()), None);

        // Client 1 tries to set at t=150 - should lose to the remove
        map.apply("key".to_string(), Some(99), ts(150, 0, 1), ClientId::new(1));
        assert_eq!(map.get(&"key".to_string()), None);
    }

    #[test]
    fn test_map_keys_and_iter() {
        let mut map: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));

        map.set("a".to_string(), 1, ts(100, 0, 1));
        map.set("b".to_string(), 2, ts(100, 1, 1));
        map.set("c".to_string(), 3, ts(100, 2, 1));
        map.remove("b".to_string(), ts(200, 0, 1));

        let mut keys: Vec<_> = map.keys().into_iter().cloned().collect();
        keys.sort();
        assert_eq!(keys, vec!["a".to_string(), "c".to_string()]);

        let mut pairs: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        pairs.sort();
        assert_eq!(
            pairs,
            vec![("a".to_string(), 1), ("c".to_string(), 3)]
        );
    }

    #[test]
    fn test_map_merge() {
        let mut map1: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));
        let mut map2: LwwMap<String, i32> = LwwMap::new(ClientId::new(2));

        // Map1: a=1 at t=100, b=2 at t=200
        map1.set("a".to_string(), 1, ts(100, 0, 1));
        map1.set("b".to_string(), 2, ts(200, 0, 1));

        // Map2: a=10 at t=200, c=3 at t=100
        map2.set("a".to_string(), 10, ts(200, 0, 2));
        map2.set("c".to_string(), 3, ts(100, 0, 2));

        // Merge map2 into map1
        map1.merge(&map2);

        // a should be 10 (newer timestamp from map2)
        assert_eq!(map1.get(&"a".to_string()), Some(&10));
        // b should be 2 (only in map1)
        assert_eq!(map1.get(&"b".to_string()), Some(&2));
        // c should be 3 (from map2)
        assert_eq!(map1.get(&"c".to_string()), Some(&3));
    }

    #[test]
    fn test_map_merge_with_removes() {
        let mut map1: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));
        let mut map2: LwwMap<String, i32> = LwwMap::new(ClientId::new(2));

        // Map1: a=1 at t=100
        map1.set("a".to_string(), 1, ts(100, 0, 1));

        // Map2: a removed at t=200
        map2.set("a".to_string(), 1, ts(50, 0, 2));
        map2.remove("a".to_string(), ts(200, 0, 2));

        // Merge - remove should win
        map1.merge(&map2);
        assert_eq!(map1.get(&"a".to_string()), None);
    }

    #[test]
    fn test_map_all_ops() {
        let mut map: LwwMap<String, i32> = LwwMap::new(ClientId::new(1));

        map.set("a".to_string(), 1, ts(100, 0, 1));
        map.set("b".to_string(), 2, ts(200, 0, 1));

        let ops = map.all_ops();
        assert_eq!(ops.len(), 2);

        for op in ops {
            match op {
                LwwOperation::Set { key, value, .. } => {
                    if key == "a" {
                        assert_eq!(value, Some(1));
                    } else if key == "b" {
                        assert_eq!(value, Some(2));
                    }
                }
            }
        }
    }

    // ========== FormattingAttributes Tests ==========

    #[test]
    fn test_formatting_attributes_builder() {
        let attrs = FormattingAttributes::new()
            .with_bold(true)
            .with_italic(true)
            .with_font_size(12.0)
            .with_color("#FF0000");

        assert_eq!(attrs.bold, Some(true));
        assert_eq!(attrs.italic, Some(true));
        assert_eq!(attrs.font_size, Some(12.0));
        assert_eq!(attrs.color, Some("#FF0000".to_string()));
        assert_eq!(attrs.underline, None);
    }

    #[test]
    fn test_formatting_attributes_merge() {
        let mut base = FormattingAttributes::new()
            .with_bold(true)
            .with_font_size(12.0);

        let overlay = FormattingAttributes::new()
            .with_bold(false)
            .with_italic(true);

        base.merge(&overlay);

        assert_eq!(base.bold, Some(false)); // Overwritten
        assert_eq!(base.italic, Some(true)); // Added
        assert_eq!(base.font_size, Some(12.0)); // Preserved
    }

    // ========== FormattingMap Tests ==========

    #[test]
    fn test_formatting_map_with_json_values() {
        let mut map: FormattingMap = LwwMap::new(ClientId::new(1));

        map.set(
            "bold".to_string(),
            serde_json::json!(true),
            ts(100, 0, 1),
        );
        map.set(
            "fontSize".to_string(),
            serde_json::json!(14.5),
            ts(100, 1, 1),
        );
        map.set(
            "color".to_string(),
            serde_json::json!("#FF0000"),
            ts(100, 2, 1),
        );

        assert_eq!(
            map.get(&"bold".to_string()),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            map.get(&"fontSize".to_string()),
            Some(&serde_json::json!(14.5))
        );
        assert_eq!(
            map.get(&"color".to_string()),
            Some(&serde_json::json!("#FF0000"))
        );
    }

    #[test]
    fn test_formatting_map_concurrent_attribute_changes() {
        let mut map: FormattingMap = LwwMap::new(ClientId::new(1));

        // User 1 sets bold at t=100
        map.apply(
            "bold".to_string(),
            Some(serde_json::json!(true)),
            ts(100, 0, 1),
            ClientId::new(1),
        );

        // User 2 sets bold at t=200 - should win
        map.apply(
            "bold".to_string(),
            Some(serde_json::json!(false)),
            ts(200, 0, 2),
            ClientId::new(2),
        );

        assert_eq!(
            map.get(&"bold".to_string()),
            Some(&serde_json::json!(false))
        );
    }

    #[test]
    fn test_scenario_real_world_formatting_conflict() {
        // Scenario: Two users simultaneously change the font of the same text
        // User 1 (client 1) changes to "Arial" at t=1000
        // User 2 (client 2) changes to "Times New Roman" at t=1000 (same physical time)
        
        let mut map: FormattingMap = LwwMap::new(ClientId::new(1));

        // User 1's change
        map.apply(
            "fontFamily".to_string(),
            Some(serde_json::json!("Arial")),
            ts(1000, 0, 1),
            ClientId::new(1),
        );

        // User 2's change (same timestamp, but higher client_id)
        map.apply(
            "fontFamily".to_string(),
            Some(serde_json::json!("Times New Roman")),
            ts(1000, 0, 2),
            ClientId::new(2),
        );

        // Client 2 wins because of higher client_id
        assert_eq!(
            map.get(&"fontFamily".to_string()),
            Some(&serde_json::json!("Times New Roman"))
        );
    }

    #[test]
    fn test_scenario_offline_sync() {
        // Scenario: User goes offline, makes changes, comes back online
        let mut server_map: FormattingMap = LwwMap::new(ClientId::new(0));
        let mut client_map: FormattingMap = LwwMap::new(ClientId::new(1));

        // Initial state: bold = true at t=100
        server_map.set("bold".to_string(), serde_json::json!(true), ts(100, 0, 0));
        client_map.set("bold".to_string(), serde_json::json!(true), ts(100, 0, 0));

        // Server gets update at t=200
        server_map.set("bold".to_string(), serde_json::json!(false), ts(200, 0, 2));

        // Client makes offline change at t=150
        client_map.set("bold".to_string(), serde_json::json!(true), ts(150, 0, 1));

        // Sync: merge client into server
        server_map.merge(&client_map);

        // Server's t=200 update should win
        assert_eq!(
            server_map.get(&"bold".to_string()),
            Some(&serde_json::json!(false))
        );

        // Sync: merge server into client
        client_map.merge(&server_map);

        // Client should also have server's value
        assert_eq!(
            client_map.get(&"bold".to_string()),
            Some(&serde_json::json!(false))
        );
    }
}

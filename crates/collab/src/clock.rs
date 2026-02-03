//! Clock implementations for distributed systems.
//!
//! This module provides three types of clocks commonly used in distributed systems:
//!
//! - **Lamport Clock**: A simple logical clock that provides a total ordering of events.
//! - **Hybrid Logical Clock (HLC)**: Combines physical wall clock time with a logical counter
//!   to provide timestamps that are both causally consistent and close to wall clock time.
//! - **Vector Clock**: Tracks causality by maintaining a counter for each client, allowing
//!   detection of concurrent events.

use crate::op_id::ClientId;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::{SystemTime, UNIX_EPOCH};

/// A simple Lamport logical clock.
///
/// Lamport clocks provide a total ordering of events in a distributed system.
/// The clock value monotonically increases and is updated on both local events
/// and receipt of messages from other processes.
///
/// # Properties
///
/// - If event A happened before event B, then `clock(A) < clock(B)`.
/// - The converse is not necessarily true (concurrent events may have any ordering).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LamportClock {
    counter: u64,
}

impl LamportClock {
    /// Creates a new Lamport clock with initial value 0.
    pub fn new() -> Self {
        Self { counter: 0 }
    }

    /// Creates a Lamport clock with a specific initial value.
    pub fn with_value(value: u64) -> Self {
        Self { counter: value }
    }

    /// Returns the current clock value without incrementing.
    pub fn value(&self) -> u64 {
        self.counter
    }

    /// Increments the clock and returns the new value.
    ///
    /// This should be called when a local event occurs.
    pub fn tick(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }

    /// Updates the clock based on a received timestamp.
    ///
    /// The clock is set to `max(current, received) + 1`.
    /// This should be called when receiving a message from another process.
    pub fn update(&mut self, received: u64) {
        self.counter = self.counter.max(received) + 1;
    }

    /// Updates the clock to be at least the given value (without incrementing).
    ///
    /// Useful for synchronization without generating a new event.
    pub fn sync(&mut self, received: u64) {
        self.counter = self.counter.max(received);
    }
}

/// A hybrid logical timestamp combining physical time with logical counters.
///
/// Timestamps are totally ordered using the following priority:
/// 1. `physical` - milliseconds since UNIX epoch
/// 2. `logical` - tie-breaker for events at the same physical time
/// 3. `client_id` - final tie-breaker for deterministic ordering
///
/// This ensures that all operations can be totally ordered across all clients,
/// even if they occur at exactly the same physical time.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Timestamp {
    /// Physical time in milliseconds since UNIX epoch
    pub physical: u64,
    /// Logical counter for tie-breaking within same physical time
    pub logical: u64,
    /// Client ID for final tie-breaking (higher wins)
    pub client_id: ClientId,
}

impl Timestamp {
    /// Create a new timestamp with explicit values.
    pub fn new(physical: u64, logical: u64, client_id: ClientId) -> Self {
        Self {
            physical,
            logical,
            client_id,
        }
    }

    /// Create a timestamp at epoch (useful for initial values).
    pub fn epoch(client_id: ClientId) -> Self {
        Self {
            physical: 0,
            logical: 0,
            client_id,
        }
    }

    /// Check if this timestamp is newer than another.
    pub fn is_newer_than(&self, other: &Timestamp) -> bool {
        self > other
    }

    /// Check if this timestamp is older than another.
    pub fn is_older_than(&self, other: &Timestamp) -> bool {
        self < other
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ts({}.{}.{})", self.physical, self.logical, self.client_id.0)
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare physical time first
        match self.physical.cmp(&other.physical) {
            Ordering::Equal => {
                // Then logical counter
                match self.logical.cmp(&other.logical) {
                    Ordering::Equal => {
                        // Finally client_id (higher wins for determinism)
                        self.client_id.cmp(&other.client_id)
                    }
                    ord => ord,
                }
            }
            ord => ord,
        }
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self {
            physical: 0,
            logical: 0,
            client_id: ClientId::new(0),
        }
    }
}

/// A Hybrid Logical Clock (HLC) for generating timestamps.
///
/// The HLC combines physical clock time with logical counters to ensure:
/// - Timestamps always advance (never go backwards)
/// - Causality is preserved (if A happens-before B, timestamp(A) < timestamp(B))
/// - Concurrent events are totally ordered via client_id tie-breaking
pub struct HybridClock {
    /// Client ID for this clock
    client_id: ClientId,
    /// Last physical time seen
    last_physical: AtomicU64,
    /// Logical counter
    logical: AtomicU64,
}

impl HybridClock {
    /// Create a new HLC for the given client.
    pub fn new(client_id: ClientId) -> Self {
        Self {
            client_id,
            last_physical: AtomicU64::new(0),
            logical: AtomicU64::new(0),
        }
    }

    /// Get the client ID for this clock.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Generate a new timestamp.
    ///
    /// This is safe to call from multiple threads.
    pub fn now(&self) -> Timestamp {
        let physical = Self::current_physical_time();
        
        loop {
            let last = self.last_physical.load(AtomicOrdering::SeqCst);
            let logical = self.logical.load(AtomicOrdering::SeqCst);
            
            if physical > last {
                // Physical time has advanced
                if self.last_physical.compare_exchange(
                    last,
                    physical,
                    AtomicOrdering::SeqCst,
                    AtomicOrdering::SeqCst,
                ).is_ok() {
                    self.logical.store(0, AtomicOrdering::SeqCst);
                    return Timestamp::new(physical, 0, self.client_id);
                }
            } else {
                // Physical time hasn't advanced, increment logical
                let new_logical = logical + 1;
                if self.logical.compare_exchange(
                    logical,
                    new_logical,
                    AtomicOrdering::SeqCst,
                    AtomicOrdering::SeqCst,
                ).is_ok() {
                    return Timestamp::new(last, new_logical, self.client_id);
                }
            }
            // CAS failed, retry
        }
    }

    /// Update the clock based on a received timestamp.
    ///
    /// This ensures that subsequent timestamps from this clock will be
    /// greater than the received timestamp, preserving causality.
    pub fn update(&self, received: Timestamp) -> Timestamp {
        loop {
            let physical = Self::current_physical_time();
            let last = self.last_physical.load(AtomicOrdering::SeqCst);
            let logical = self.logical.load(AtomicOrdering::SeqCst);
            
            let max_physical = physical.max(last).max(received.physical);
            
            let new_logical = if max_physical == received.physical && max_physical == last {
                logical.max(received.logical) + 1
            } else if max_physical == received.physical {
                received.logical + 1
            } else if max_physical == last {
                logical + 1
            } else {
                0
            };
            
            if self.last_physical.compare_exchange(
                last,
                max_physical,
                AtomicOrdering::SeqCst,
                AtomicOrdering::SeqCst,
            ).is_ok() && self.logical.compare_exchange(
                logical,
                new_logical,
                AtomicOrdering::SeqCst,
                AtomicOrdering::SeqCst,
            ).is_ok() {
                return Timestamp::new(max_physical, new_logical, self.client_id);
            }
            // CAS failed, retry
        }
    }

    /// Get current physical time in milliseconds since UNIX epoch.
    fn current_physical_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

impl Clone for HybridClock {
    fn clone(&self) -> Self {
        Self {
            client_id: self.client_id,
            last_physical: AtomicU64::new(self.last_physical.load(AtomicOrdering::SeqCst)),
            logical: AtomicU64::new(self.logical.load(AtomicOrdering::SeqCst)),
        }
    }
}

impl fmt::Debug for HybridClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HybridClock")
            .field("client_id", &self.client_id)
            .field("last_physical", &self.last_physical.load(AtomicOrdering::SeqCst))
            .field("logical", &self.logical.load(AtomicOrdering::SeqCst))
            .finish()
    }
}

/// Type alias for HybridClock for API consistency.
pub type HybridLogicalClock = HybridClock;

/// Vector clock for tracking causal dependencies across clients.
///
/// Each entry maps a client ID to the highest sequence number seen from that client.
/// Used for:
/// - Determining if operations are causally related
/// - Finding operations that a client hasn't seen yet
/// - Detecting concurrent operations
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VectorClock {
    /// Map from client ID to highest seen sequence number
    entries: std::collections::HashMap<ClientId, u64>,
}

impl VectorClock {
    /// Create a new empty vector clock.
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }

    /// Get the sequence number for a client.
    pub fn get(&self, client_id: ClientId) -> u64 {
        self.entries.get(&client_id).copied().unwrap_or(0)
    }

    /// Set the sequence number for a client.
    pub fn set(&mut self, client_id: ClientId, seq: u64) {
        self.entries.insert(client_id, seq);
    }

    /// Increment and return the sequence number for a client.
    pub fn increment(&mut self, client_id: ClientId) -> u64 {
        let seq = self.get(client_id) + 1;
        self.set(client_id, seq);
        seq
    }

    /// Merge another clock into this one (taking max of each entry).
    pub fn merge(&mut self, other: &VectorClock) {
        for (&client_id, &seq) in &other.entries {
            let current = self.get(client_id);
            if seq > current {
                self.set(client_id, seq);
            }
        }
    }

    /// Check if this clock dominates another (all entries >= other's entries).
    pub fn dominates(&self, other: &VectorClock) -> bool {
        for (&client_id, &seq) in &other.entries {
            if self.get(client_id) < seq {
                return false;
            }
        }
        true
    }

    /// Check if this clock is concurrent with another (neither dominates the other).
    pub fn is_concurrent_with(&self, other: &VectorClock) -> bool {
        !self.dominates(other) && !other.dominates(self)
    }

    /// Get all entries in the clock.
    pub fn entries(&self) -> &std::collections::HashMap<ClientId, u64> {
        &self.entries
    }

    /// Check if the clock contains any entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of clients tracked by this clock.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Checks if this clock happened-before the other clock.
    ///
    /// Returns true if all components of this clock are <= the corresponding
    /// components of the other clock, and at least one component is strictly less.
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;

        // Check all clients in self
        for (&client_id, &value) in &self.entries {
            let other_value = other.get(client_id);
            if value > other_value {
                return false; // self has a larger value, so not happened-before
            }
            if value < other_value {
                strictly_less = true;
            }
        }

        // Check clients only in other
        for (&client_id, &value) in &other.entries {
            if !self.entries.contains_key(&client_id) && value > 0 {
                strictly_less = true;
            }
        }

        // If we have no entries and other has entries, we happened-before
        if self.entries.is_empty() && !other.entries.is_empty() {
            return true;
        }

        strictly_less
    }

    /// Checks if this clock and the other clock are concurrent.
    ///
    /// Events are concurrent if neither happened-before the other.
    pub fn concurrent(&self, other: &VectorClock) -> bool {
        !self.happened_before(other) && !other.happened_before(self) && self != other
    }

    /// Returns an iterator over all (client_id, value) pairs.
    pub fn iter(&self) -> impl Iterator<Item = (ClientId, u64)> + '_ {
        self.entries.iter().map(|(&k, &v)| (k, v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_timestamp_ordering() {
        let t1 = Timestamp::new(100, 0, ClientId::new(1));
        let t2 = Timestamp::new(100, 1, ClientId::new(1));
        let t3 = Timestamp::new(101, 0, ClientId::new(1));

        // Physical time takes priority
        assert!(t1 < t3);
        assert!(t2 < t3);

        // Logical breaks tie when physical is equal
        assert!(t1 < t2);
    }

    #[test]
    fn test_timestamp_client_id_tiebreaker() {
        let t1 = Timestamp::new(100, 0, ClientId::new(1));
        let t2 = Timestamp::new(100, 0, ClientId::new(2));

        // Higher client_id wins when physical and logical are equal
        assert!(t1 < t2);
    }

    #[test]
    fn test_timestamp_is_newer_than() {
        let t1 = Timestamp::new(100, 0, ClientId::new(1));
        let t2 = Timestamp::new(100, 1, ClientId::new(1));

        assert!(t2.is_newer_than(&t1));
        assert!(t1.is_older_than(&t2));
        assert!(!t1.is_newer_than(&t2));
    }

    #[test]
    fn test_hlc_generates_increasing_timestamps() {
        let clock = HybridClock::new(ClientId::new(1));
        
        let t1 = clock.now();
        let t2 = clock.now();
        let t3 = clock.now();

        assert!(t1 < t2);
        assert!(t2 < t3);
    }

    #[test]
    fn test_hlc_update_preserves_causality() {
        let clock1 = HybridClock::new(ClientId::new(1));
        let clock2 = HybridClock::new(ClientId::new(2));

        let t1 = clock1.now();
        
        // Simulate receiving t1 at clock2
        let t2 = clock2.update(t1);
        
        // t2 should be greater than t1
        assert!(t2 > t1);
    }

    #[test]
    fn test_timestamp_epoch() {
        let t = Timestamp::epoch(ClientId::new(42));
        assert_eq!(t.physical, 0);
        assert_eq!(t.logical, 0);
        assert_eq!(t.client_id, ClientId::new(42));
    }

    #[test]
    fn test_timestamp_display() {
        let t = Timestamp::new(1000, 5, ClientId::new(42));
        assert_eq!(format!("{}", t), "Ts(1000.5.42)");
    }

    // ========== LamportClock Tests ==========

    #[test]
    fn test_lamport_clock_new() {
        let clock = LamportClock::new();
        assert_eq!(clock.value(), 0);
    }

    #[test]
    fn test_lamport_clock_with_value() {
        let clock = LamportClock::with_value(42);
        assert_eq!(clock.value(), 42);
    }

    #[test]
    fn test_lamport_clock_tick() {
        let mut clock = LamportClock::new();
        assert_eq!(clock.tick(), 1);
        assert_eq!(clock.tick(), 2);
        assert_eq!(clock.tick(), 3);
        assert_eq!(clock.value(), 3);
    }

    #[test]
    fn test_lamport_clock_update() {
        let mut clock = LamportClock::new();
        clock.tick(); // 1
        clock.tick(); // 2

        // Update with smaller value: should become max(2, 1) + 1 = 3
        clock.update(1);
        assert_eq!(clock.value(), 3);

        // Update with larger value: should become max(3, 10) + 1 = 11
        clock.update(10);
        assert_eq!(clock.value(), 11);

        // Update with equal value: should become max(11, 11) + 1 = 12
        clock.update(11);
        assert_eq!(clock.value(), 12);
    }

    #[test]
    fn test_lamport_clock_sync() {
        let mut clock = LamportClock::new();
        clock.tick(); // 1

        // Sync with larger value
        clock.sync(10);
        assert_eq!(clock.value(), 10);

        // Sync with smaller value (no change)
        clock.sync(5);
        assert_eq!(clock.value(), 10);
    }

    #[test]
    fn test_lamport_clock_serialization() {
        let mut clock = LamportClock::new();
        clock.tick();
        clock.tick();

        let json = serde_json::to_string(&clock).unwrap();
        let deserialized: LamportClock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.value(), 2);
    }

    // ========== VectorClock Tests ==========

    #[test]
    fn test_vector_clock_new() {
        let vc = VectorClock::new();
        assert!(vc.is_empty());
        assert_eq!(vc.len(), 0);
    }

    #[test]
    fn test_vector_clock_get_default() {
        let vc = VectorClock::new();
        assert_eq!(vc.get(ClientId::new(1)), 0);
    }

    #[test]
    fn test_vector_clock_set_and_get() {
        let mut vc = VectorClock::new();
        vc.set(ClientId::new(1), 5);
        assert_eq!(vc.get(ClientId::new(1)), 5);
        assert_eq!(vc.get(ClientId::new(2)), 0);
    }

    #[test]
    fn test_vector_clock_increment() {
        let mut vc = VectorClock::new();
        assert_eq!(vc.increment(ClientId::new(1)), 1);
        assert_eq!(vc.increment(ClientId::new(1)), 2);
        assert_eq!(vc.increment(ClientId::new(2)), 1);
        assert_eq!(vc.get(ClientId::new(1)), 2);
        assert_eq!(vc.get(ClientId::new(2)), 1);
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut vc1 = VectorClock::new();
        vc1.set(ClientId::new(1), 3);
        vc1.set(ClientId::new(2), 5);

        let mut vc2 = VectorClock::new();
        vc2.set(ClientId::new(1), 5);
        vc2.set(ClientId::new(3), 2);

        vc1.merge(&vc2);

        assert_eq!(vc1.get(ClientId::new(1)), 5); // max(3, 5)
        assert_eq!(vc1.get(ClientId::new(2)), 5); // unchanged
        assert_eq!(vc1.get(ClientId::new(3)), 2); // new
    }

    #[test]
    fn test_vector_clock_happened_before_simple() {
        let mut vc1 = VectorClock::new();
        vc1.set(ClientId::new(1), 1);

        let mut vc2 = VectorClock::new();
        vc2.set(ClientId::new(1), 2);

        assert!(vc1.happened_before(&vc2));
        assert!(!vc2.happened_before(&vc1));
    }

    #[test]
    fn test_vector_clock_happened_before_multiple_clients() {
        let mut vc1 = VectorClock::new();
        vc1.set(ClientId::new(1), 1);
        vc1.set(ClientId::new(2), 2);

        let mut vc2 = VectorClock::new();
        vc2.set(ClientId::new(1), 2);
        vc2.set(ClientId::new(2), 3);

        assert!(vc1.happened_before(&vc2));
        assert!(!vc2.happened_before(&vc1));
    }

    #[test]
    fn test_vector_clock_concurrent() {
        let mut vc1 = VectorClock::new();
        vc1.set(ClientId::new(1), 1);

        let mut vc2 = VectorClock::new();
        vc2.set(ClientId::new(2), 1);

        assert!(vc1.concurrent(&vc2));
        assert!(vc2.concurrent(&vc1));
        assert!(!vc1.happened_before(&vc2));
        assert!(!vc2.happened_before(&vc1));
    }

    #[test]
    fn test_vector_clock_concurrent_divergent() {
        let mut vc1 = VectorClock::new();
        vc1.set(ClientId::new(1), 2);
        vc1.set(ClientId::new(2), 1);

        let mut vc2 = VectorClock::new();
        vc2.set(ClientId::new(1), 1);
        vc2.set(ClientId::new(2), 2);

        // Neither dominates the other
        assert!(vc1.concurrent(&vc2));
    }

    #[test]
    fn test_vector_clock_equal_not_concurrent() {
        let mut vc1 = VectorClock::new();
        vc1.set(ClientId::new(1), 1);

        let mut vc2 = VectorClock::new();
        vc2.set(ClientId::new(1), 1);

        // Equal clocks are not concurrent (same event)
        assert!(!vc1.concurrent(&vc2));
        assert!(!vc1.happened_before(&vc2));
        assert!(!vc2.happened_before(&vc1));
    }

    #[test]
    fn test_vector_clock_dominates() {
        let mut vc1 = VectorClock::new();
        vc1.set(ClientId::new(1), 1);
        vc1.set(ClientId::new(2), 2);

        let mut vc2 = VectorClock::new();
        vc2.set(ClientId::new(1), 2);
        vc2.set(ClientId::new(2), 3);

        assert!(vc2.dominates(&vc1));
        assert!(!vc1.dominates(&vc2));
    }

    #[test]
    fn test_vector_clock_iter() {
        let mut vc = VectorClock::new();
        vc.set(ClientId::new(1), 3);
        vc.set(ClientId::new(2), 5);

        let entries: HashMap<ClientId, u64> = vc.iter().collect();
        assert_eq!(entries.get(&ClientId::new(1)), Some(&3));
        assert_eq!(entries.get(&ClientId::new(2)), Some(&5));
    }

    #[test]
    fn test_vector_clock_serialization() {
        let mut vc = VectorClock::new();
        vc.set(ClientId::new(1), 3);
        vc.set(ClientId::new(2), 5);

        let json = serde_json::to_string(&vc).unwrap();
        let deserialized: VectorClock = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.get(ClientId::new(1)), 3);
        assert_eq!(deserialized.get(ClientId::new(2)), 5);
    }

    #[test]
    fn test_vector_clock_causality_scenario() {
        // Simulate a scenario:
        // Client 1 creates event A
        // Client 1 sends message to Client 2
        // Client 2 creates event B (after receiving A)
        // Event A happened-before Event B

        let client1 = ClientId::new(1);
        let client2 = ClientId::new(2);

        // Client 1 creates event A
        let mut vc_a = VectorClock::new();
        vc_a.increment(client1);

        // Client 2 receives message from Client 1, creates event B
        let mut vc_b = vc_a.clone();
        vc_b.increment(client2);

        // A happened-before B
        assert!(vc_a.happened_before(&vc_b));
        assert!(!vc_b.happened_before(&vc_a));
        assert!(!vc_a.concurrent(&vc_b));
    }

    #[test]
    fn test_vector_clock_concurrent_scenario() {
        // Simulate concurrent events:
        // Client 1 and Client 2 both create events without seeing each other

        let client1 = ClientId::new(1);
        let client2 = ClientId::new(2);

        // Client 1 creates event A
        let mut vc_a = VectorClock::new();
        vc_a.increment(client1);

        // Client 2 creates event B (independently)
        let mut vc_b = VectorClock::new();
        vc_b.increment(client2);

        // A and B are concurrent
        assert!(vc_a.concurrent(&vc_b));
        assert!(!vc_a.happened_before(&vc_b));
        assert!(!vc_b.happened_before(&vc_a));
    }
}

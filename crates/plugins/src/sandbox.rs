//! Sandbox configuration and resource limits for plugins
//!
//! This module defines the sandbox boundaries and resource limits
//! that constrain plugin execution for security and stability.

use crate::manifest::Permission;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;

/// Configuration for the plugin sandbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum CPU time per operation
    pub max_cpu_time: Duration,
    /// Maximum execution time for any operation
    pub max_execution_time: Duration,
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: u32,
    /// Maximum number of API calls per minute
    pub max_api_calls_per_minute: u32,
    /// Maximum size of stored data in bytes
    pub max_storage_bytes: u64,
    /// Maximum network request size in bytes
    pub max_network_request_size: u64,
    /// Maximum network response size in bytes
    pub max_network_response_size: u64,
    /// Allowed network hosts (empty means all allowed if Network permission granted)
    pub allowed_hosts: Vec<String>,
    /// Blocked network hosts
    pub blocked_hosts: Vec<String>,
    /// Whether file system access is allowed (always sandboxed to plugin directory)
    pub allow_file_access: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory_bytes: 64 * 1024 * 1024,       // 64 MB
            max_cpu_time: Duration::from_secs(5),     // 5 seconds CPU time
            max_execution_time: Duration::from_secs(30), // 30 seconds wall time
            max_concurrent_operations: 4,
            max_api_calls_per_minute: 1000,
            max_storage_bytes: 10 * 1024 * 1024,      // 10 MB
            max_network_request_size: 1024 * 1024,    // 1 MB
            max_network_response_size: 10 * 1024 * 1024, // 10 MB
            allowed_hosts: Vec::new(),
            blocked_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
            ],
            allow_file_access: false,
        }
    }
}

impl SandboxConfig {
    /// Create a new sandbox configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a restrictive sandbox configuration
    pub fn restrictive() -> Self {
        Self {
            max_memory_bytes: 16 * 1024 * 1024,       // 16 MB
            max_cpu_time: Duration::from_secs(1),
            max_execution_time: Duration::from_secs(10),
            max_concurrent_operations: 1,
            max_api_calls_per_minute: 100,
            max_storage_bytes: 1024 * 1024,           // 1 MB
            max_network_request_size: 256 * 1024,     // 256 KB
            max_network_response_size: 1024 * 1024,   // 1 MB
            allowed_hosts: Vec::new(),
            blocked_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
            ],
            allow_file_access: false,
        }
    }

    /// Create a permissive sandbox configuration (for trusted plugins)
    pub fn permissive() -> Self {
        Self {
            max_memory_bytes: 256 * 1024 * 1024,      // 256 MB
            max_cpu_time: Duration::from_secs(30),
            max_execution_time: Duration::from_secs(120),
            max_concurrent_operations: 16,
            max_api_calls_per_minute: 10000,
            max_storage_bytes: 100 * 1024 * 1024,     // 100 MB
            max_network_request_size: 10 * 1024 * 1024, // 10 MB
            max_network_response_size: 100 * 1024 * 1024, // 100 MB
            allowed_hosts: Vec::new(),
            blocked_hosts: Vec::new(),
            allow_file_access: true,
        }
    }

    /// Set maximum memory
    pub fn with_max_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = bytes;
        self
    }

    /// Set maximum CPU time
    pub fn with_max_cpu_time(mut self, duration: Duration) -> Self {
        self.max_cpu_time = duration;
        self
    }

    /// Set maximum execution time
    pub fn with_max_execution_time(mut self, duration: Duration) -> Self {
        self.max_execution_time = duration;
        self
    }

    /// Add allowed host
    pub fn with_allowed_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.push(host.into());
        self
    }

    /// Add blocked host
    pub fn with_blocked_host(mut self, host: impl Into<String>) -> Self {
        self.blocked_hosts.push(host.into());
        self
    }

    /// Check if a host is allowed for network requests
    pub fn is_host_allowed(&self, host: &str) -> bool {
        // Check blocked list first
        if self.blocked_hosts.iter().any(|h| host.contains(h)) {
            return false;
        }

        // If allowed list is empty, all non-blocked hosts are allowed
        if self.allowed_hosts.is_empty() {
            return true;
        }

        // Check allowed list
        self.allowed_hosts.iter().any(|h| host.contains(h))
    }
}

/// API surface restrictions based on permissions
#[derive(Debug, Clone)]
pub struct ApiRestrictions {
    /// Available API methods based on granted permissions
    available_methods: HashSet<String>,
}

impl ApiRestrictions {
    /// Create API restrictions based on granted permissions
    pub fn from_permissions(permissions: &[Permission]) -> Self {
        let mut methods = HashSet::new();

        for permission in permissions {
            match permission {
                Permission::DocumentRead => {
                    methods.insert("document.get".to_string());
                    methods.insert("document.getText".to_string());
                    methods.insert("document.getSelection".to_string());
                    methods.insert("document.search".to_string());
                }
                Permission::DocumentWrite => {
                    methods.insert("document.set".to_string());
                    methods.insert("document.insert".to_string());
                    methods.insert("document.delete".to_string());
                    methods.insert("document.replace".to_string());
                    methods.insert("document.format".to_string());
                }
                Permission::UiToolbar => {
                    methods.insert("ui.addToolbarItem".to_string());
                    methods.insert("ui.removeToolbarItem".to_string());
                    methods.insert("ui.updateToolbarItem".to_string());
                }
                Permission::UiPanel => {
                    methods.insert("ui.createPanel".to_string());
                    methods.insert("ui.showPanel".to_string());
                    methods.insert("ui.hidePanel".to_string());
                    methods.insert("ui.updatePanel".to_string());
                }
                Permission::UiDialog => {
                    methods.insert("ui.showDialog".to_string());
                    methods.insert("ui.showMessage".to_string());
                    methods.insert("ui.showInput".to_string());
                    methods.insert("ui.showConfirm".to_string());
                }
                Permission::Network => {
                    methods.insert("network.fetch".to_string());
                    methods.insert("network.httpGet".to_string());
                    methods.insert("network.httpPost".to_string());
                }
                Permission::Storage => {
                    methods.insert("storage.get".to_string());
                    methods.insert("storage.set".to_string());
                    methods.insert("storage.delete".to_string());
                    methods.insert("storage.list".to_string());
                }
                Permission::Clipboard => {
                    methods.insert("clipboard.read".to_string());
                    methods.insert("clipboard.write".to_string());
                }
            }
        }

        // Always allow these basic methods
        methods.insert("log.info".to_string());
        methods.insert("log.warn".to_string());
        methods.insert("log.error".to_string());

        Self {
            available_methods: methods,
        }
    }

    /// Check if a method is allowed
    pub fn is_method_allowed(&self, method: &str) -> bool {
        self.available_methods.contains(method)
    }

    /// Get all available methods
    pub fn get_available_methods(&self) -> Vec<&str> {
        self.available_methods.iter().map(|s| s.as_str()).collect()
    }

    /// Add a custom method
    pub fn add_method(&mut self, method: impl Into<String>) {
        self.available_methods.insert(method.into());
    }

    /// Remove a method
    pub fn remove_method(&mut self, method: &str) {
        self.available_methods.remove(method);
    }
}

/// Resource usage tracking for a plugin
#[derive(Debug, Clone, Default)]
pub struct ResourceUsage {
    /// Current memory usage in bytes
    pub memory_bytes: u64,
    /// Total CPU time used
    pub cpu_time: Duration,
    /// Number of API calls in the current minute
    pub api_calls_this_minute: u32,
    /// Storage bytes used
    pub storage_bytes: u64,
    /// Number of active operations
    pub active_operations: u32,
}

impl ResourceUsage {
    /// Create new resource usage tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if memory limit is exceeded
    pub fn is_memory_exceeded(&self, config: &SandboxConfig) -> bool {
        self.memory_bytes > config.max_memory_bytes
    }

    /// Check if CPU time limit is exceeded
    pub fn is_cpu_time_exceeded(&self, config: &SandboxConfig) -> bool {
        self.cpu_time > config.max_cpu_time
    }

    /// Check if API rate limit is exceeded
    pub fn is_rate_limit_exceeded(&self, config: &SandboxConfig) -> bool {
        self.api_calls_this_minute > config.max_api_calls_per_minute
    }

    /// Check if storage limit is exceeded
    pub fn is_storage_exceeded(&self, config: &SandboxConfig) -> bool {
        self.storage_bytes > config.max_storage_bytes
    }

    /// Check if concurrent operation limit is exceeded
    pub fn is_concurrent_limit_exceeded(&self, config: &SandboxConfig) -> bool {
        self.active_operations >= config.max_concurrent_operations
    }

    /// Check if any limit is exceeded
    pub fn is_any_limit_exceeded(&self, config: &SandboxConfig) -> Option<ResourceLimitViolation> {
        if self.is_memory_exceeded(config) {
            return Some(ResourceLimitViolation::Memory {
                used: self.memory_bytes,
                limit: config.max_memory_bytes,
            });
        }
        if self.is_cpu_time_exceeded(config) {
            return Some(ResourceLimitViolation::CpuTime {
                used: self.cpu_time,
                limit: config.max_cpu_time,
            });
        }
        if self.is_rate_limit_exceeded(config) {
            return Some(ResourceLimitViolation::RateLimit {
                calls: self.api_calls_this_minute,
                limit: config.max_api_calls_per_minute,
            });
        }
        if self.is_storage_exceeded(config) {
            return Some(ResourceLimitViolation::Storage {
                used: self.storage_bytes,
                limit: config.max_storage_bytes,
            });
        }
        if self.is_concurrent_limit_exceeded(config) {
            return Some(ResourceLimitViolation::ConcurrentOperations {
                active: self.active_operations,
                limit: config.max_concurrent_operations,
            });
        }
        None
    }

    /// Reset API call counter (called every minute)
    pub fn reset_rate_limit(&mut self) {
        self.api_calls_this_minute = 0;
    }

    /// Increment API call counter
    pub fn record_api_call(&mut self) {
        self.api_calls_this_minute += 1;
    }

    /// Start an operation
    pub fn start_operation(&mut self) {
        self.active_operations += 1;
    }

    /// End an operation
    pub fn end_operation(&mut self) {
        self.active_operations = self.active_operations.saturating_sub(1);
    }

    /// Update memory usage
    pub fn update_memory(&mut self, bytes: u64) {
        self.memory_bytes = bytes;
    }

    /// Add CPU time
    pub fn add_cpu_time(&mut self, duration: Duration) {
        self.cpu_time += duration;
    }

    /// Update storage usage
    pub fn update_storage(&mut self, bytes: u64) {
        self.storage_bytes = bytes;
    }
}

/// Types of resource limit violations
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceLimitViolation {
    Memory { used: u64, limit: u64 },
    CpuTime { used: Duration, limit: Duration },
    RateLimit { calls: u32, limit: u32 },
    Storage { used: u64, limit: u64 },
    ConcurrentOperations { active: u32, limit: u32 },
}

impl std::fmt::Display for ResourceLimitViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Memory { used, limit } => {
                write!(f, "Memory limit exceeded: {} / {} bytes", used, limit)
            }
            Self::CpuTime { used, limit } => {
                write!(f, "CPU time limit exceeded: {:?} / {:?}", used, limit)
            }
            Self::RateLimit { calls, limit } => {
                write!(f, "Rate limit exceeded: {} / {} calls/minute", calls, limit)
            }
            Self::Storage { used, limit } => {
                write!(f, "Storage limit exceeded: {} / {} bytes", used, limit)
            }
            Self::ConcurrentOperations { active, limit } => {
                write!(
                    f,
                    "Concurrent operation limit exceeded: {} / {}",
                    active, limit
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.max_memory_bytes, 64 * 1024 * 1024);
        assert_eq!(config.max_cpu_time, Duration::from_secs(5));
    }

    #[test]
    fn test_sandbox_config_restrictive() {
        let config = SandboxConfig::restrictive();
        assert!(config.max_memory_bytes < SandboxConfig::default().max_memory_bytes);
    }

    #[test]
    fn test_sandbox_config_permissive() {
        let config = SandboxConfig::permissive();
        assert!(config.max_memory_bytes > SandboxConfig::default().max_memory_bytes);
    }

    #[test]
    fn test_sandbox_config_builder() {
        let config = SandboxConfig::new()
            .with_max_memory(1024)
            .with_max_cpu_time(Duration::from_secs(1))
            .with_allowed_host("api.example.com");

        assert_eq!(config.max_memory_bytes, 1024);
        assert_eq!(config.max_cpu_time, Duration::from_secs(1));
        assert!(config.allowed_hosts.contains(&"api.example.com".to_string()));
    }

    #[test]
    fn test_host_allowed_blocked() {
        let config = SandboxConfig::default();
        assert!(!config.is_host_allowed("localhost"));
        assert!(!config.is_host_allowed("127.0.0.1"));
        assert!(config.is_host_allowed("api.example.com"));
    }

    #[test]
    fn test_host_allowed_whitelist() {
        let config = SandboxConfig::new()
            .with_allowed_host("api.example.com")
            .with_allowed_host("cdn.example.com");

        assert!(config.is_host_allowed("api.example.com"));
        assert!(config.is_host_allowed("cdn.example.com"));
        assert!(!config.is_host_allowed("other.com"));
    }

    #[test]
    fn test_api_restrictions_from_permissions() {
        let restrictions =
            ApiRestrictions::from_permissions(&[Permission::DocumentRead, Permission::UiDialog]);

        assert!(restrictions.is_method_allowed("document.get"));
        assert!(restrictions.is_method_allowed("ui.showDialog"));
        assert!(!restrictions.is_method_allowed("document.set"));
        assert!(!restrictions.is_method_allowed("network.fetch"));
    }

    #[test]
    fn test_api_restrictions_basic_methods() {
        let restrictions = ApiRestrictions::from_permissions(&[]);

        // Basic logging should always be available
        assert!(restrictions.is_method_allowed("log.info"));
        assert!(restrictions.is_method_allowed("log.warn"));
        assert!(restrictions.is_method_allowed("log.error"));
    }

    #[test]
    fn test_api_restrictions_add_remove() {
        let mut restrictions = ApiRestrictions::from_permissions(&[]);

        restrictions.add_method("custom.method");
        assert!(restrictions.is_method_allowed("custom.method"));

        restrictions.remove_method("custom.method");
        assert!(!restrictions.is_method_allowed("custom.method"));
    }

    #[test]
    fn test_resource_usage_new() {
        let usage = ResourceUsage::new();
        assert_eq!(usage.memory_bytes, 0);
        assert_eq!(usage.api_calls_this_minute, 0);
    }

    #[test]
    fn test_resource_usage_limits() {
        let config = SandboxConfig::new().with_max_memory(1000);
        let mut usage = ResourceUsage::new();

        usage.update_memory(500);
        assert!(!usage.is_memory_exceeded(&config));

        usage.update_memory(1500);
        assert!(usage.is_memory_exceeded(&config));
    }

    #[test]
    fn test_resource_usage_rate_limit() {
        let mut config = SandboxConfig::new();
        config.max_api_calls_per_minute = 10;

        let mut usage = ResourceUsage::new();
        for _ in 0..10 {
            usage.record_api_call();
        }
        assert!(!usage.is_rate_limit_exceeded(&config));

        usage.record_api_call();
        assert!(usage.is_rate_limit_exceeded(&config));

        usage.reset_rate_limit();
        assert!(!usage.is_rate_limit_exceeded(&config));
    }

    #[test]
    fn test_resource_usage_operations() {
        let mut config = SandboxConfig::new();
        config.max_concurrent_operations = 2;

        let mut usage = ResourceUsage::new();
        usage.start_operation();
        usage.start_operation();
        assert!(usage.is_concurrent_limit_exceeded(&config));

        usage.end_operation();
        assert!(!usage.is_concurrent_limit_exceeded(&config));
    }

    #[test]
    fn test_resource_usage_any_limit_exceeded() {
        let config = SandboxConfig::new().with_max_memory(100);
        let mut usage = ResourceUsage::new();

        assert!(usage.is_any_limit_exceeded(&config).is_none());

        usage.update_memory(200);
        let violation = usage.is_any_limit_exceeded(&config);
        assert!(matches!(violation, Some(ResourceLimitViolation::Memory { .. })));
    }

    #[test]
    fn test_resource_limit_violation_display() {
        let violation = ResourceLimitViolation::Memory {
            used: 100,
            limit: 50,
        };
        assert!(violation.to_string().contains("Memory limit exceeded"));
    }

    #[test]
    fn test_sandbox_config_serialization() {
        let config = SandboxConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: SandboxConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.max_memory_bytes, parsed.max_memory_bytes);
    }
}

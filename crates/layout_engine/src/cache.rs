//! Layout cache for efficient incremental layout
//!
//! This module provides caching for layout computations, enabling efficient
//! incremental updates when the document is edited. The cache stores:
//! - Line break results for paragraphs
//! - Page break positions
//! - Layout metrics for quick access
//!
//! Cache invalidation is granular: editing a paragraph only invalidates
//! that paragraph's cached data, allowing reuse of other cached layouts.
//!
//! # Features
//!
//! - **LRU eviction policy**: Automatically evicts least-recently-used entries
//!   when the cache reaches its size limit
//! - **Content-based cache keys**: Uses content hash + formatting properties
//!   to detect when cached layouts can be reused
//! - **Partial invalidation**: Only invalidates affected pages when content changes
//! - **Statistics tracking**: Monitors cache hit/miss rates for performance tuning

use crate::LineBox;
use doc_model::NodeId;
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};

// Use the standard library's DefaultHasher for content hashing
// In production, consider xxhash or similar for better performance
use std::collections::hash_map::DefaultHasher;

/// Configuration for the layout cache
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of paragraph entries in the cache
    pub max_paragraph_entries: usize,
    /// Maximum number of page entries in the cache
    pub max_page_entries: usize,
    /// Enable detailed statistics collection
    pub collect_detailed_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_paragraph_entries: 1000,
            max_page_entries: 100,
            collect_detailed_stats: false,
        }
    }
}

impl CacheConfig {
    /// Create a cache config with specified limits
    pub fn with_limits(max_paragraphs: usize, max_pages: usize) -> Self {
        Self {
            max_paragraph_entries: max_paragraphs,
            max_page_entries: max_pages,
            collect_detailed_stats: false,
        }
    }

    /// Enable detailed statistics collection
    pub fn with_detailed_stats(mut self) -> Self {
        self.collect_detailed_stats = true;
        self
    }
}

/// Cache key for paragraph layouts based on content and formatting
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParagraphCacheKey {
    /// Hash of the paragraph content
    pub content_hash: u64,
    /// Available width (stored as fixed-point to allow hashing)
    pub width_fixed: u32,
    /// Hash of the paragraph style properties
    pub style_hash: u64,
}

impl ParagraphCacheKey {
    /// Create a new cache key
    pub fn new(content_hash: u64, width: f32, style_hash: u64) -> Self {
        // Convert width to fixed-point (hundredths of a point)
        let width_fixed = (width * 100.0) as u32;
        Self {
            content_hash,
            width_fixed,
            style_hash,
        }
    }

    /// Get the width as a float
    pub fn width(&self) -> f32 {
        self.width_fixed as f32 / 100.0
    }
}

/// Cached line information for quick height calculations
#[derive(Debug, Clone)]
pub struct CachedLineInfo {
    /// Height of the line
    pub height: f32,
    /// Width of content (excluding trailing whitespace)
    pub content_width: f32,
    /// Baseline position within the line
    pub baseline: f32,
}

impl CachedLineInfo {
    /// Create from a LineBox
    pub fn from_line_box(line: &LineBox) -> Self {
        Self {
            height: line.bounds.height,
            content_width: line.bounds.width,
            baseline: line.baseline,
        }
    }
}

/// Cached paragraph layout information
#[derive(Debug, Clone)]
pub struct CachedParagraphLayout {
    /// Version when this was cached
    pub version: u64,
    /// Available width when cached
    pub width: f32,
    /// Cached line information
    pub lines: Vec<CachedLineInfo>,
    /// Full line boxes for reuse
    pub line_boxes: Vec<LineBox>,
    /// Total height of the paragraph
    pub total_height: f32,
    /// Number of lines
    pub line_count: usize,
    /// Content hash when cached
    pub content_hash: u64,
    /// Style hash when cached
    pub style_hash: u64,
}

impl CachedParagraphLayout {
    /// Create from line boxes
    pub fn from_lines(version: u64, width: f32, lines: &[LineBox], total_height: f32) -> Self {
        Self::from_lines_with_hashes(version, width, lines, total_height, 0, 0)
    }

    /// Create from line boxes with content and style hashes
    pub fn from_lines_with_hashes(
        version: u64,
        width: f32,
        lines: &[LineBox],
        total_height: f32,
        content_hash: u64,
        style_hash: u64,
    ) -> Self {
        let cached_lines: Vec<CachedLineInfo> = lines
            .iter()
            .map(CachedLineInfo::from_line_box)
            .collect();
        let line_count = cached_lines.len();

        Self {
            version,
            width,
            lines: cached_lines,
            line_boxes: lines.to_vec(),
            total_height,
            line_count,
            content_hash,
            style_hash,
        }
    }

    /// Get the height of lines up to (but not including) a given line index
    pub fn height_before_line(&self, line_index: usize) -> f32 {
        self.lines
            .iter()
            .take(line_index)
            .map(|l| l.height)
            .sum()
    }

    /// Get the height of lines from a given index to the end
    pub fn height_from_line(&self, line_index: usize) -> f32 {
        self.lines
            .iter()
            .skip(line_index)
            .map(|l| l.height)
            .sum()
    }
}

/// Page break information for incremental reflow
#[derive(Debug, Clone, PartialEq)]
pub struct PageBreakInfo {
    /// Paragraph where the break occurs
    pub paragraph_id: NodeId,
    /// Line index within the paragraph (0 = break before paragraph)
    pub line_index: usize,
    /// Y offset within the page where this content starts
    pub y_offset: f32,
}

/// Cached page layout information
#[derive(Debug, Clone)]
pub struct CachedPageLayout {
    /// Page index
    pub page_index: usize,
    /// Paragraphs on this page (node IDs)
    pub paragraphs: Vec<NodeId>,
    /// First line index for partial paragraphs at page start
    pub first_line_index: usize,
    /// Last line index for partial paragraphs at page end
    pub last_line_index: Option<usize>,
    /// Total content height on this page
    pub content_height: f32,
    /// Version when cached
    pub version: u64,
}

/// Cache statistics for monitoring performance
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cache hits for paragraphs
    pub paragraph_hits: u64,
    /// Number of cache misses for paragraphs
    pub paragraph_misses: u64,
    /// Number of cache hits for pages
    pub page_hits: u64,
    /// Number of cache misses for pages
    pub page_misses: u64,
    /// Number of evictions due to cache size limits
    pub evictions: u64,
    /// Number of invalidations
    pub invalidations: u64,
    /// Number of full cache clears
    pub full_clears: u64,
}

impl CacheStats {
    /// Get the paragraph cache hit ratio (0.0 to 1.0)
    pub fn paragraph_hit_ratio(&self) -> f32 {
        let total = self.paragraph_hits + self.paragraph_misses;
        if total == 0 {
            0.0
        } else {
            self.paragraph_hits as f32 / total as f32
        }
    }

    /// Get the page cache hit ratio (0.0 to 1.0)
    pub fn page_hit_ratio(&self) -> f32 {
        let total = self.page_hits + self.page_misses;
        if total == 0 {
            0.0
        } else {
            self.page_hits as f32 / total as f32
        }
    }

    /// Get the overall hit ratio (0.0 to 1.0)
    pub fn overall_hit_ratio(&self) -> f32 {
        let total_hits = self.paragraph_hits + self.page_hits;
        let total_misses = self.paragraph_misses + self.page_misses;
        let total = total_hits + total_misses;
        if total == 0 {
            0.0
        } else {
            total_hits as f32 / total as f32
        }
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Layout cache for storing computed layouts
///
/// This cache implements an LRU (Least Recently Used) eviction policy
/// to manage memory usage while maximizing cache hit rates.
#[derive(Debug)]
pub struct LayoutCache {
    /// Cached paragraph layouts by node ID
    paragraph_cache: HashMap<NodeId, CachedParagraphLayout>,
    /// Secondary index: cache key to node ID for content-based lookups
    key_to_node: HashMap<ParagraphCacheKey, NodeId>,
    /// LRU order for paragraphs (front = most recently used)
    paragraph_lru: VecDeque<NodeId>,
    /// Cached page layouts
    page_cache: HashMap<usize, CachedPageLayout>,
    /// LRU order for pages (front = most recently used)
    page_lru: VecDeque<usize>,
    /// Cached page breaks for incremental reflow
    page_breaks: Vec<PageBreakInfo>,
    /// Current document version
    document_version: u64,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: CacheStats,
}

impl LayoutCache {
    /// Create a new empty layout cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new layout cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            paragraph_cache: HashMap::new(),
            key_to_node: HashMap::new(),
            paragraph_lru: VecDeque::new(),
            page_cache: HashMap::new(),
            page_lru: VecDeque::new(),
            page_breaks: Vec::new(),
            document_version: 0,
            config,
            stats: CacheStats::default(),
        }
    }

    /// Get the cache configuration
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Update the cache configuration
    pub fn set_config(&mut self, config: CacheConfig) {
        self.config = config;
        // Enforce new limits
        self.enforce_paragraph_limit();
        self.enforce_page_limit();
    }

    /// Get the current document version
    pub fn document_version(&self) -> u64 {
        self.document_version
    }

    /// Get the number of cached paragraphs
    pub fn cached_count(&self) -> usize {
        self.paragraph_cache.len()
    }

    /// Get the number of cached pages
    pub fn cached_page_count(&self) -> usize {
        self.page_cache.len()
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get cache hit ratio (backward compatibility)
    pub fn hit_ratio(&self) -> f32 {
        self.stats.paragraph_hit_ratio()
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats.reset();
    }

    // === Paragraph Cache Operations ===

    /// Check if a paragraph layout is cached and valid using the old API
    pub fn is_cached(&mut self, node_id: NodeId, version: u64, width: f32) -> bool {
        if let Some(cached) = self.paragraph_cache.get(&node_id) {
            let valid = cached.version == version && (cached.width - width).abs() < 0.01;
            if valid {
                self.stats.paragraph_hits += 1;
                self.touch_paragraph(node_id);
            } else {
                self.stats.paragraph_misses += 1;
            }
            valid
        } else {
            self.stats.paragraph_misses += 1;
            false
        }
    }

    /// Check if a paragraph layout is cached using content-based key
    pub fn is_cached_by_key(&mut self, key: &ParagraphCacheKey) -> bool {
        if let Some(&node_id) = self.key_to_node.get(key) {
            if self.paragraph_cache.contains_key(&node_id) {
                self.stats.paragraph_hits += 1;
                self.touch_paragraph(node_id);
                return true;
            }
        }
        self.stats.paragraph_misses += 1;
        false
    }

    /// Get cached paragraph layout if valid (old API)
    pub fn get_cached(
        &mut self,
        node_id: NodeId,
        version: u64,
        width: f32,
    ) -> Option<&CachedParagraphLayout> {
        if self.is_cached(node_id, version, width) {
            self.paragraph_cache.get(&node_id)
        } else {
            None
        }
    }

    /// Get cached paragraph layout by content-based key
    pub fn get_cached_by_key(&mut self, key: &ParagraphCacheKey) -> Option<&CachedParagraphLayout> {
        // First check if we have this key and the corresponding cache entry
        let node_id = match self.key_to_node.get(key) {
            Some(&id) if self.paragraph_cache.contains_key(&id) => id,
            _ => {
                self.stats.paragraph_misses += 1;
                return None;
            }
        };

        // Update stats and LRU
        self.stats.paragraph_hits += 1;
        self.touch_paragraph(node_id);

        // Now return the cached value
        self.paragraph_cache.get(&node_id)
    }

    /// Get cached line boxes for a paragraph
    pub fn get_cached_lines(&mut self, node_id: NodeId, version: u64, width: f32) -> Option<Vec<LineBox>> {
        if let Some(cached) = self.get_cached(node_id, version, width) {
            Some(cached.line_boxes.clone())
        } else {
            None
        }
    }

    /// Store a paragraph layout in the cache (old API for backward compatibility)
    pub fn store(
        &mut self,
        node_id: NodeId,
        version: u64,
        width: f32,
        lines: &[LineBox],
        total_height: f32,
    ) {
        // Use default hashes for backward compatibility
        self.store_with_hashes(node_id, version, width, lines, total_height, 0, 0);
    }

    /// Store a paragraph layout with content and style hashes
    pub fn store_with_hashes(
        &mut self,
        node_id: NodeId,
        version: u64,
        width: f32,
        lines: &[LineBox],
        total_height: f32,
        content_hash: u64,
        style_hash: u64,
    ) {
        // Remove old key-to-node mapping if exists
        if let Some(old_cached) = self.paragraph_cache.get(&node_id) {
            let old_key = ParagraphCacheKey::new(
                old_cached.content_hash,
                old_cached.width,
                old_cached.style_hash,
            );
            self.key_to_node.remove(&old_key);
        }

        // Create the cached layout
        let cached = CachedParagraphLayout::from_lines_with_hashes(
            version,
            width,
            lines,
            total_height,
            content_hash,
            style_hash,
        );

        // Create and store the key mapping
        let key = ParagraphCacheKey::new(content_hash, width, style_hash);
        self.key_to_node.insert(key, node_id);

        // Insert into cache
        let is_new = !self.paragraph_cache.contains_key(&node_id);
        self.paragraph_cache.insert(node_id, cached);

        // Update LRU
        if is_new {
            self.paragraph_lru.push_front(node_id);
        } else {
            self.touch_paragraph(node_id);
        }

        // Enforce size limit
        self.enforce_paragraph_limit();
    }

    /// Get cached total height for a paragraph
    pub fn get_height(&self, node_id: NodeId) -> Option<f32> {
        self.paragraph_cache.get(&node_id).map(|c| c.total_height)
    }

    /// Get cached line count for a paragraph
    pub fn get_line_count(&self, node_id: NodeId) -> Option<usize> {
        self.paragraph_cache.get(&node_id).map(|c| c.line_count)
    }

    // === Page Cache Operations ===

    /// Store a page layout in the cache
    pub fn store_page(&mut self, page: CachedPageLayout) {
        let page_index = page.page_index;
        let is_new = !self.page_cache.contains_key(&page_index);
        self.page_cache.insert(page_index, page);

        if is_new {
            self.page_lru.push_front(page_index);
        } else {
            self.touch_page(page_index);
        }

        self.enforce_page_limit();
    }

    /// Get a cached page layout
    pub fn get_page(&mut self, page_index: usize) -> Option<&CachedPageLayout> {
        if self.page_cache.contains_key(&page_index) {
            self.stats.page_hits += 1;
            self.touch_page(page_index);
            self.page_cache.get(&page_index)
        } else {
            self.stats.page_misses += 1;
            None
        }
    }

    /// Check if a page is cached and valid
    pub fn is_page_cached(&mut self, page_index: usize, version: u64) -> bool {
        if let Some(cached) = self.page_cache.get(&page_index) {
            if cached.version == version {
                self.stats.page_hits += 1;
                self.touch_page(page_index);
                return true;
            }
        }
        self.stats.page_misses += 1;
        false
    }

    // === Page Break Operations ===

    /// Store page break information
    pub fn store_page_breaks(&mut self, breaks: Vec<PageBreakInfo>) {
        self.page_breaks = breaks;
    }

    /// Get stored page breaks
    pub fn get_page_breaks(&self) -> &[PageBreakInfo] {
        &self.page_breaks
    }

    /// Check if page breaks have changed
    pub fn page_breaks_changed(&self, new_breaks: &[PageBreakInfo]) -> bool {
        self.page_breaks != new_breaks
    }

    /// Find the page containing a paragraph
    pub fn find_page_for_paragraph(&self, para_id: NodeId) -> Option<usize> {
        for (i, page_break) in self.page_breaks.iter().enumerate() {
            if page_break.paragraph_id == para_id {
                return Some(i);
            }
        }
        None
    }

    /// Get paragraphs that need re-layout starting from a given paragraph
    pub fn get_affected_paragraphs(&self, start_para_id: NodeId) -> Vec<NodeId> {
        let mut affected = Vec::new();
        let mut found_start = false;

        for page_break in &self.page_breaks {
            if page_break.paragraph_id == start_para_id {
                found_start = true;
            }
            if found_start {
                affected.push(page_break.paragraph_id);
            }
        }

        affected
    }

    // === Invalidation Operations ===

    /// Invalidate a paragraph's cached layout
    pub fn invalidate_paragraph(&mut self, node_id: NodeId) {
        if let Some(cached) = self.paragraph_cache.remove(&node_id) {
            // Remove key-to-node mapping
            let key = ParagraphCacheKey::new(
                cached.content_hash,
                cached.width,
                cached.style_hash,
            );
            self.key_to_node.remove(&key);

            // Remove from LRU
            self.paragraph_lru.retain(|&id| id != node_id);

            self.stats.invalidations += 1;
        }

        // Also invalidate page breaks since they may be affected
        self.page_breaks.clear();
    }

    /// Invalidate pages starting from a specific page index
    pub fn invalidate_pages_from(&mut self, start_page: usize) {
        // Remove all pages from start_page onwards
        let pages_to_remove: Vec<usize> = self.page_cache
            .keys()
            .filter(|&&idx| idx >= start_page)
            .copied()
            .collect();

        for page_idx in pages_to_remove {
            self.page_cache.remove(&page_idx);
            self.page_lru.retain(|&idx| idx != page_idx);
            self.stats.invalidations += 1;
        }
    }

    /// Invalidate a specific page
    pub fn invalidate_page(&mut self, page_index: usize) {
        if self.page_cache.remove(&page_index).is_some() {
            self.page_lru.retain(|&idx| idx != page_index);
            self.stats.invalidations += 1;
        }
    }

    /// Invalidate all cached layouts
    pub fn invalidate_all(&mut self) {
        self.paragraph_cache.clear();
        self.key_to_node.clear();
        self.paragraph_lru.clear();
        self.page_cache.clear();
        self.page_lru.clear();
        self.page_breaks.clear();
        self.document_version += 1;
        self.stats.full_clears += 1;
    }

    // === LRU Management ===

    /// Move a paragraph to the front of the LRU queue (most recently used)
    fn touch_paragraph(&mut self, node_id: NodeId) {
        // Remove from current position
        self.paragraph_lru.retain(|&id| id != node_id);
        // Add to front
        self.paragraph_lru.push_front(node_id);
    }

    /// Move a page to the front of the LRU queue
    fn touch_page(&mut self, page_index: usize) {
        self.page_lru.retain(|&idx| idx != page_index);
        self.page_lru.push_front(page_index);
    }

    /// Enforce the paragraph cache size limit using LRU eviction
    fn enforce_paragraph_limit(&mut self) {
        while self.paragraph_cache.len() > self.config.max_paragraph_entries {
            if let Some(node_id) = self.paragraph_lru.pop_back() {
                if let Some(cached) = self.paragraph_cache.remove(&node_id) {
                    let key = ParagraphCacheKey::new(
                        cached.content_hash,
                        cached.width,
                        cached.style_hash,
                    );
                    self.key_to_node.remove(&key);
                    self.stats.evictions += 1;
                }
            } else {
                break;
            }
        }
    }

    /// Enforce the page cache size limit using LRU eviction
    fn enforce_page_limit(&mut self) {
        while self.page_cache.len() > self.config.max_page_entries {
            if let Some(page_index) = self.page_lru.pop_back() {
                self.page_cache.remove(&page_index);
                self.stats.evictions += 1;
            } else {
                break;
            }
        }
    }
}

impl Default for LayoutCache {
    fn default() -> Self {
        Self::new()
    }
}

// === Hashing Utilities ===

/// Compute a hash for paragraph content
pub fn hash_paragraph_content(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

/// Compute a hash for paragraph style properties
///
/// This includes properties that affect layout:
/// - Font size
/// - Line spacing
/// - Indentation
/// - Alignment
pub fn hash_paragraph_style(
    font_size: f32,
    line_spacing: f32,
    first_line_indent: f32,
    left_indent: f32,
    right_indent: f32,
    alignment: u8,
) -> u64 {
    let mut hasher = DefaultHasher::new();

    // Convert floats to fixed-point for consistent hashing
    let font_size_fixed = (font_size * 100.0) as u32;
    let line_spacing_fixed = (line_spacing * 1000.0) as u32;
    let first_indent_fixed = (first_line_indent * 100.0) as i32;
    let left_indent_fixed = (left_indent * 100.0) as i32;
    let right_indent_fixed = (right_indent * 100.0) as i32;

    font_size_fixed.hash(&mut hasher);
    line_spacing_fixed.hash(&mut hasher);
    first_indent_fixed.hash(&mut hasher);
    left_indent_fixed.hash(&mut hasher);
    right_indent_fixed.hash(&mut hasher);
    alignment.hash(&mut hasher);

    hasher.finish()
}

/// Compute a combined hash for all runs in a paragraph
pub fn hash_paragraph_runs<'a, I>(runs: I) -> u64
where
    I: Iterator<Item = (&'a str, f32, bool, bool)>, // (text, font_size, bold, italic)
{
    let mut hasher = DefaultHasher::new();

    for (text, font_size, bold, italic) in runs {
        text.hash(&mut hasher);
        ((font_size * 100.0) as u32).hash(&mut hasher);
        bold.hash(&mut hasher);
        italic.hash(&mut hasher);
    }

    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Direction, Rect};

    fn create_test_lines(count: usize, height: f32) -> Vec<LineBox> {
        (0..count)
            .map(|i| LineBox {
                bounds: Rect::new(0.0, i as f32 * height, 400.0, height),
                baseline: height * 0.8,
                direction: Direction::Ltr,
                inlines: Vec::new(),
            })
            .collect()
    }

    #[test]
    fn test_cache_creation() {
        let cache = LayoutCache::new();
        assert_eq!(cache.cached_count(), 0);
        assert_eq!(cache.document_version(), 0);
    }

    #[test]
    fn test_cache_with_config() {
        let config = CacheConfig::with_limits(50, 10);
        let cache = LayoutCache::with_config(config);
        assert_eq!(cache.config().max_paragraph_entries, 50);
        assert_eq!(cache.config().max_page_entries, 10);
    }

    #[test]
    fn test_store_and_retrieve() {
        let mut cache = LayoutCache::new();
        let node_id = NodeId::new();
        let lines = create_test_lines(3, 20.0);

        cache.store(node_id, 1, 400.0, &lines, 60.0);

        assert_eq!(cache.cached_count(), 1);
        assert_eq!(cache.get_height(node_id), Some(60.0));
        assert_eq!(cache.get_line_count(node_id), Some(3));
    }

    #[test]
    fn test_store_with_hashes() {
        let mut cache = LayoutCache::new();
        let node_id = NodeId::new();
        let lines = create_test_lines(3, 20.0);

        let content_hash = hash_paragraph_content("Hello world");
        let style_hash = hash_paragraph_style(12.0, 1.0, 0.0, 0.0, 0.0, 0);

        cache.store_with_hashes(node_id, 1, 400.0, &lines, 60.0, content_hash, style_hash);

        // Should be retrievable by key
        let key = ParagraphCacheKey::new(content_hash, 400.0, style_hash);
        assert!(cache.is_cached_by_key(&key));
    }

    #[test]
    fn test_cache_validity() {
        let mut cache = LayoutCache::new();
        let node_id = NodeId::new();
        let lines = create_test_lines(3, 20.0);

        cache.store(node_id, 1, 400.0, &lines, 60.0);

        // Same version and width should be valid
        assert!(cache.is_cached(node_id, 1, 400.0));

        // Different version should be invalid
        assert!(!cache.is_cached(node_id, 2, 400.0));

        // Different width should be invalid
        assert!(!cache.is_cached(node_id, 1, 500.0));

        // Non-existent should be invalid
        let other_id = NodeId::new();
        assert!(!cache.is_cached(other_id, 1, 400.0));
    }

    #[test]
    fn test_invalidation() {
        let mut cache = LayoutCache::new();
        let node_id = NodeId::new();
        let lines = create_test_lines(3, 20.0);

        cache.store(node_id, 1, 400.0, &lines, 60.0);
        assert_eq!(cache.cached_count(), 1);

        cache.invalidate_paragraph(node_id);
        assert_eq!(cache.cached_count(), 0);
        assert!(!cache.is_cached(node_id, 1, 400.0));
    }

    #[test]
    fn test_invalidate_all() {
        let mut cache = LayoutCache::new();
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        let lines = create_test_lines(3, 20.0);

        cache.store(id1, 1, 400.0, &lines, 60.0);
        cache.store(id2, 1, 400.0, &lines, 60.0);
        assert_eq!(cache.cached_count(), 2);

        let old_version = cache.document_version();
        cache.invalidate_all();

        assert_eq!(cache.cached_count(), 0);
        assert!(cache.document_version() > old_version);
    }

    #[test]
    fn test_lru_eviction() {
        let config = CacheConfig::with_limits(3, 10);
        let mut cache = LayoutCache::with_config(config);
        let lines = create_test_lines(2, 20.0);

        // Add 4 items to a cache with limit of 3
        let ids: Vec<NodeId> = (0..4).map(|_| NodeId::new()).collect();

        for &id in &ids {
            cache.store(id, 1, 400.0, &lines, 40.0);
        }

        // Should only have 3 items
        assert_eq!(cache.cached_count(), 3);

        // First item should have been evicted
        assert!(!cache.paragraph_cache.contains_key(&ids[0]));

        // Stats should show eviction
        assert_eq!(cache.stats().evictions, 1);
    }

    #[test]
    fn test_lru_touch_on_access() {
        let config = CacheConfig::with_limits(3, 10);
        let mut cache = LayoutCache::with_config(config);
        let lines = create_test_lines(2, 20.0);

        let ids: Vec<NodeId> = (0..3).map(|_| NodeId::new()).collect();

        // Add 3 items
        for &id in &ids {
            cache.store(id, 1, 400.0, &lines, 40.0);
        }

        // Access the first item (moves it to front of LRU)
        cache.is_cached(ids[0], 1, 400.0);

        // Add a 4th item - should evict the second item (least recently used)
        let new_id = NodeId::new();
        cache.store(new_id, 1, 400.0, &lines, 40.0);

        // First item should still be present (was touched)
        assert!(cache.paragraph_cache.contains_key(&ids[0]));

        // Second item should have been evicted
        assert!(!cache.paragraph_cache.contains_key(&ids[1]));
    }

    #[test]
    fn test_statistics() {
        let mut cache = LayoutCache::new();
        let node_id = NodeId::new();
        let lines = create_test_lines(3, 20.0);

        cache.store(node_id, 1, 400.0, &lines, 60.0);

        // Reset stats
        cache.reset_stats();

        // 2 hits
        cache.is_cached(node_id, 1, 400.0);
        cache.is_cached(node_id, 1, 400.0);

        // 1 miss
        cache.is_cached(node_id, 2, 400.0);

        // Hit ratio should be 2/3
        let stats = cache.stats();
        assert_eq!(stats.paragraph_hits, 2);
        assert_eq!(stats.paragraph_misses, 1);
        assert!((stats.paragraph_hit_ratio() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_hit_ratio() {
        let mut cache = LayoutCache::new();
        let node_id = NodeId::new();
        let lines = create_test_lines(3, 20.0);

        cache.store(node_id, 1, 400.0, &lines, 60.0);

        // Reset stats
        cache.reset_stats();

        // 2 hits
        cache.is_cached(node_id, 1, 400.0);
        cache.is_cached(node_id, 1, 400.0);

        // 1 miss
        cache.is_cached(node_id, 2, 400.0);

        // Hit ratio should be 2/3
        assert!((cache.hit_ratio() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_cached_paragraph_layout() {
        let lines = create_test_lines(5, 20.0);
        let cached = CachedParagraphLayout::from_lines(1, 400.0, &lines, 100.0);

        assert_eq!(cached.line_count, 5);
        assert_eq!(cached.total_height, 100.0);

        // Height before line 2 should be 2 * 20 = 40
        assert_eq!(cached.height_before_line(2), 40.0);

        // Height from line 2 should be 3 * 20 = 60
        assert_eq!(cached.height_from_line(2), 60.0);
    }

    #[test]
    fn test_page_breaks() {
        let mut cache = LayoutCache::new();
        let id1 = NodeId::new();
        let id2 = NodeId::new();

        let breaks = vec![
            PageBreakInfo {
                paragraph_id: id1,
                line_index: 0,
                y_offset: 0.0,
            },
            PageBreakInfo {
                paragraph_id: id2,
                line_index: 3,
                y_offset: 100.0,
            },
        ];

        cache.store_page_breaks(breaks.clone());
        assert_eq!(cache.get_page_breaks().len(), 2);

        // Should find page for paragraph
        assert_eq!(cache.find_page_for_paragraph(id1), Some(0));
        assert_eq!(cache.find_page_for_paragraph(id2), Some(1));

        // Unknown paragraph
        let id3 = NodeId::new();
        assert_eq!(cache.find_page_for_paragraph(id3), None);

        // Check if page breaks changed
        assert!(!cache.page_breaks_changed(&breaks));

        let different_breaks = vec![PageBreakInfo {
            paragraph_id: id1,
            line_index: 0,
            y_offset: 0.0,
        }];
        assert!(cache.page_breaks_changed(&different_breaks));
    }

    #[test]
    fn test_page_cache() {
        let mut cache = LayoutCache::new();

        let page = CachedPageLayout {
            page_index: 0,
            paragraphs: vec![NodeId::new(), NodeId::new()],
            first_line_index: 0,
            last_line_index: None,
            content_height: 500.0,
            version: 1,
        };

        cache.store_page(page);
        assert_eq!(cache.cached_page_count(), 1);

        assert!(cache.is_page_cached(0, 1));
        assert!(!cache.is_page_cached(0, 2)); // wrong version
        assert!(!cache.is_page_cached(1, 1)); // wrong index
    }

    #[test]
    fn test_partial_page_invalidation() {
        let mut cache = LayoutCache::new();

        // Add pages 0, 1, 2, 3
        for i in 0..4 {
            let page = CachedPageLayout {
                page_index: i,
                paragraphs: vec![NodeId::new()],
                first_line_index: 0,
                last_line_index: None,
                content_height: 500.0,
                version: 1,
            };
            cache.store_page(page);
        }

        assert_eq!(cache.cached_page_count(), 4);

        // Invalidate from page 2 onwards
        cache.invalidate_pages_from(2);

        assert_eq!(cache.cached_page_count(), 2);
        assert!(cache.page_cache.contains_key(&0));
        assert!(cache.page_cache.contains_key(&1));
        assert!(!cache.page_cache.contains_key(&2));
        assert!(!cache.page_cache.contains_key(&3));
    }

    #[test]
    fn test_hash_functions() {
        // Content hash should be consistent
        let hash1 = hash_paragraph_content("Hello world");
        let hash2 = hash_paragraph_content("Hello world");
        assert_eq!(hash1, hash2);

        // Different content should have different hashes
        let hash3 = hash_paragraph_content("Different text");
        assert_ne!(hash1, hash3);

        // Style hash should be consistent
        let style1 = hash_paragraph_style(12.0, 1.0, 0.0, 0.0, 0.0, 0);
        let style2 = hash_paragraph_style(12.0, 1.0, 0.0, 0.0, 0.0, 0);
        assert_eq!(style1, style2);

        // Different style should have different hash
        let style3 = hash_paragraph_style(14.0, 1.0, 0.0, 0.0, 0.0, 0);
        assert_ne!(style1, style3);
    }

    #[test]
    fn test_paragraph_cache_key() {
        let key1 = ParagraphCacheKey::new(12345, 400.0, 67890);
        let key2 = ParagraphCacheKey::new(12345, 400.0, 67890);

        assert_eq!(key1, key2);
        assert_eq!(key1.width(), 400.0);

        // Small width differences should result in same key (fixed point)
        let key3 = ParagraphCacheKey::new(12345, 400.001, 67890);
        assert_eq!(key1.width_fixed, key3.width_fixed);
    }

    #[test]
    fn test_get_cached_lines() {
        let mut cache = LayoutCache::new();
        let node_id = NodeId::new();
        let lines = create_test_lines(3, 20.0);

        cache.store(node_id, 1, 400.0, &lines, 60.0);

        // Should return cloned lines
        let cached_lines = cache.get_cached_lines(node_id, 1, 400.0);
        assert!(cached_lines.is_some());
        assert_eq!(cached_lines.unwrap().len(), 3);

        // Invalid request should return None
        let invalid_lines = cache.get_cached_lines(node_id, 2, 400.0);
        assert!(invalid_lines.is_none());
    }

    #[test]
    fn test_page_lru_eviction() {
        let config = CacheConfig::with_limits(100, 2);
        let mut cache = LayoutCache::with_config(config);

        // Add 3 pages to a cache with limit of 2
        for i in 0..3 {
            let page = CachedPageLayout {
                page_index: i,
                paragraphs: vec![NodeId::new()],
                first_line_index: 0,
                last_line_index: None,
                content_height: 500.0,
                version: 1,
            };
            cache.store_page(page);
        }

        // Should only have 2 pages
        assert_eq!(cache.cached_page_count(), 2);

        // First page should have been evicted
        assert!(!cache.page_cache.contains_key(&0));
        assert!(cache.page_cache.contains_key(&1));
        assert!(cache.page_cache.contains_key(&2));
    }

    #[test]
    fn test_cache_stats_overall() {
        let mut cache = LayoutCache::new();
        let node_id = NodeId::new();
        let lines = create_test_lines(2, 20.0);

        cache.store(node_id, 1, 400.0, &lines, 40.0);

        let page = CachedPageLayout {
            page_index: 0,
            paragraphs: vec![node_id],
            first_line_index: 0,
            last_line_index: None,
            content_height: 40.0,
            version: 1,
        };
        cache.store_page(page);

        cache.reset_stats();

        // Paragraph hit
        cache.is_cached(node_id, 1, 400.0);
        // Page hit
        cache.is_page_cached(0, 1);
        // Paragraph miss
        cache.is_cached(NodeId::new(), 1, 400.0);
        // Page miss
        cache.is_page_cached(1, 1);

        let stats = cache.stats();
        assert_eq!(stats.paragraph_hits, 1);
        assert_eq!(stats.paragraph_misses, 1);
        assert_eq!(stats.page_hits, 1);
        assert_eq!(stats.page_misses, 1);
        assert_eq!(stats.overall_hit_ratio(), 0.5);
    }

    #[test]
    fn test_invalidate_single_page() {
        let mut cache = LayoutCache::new();

        for i in 0..3 {
            let page = CachedPageLayout {
                page_index: i,
                paragraphs: vec![NodeId::new()],
                first_line_index: 0,
                last_line_index: None,
                content_height: 500.0,
                version: 1,
            };
            cache.store_page(page);
        }

        cache.invalidate_page(1);

        assert_eq!(cache.cached_page_count(), 2);
        assert!(cache.page_cache.contains_key(&0));
        assert!(!cache.page_cache.contains_key(&1));
        assert!(cache.page_cache.contains_key(&2));
    }

    #[test]
    fn test_hash_paragraph_runs() {
        let runs1 = vec![
            ("Hello", 12.0, false, false),
            (" world", 12.0, true, false),
        ];
        let runs2 = vec![
            ("Hello", 12.0, false, false),
            (" world", 12.0, true, false),
        ];
        let runs3 = vec![
            ("Hello", 12.0, false, false),
            (" world", 14.0, true, false),
        ];

        let hash1 = hash_paragraph_runs(runs1.iter().map(|r| (r.0, r.1, r.2, r.3)));
        let hash2 = hash_paragraph_runs(runs2.iter().map(|r| (r.0, r.1, r.2, r.3)));
        let hash3 = hash_paragraph_runs(runs3.iter().map(|r| (r.0, r.1, r.2, r.3)));

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}

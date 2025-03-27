// Resource Query Pagination
//
// This module provides pagination capabilities for resource queries,
// allowing results to be split into pages.

use std::fmt::{Debug, Display};
use serde::{Serialize, Deserialize};

/// Pagination specification for queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pagination {
    /// Maximum number of results to return
    pub limit: Option<usize>,
    
    /// Number of results to skip
    pub offset: Option<usize>,
}

impl Pagination {
    /// Create a new pagination with the given limit and offset
    pub fn new(limit: Option<usize>, offset: Option<usize>) -> Self {
        Self { limit, offset }
    }
    
    /// Create a new pagination with just a limit
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit: Some(limit),
            offset: None,
        }
    }
    
    /// Create a new pagination with limit and offset
    pub fn with_limit_offset(limit: usize, offset: usize) -> Self {
        Self {
            limit: Some(limit),
            offset: Some(offset),
        }
    }
    
    /// Apply pagination to a collection of items
    pub fn apply<T>(&self, items: Vec<T>) -> Vec<T> {
        let offset = self.offset.unwrap_or(0);
        let limit = self.limit.unwrap_or(usize::MAX);
        
        items.into_iter()
            .skip(offset)
            .take(limit)
            .collect()
    }
    
    /// Check if this pagination has a limit
    pub fn has_limit(&self) -> bool {
        self.limit.is_some()
    }
    
    /// Check if this pagination has an offset
    pub fn has_offset(&self) -> bool {
        self.offset.is_some()
    }
    
    /// Get the effective limit (or default)
    pub fn effective_limit(&self, default: usize) -> usize {
        self.limit.unwrap_or(default)
    }
    
    /// Get the effective offset (or default)
    pub fn effective_offset(&self, default: usize) -> usize {
        self.offset.unwrap_or(default)
    }
    
    /// Create pagination for the next page
    pub fn next_page(&self) -> Self {
        let current_offset = self.offset.unwrap_or(0);
        let limit = self.limit.unwrap_or(10);
        
        Self {
            limit: self.limit,
            offset: Some(current_offset + limit),
        }
    }
    
    /// Create pagination for the previous page
    pub fn previous_page(&self) -> Option<Self> {
        let current_offset = self.offset.unwrap_or(0);
        let limit = self.limit.unwrap_or(10);
        
        if current_offset < limit {
            return None; // Already on the first page
        }
        
        Some(Self {
            limit: self.limit,
            offset: Some(current_offset - limit),
        })
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: Some(10), // Default to 10 items per page
            offset: None,
        }
    }
}

/// Advanced pagination options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationOptions {
    /// Basic pagination parameters
    pub pagination: Pagination,
    
    /// Whether to include total count in the result
    pub include_total: bool,
    
    /// Whether to use cursor-based pagination
    pub use_cursor: bool,
    
    /// Cursor value for continuing from a previous page
    pub cursor: Option<String>,
    
    /// Cursor field for cursor-based pagination
    pub cursor_field: Option<String>,
}

impl PaginationOptions {
    /// Create new pagination options
    pub fn new(pagination: Pagination) -> Self {
        Self {
            pagination,
            include_total: false,
            use_cursor: false,
            cursor: None,
            cursor_field: None,
        }
    }
    
    /// Create pagination options with cursor-based pagination
    pub fn with_cursor(
        limit: usize,
        cursor: Option<String>,
        cursor_field: impl Into<String>,
    ) -> Self {
        Self {
            pagination: Pagination::with_limit(limit),
            include_total: false,
            use_cursor: true,
            cursor,
            cursor_field: Some(cursor_field.into()),
        }
    }
    
    /// Include total count in the result
    pub fn with_total(mut self, include_total: bool) -> Self {
        self.include_total = include_total;
        self
    }
}

impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            pagination: Pagination::default(),
            include_total: false,
            use_cursor: false,
            cursor: None,
            cursor_field: None,
        }
    }
}

/// Result of a paginated query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationResult {
    /// Offset used for this page
    pub offset: usize,
    
    /// Limit used for this page
    pub limit: usize,
    
    /// Total number of items across all pages (if requested and available)
    pub total: Option<usize>,
    
    /// Number of items returned on this page
    pub count: usize,
    
    /// Whether there are more items available
    pub has_more: bool,
    
    /// Cursor for the next page (if using cursor-based pagination)
    pub next_cursor: Option<String>,
    
    /// Cursor for the previous page (if using cursor-based pagination and not on first page)
    pub previous_cursor: Option<String>,
}

impl PaginationResult {
    /// Create a new pagination result
    pub fn new(
        offset: usize,
        limit: usize,
        count: usize,
        total: Option<usize>,
    ) -> Self {
        let has_more = match total {
            Some(total) => offset + count < total,
            None => false, // Unknown if there are more
        };
        
        Self {
            offset,
            limit,
            total,
            count,
            has_more,
            next_cursor: None,
            previous_cursor: None,
        }
    }
    
    /// Create a pagination result with cursors
    pub fn with_cursors(
        offset: usize,
        limit: usize,
        count: usize,
        total: Option<usize>,
        next_cursor: Option<String>,
        previous_cursor: Option<String>,
    ) -> Self {
        let has_more = next_cursor.is_some() || match total {
            Some(total) => offset + count < total,
            None => false,
        };
        
        Self {
            offset,
            limit,
            total,
            count,
            has_more,
            next_cursor,
            previous_cursor,
        }
    }
    
    /// Get the next page number (1-based)
    pub fn next_page(&self) -> Option<usize> {
        if !self.has_more {
            return None;
        }
        
        let current_page = (self.offset / self.limit) + 1;
        Some(current_page + 1)
    }
    
    /// Get the previous page number (1-based)
    pub fn previous_page(&self) -> Option<usize> {
        if self.offset == 0 {
            return None;
        }
        
        let current_page = (self.offset / self.limit) + 1;
        Some(current_page - 1)
    }
    
    /// Get the current page number (1-based)
    pub fn current_page(&self) -> usize {
        (self.offset / self.limit) + 1
    }
    
    /// Get the total number of pages
    pub fn total_pages(&self) -> Option<usize> {
        self.total.map(|total| {
            let pages = total / self.limit;
            if total % self.limit > 0 {
                pages + 1
            } else {
                pages
            }
        })
    }
} 
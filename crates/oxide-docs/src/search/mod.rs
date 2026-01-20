//! Search index for offline documentation
//!
//! Provides full-text search capabilities using Tantivy.

mod index;
mod query;

pub use index::DocIndex;
pub use query::{SearchQuery, SearchResult};

use crate::DocsResult;
use std::path::Path;

/// Initialize a new search index at the given path
pub fn create_index(path: &Path) -> DocsResult<DocIndex> {
    DocIndex::create(path)
}

/// Load an existing search index
pub fn load_index(path: &Path) -> DocsResult<DocIndex> {
    DocIndex::load(path)
}

/// Quick search helper for simple queries
pub fn quick_search(index: &DocIndex, query: &str, limit: usize) -> DocsResult<Vec<SearchResult>> {
    index.search(query, limit)
}

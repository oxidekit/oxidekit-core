//! Search query and result types

use serde::{Deserialize, Serialize};

/// A search query with optional filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// The search text
    pub text: String,
    /// Limit results to specific categories
    pub categories: Option<Vec<String>>,
    /// Limit results to specific tags
    pub tags: Option<Vec<String>>,
    /// Maximum number of results
    pub limit: usize,
    /// Offset for pagination
    pub offset: usize,
}

impl SearchQuery {
    /// Create a new search query
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            categories: None,
            tags: None,
            limit: 20,
            offset: 0,
        }
    }

    /// Filter by categories
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.categories = Some(categories);
        self
    }

    /// Filter by tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set result offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Build the query string for Tantivy
    pub fn build_query_string(&self) -> String {
        let mut parts = vec![self.text.clone()];

        if let Some(ref tags) = self.tags {
            for tag in tags {
                parts.push(format!("tags:{}", tag));
            }
        }

        parts.join(" ")
    }
}

/// A search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Document ID
    pub id: String,
    /// Document title
    pub title: String,
    /// Content snippet with matching text
    pub snippet: String,
    /// Relevance score
    pub score: f32,
    /// Document tags
    pub tags: Vec<String>,
    /// Path to the document
    pub path: String,
}

impl SearchResult {
    /// Create a highlighted snippet from content
    pub fn create_snippet(content: &str, query: &str, context_chars: usize) -> String {
        let content_lower = content.to_lowercase();
        let query_lower = query.to_lowercase();

        // Find the query in the content
        if let Some(pos) = content_lower.find(&query_lower) {
            let start = pos.saturating_sub(context_chars);
            let end = (pos + query.len() + context_chars).min(content.len());

            let mut snippet = String::new();

            if start > 0 {
                snippet.push_str("...");
            }

            snippet.push_str(&content[start..end]);

            if end < content.len() {
                snippet.push_str("...");
            }

            snippet
        } else {
            // Return first N characters if query not found
            let end = context_chars.min(content.len());
            let mut snippet = content[..end].to_string();
            if end < content.len() {
                snippet.push_str("...");
            }
            snippet
        }
    }
}

/// Search results container with pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// The results
    pub results: Vec<SearchResult>,
    /// Total number of matching documents
    pub total: usize,
    /// Query that was executed
    pub query: String,
    /// Time taken in milliseconds
    pub took_ms: u64,
}

impl SearchResults {
    /// Create new search results
    pub fn new(results: Vec<SearchResult>, total: usize, query: String, took_ms: u64) -> Self {
        Self {
            results,
            total,
            query,
            took_ms,
        }
    }

    /// Check if there are more results
    pub fn has_more(&self, offset: usize, limit: usize) -> bool {
        offset + limit < self.total
    }

    /// Check if the results are empty
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get the number of results
    pub fn len(&self) -> usize {
        self.results.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("getting started")
            .with_tags(vec!["beginner".to_string()])
            .limit(10);

        assert_eq!(query.text, "getting started");
        assert_eq!(query.limit, 10);
        assert!(query.tags.is_some());
    }

    #[test]
    fn test_create_snippet() {
        let content = "This is a long piece of content about getting started with OxideKit development.";
        let snippet = SearchResult::create_snippet(content, "getting started", 20);

        assert!(snippet.contains("getting started"));
        assert!(snippet.starts_with("...") || snippet.len() < content.len());
    }

    #[test]
    fn test_search_results() {
        let results = SearchResults::new(
            vec![SearchResult {
                id: "test".to_string(),
                title: "Test".to_string(),
                snippet: "Test content".to_string(),
                score: 1.0,
                tags: vec![],
                path: "/test".to_string(),
            }],
            1,
            "test".to_string(),
            5,
        );

        assert_eq!(results.len(), 1);
        assert!(!results.is_empty());
        assert!(!results.has_more(0, 10));
    }
}

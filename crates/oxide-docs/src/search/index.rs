//! Search index implementation using Tantivy

use crate::{DocsError, DocsResult};
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, STORED, TEXT, IndexRecordOption, Value};
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy, TantivyDocument};
use tracing::{debug, info};

use super::SearchResult;

/// Full-text search index for documentation
pub struct DocIndex {
    index: Index,
    reader: Option<IndexReader>,
    writer: Option<IndexWriter>,
    schema: Schema,
    // Field handles
    id_field: Field,
    title_field: Field,
    content_field: Field,
    tags_field: Field,
}

impl DocIndex {
    /// Create a new index at the specified path
    pub fn create(path: &Path) -> DocsResult<Self> {
        std::fs::create_dir_all(path)?;

        let schema = Self::build_schema();
        let index = Index::create_in_dir(path, schema.clone())?;

        let id_field = schema.get_field("id").unwrap();
        let title_field = schema.get_field("title").unwrap();
        let content_field = schema.get_field("content").unwrap();
        let tags_field = schema.get_field("tags").unwrap();

        let writer = index.writer(50_000_000)?; // 50MB buffer

        info!("Created new search index at {:?}", path);

        Ok(Self {
            index,
            reader: None,
            writer: Some(writer),
            schema,
            id_field,
            title_field,
            content_field,
            tags_field,
        })
    }

    /// Load an existing index from disk
    pub fn load(path: &Path) -> DocsResult<Self> {
        if !path.exists() {
            return Err(DocsError::SearchIndex(format!(
                "Index not found at {:?}",
                path
            )));
        }

        let index = Index::open_in_dir(path)?;
        let schema = index.schema();

        let id_field = schema.get_field("id").unwrap();
        let title_field = schema.get_field("title").unwrap();
        let content_field = schema.get_field("content").unwrap();
        let tags_field = schema.get_field("tags").unwrap();

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        info!("Loaded search index from {:?}", path);

        Ok(Self {
            index,
            reader: Some(reader),
            writer: None,
            schema,
            id_field,
            title_field,
            content_field,
            tags_field,
        })
    }

    /// Build the schema for the search index
    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();

        // Document ID (stored, not indexed for search)
        schema_builder.add_text_field("id", STORED);

        // Title (searchable and stored)
        schema_builder.add_text_field("title", TEXT | STORED);

        // Content (searchable but not stored - too large)
        let content_options = tantivy::schema::TextOptions::default()
            .set_indexing_options(
                tantivy::schema::TextFieldIndexing::default()
                    .set_tokenizer("default")
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            );
        schema_builder.add_text_field("content", content_options);

        // Tags (searchable and stored)
        schema_builder.add_text_field("tags", TEXT | STORED);

        schema_builder.build()
    }

    /// Add a document to the index
    pub fn add_document(
        &mut self,
        id: &str,
        title: &str,
        content: &str,
        tags: &[String],
    ) -> DocsResult<()> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| DocsError::SearchIndex("Index opened in read-only mode".to_string()))?;

        let tags_str = tags.join(" ");

        writer.add_document(doc!(
            self.id_field => id,
            self.title_field => title,
            self.content_field => content,
            self.tags_field => tags_str,
        ))?;

        debug!("Added document to index: {}", id);
        Ok(())
    }

    /// Commit changes to the index
    pub fn commit(&mut self) -> DocsResult<()> {
        if let Some(ref mut writer) = self.writer {
            writer.commit()?;

            // Create reader for searching
            self.reader = Some(
                self.index
                    .reader_builder()
                    .reload_policy(ReloadPolicy::OnCommitWithDelay)
                    .try_into()?,
            );

            info!("Committed search index changes");
        }
        Ok(())
    }

    /// Search the index
    pub fn search(&self, query_str: &str, limit: usize) -> DocsResult<Vec<SearchResult>> {
        let reader = self
            .reader
            .as_ref()
            .ok_or_else(|| DocsError::SearchIndex("Index not ready for search".to_string()))?;

        let searcher = reader.searcher();

        // Create query parser for title and content fields
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.title_field, self.content_field, self.tags_field],
        );

        // Parse the query
        let query = query_parser.parse_query(query_str)?;

        // Execute search
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        // Convert results
        let mut results = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = searcher.doc(doc_address)?;

            let id = doc
                .get_first(self.id_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let title = doc
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tags = doc
                .get_first(self.tags_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(SearchResult {
                id,
                title,
                snippet: String::new(), // Will be generated from content
                score,
                tags: tags.split_whitespace().map(String::from).collect(),
                path: String::new(), // Will be filled from manifest
            });
        }

        debug!("Search for '{}' returned {} results", query_str, results.len());
        Ok(results)
    }

    /// Get the total number of documents in the index
    pub fn doc_count(&self) -> DocsResult<u64> {
        let reader = self
            .reader
            .as_ref()
            .ok_or_else(|| DocsError::SearchIndex("Index not ready".to_string()))?;

        let searcher = reader.searcher();
        Ok(searcher.num_docs())
    }

    /// Clear all documents from the index
    pub fn clear(&mut self) -> DocsResult<()> {
        if let Some(ref mut writer) = self.writer {
            writer.delete_all_documents()?;
            writer.commit()?;
            info!("Cleared search index");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_and_search_index() {
        let dir = TempDir::new().unwrap();
        let mut index = DocIndex::create(dir.path()).unwrap();

        // Add documents
        index
            .add_document(
                "getting-started",
                "Getting Started with OxideKit",
                "This guide will help you get started with OxideKit application development.",
                &["beginner".to_string(), "tutorial".to_string()],
            )
            .unwrap();

        index
            .add_document(
                "components",
                "Component Reference",
                "OxideKit provides a rich set of UI components for building applications.",
                &["api".to_string(), "reference".to_string()],
            )
            .unwrap();

        index.commit().unwrap();

        // Search
        let results = index.search("getting started", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "getting-started");

        let results = index.search("components", 10).unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_doc_count() {
        let dir = TempDir::new().unwrap();
        let mut index = DocIndex::create(dir.path()).unwrap();

        index.add_document("doc1", "Title 1", "Content 1", &[]).unwrap();
        index.add_document("doc2", "Title 2", "Content 2", &[]).unwrap();
        index.commit().unwrap();

        assert_eq!(index.doc_count().unwrap(), 2);
    }
}

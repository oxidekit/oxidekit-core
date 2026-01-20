//! Documentation bundle builder

use crate::manifest::{DocsManifest, PageEntry};
use crate::search::DocIndex;
use crate::types::{ContentFormat, DocCategory, DocPage, NavItem, VersionInfo};
use crate::{DocsConfig, DocsResult};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use super::renderer::MarkdownRenderer;
use super::DocBundle;

/// Builder for creating documentation bundles
pub struct DocBundler {
    config: DocsConfig,
    renderer: MarkdownRenderer,
    pages: Vec<DocPage>,
    nav_items: Vec<NavItem>,
}

impl DocBundler {
    /// Create a new bundler with configuration
    pub fn new(config: DocsConfig) -> Self {
        Self {
            config,
            renderer: MarkdownRenderer::new(),
            pages: Vec::new(),
            nav_items: Vec::new(),
        }
    }

    /// Build the documentation bundle
    pub fn build(mut self) -> DocsResult<DocBundle> {
        info!("Building documentation bundle...");

        // Create output directory
        fs::create_dir_all(&self.config.output_dir)?;

        // Collect and process documentation pages
        self.collect_pages()?;

        // Build navigation structure
        self.build_navigation();

        // Render all pages to HTML
        self.render_pages()?;

        // Build search index
        let index_path = self.build_search_index()?;

        // Copy static assets
        self.copy_assets()?;

        // Create manifest
        let manifest = self.create_manifest(index_path)?;

        // Save manifest
        let manifest_path = self.config.output_dir.join("manifest.json");
        manifest.save(&manifest_path)?;

        info!(
            "Documentation bundle built: {} pages, {} examples",
            manifest.page_count(),
            manifest.examples.len()
        );

        DocBundle::load(&self.config.output_dir)
    }

    /// Collect documentation pages from source directory
    fn collect_pages(&mut self) -> DocsResult<()> {
        let source = &self.config.source_dir;

        if !source.exists() {
            warn!("Source directory does not exist: {:?}", source);
            return Ok(());
        }

        for entry in WalkDir::new(source)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Process markdown files
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(page) = self.process_markdown_file(path)? {
                    self.pages.push(page);
                }
            }
        }

        debug!("Collected {} pages", self.pages.len());
        Ok(())
    }

    /// Process a markdown file into a DocPage
    fn process_markdown_file(&self, path: &Path) -> DocsResult<Option<DocPage>> {
        let content = fs::read_to_string(path)?;
        let rel_path = path.strip_prefix(&self.config.source_dir)
            .unwrap_or(path);

        // Parse frontmatter and content
        let (frontmatter, markdown) = parse_frontmatter(&content);

        // Determine page properties from frontmatter or path
        let title = frontmatter
            .get("title")
            .map(|s| s.to_string())
            .unwrap_or_else(|| extract_title_from_path(rel_path));

        let category = frontmatter
            .get("category")
            .and_then(|s| parse_category(s))
            .unwrap_or_else(|| infer_category_from_path(rel_path));

        let tags: Vec<String> = frontmatter
            .get("tags")
            .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
            .unwrap_or_default();

        let id = generate_page_id(rel_path);

        let page = DocPage {
            id,
            title,
            path: rel_path.to_path_buf(),
            content: markdown,
            format: ContentFormat::Markdown,
            category,
            tags,
            parent: None,
            children: Vec::new(),
            modified: Utc::now(),
            metadata: frontmatter,
        };

        Ok(Some(page))
    }

    /// Build navigation structure from pages
    fn build_navigation(&mut self) {
        // Group pages by category
        let mut by_category: HashMap<DocCategory, Vec<&DocPage>> = HashMap::new();

        for page in &self.pages {
            by_category
                .entry(page.category)
                .or_default()
                .push(page);
        }

        // Build nav items for each category
        let category_order = [
            DocCategory::GettingStarted,
            DocCategory::Concepts,
            DocCategory::Tutorials,
            DocCategory::Guides,
            DocCategory::Components,
            DocCategory::ApiReference,
            DocCategory::Cli,
            DocCategory::Examples,
            DocCategory::Architecture,
            DocCategory::Contributing,
            DocCategory::Faq,
            DocCategory::Changelog,
        ];

        for category in category_order {
            if let Some(pages) = by_category.get(&category) {
                let children: Vec<NavItem> = pages
                    .iter()
                    .map(|page| NavItem {
                        title: page.title.clone(),
                        path: format!("/{}/{}.html", category.slug(), page.id),
                        children: Vec::new(),
                        expanded: false,
                        icon: None,
                    })
                    .collect();

                if !children.is_empty() {
                    self.nav_items.push(NavItem {
                        title: category.display_name().to_string(),
                        path: format!("/{}/", category.slug()),
                        children,
                        expanded: category == DocCategory::GettingStarted,
                        icon: Some(category_icon(category).to_string()),
                    });
                }
            }
        }
    }

    /// Render pages to HTML
    fn render_pages(&self) -> DocsResult<()> {
        for page in &self.pages {
            let output_dir = self.config.output_dir.join(page.category.slug());
            fs::create_dir_all(&output_dir)?;

            let html = self.renderer.render_page(page, &self.config)?;
            let output_path = output_dir.join(format!("{}.html", page.id));

            let file = fs::File::create(&output_path)?;
            let mut writer = BufWriter::new(file);
            writer.write_all(html.as_bytes())?;

            debug!("Rendered: {:?}", output_path);
        }

        // Generate index page
        self.generate_index_page()?;

        Ok(())
    }

    /// Generate the main index page
    fn generate_index_page(&self) -> DocsResult<()> {
        let html = self.renderer.render_index(&self.nav_items, &self.config)?;
        let output_path = self.config.output_dir.join("index.html");

        fs::write(&output_path, html)?;
        debug!("Generated index page");

        Ok(())
    }

    /// Build the search index
    fn build_search_index(&self) -> DocsResult<PathBuf> {
        let index_dir = self.config.output_dir.join("search_index");
        fs::create_dir_all(&index_dir)?;

        let mut index = DocIndex::create(&index_dir)?;

        for page in &self.pages {
            index.add_document(&page.id, &page.title, &page.content, &page.tags)?;
        }

        index.commit()?;
        debug!("Built search index with {} documents", self.pages.len());

        Ok(PathBuf::from("search_index"))
    }

    /// Copy static assets (CSS, JS, images)
    fn copy_assets(&self) -> DocsResult<()> {
        let assets_dir = self.config.output_dir.join("assets");
        fs::create_dir_all(&assets_dir)?;

        // Write built-in CSS
        fs::write(assets_dir.join("style.css"), BUILT_IN_CSS)?;

        // Write built-in JS
        fs::write(assets_dir.join("script.js"), BUILT_IN_JS)?;

        // Copy custom styles if provided
        if let Some(custom_css) = &self.config.custom_styles {
            fs::write(assets_dir.join("custom.css"), custom_css)?;
        }

        // Copy source assets directory if it exists
        let source_assets = self.config.source_dir.join("assets");
        if source_assets.exists() {
            copy_dir_recursive(&source_assets, &assets_dir)?;
        }

        Ok(())
    }

    /// Create the bundle manifest
    fn create_manifest(&self, search_index_path: PathBuf) -> DocsResult<DocsManifest> {
        let version = VersionInfo {
            version: self.config.version.clone(),
            is_latest: true,
            release_date: Utc::now(),
            min_core_version: self.config.version.clone(),
        };

        let mut manifest = DocsManifest::new(version);
        manifest.navigation = self.nav_items.clone();
        manifest.search_index_path = Some(search_index_path);

        for page in &self.pages {
            manifest.add_page(
                page.id.clone(),
                PageEntry {
                    title: page.title.clone(),
                    path: PathBuf::from(format!("{}/{}.html", page.category.slug(), page.id)),
                    category: page.category,
                    indexed: true,
                },
            );
        }

        // Calculate bundle size
        manifest.bundle_size = calculate_dir_size(&self.config.output_dir)?;

        // Calculate checksum
        manifest.checksum = calculate_checksum(&self.config.output_dir)?;

        Ok(manifest)
    }
}

// Helper functions

fn parse_frontmatter(content: &str) -> (HashMap<String, String>, String) {
    let mut metadata = HashMap::new();
    let mut markdown = content.to_string();

    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let frontmatter_str = &content[3..3 + end];
            markdown = content[3 + end + 3..].trim().to_string();

            for line in frontmatter_str.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    metadata.insert(
                        key.trim().to_string(),
                        value.trim().trim_matches('"').to_string(),
                    );
                }
            }
        }
    }

    (metadata, markdown)
}

fn extract_title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| {
            s.replace('-', " ")
                .replace('_', " ")
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_else(|| "Untitled".to_string())
}

fn parse_category(s: &str) -> Option<DocCategory> {
    match s.to_lowercase().as_str() {
        "getting-started" | "getting_started" | "gettingstarted" => Some(DocCategory::GettingStarted),
        "concepts" | "core-concepts" => Some(DocCategory::Concepts),
        "api" | "api-reference" | "reference" => Some(DocCategory::ApiReference),
        "components" => Some(DocCategory::Components),
        "tutorials" | "tutorial" => Some(DocCategory::Tutorials),
        "guides" | "guide" | "how-to" => Some(DocCategory::Guides),
        "examples" | "example" => Some(DocCategory::Examples),
        "cli" | "cli-reference" => Some(DocCategory::Cli),
        "architecture" | "arch" => Some(DocCategory::Architecture),
        "contributing" | "contribute" => Some(DocCategory::Contributing),
        "faq" => Some(DocCategory::Faq),
        "changelog" | "changes" | "releases" => Some(DocCategory::Changelog),
        _ => None,
    }
}

fn infer_category_from_path(path: &Path) -> DocCategory {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("getting-started") || path_str.contains("quickstart") {
        DocCategory::GettingStarted
    } else if path_str.contains("concept") {
        DocCategory::Concepts
    } else if path_str.contains("api") || path_str.contains("reference") {
        DocCategory::ApiReference
    } else if path_str.contains("component") {
        DocCategory::Components
    } else if path_str.contains("tutorial") {
        DocCategory::Tutorials
    } else if path_str.contains("guide") || path_str.contains("how-to") {
        DocCategory::Guides
    } else if path_str.contains("example") {
        DocCategory::Examples
    } else if path_str.contains("cli") {
        DocCategory::Cli
    } else if path_str.contains("arch") {
        DocCategory::Architecture
    } else if path_str.contains("contributing") {
        DocCategory::Contributing
    } else if path_str.contains("faq") {
        DocCategory::Faq
    } else if path_str.contains("changelog") {
        DocCategory::Changelog
    } else {
        DocCategory::Other
    }
}

fn generate_page_id(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase().replace(' ', "-"))
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
}

fn category_icon(category: DocCategory) -> &'static str {
    match category {
        DocCategory::GettingStarted => "rocket",
        DocCategory::Concepts => "lightbulb",
        DocCategory::ApiReference => "book",
        DocCategory::Components => "puzzle",
        DocCategory::Tutorials => "graduation-cap",
        DocCategory::Guides => "compass",
        DocCategory::Examples => "code",
        DocCategory::Cli => "terminal",
        DocCategory::Architecture => "sitemap",
        DocCategory::Contributing => "users",
        DocCategory::Faq => "question-circle",
        DocCategory::Changelog => "history",
        DocCategory::Other => "file",
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> DocsResult<()> {
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(src).unwrap();
        let dst_path = dst.join(rel_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&dst_path)?;
        } else {
            fs::copy(src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn calculate_dir_size(dir: &Path) -> DocsResult<u64> {
    let mut total = 0;
    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            total += entry.metadata()?.len();
        }
    }
    Ok(total)
}

fn calculate_checksum(dir: &Path) -> DocsResult<String> {
    // Simple checksum based on file count and total size
    let mut count = 0u64;
    let mut size = 0u64;

    for entry in WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            count += 1;
            size += entry.metadata()?.len();
        }
    }

    Ok(format!("{:x}-{:x}", count, size))
}

// Built-in CSS for documentation
const BUILT_IN_CSS: &str = r#"
:root {
    --bg-primary: #0f1419;
    --bg-secondary: #1a1f26;
    --bg-tertiary: #242a33;
    --text-primary: #e6edf3;
    --text-secondary: #8b949e;
    --accent: #2f81f7;
    --accent-hover: #58a6ff;
    --border: #30363d;
    --code-bg: #161b22;
    --success: #3fb950;
    --warning: #d29922;
    --error: #f85149;
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background: var(--bg-primary);
    color: var(--text-primary);
    line-height: 1.6;
}

.container {
    display: flex;
    min-height: 100vh;
}

.sidebar {
    width: 280px;
    background: var(--bg-secondary);
    border-right: 1px solid var(--border);
    padding: 1.5rem;
    position: fixed;
    height: 100vh;
    overflow-y: auto;
}

.main-content {
    flex: 1;
    margin-left: 280px;
    padding: 2rem 3rem;
    max-width: 900px;
}

.logo {
    font-size: 1.5rem;
    font-weight: 700;
    color: var(--accent);
    margin-bottom: 2rem;
    text-decoration: none;
    display: block;
}

.nav-section {
    margin-bottom: 1.5rem;
}

.nav-section-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.5rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
}

.nav-links {
    list-style: none;
}

.nav-links a {
    display: block;
    padding: 0.375rem 0.75rem;
    color: var(--text-primary);
    text-decoration: none;
    border-radius: 4px;
    font-size: 0.9rem;
}

.nav-links a:hover {
    background: var(--bg-tertiary);
    color: var(--accent-hover);
}

.nav-links a.active {
    background: var(--accent);
    color: white;
}

h1, h2, h3, h4, h5, h6 {
    margin-top: 1.5rem;
    margin-bottom: 0.75rem;
    font-weight: 600;
}

h1 { font-size: 2rem; margin-top: 0; }
h2 { font-size: 1.5rem; border-bottom: 1px solid var(--border); padding-bottom: 0.5rem; }
h3 { font-size: 1.25rem; }

p {
    margin-bottom: 1rem;
}

a {
    color: var(--accent);
}

a:hover {
    color: var(--accent-hover);
}

code {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    background: var(--code-bg);
    padding: 0.125rem 0.375rem;
    border-radius: 4px;
    font-size: 0.875em;
}

pre {
    background: var(--code-bg);
    padding: 1rem;
    border-radius: 8px;
    overflow-x: auto;
    margin: 1rem 0;
    border: 1px solid var(--border);
}

pre code {
    background: none;
    padding: 0;
}

.search-box {
    width: 100%;
    padding: 0.5rem 0.75rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 0.9rem;
    margin-bottom: 1.5rem;
}

.search-box:focus {
    outline: none;
    border-color: var(--accent);
}

.breadcrumb {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 1rem;
}

.breadcrumb a {
    color: var(--text-secondary);
}

.breadcrumb a:hover {
    color: var(--accent);
}

.callout {
    padding: 1rem;
    border-radius: 8px;
    margin: 1rem 0;
    border-left: 4px solid;
}

.callout-info {
    background: rgba(47, 129, 247, 0.1);
    border-color: var(--accent);
}

.callout-warning {
    background: rgba(210, 153, 34, 0.1);
    border-color: var(--warning);
}

.callout-error {
    background: rgba(248, 81, 73, 0.1);
    border-color: var(--error);
}

.callout-success {
    background: rgba(63, 185, 80, 0.1);
    border-color: var(--success);
}

table {
    width: 100%;
    border-collapse: collapse;
    margin: 1rem 0;
}

th, td {
    padding: 0.75rem;
    text-align: left;
    border: 1px solid var(--border);
}

th {
    background: var(--bg-secondary);
    font-weight: 600;
}

tr:nth-child(even) {
    background: var(--bg-secondary);
}

.version-badge {
    display: inline-block;
    padding: 0.125rem 0.5rem;
    background: var(--bg-tertiary);
    border-radius: 9999px;
    font-size: 0.75rem;
    color: var(--text-secondary);
}

@media (max-width: 768px) {
    .sidebar {
        width: 100%;
        position: relative;
        height: auto;
    }

    .main-content {
        margin-left: 0;
        padding: 1rem;
    }

    .container {
        flex-direction: column;
    }
}
"#;

// Built-in JavaScript for documentation
const BUILT_IN_JS: &str = r#"
// OxideKit Documentation Scripts

document.addEventListener('DOMContentLoaded', function() {
    // Search functionality
    const searchBox = document.querySelector('.search-box');
    if (searchBox) {
        searchBox.addEventListener('input', debounce(handleSearch, 300));
    }

    // Code copy buttons
    document.querySelectorAll('pre code').forEach(block => {
        const button = document.createElement('button');
        button.className = 'copy-button';
        button.textContent = 'Copy';
        button.onclick = () => copyCode(block, button);
        block.parentNode.insertBefore(button, block);
    });

    // Active nav highlighting
    highlightActiveNav();
});

function debounce(fn, delay) {
    let timeout;
    return function(...args) {
        clearTimeout(timeout);
        timeout = setTimeout(() => fn.apply(this, args), delay);
    };
}

async function handleSearch(e) {
    const query = e.target.value.trim();
    if (query.length < 2) {
        hideSearchResults();
        return;
    }

    try {
        const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`);
        const results = await response.json();
        displaySearchResults(results);
    } catch (err) {
        console.error('Search error:', err);
    }
}

function displaySearchResults(results) {
    let container = document.getElementById('search-results');
    if (!container) {
        container = document.createElement('div');
        container.id = 'search-results';
        document.querySelector('.search-box').after(container);
    }

    if (results.length === 0) {
        container.innerHTML = '<p class="no-results">No results found</p>';
        return;
    }

    container.innerHTML = results.map(r => `
        <a href="${r.path}" class="search-result">
            <strong>${escapeHtml(r.title)}</strong>
            <p>${escapeHtml(r.snippet)}</p>
        </a>
    `).join('');
}

function hideSearchResults() {
    const container = document.getElementById('search-results');
    if (container) {
        container.innerHTML = '';
    }
}

function copyCode(block, button) {
    navigator.clipboard.writeText(block.textContent).then(() => {
        button.textContent = 'Copied!';
        setTimeout(() => button.textContent = 'Copy', 2000);
    });
}

function highlightActiveNav() {
    const currentPath = window.location.pathname;
    document.querySelectorAll('.nav-links a').forEach(link => {
        if (link.getAttribute('href') === currentPath) {
            link.classList.add('active');
        }
    });
}

function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}
"#;

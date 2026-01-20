//! Markdown to HTML rendering with syntax highlighting

use crate::types::{DocPage, NavItem, TableOfContents, TocEntry};
use crate::{DocsConfig, DocsError, DocsResult};
use minijinja::{context, Environment};
use pulldown_cmark::{html, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

/// Markdown renderer with syntax highlighting support
pub struct MarkdownRenderer {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    templates: Environment<'static>,
}

impl MarkdownRenderer {
    /// Create a new markdown renderer
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        let mut templates = Environment::new();
        templates.add_template("page", PAGE_TEMPLATE).unwrap();
        templates.add_template("index", INDEX_TEMPLATE).unwrap();

        Self {
            syntax_set,
            theme_set,
            templates,
        }
    }

    /// Render a documentation page to HTML
    pub fn render_page(&self, page: &DocPage, config: &DocsConfig) -> DocsResult<String> {
        // Parse markdown and extract headings for TOC
        let (html_content, toc) = self.render_markdown(&page.content)?;

        // Render with template
        let template = self.templates.get_template("page")?;
        let html = template.render(context! {
            title => &page.title,
            content => &html_content,
            toc => &toc.entries,
            category => page.category.display_name(),
            tags => &page.tags,
            version => &config.version,
            base_url => &config.base_url,
        })?;

        Ok(html)
    }

    /// Render the main index page
    pub fn render_index(&self, nav_items: &[NavItem], config: &DocsConfig) -> DocsResult<String> {
        let template = self.templates.get_template("index")?;
        let html = template.render(context! {
            title => "OxideKit Documentation",
            nav_items => nav_items,
            version => &config.version,
            base_url => &config.base_url,
        })?;

        Ok(html)
    }

    /// Render markdown to HTML with syntax highlighting
    fn render_markdown(&self, markdown: &str) -> DocsResult<(String, TableOfContents)> {
        let options = Options::all();
        let parser = Parser::new_ext(markdown, options);

        let mut toc_entries: Vec<TocEntry> = Vec::new();
        let mut current_heading: Option<(HeadingLevel, String)> = None;
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_content = String::new();

        let mut events: Vec<Event> = Vec::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    current_heading = Some((level, String::new()));
                    events.push(event);
                }
                Event::End(TagEnd::Heading(level)) => {
                    if let Some((_, ref text)) = current_heading {
                        let anchor = slugify(text);
                        let toc_level = match level {
                            HeadingLevel::H1 => 1,
                            HeadingLevel::H2 => 2,
                            HeadingLevel::H3 => 3,
                            HeadingLevel::H4 => 4,
                            HeadingLevel::H5 => 5,
                            HeadingLevel::H6 => 6,
                        };

                        toc_entries.push(TocEntry {
                            title: text.clone(),
                            anchor: anchor.clone(),
                            level: toc_level,
                            children: Vec::new(),
                        });

                        // Add anchor link
                        events.push(Event::Html(format!(r##"<a id="{}" class="anchor" href="#{}"></a>"##, anchor, anchor).into()));
                    }
                    current_heading = None;
                    events.push(Event::End(TagEnd::Heading(level)));
                }
                Event::Text(text) => {
                    if let Some((_, ref mut heading_text)) = current_heading {
                        heading_text.push_str(&text);
                    }
                    if in_code_block {
                        code_content.push_str(&text);
                    } else {
                        events.push(Event::Text(text));
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => "text".to_string(),
                    };
                    code_content.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;

                    // Apply syntax highlighting
                    let highlighted = self.highlight_code(&code_content, &code_lang)?;
                    events.push(Event::Html(highlighted.into()));
                }
                _ => {
                    if !in_code_block {
                        events.push(event);
                    }
                }
            }
        }

        // Render to HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, events.into_iter());

        // Process callouts
        let html_output = process_callouts(&html_output);

        // Build hierarchical TOC
        let toc = build_toc_hierarchy(toc_entries);

        Ok((html_output, toc))
    }

    /// Apply syntax highlighting to a code block
    fn highlight_code(&self, code: &str, lang: &str) -> DocsResult<String> {
        let syntax = self
            .syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];

        match highlighted_html_for_string(code, &self.syntax_set, syntax, theme) {
            Ok(html) => Ok(format!(
                r#"<pre class="highlight"><code class="language-{}">{}</code></pre>"#,
                lang, html
            )),
            Err(e) => Err(DocsError::Highlight(e.to_string())),
        }
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn build_toc_hierarchy(entries: Vec<TocEntry>) -> TableOfContents {
    // Simple flattened TOC for now - can be enhanced for nesting
    TableOfContents { entries }
}

fn process_callouts(html: &str) -> String {
    let mut result = html.to_string();

    // Convert blockquotes with special markers to callouts
    let patterns = [
        (r#"<blockquote>\s*<p>ℹ️"#, r#"<div class="callout callout-info"><p>"#),
        (r#"<blockquote>\s*<p>⚠️"#, r#"<div class="callout callout-warning"><p>"#),
        (r#"<blockquote>\s*<p>❌"#, r#"<div class="callout callout-error"><p>"#),
        (r#"<blockquote>\s*<p>✅"#, r#"<div class="callout callout-success"><p>"#),
        (r#"<blockquote>\s*<p>\[!NOTE\]"#, r#"<div class="callout callout-info"><p>"#),
        (r#"<blockquote>\s*<p>\[!WARNING\]"#, r#"<div class="callout callout-warning"><p>"#),
        (r#"<blockquote>\s*<p>\[!IMPORTANT\]"#, r#"<div class="callout callout-error"><p>"#),
        (r#"<blockquote>\s*<p>\[!TIP\]"#, r#"<div class="callout callout-success"><p>"#),
    ];

    for (pattern, replacement) in patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }

    // Close callout divs
    result = result.replace("</blockquote>", "</div>");

    result
}

// HTML Templates

const PAGE_TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }} - OxideKit Docs</title>
    <link rel="stylesheet" href="{{ base_url }}assets/style.css">
</head>
<body>
    <div class="container">
        <aside class="sidebar">
            <a href="{{ base_url }}" class="logo">OxideKit</a>
            <span class="version-badge">v{{ version }}</span>
            <input type="text" class="search-box" placeholder="Search docs...">
            <nav id="nav"></nav>
        </aside>
        <main class="main-content">
            <div class="breadcrumb">
                <a href="{{ base_url }}">Docs</a> / {{ category }}
            </div>
            <article>
                <h1>{{ title }}</h1>
                {% if tags %}
                <div class="tags">
                    {% for tag in tags %}
                    <span class="tag">{{ tag }}</span>
                    {% endfor %}
                </div>
                {% endif %}
                <div class="content">
                    {{ content | safe }}
                </div>
            </article>
            {% if toc %}
            <aside class="toc">
                <h4>On this page</h4>
                <ul>
                {% for entry in toc %}
                    <li><a href="#{{ entry.anchor }}">{{ entry.title }}</a></li>
                {% endfor %}
                </ul>
            </aside>
            {% endif %}
        </main>
    </div>
    <script src="{{ base_url }}assets/script.js"></script>
</body>
</html>
"##;

const INDEX_TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>OxideKit Documentation</title>
    <link rel="stylesheet" href="{{ base_url }}assets/style.css">
    <style>
        .hero {
            text-align: center;
            padding: 4rem 2rem;
            background: linear-gradient(135deg, var(--bg-secondary), var(--bg-tertiary));
            border-radius: 12px;
            margin-bottom: 3rem;
        }
        .hero h1 {
            font-size: 3rem;
            margin-bottom: 1rem;
        }
        .hero p {
            font-size: 1.25rem;
            color: var(--text-secondary);
            max-width: 600px;
            margin: 0 auto;
        }
        .categories {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 1.5rem;
        }
        .category-card {
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 1.5rem;
            transition: border-color 0.2s;
        }
        .category-card:hover {
            border-color: var(--accent);
        }
        .category-card h3 {
            margin-top: 0;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }
        .category-card ul {
            list-style: none;
            margin-top: 1rem;
        }
        .category-card li {
            padding: 0.25rem 0;
        }
        .category-card a {
            color: var(--text-primary);
            text-decoration: none;
        }
        .category-card a:hover {
            color: var(--accent);
        }
    </style>
</head>
<body>
    <div class="container">
        <aside class="sidebar">
            <a href="{{ base_url }}" class="logo">OxideKit</a>
            <span class="version-badge">v{{ version }}</span>
            <input type="text" class="search-box" placeholder="Search docs...">
            <nav>
                {% for item in nav_items %}
                <div class="nav-section">
                    <div class="nav-section-title">{{ item.title }}</div>
                    <ul class="nav-links">
                        {% for child in item.children %}
                        <li><a href="{{ child.path }}">{{ child.title }}</a></li>
                        {% endfor %}
                    </ul>
                </div>
                {% endfor %}
            </nav>
        </aside>
        <main class="main-content">
            <div class="hero">
                <h1>OxideKit Documentation</h1>
                <p>A Rust-native application platform to replace Electron/Tauri. Build fast, lightweight desktop apps with the power of Rust.</p>
            </div>
            <div class="categories">
                {% for item in nav_items %}
                <div class="category-card">
                    <h3>{{ item.title }}</h3>
                    <ul>
                        {% for child in item.children %}
                        <li><a href="{{ child.path }}">{{ child.title }}</a></li>
                        {% endfor %}
                    </ul>
                </div>
                {% endfor %}
            </div>
        </main>
    </div>
    <script src="{{ base_url }}assets/script.js"></script>
</body>
</html>
"##;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Getting Started!"), "getting-started");
        assert_eq!(slugify("API Reference (v2)"), "api-reference-v2");
    }

    #[test]
    fn test_render_basic_markdown() {
        let renderer = MarkdownRenderer::new();
        let (html, _toc) = renderer.render_markdown("# Hello\n\nThis is a test.").unwrap();

        assert!(html.contains("Hello"));
        assert!(html.contains("This is a test"));
    }
}

# Building Static HTML

OxideKit can compile your application to static HTML, CSS, and JavaScript for deployment as a website. This is perfect for documentation sites, marketing pages, and web applications.

## Quick Start

```bash
# Build static site
oxide build --target static

# Build optimized for production
oxide build --target static --release

# Preview locally
oxide serve
```

## Output Structure

```
build/static/
  index.html          # Main HTML file
  app.js              # Application JavaScript
  app.css             # Compiled styles
  assets/             # Images, fonts, etc.
    images/
    fonts/
```

## Configuration

Configure static builds in `oxide.toml`:

```toml
[build]
target = ["static"]

[build.static]
out_dir = "dist"              # Output directory
base_path = "/"               # Base URL path (e.g., "/docs" for subdirectory)
minify = true                 # Minify output files
source_maps = false           # Generate source maps

# Asset handling
[build.static.assets]
inline_small = true           # Inline small assets as data URLs
inline_threshold = 4096       # Threshold in bytes

# HTML options
[build.static.html]
lang = "en"
title = "My App"
favicon = "assets/favicon.ico"

# Additional head elements
head = """
<meta name="theme-color" content="#3B82F6">
<link rel="preconnect" href="https://fonts.googleapis.com">
"""
```

## SEO Configuration

```toml
[build.static.seo]
title = "My App - Build Amazing Products"
description = "A comprehensive platform for building modern applications"
keywords = ["app", "platform", "modern"]

# Open Graph
open_graph = true
og_image = "assets/og-image.png"
og_type = "website"

# Twitter Card
twitter_card = "summary_large_image"
twitter_site = "@myapp"

# Robots
robots = "index, follow"

# Sitemap
generate_sitemap = true
sitemap_priority = 0.8
```

## Multi-Page Sites

Create multiple pages for your static site:

```
ui/
  app.oui           # Main app with router
  pages/
    index.oui       # Home page
    about.oui       # About page
    docs/
      index.oui     # Docs landing
      guide.oui     # Guide page
```

```oui
// ui/app.oui
app MySite {
    Router {
        Route { path: "/" page: "./pages/index.oui" }
        Route { path: "/about" page: "./pages/about.oui" }
        Route { path: "/docs" page: "./pages/docs/index.oui" }
        Route { path: "/docs/guide" page: "./pages/docs/guide.oui" }
    }
}
```

Output:

```
dist/
  index.html
  about/
    index.html
  docs/
    index.html
    guide/
      index.html
```

## Static Site Generation

For dynamic content, use static site generation (SSG):

```toml
[build.static]
mode = "ssg"          # Static site generation

# Data sources
[[build.static.data]]
name = "posts"
source = "content/posts/*.md"
```

```oui
// Generate pages from data
@for post in data.posts {
    Page {
        path: "/blog/{post.slug}"

        Column {
            Text { content: post.title role: "display" }
            Markdown { content: post.content }
        }
    }
}
```

## Optimizations

### Code Splitting

```toml
[build.static]
code_splitting = true
chunk_size = 50000    # Max chunk size in bytes
```

### Tree Shaking

Unused code is automatically removed:

```toml
[build.static]
tree_shaking = true
```

### Asset Optimization

```toml
[build.static.assets]
# Image optimization
optimize_images = true
image_quality = 85
webp = true           # Generate WebP versions

# Font optimization
subset_fonts = true   # Only include used characters
```

### Compression

```toml
[build.static]
# Pre-compress for servers that support it
gzip = true
brotli = true
```

## Deployment

### Netlify

```toml
# netlify.toml
[build]
command = "oxide build --target static --release"
publish = "dist"

[[redirects]]
from = "/*"
to = "/index.html"
status = 200
```

### Vercel

```json
// vercel.json
{
  "buildCommand": "oxide build --target static --release",
  "outputDirectory": "dist",
  "rewrites": [
    { "source": "/(.*)", "destination": "/index.html" }
  ]
}
```

### GitHub Pages

```yaml
# .github/workflows/deploy.yml
name: Deploy to GitHub Pages

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install OxideKit
        run: cargo install oxide-cli

      - name: Build
        run: oxide build --target static --release

      - name: Deploy
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./dist
```

### Custom Server

```bash
# Build the static site
oxide build --target static --release

# Serve with any static server
cd dist

# Python
python -m http.server 8000

# Node.js
npx serve

# Nginx (copy to web root)
cp -r dist/* /var/www/html/
```

## Preview Server

Start a local preview server:

```bash
# Build and serve
oxide build --target static && oxide serve

# With options
oxide serve --port 3000 --host 0.0.0.0

# Hot reload during development
oxide dev --target static
```

## Custom HTML Template

Override the default HTML template:

```html
<!-- templates/index.html -->
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{{ title }}</title>
    {{ head }}
    <link rel="stylesheet" href="{{ css }}">
</head>
<body>
    <div id="app">{{ content }}</div>
    <script src="{{ js }}"></script>
    <!-- Custom analytics -->
    <script>
        // Your analytics code
    </script>
</body>
</html>
```

```toml
[build.static.html]
template = "templates/index.html"
```

## Environment Variables

```toml
[build.static.env]
API_URL = "https://api.myapp.com"
PUBLIC_KEY = "pk_live_xxx"
```

Access in code:

```oui
Text {
    content: "API: {env.API_URL}"
}
```

## Comparing Build Modes

| Feature | Dev | Static | Desktop |
|---------|-----|--------|---------|
| Output | In-memory | HTML/CSS/JS | Binary |
| Hot Reload | Yes | Watch mode | Yes |
| Size | N/A | ~50-200KB | 5-15MB |
| SEO | N/A | Full support | N/A |
| Offline | No | Possible (PWA) | Yes |

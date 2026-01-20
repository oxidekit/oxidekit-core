# OxideKit Website - oxidekit.com

The official OxideKit website, built with OxideKit (dogfooding).

## Overview

This is the source for [oxidekit.com](https://oxidekit.com), demonstrating that OxideKit can build production-ready marketing websites without web frameworks.

**No React. No Next.js. No Zola. Just OxideKit.**

## Structure

```
oxidekit-site/
├── oxide.toml           # Project manifest
├── ui/
│   ├── app.oui          # Main app entry
│   ├── components/
│   │   ├── navbar.oui   # Navigation bar
│   │   └── footer.oui   # Site footer
│   └── pages/
│       ├── home.oui     # Landing page
│       ├── why-oxidekit.oui  # Why OxideKit page
│       ├── roadmap.oui  # Public roadmap
│       └── download.oui # Installation guide
└── README.md
```

## Pages

- **/** - Landing page with hero, features, comparison, and CTA
- **/why** - Detailed explanation of OxideKit's advantages
- **/docs** - Links to documentation site
- **/roadmap** - Public roadmap and version history
- **/download** - Installation instructions for all platforms

## Development

```bash
cd oxidekit-site

# Run development server
oxide dev

# Build for production
oxide build --target static

# Output will be in dist/
```

## Features Demonstrated

- **Responsive design** - Works on mobile, tablet, and desktop
- **Dark/light theme** - Toggle in navbar
- **Mobile menu** - Hamburger menu for small screens
- **Component reuse** - Shared navbar, footer, cards
- **Routing** - Hash-based routing for static deployment
- **SEO** - Open Graph tags, meta descriptions

## Deployment

Build outputs static HTML/CSS/JS that can be deployed anywhere:

```bash
oxide build --target static

# Deploy to GitHub Pages, Netlify, Vercel, etc.
```

## Design

The site uses OxideKit's design token system:

```oui
background: token("color.surface")
color: token("color.text.primary")
padding: token("spacing.6")
```

All colors, spacing, and typography come from the theme system, ensuring consistency and easy theming.

## License

Apache-2.0

# Landing Page Demo

A complete marketing landing page demonstrating OxideKit's capabilities for building modern, responsive websites.

## Features

- Navigation bar with logo and links
- Hero section with headline and CTAs
- Social proof with company logos
- Feature cards grid
- Pricing table with toggle (monthly/annual)
- Call-to-action section
- Full footer with links

## What This Demo Shows

- **Complex Layouts**: Full-page marketing layout with multiple sections
- **State Management**: Pricing toggle between monthly/annual billing
- **Components**: Reusable FeatureCard, PricingCard, FooterColumn, etc.
- **Lists**: Dynamic rendering of features and links with `@for`
- **Conditionals**: "Most Popular" badge with `@if`
- **Scrolling**: Full-page scroll with fixed navigation
- **Typography**: Multiple text sizes and colors for hierarchy
- **Spacing**: Consistent padding and margins throughout

## Project Structure

```
landing-page/
  oxide.toml      # Project manifest
  ui/
    app.oui       # Landing page UI
  README.md       # This file
```

## Running

```bash
cd landing-page
oxide dev          # Development with hot reload
oxide build --target static  # Build for static deployment
```

## Sections

### Hero Section

Large headline with subtext, CTA buttons, and social proof.

```oui
Column {
    align: center
    gap: 32

    Text { content: "Build Amazing Products" size: 64 }
    Text { content: "Faster Than Ever" size: 64 color: "#3B82F6" }

    // CTA buttons
    Row {
        Button { text: "Start Free Trial" }
        Button { text: "Watch Demo" variant: "outline" }
    }
}
```

### Feature Cards

Grid of feature cards with icon, title, and description.

```oui
FeatureCard {
    icon: "lightning"
    title: "Lightning Fast"
    description: "Built for speed..."
    color: "#F59E0B"
}
```

### Pricing Table

Three-tier pricing with monthly/annual toggle.

```oui
state {
    is_annual: bool = true
}

PricingCard {
    name: "Pro"
    price: state.is_annual ? "29" : "39"
    period: state.is_annual ? "/month, billed annually" : "/month"
    features: ["Unlimited projects", "Advanced analytics", ...]
    popular: true
}
```

### Footer

Multi-column footer with company info, links, and social icons.

```oui
FooterColumn {
    title: "Product"
    links: ["Features", "Pricing", "Integrations", "Changelog"]
}
```

## Components

### FeatureCard

```oui
component FeatureCard {
    prop icon: String
    prop title: String
    prop description: String
    prop color: String = "#3B82F6"
    // ...
}
```

### PricingCard

```oui
component PricingCard {
    prop name: String
    prop price: String
    prop period: String
    prop features: List<String>
    prop cta: String
    prop popular: bool = false
    // ...
}
```

### FooterColumn

```oui
component FooterColumn {
    prop title: String
    prop links: List<String>
    // ...
}
```

## Design Patterns

1. **Max Width Container**: Content is centered with `max_width: 1200` for readability
2. **Section Backgrounds**: Alternating backgrounds create visual separation
3. **Consistent Spacing**: Uses 16/24/32/48/64/80px spacing scale
4. **Color Hierarchy**: Primary, secondary, and muted text colors
5. **Interactive Elements**: Hover states and click handlers for buttons

## Static Build

This landing page is designed for static deployment:

```bash
oxide build --target static
```

The output can be deployed to any static hosting service (Netlify, Vercel, GitHub Pages, etc.).

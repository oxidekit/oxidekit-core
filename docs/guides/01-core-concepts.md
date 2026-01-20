# Core Concepts: How to Think in OxideKit

This guide teaches you the mental model for building applications with OxideKit. Understanding this flow is essential for productive development.

## The Conceptual Hierarchy

OxideKit organizes everything into a clear progression:

```
Tokens -> Components -> Packs -> Design -> Starters
```

Each layer builds on the previous one. Think of it like building blocks:

```
+-------------------+
|    STARTERS       |  Complete project templates
+-------------------+
         |
+-------------------+
|     DESIGN        |  Visual systems & layouts
+-------------------+
         |
+-------------------+
|      PACKS        |  Grouped functionality
+-------------------+
         |
+-------------------+
|   COMPONENTS      |  UI building blocks
+-------------------+
         |
+-------------------+
|     TOKENS        |  Design primitives
+-------------------+
```

---

## 1. Tokens: The Foundation

**Tokens are the atomic units of design.** They define the raw values that everything else uses.

### What Are Tokens?

Tokens are named, semantic values for:

- **Colors**: `color.primary`, `color.text`, `color.danger`
- **Spacing**: `spacing.sm`, `spacing.md`, `spacing.lg`
- **Typography**: `font.size.md`, `font.weight.bold`, `line-height.normal`
- **Radii**: `radius.md`, `radius.full`
- **Shadows**: `shadow.md`, `shadow.dialog`
- **Motion**: `duration.fast`, `easing.ease-out`

### Why Tokens Matter

Instead of hardcoding values:

```oui
// Bad: Magic numbers, hard to maintain
Button {
    background: "#3B82F6"
    padding: 12
    border-radius: 6
}
```

Use tokens:

```oui
// Good: Semantic, themeable, consistent
Button {
    background: $color.primary
    padding: $spacing.md
    border-radius: $radius.button
}
```

### Token Resolution Flow

```
$color.primary
      |
      v
+------------------+
|  Current Theme   |  (dark.toml / light.toml)
+------------------+
      |
      v
+------------------+
|  Resolved Value  |  "#3B82F6"
+------------------+
```

When the theme changes, all token references automatically update.

### Key Insight

> **Tokens let you separate "what it means" from "what the value is."**
>
> `$color.primary` means "the main brand color" - the actual hex value comes from the theme.

---

## 2. Components: The Building Blocks

**Components are reusable UI elements with defined contracts.**

### Component Anatomy

Every component has:

```
+----------------------------------+
|           ComponentSpec          |
+----------------------------------+
|  ID        | "ui.Button"         |
|  Pack      | "ui.core"           |
|  Props     | label, variant, ... |
|  Events    | on_click, on_hover  |
|  Slots     | default (children)  |
|  Variants  | primary, secondary  |
|  A11y      | role, keyboard      |
+----------------------------------+
```

### Props: The Component API

Props define what you can configure:

```oui
Button {
    label: "Submit"           // Required: string
    variant: "primary"        // Optional: enum (primary|secondary|outline)
    disabled: false           // Optional: bool
    icon: Icon { name: "check" }  // Optional: component
}
```

Props have types, defaults, and constraints - all machine-readable.

### Events: Component Communication

Components emit events to communicate:

```oui
Button {
    label: "Save"
    on_click: { save_data() }
    on_hover: { show_tooltip("Save your work") }
}
```

### Slots: Composition Points

Slots let components accept children:

```oui
Card {
    slot header {
        Text { "Card Title" }
    }
    slot default {
        Text { "Card content goes here" }
    }
    slot footer {
        Button { label: "Action" }
    }
}
```

### The Component Contract

Components guarantee:

1. **Consistent Props**: Same API across all uses
2. **Theme Integration**: Token-aware styling
3. **Accessibility**: Built-in a11y support
4. **Validation**: Props are checked at compile time

### Key Insight

> **Components are contracts, not just views.**
>
> A `Button` component isn't just "something that looks like a button" - it's a well-defined interface with explicit capabilities and constraints.

---

## 3. Packs: Grouped Functionality

**Packs bundle related components, tokens, and logic together.**

### Pack Types

```
+------------------+------------------------------------------+
|  Pack Type       |  Contains                                |
+------------------+------------------------------------------+
|  ui.core         |  Button, Input, Card, Text, etc.         |
|  ui.tables       |  DataTable, Column, Cell, etc.           |
|  ui.charts       |  BarChart, LineChart, etc.               |
|  theme.dark      |  Dark mode tokens and variants           |
|  native.fs       |  Filesystem access capabilities          |
|  service.auth    |  Authentication logic and UI             |
+------------------+------------------------------------------+
```

### Pack Structure

```
ui.tables/
  ├── manifest.toml        # Pack metadata & dependencies
  ├── components/
  │   ├── DataTable.oui    # Table component
  │   ├── Column.oui       # Column component
  │   └── Cell.oui         # Cell component
  ├── tokens/
  │   └── table.toml       # Table-specific tokens
  └── examples/
      └── basic.oui        # Usage examples
```

### Adding Packs

```bash
oxide add ui.tables              # From registry
oxide add git github.com/acme/auth@v1.0  # From Git
oxide add path ../local-pack     # Local development
```

### Key Insight

> **Packs are the unit of distribution and composition.**
>
> You don't install individual components - you install packs that provide a cohesive set of functionality.

---

## 4. Design: Visual Systems

**Design brings tokens and components together into cohesive visual systems.**

### What Design Includes

- **Themes**: Token value sets (dark, light, brand)
- **Typography Systems**: Font families, scales, roles
- **Layout Patterns**: Common UI structures
- **Admin Shells**: Full dashboard layouts

### Themes in Practice

```toml
# themes/acme-dark.toml
name = "ACME Dark"
extends = "oxidekit-dark"  # Start from base

[tokens.color]
primary = { value = "#FF6B00", light = "#FF8533", dark = "#CC5500" }
background = { value = "#0A0A0F" }

[tokens.radius]
button = 8
card = 16
```

### Typography Roles

Typography uses semantic roles, not just sizes:

```
+----------------+------------------------------------------+
|  Role          |  Usage                                   |
+----------------+------------------------------------------+
|  heading.hero  |  Main page title                         |
|  heading.h1    |  Section headings                        |
|  body.default  |  Regular text                            |
|  body.small    |  Secondary text, captions                |
|  ui.label      |  Form labels, buttons                    |
|  ui.caption    |  Helper text, metadata                   |
|  mono.code     |  Code blocks                             |
+----------------+------------------------------------------+
```

```oui
Text {
    role: "heading.h1"
    content: "Welcome to OxideKit"
}
```

### Key Insight

> **Design is not decoration - it's system.**
>
> A well-designed theme ensures consistency even when different developers build different parts of the app.

---

## 5. Starters: Complete Project Templates

**Starters are production-ready project scaffolds.**

### What Starters Provide

- Complete project structure
- Pre-installed packs and plugins
- Working layouts and pages
- Build configuration
- Example content

### Available Starters

```bash
oxide starters list
```

| Starter | Description | Target |
|---------|-------------|--------|
| `admin-panel` | Dashboard with tables, charts, sidebar | Desktop |
| `docs-site` | Documentation site with search | Static |
| `desktop-wallet` | Secure wallet UI | Desktop |
| `website-single` | Single-page marketing site | Static |
| `logger-dashboard` | Real-time log monitoring | Desktop |

### Using a Starter

```bash
# Create new project from starter
oxide new my-dashboard --starter admin-panel

# Initialize current directory
oxide init --starter admin-panel
```

### Starter Anatomy

```
admin-panel/
  ├── oxide.toml           # Project config
  ├── src/
  │   ├── app.oui          # Root component
  │   ├── pages/
  │   │   ├── dashboard.oui
  │   │   └── settings.oui
  │   └── layouts/
  │       └── admin.oui    # Admin shell layout
  ├── theme.toml           # Theme customization
  └── extensions.lock      # Locked dependencies
```

### Key Insight

> **Starters are not samples - they're foundations.**
>
> A starter gives you a production-grade starting point, not a toy example to learn from.

---

## The Complete Mental Model

Here's how everything connects:

```
                     Your Application
                           |
              +------------+------------+
              |                         |
        Starters                    Custom Build
              |                         |
    +---------+---------+     +---------+---------+
    |                   |     |                   |
  Design            Built-in  Your Tokens     Your Components
    |                Packs        |                |
    |                   |         +--------+-------+
    +-------------------+                  |
              |                         Your Packs
              |                            |
         All Tokens                   All Components
              |                            |
              +------------+---------------+
                           |
                     Component Tree
                           |
                       Rendering
```

### Development Flow

1. **Start with a Starter** (or `oxide new`)
2. **Customize Tokens** for your brand
3. **Use Components** from packs
4. **Add Packs** as needed
5. **Build custom Components** for unique needs
6. **Create custom Packs** to share functionality

### Key Principles

1. **Tokens are semantic**: Name by meaning, not by value
2. **Components are contracts**: Explicit props, events, slots
3. **Packs are cohesive**: Group related functionality
4. **Design is systematic**: Themes, not ad-hoc styling
5. **Starters are foundations**: Production-ready, not demos

---

## Quick Reference

### Hierarchy

| Level | Purpose | Example |
|-------|---------|---------|
| Token | Design primitive | `$color.primary` |
| Component | UI element | `Button`, `Card` |
| Pack | Feature bundle | `ui.tables` |
| Design | Visual system | Theme, typography |
| Starter | Project template | `admin-panel` |

### Commands

```bash
oxide new my-app --starter admin-panel  # Start with template
oxide add ui.tables                      # Add a pack
oxide export theme dark                  # Export theme tokens
oxide dev                                # Run dev server
oxide build --release                    # Production build
```

### File Types

| Extension | Purpose |
|-----------|---------|
| `.oui` | UI components |
| `.toml` | Configuration (oxide.toml, theme.toml) |
| `.lock` | Locked dependencies (extensions.lock) |

---

## Next Steps

- [Plugin Categories](./02-plugin-categories.md) - Understand UI, Service, Native, and Tooling plugins
- [Build Modes](./03-build-modes.md) - Dev vs Release vs Diagnostics
- [AI Assistant Philosophy](./04-ai-philosophy.md) - How AI helps without taking over

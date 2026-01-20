# Design Tokens

Design tokens are named values for colors, spacing, typography, and other visual properties. They provide consistency, theming support, and maintainability across your application.

## What Are Design Tokens?

Instead of hardcoding values throughout your application:

```oui
// Bad - hardcoded values
Text {
    color: "#E5E7EB"     // What does this mean?
    size: 16              // Is this consistent?
}
```

Use semantic tokens:

```oui
// Good - semantic tokens
Text {
    color: tokens.color.text.primary
    size: tokens.font.size.md
}
```

### Benefits

- **Consistency**: Same values everywhere in your app
- **Theming**: Change tokens, UI updates automatically
- **Maintainability**: Change once, update everywhere
- **Accessibility**: Semantic naming reveals purpose
- **Design System**: Enforces design decisions

## Color Tokens

### Semantic Colors

| Token | Purpose | Example |
|-------|---------|---------|
| `color.primary` | Primary brand color | Buttons, links, accents |
| `color.secondary` | Secondary actions | Secondary buttons |
| `color.success` | Success states | Success messages |
| `color.warning` | Warning states | Warning alerts |
| `color.danger` | Error/destructive | Delete buttons, errors |
| `color.info` | Informational | Info banners, tips |

```oui
Button {
    background: tokens.color.primary

    // Variants available
    background: tokens.color.primary.light   // Lighter shade
    background: tokens.color.primary.dark    // Darker shade
    background: tokens.color.primary.contrast // For text on primary
}
```

### Surface Colors

| Token | Purpose |
|-------|---------|
| `color.background` | Page/app background |
| `color.surface` | Card/panel background |
| `color.surface.variant` | Elevated surface |

```oui
Container {
    background: tokens.color.surface

    Container {
        background: tokens.color.surface.variant  // Elevated
    }
}
```

### Text Colors

| Token | Purpose |
|-------|---------|
| `color.text.primary` | Main text content |
| `color.text.secondary` | Supporting text |
| `color.text.disabled` | Disabled text |
| `color.text.inverse` | Text on dark/light opposite |

```oui
Column {
    Text {
        content: "Main Title"
        color: tokens.color.text.primary
    }

    Text {
        content: "Supporting description"
        color: tokens.color.text.secondary
    }
}
```

### Border & Divider Colors

| Token | Purpose |
|-------|---------|
| `color.border` | Default border |
| `color.border.strong` | Emphasized border |
| `color.divider` | Separator lines |

### State Colors

| Token | Purpose |
|-------|---------|
| `color.hover` | Hover state overlay |
| `color.focus` | Focus ring color |
| `color.active` | Active/pressed state |
| `color.disabled` | Disabled state overlay |

## Spacing Tokens

Consistent spacing scale based on a 4px base unit:

| Token | Value | Use Case |
|-------|-------|----------|
| `space.xs` | 4px | Tight spacing |
| `space.sm` | 8px | Compact elements |
| `space.md` | 16px | Default spacing |
| `space.lg` | 24px | Section gaps |
| `space.xl` | 32px | Large gaps |
| `space.xxl` | 48px | Page sections |

```oui
Column {
    gap: tokens.space.md        // 16px gap
    padding: tokens.space.lg    // 24px padding
    padding_x: tokens.space.xl  // 32px horizontal padding

    // Component-specific spacing
    padding: tokens.space.button  // 12px vertical, 16px horizontal
    padding: tokens.space.input   // 8px vertical, 12px horizontal
    padding: tokens.space.card    // 16px all around
}
```

## Border Radius Tokens

| Token | Value | Use Case |
|-------|-------|----------|
| `radius.none` | 0px | Sharp corners |
| `radius.sm` | 4px | Subtle rounding |
| `radius.md` | 8px | Default rounding |
| `radius.lg` | 12px | Cards, panels |
| `radius.xl` | 16px | Dialogs, modals |
| `radius.full` | 9999px | Pills, avatars |

```oui
Container {
    radius: tokens.radius.lg
}

// Component-specific radii
Button {
    radius: tokens.radius.button   // 6px
}

Card {
    radius: tokens.radius.card     // 12px
}
```

## Shadow Tokens

| Token | Description |
|-------|-------------|
| `shadow.none` | No shadow |
| `shadow.sm` | Subtle shadow |
| `shadow.md` | Default elevation |
| `shadow.lg` | Higher elevation |
| `shadow.xl` | Highest elevation |

```oui
// General shadows
Card {
    shadow: tokens.shadow.md
}

// Component-specific shadows
Card {
    shadow: tokens.shadow.card
}

Modal {
    shadow: tokens.shadow.dialog
}

Dropdown {
    shadow: tokens.shadow.dropdown
}
```

## Typography Tokens

### Font Families

| Token | Default |
|-------|---------|
| `font.family.sans` | Inter, system-ui, sans-serif |
| `font.family.serif` | Georgia, serif |
| `font.family.mono` | JetBrains Mono, monospace |

### Font Sizes

| Token | Value |
|-------|-------|
| `font.size.xs` | 12px |
| `font.size.sm` | 14px |
| `font.size.md` | 16px |
| `font.size.lg` | 18px |
| `font.size.xl` | 20px |
| `font.size.xxl` | 24px |
| `font.size.xxxl` | 32px |

### Font Weights

| Token | Value |
|-------|-------|
| `font.weight.thin` | 100 |
| `font.weight.light` | 300 |
| `font.weight.normal` | 400 |
| `font.weight.medium` | 500 |
| `font.weight.semibold` | 600 |
| `font.weight.bold` | 700 |
| `font.weight.extrabold` | 800 |

## Motion Tokens

### Durations

| Token | Value | Use Case |
|-------|-------|----------|
| `motion.duration.instant` | 50ms | Immediate feedback |
| `motion.duration.fast` | 150ms | Quick transitions |
| `motion.duration.normal` | 300ms | Default animations |
| `motion.duration.slow` | 500ms | Elaborate animations |

### Easing Functions

| Token | Description |
|-------|-------------|
| `motion.easing.linear` | Constant speed |
| `motion.easing.ease_in` | Slow start |
| `motion.easing.ease_out` | Slow end |
| `motion.easing.ease_in_out` | Slow start and end |

## Using Tokens in Code

```oui
// Reference tokens with the tokens namespace
Container {
    background: tokens.color.surface
    border: 1
    border_color: tokens.color.border
    radius: tokens.radius.lg
    shadow: tokens.shadow.md
    padding: tokens.space.lg

    Column {
        gap: tokens.space.md

        Text {
            content: "Card Title"
            size: tokens.font.size.xl
            weight: tokens.font.weight.semibold
            color: tokens.color.text.primary
        }

        Text {
            content: "Card description text"
            size: tokens.font.size.md
            color: tokens.color.text.secondary
        }
    }
}
```

## Defining Custom Tokens

In your `oxide.toml`:

```toml
[tokens]
# Custom colors
color.brand = "#8B5CF6"
color.brand.light = "#A78BFA"
color.brand.dark = "#7C3AED"

# Custom spacing
space.header = 64
space.sidebar = 280

# Custom radius
radius.card_large = 20
```

Then use them:

```oui
Container {
    background: tokens.color.brand
    height: tokens.space.header
    radius: tokens.radius.card_large
}
```

## Theme-Aware Tokens

Tokens automatically adapt to the current theme:

```oui
// These values change based on light/dark theme
Text {
    color: tokens.color.text.primary  // White in dark, black in light
}

Container {
    background: tokens.color.surface  // Dark gray in dark, white in light
}
```

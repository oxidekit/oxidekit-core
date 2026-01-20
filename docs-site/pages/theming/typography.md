# Typography

OxideKit provides a comprehensive typography system with semantic roles, configurable fonts, and design tokens for consistent text styling across your application.

## Typography Roles

Use semantic roles for consistent, purposeful typography:

```oui
Column {
    gap: 16

    Text { content: "Welcome" role: "display" }
    Text { content: "Section Title" role: "headline" }
    Text { content: "Card Title" role: "title" }
    Text { content: "Subtitle" role: "subtitle" }
    Text { content: "Body text" role: "body" }
    Text { content: "Small text" role: "body_small" }
    Text { content: "Label" role: "label" }
    Text { content: "Caption" role: "caption" }
    Text { content: "OVERLINE" role: "overline" }
}
```

### Role Reference

| Role | Size | Weight | Line Height | Use Case |
|------|------|--------|-------------|----------|
| `display` | 48px | Bold | 1.2 | Page titles |
| `headline` | 32px | Semibold | 1.3 | Section headings |
| `title` | 24px | Semibold | 1.4 | Card titles |
| `subtitle` | 18px | Medium | 1.4 | Subtitles |
| `body` | 16px | Normal | 1.6 | Body text |
| `body_small` | 14px | Normal | 1.6 | Secondary text |
| `button` | 14px | Medium | 1.0 | Button labels |
| `label` | 14px | Medium | 1.4 | Form labels |
| `caption` | 12px | Normal | 1.4 | Captions, hints |
| `code` | 14px | Normal | 1.6 | Code snippets |
| `overline` | 10px | Semibold | 1.2 | Section labels |

## Font Families

### Sans-Serif (Default)

```oui
Text {
    content: "Inter is the default sans-serif font"
    font: "sans"
}
```

### Serif

```oui
Text {
    content: "Georgia for serif text"
    font: "serif"
}
```

### Monospace

```oui
Text {
    content: "JetBrains Mono for code"
    font: "mono"
}
```

### Custom Fonts

Configure custom fonts in `oxide.toml`:

```toml
[fonts]
sans = "Plus Jakarta Sans"
serif = "Playfair Display"
mono = "Fira Code"

# Or with full configuration
[fonts.custom]
name = "Plus Jakarta Sans"
src = "assets/fonts/PlusJakartaSans.ttf"
weight = [400, 500, 600, 700]
```

## Font Sizes

Using tokens for consistent sizing:

```oui
Text { content: "Extra Small (12px)" size: tokens.font.size.xs }
Text { content: "Small (14px)" size: tokens.font.size.sm }
Text { content: "Medium (16px)" size: tokens.font.size.md }
Text { content: "Large (18px)" size: tokens.font.size.lg }
Text { content: "Extra Large (20px)" size: tokens.font.size.xl }
Text { content: "2XL (24px)" size: tokens.font.size.xxl }
Text { content: "3XL (32px)" size: tokens.font.size.xxxl }
```

### Size Scale

| Token | Value | Typical Use |
|-------|-------|-------------|
| `xs` | 12px | Badges, captions |
| `sm` | 14px | Labels, secondary text |
| `md` | 16px | Body text (default) |
| `lg` | 18px | Lead paragraphs |
| `xl` | 20px | Small headings |
| `xxl` | 24px | Section headings |
| `xxxl` | 32px | Page titles |
| `4xl` | 40px | Hero text |
| `5xl` | 48px | Display text |

## Font Weights

```oui
Column {
    gap: 8

    Text { content: "Thin (100)" weight: thin }
    Text { content: "Light (300)" weight: light }
    Text { content: "Normal (400)" weight: normal }
    Text { content: "Medium (500)" weight: medium }
    Text { content: "Semibold (600)" weight: semibold }
    Text { content: "Bold (700)" weight: bold }
    Text { content: "Extrabold (800)" weight: extrabold }
}
```

## Line Height

```oui
// Tight (1.2) - for headings
Text {
    content: "Tight line height"
    line_height: 1.2
}

// Normal (1.5) - for body
Text {
    content: "Normal line height for better readability"
    line_height: 1.5
}

// Relaxed (1.8) - for easier reading
Text {
    content: "Relaxed line height"
    line_height: 1.8
}
```

## Letter Spacing

```oui
// Tight tracking
Text {
    content: "TIGHTER"
    letter_spacing: -0.5
}

// Normal
Text {
    content: "Normal tracking"
    letter_spacing: 0
}

// Wide tracking (good for overlines)
Text {
    content: "WIDE TRACKING"
    letter_spacing: 1.5
    transform: uppercase
}
```

## Text Alignment

```oui
Column {
    width: fill
    gap: 12

    Text { content: "Left aligned (default)" align: start width: fill }
    Text { content: "Center aligned" align: center width: fill }
    Text { content: "Right aligned" align: end width: fill }
    Text { content: "Justified text spreads to fill the width" align: justify width: fill }
}
```

## Text Decoration

```oui
Column {
    gap: 8

    Text { content: "Underlined text" decoration: underline }
    Text { content: "Strikethrough text" decoration: line_through }

    // With color
    Text {
        content: "Custom underline"
        decoration: underline
        decoration_color: "#3B82F6"
        decoration_thickness: 2
    }
}
```

## Text Transform

```oui
Column {
    gap: 8

    Text { content: "UPPERCASE TEXT" transform: uppercase }
    Text { content: "lowercase text" transform: lowercase }
    Text { content: "capitalize each word" transform: capitalize }
}
```

## Truncation

### Single Line Ellipsis

```oui
Text {
    content: "This is a very long text that will be truncated with an ellipsis..."
    max_lines: 1
    overflow: ellipsis
    width: 200
}
```

### Multi-line Truncation

```oui
Text {
    content: "This is a longer paragraph that will span multiple lines before being truncated..."
    max_lines: 3
    overflow: ellipsis
}
```

## Responsive Typography

```oui
Text {
    content: "Responsive Heading"
    role: "display"

    // Breakpoint-based sizing
    size: {
        default: 32,
        sm: 40,
        md: 48,
        lg: 56
    }
}
```

Or using media queries:

```oui
Text {
    content: "Responsive Text"
    size: 24

    @media (min-width: 768px) {
        size: 32
    }

    @media (min-width: 1024px) {
        size: 48
    }
}
```

## Complete Typography Example

```oui
// Article layout
Column {
    gap: 24
    max_width: 700

    // Category
    Text {
        content: "ENGINEERING"
        role: "overline"
        color: tokens.color.primary
        letter_spacing: 1.5
    }

    // Title
    Text {
        content: "Building Performant Native Applications"
        role: "display"
        color: tokens.color.text.primary
    }

    // Subtitle
    Text {
        content: "A deep dive into GPU-accelerated rendering and memory optimization"
        role: "subtitle"
        color: tokens.color.text.secondary
    }

    // Meta info
    Row {
        gap: 16
        align: center

        Text {
            content: "John Doe"
            role: "body_small"
            color: tokens.color.text.secondary
        }

        Text {
            content: "5 min read"
            role: "caption"
            color: tokens.color.text.muted
        }
    }

    // Body
    Text {
        content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit..."
        role: "body"
        color: tokens.color.text.primary
        line_height: 1.7
    }

    // Code
    Text {
        content: "let app = OxideKit::new();"
        role: "code"
        font: "mono"
        color: tokens.color.text.primary
    }
}
```

## Accessibility

- Maintain minimum contrast ratio of 4.5:1 for normal text, 3:1 for large text
- Use relative units (rem, em) for user-scalable text
- Ensure proper heading hierarchy
- Don't rely on color alone for meaning

```oui
// Good - semantic and accessible
Column {
    Text { content: "Page Title" role: "display" aria_level: 1 }
    Text { content: "Section" role: "headline" aria_level: 2 }
    Text { content: "Subsection" role: "title" aria_level: 3 }
    Text { content: "Body content" role: "body" }
}
```

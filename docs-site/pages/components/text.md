# Text Component

The Text component displays text content with configurable styling, typography roles, and responsive behavior.

## Preview

```oui-preview
Column {
    gap: 16

    Text { content: "Display Text" role: "display" color: "#FFFFFF" }
    Text { content: "Headline Text" role: "headline" color: "#FFFFFF" }
    Text { content: "Body Text" role: "body" color: "#CBD5E1" }
    Text { content: "Caption Text" role: "caption" color: "#64748B" }
}
```

## Basic Usage

```oui
Text {
    content: "Hello, World!"
    size: 16
    color: "#FFFFFF"
}
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `content` | String | required | The text to display |
| `size` | Number | 16 | Font size in pixels |
| `color` | Color | inherit | Text color |
| `weight` | Enum | `normal` | Font weight |
| `role` | Enum | - | Typography preset role |
| `font` | String | `sans` | Font family (sans, serif, mono) |
| `align` | Enum | `start` | Text alignment |
| `decoration` | Enum | `none` | Text decoration |
| `transform` | Enum | `none` | Text transform |
| `max_lines` | Number | - | Maximum lines before truncation |
| `overflow` | Enum | `visible` | Overflow behavior |

## Typography Roles

Use semantic roles for consistent typography:

```oui
Column {
    gap: 16

    Text { content: "Welcome" role: "display" }           // 48px, Bold
    Text { content: "Section Title" role: "headline" }    // 32px, Semibold
    Text { content: "Card Title" role: "title" }          // 24px, Semibold
    Text { content: "Subtitle" role: "subtitle" }         // 18px, Medium
    Text { content: "Body text" role: "body" }            // 16px, Normal
    Text { content: "Small text" role: "body_small" }     // 14px, Normal
    Text { content: "Label" role: "label" }               // 14px, Medium
    Text { content: "Caption" role: "caption" }           // 12px, Normal
    Text { content: "OVERLINE" role: "overline" }         // 10px, Semibold
}
```

### Role Reference

| Role | Size | Weight | Use Case |
|------|------|--------|----------|
| display | 48px | Bold | Page titles |
| headline | 32px | Semibold | Section headings |
| title | 24px | Semibold | Card titles |
| subtitle | 18px | Medium | Subtitles |
| body | 16px | Normal | Body text |
| body_small | 14px | Normal | Secondary text |
| button | 14px | Medium | Button labels |
| label | 14px | Medium | Form labels |
| caption | 12px | Normal | Captions, hints |
| code | 14px | Normal (mono) | Code snippets |
| overline | 10px | Semibold | Section labels |

## Font Weights

```oui
Column {
    gap: 8

    Text { content: "Thin" weight: thin }         // 100
    Text { content: "Light" weight: light }       // 300
    Text { content: "Normal" weight: normal }     // 400
    Text { content: "Medium" weight: medium }     // 500
    Text { content: "Semibold" weight: semibold } // 600
    Text { content: "Bold" weight: bold }         // 700
    Text { content: "Extrabold" weight: extrabold } // 800
}
```

## Font Families

```oui
Column {
    gap: 12

    Text { content: "Sans-serif font" font: "sans" }
    Text { content: "Serif font" font: "serif" }
    Text { content: "Monospace font" font: "mono" }
}
```

## Text Alignment

```oui
Column {
    gap: 8
    width: fill

    Text { content: "Left aligned" align: start width: fill }
    Text { content: "Center aligned" align: center width: fill }
    Text { content: "Right aligned" align: end width: fill }
}
```

## Text Decoration

```oui
Column {
    gap: 8

    Text { content: "Normal text" }
    Text { content: "Underlined text" decoration: underline }
    Text { content: "Line-through text" decoration: line_through }
}
```

## Text Transform

```oui
Column {
    gap: 8

    Text { content: "normal case" }
    Text { content: "uppercase text" transform: uppercase }
    Text { content: "LOWERCASE TEXT" transform: lowercase }
    Text { content: "capitalize words" transform: capitalize }
}
```

## Truncation

### Single Line

```oui
Text {
    content: "This is a very long text that will be truncated..."
    max_lines: 1
    overflow: ellipsis
    width: 200
}
```

### Multi-line

```oui
Text {
    content: "This is a longer paragraph that spans multiple lines and will be truncated after the specified number of lines..."
    max_lines: 3
    overflow: ellipsis
}
```

## Using Design Tokens

```oui
Text {
    content: "Themed Text"
    color: tokens.color.text.primary
    size: tokens.font.size.md
}

Text {
    content: "Secondary Text"
    color: tokens.color.text.secondary
    size: tokens.font.size.sm
}
```

## Dynamic Content

```oui
app Counter {
    state {
        count: i32 = 0
    }

    Column {
        Text {
            content: "Count: {state.count}"
            size: 24
        }

        Button {
            label: "Increment"
            on_click: state.count += 1
        }
    }
}
```

## Clickable Text (Links)

```oui
Text {
    content: "Learn more"
    color: "#3B82F6"
    decoration: underline

    style {
        cursor: pointer
    }

    on click => open_link("/docs")
}
```

## Rich Text Pattern

For text with mixed styling, use multiple Text components:

```oui
Row {
    gap: 0
    wrap: wrap

    Text { content: "This is " color: "#FFFFFF" }
    Text { content: "important" color: "#EF4444" weight: bold }
    Text { content: " text with " color: "#FFFFFF" }
    Text { content: "mixed styles" color: "#3B82F6" decoration: underline }
    Text { content: "." color: "#FFFFFF" }
}
```

## Responsive Text

```oui
Text {
    content: "Responsive Heading"

    size: {
        mobile: 24,
        tablet: 32,
        desktop: 48
    }

    // Or use breakpoint syntax
    @media (min-width: 768px) {
        size: 32
    }

    @media (min-width: 1024px) {
        size: 48
    }
}
```

## Accessibility

- Use semantic roles for proper document structure
- Ensure sufficient color contrast (WCAG AA minimum)
- Don't use color alone to convey meaning
- Use proper heading hierarchy

```oui
// Good - semantic structure
Column {
    Text { content: "Page Title" role: "display" }
    Text { content: "Section" role: "headline" }
    Text { content: "Body content" role: "body" }
}
```

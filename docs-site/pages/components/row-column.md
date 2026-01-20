# Row & Column Components

Row and Column are the primary layout primitives for arranging children horizontally and vertically. They provide a flexbox-like layout system.

## Preview

```oui-preview
Row {
    gap: 16

    // Row example
    Container {
        padding: 16
        background: "#0F172A"
        radius: 8

        Column {
            gap: 8

            Text { content: "Row" size: 12 color: "#64748B" }

            Row {
                gap: 8

                Container { width: 40 height: 40 background: "#3B82F6" radius: 4 }
                Container { width: 40 height: 40 background: "#8B5CF6" radius: 4 }
                Container { width: 40 height: 40 background: "#EC4899" radius: 4 }
            }
        }
    }

    // Column example
    Container {
        padding: 16
        background: "#0F172A"
        radius: 8

        Column {
            gap: 8

            Text { content: "Column" size: 12 color: "#64748B" }

            Column {
                gap: 8

                Container { width: 100 height: 24 background: "#3B82F6" radius: 4 }
                Container { width: 100 height: 24 background: "#8B5CF6" radius: 4 }
                Container { width: 100 height: 24 background: "#EC4899" radius: 4 }
            }
        }
    }
}
```

## Basic Usage

### Row (Horizontal)

```oui
Row {
    gap: 16

    Text { content: "Item 1" }
    Text { content: "Item 2" }
    Text { content: "Item 3" }
}
```

### Column (Vertical)

```oui
Column {
    gap: 16

    Text { content: "Item 1" }
    Text { content: "Item 2" }
    Text { content: "Item 3" }
}
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `gap` | Number | 0 | Space between children |
| `align` | Enum | `start` | Cross-axis alignment |
| `justify` | Enum | `start` | Main-axis alignment |
| `wrap` | bool/Enum | `false` | Allow wrapping |
| `width` | Size | auto | Container width |
| `height` | Size | auto | Container height |
| `padding` | Number | 0 | Internal padding |
| `background` | Color | transparent | Background color |
| `flex` | Number | - | Flex grow factor |

## Gap (Spacing)

```oui
// No gap
Row {
    gap: 0
    // children touch each other
}

// Small gap
Row {
    gap: 8
}

// Medium gap
Row {
    gap: 16
}

// Large gap
Row {
    gap: 24
}
```

## Alignment

### Cross-Axis Alignment (align)

For Row, `align` controls vertical alignment. For Column, it controls horizontal alignment.

```oui
// Row with different alignments
Row {
    height: 100
    background: "#1E293B"

    // align: start - items at top
    // align: center - items in middle
    // align: end - items at bottom
    // align: stretch - items stretch to fill
    align: center
}

// Column with different alignments
Column {
    width: fill
    background: "#1E293B"

    // align: start - items at left
    // align: center - items in center
    // align: end - items at right
    // align: stretch - items stretch to fill
    align: center
}
```

### Main-Axis Alignment (justify)

For Row, `justify` controls horizontal distribution. For Column, it controls vertical distribution.

```oui
Row {
    width: fill
    justify: start          // Items at start (default)
    // justify: center      // Items centered
    // justify: end         // Items at end
    // justify: space_between  // Space between items
    // justify: space_around   // Space around items
    // justify: space_evenly   // Even space
}
```

### Common Alignment Patterns

```oui
// Center everything
Column {
    align: center
    justify: center
    width: fill
    height: fill

    Text { content: "Centered" }
}

// Space between
Row {
    justify: space_between
    width: fill

    Text { content: "Left" }
    Text { content: "Right" }
}

// Right-align items
Row {
    justify: end
    width: fill

    Button { label: "Cancel" }
    Button { label: "Save" }
}
```

## Flex Layout

### Flex Grow

```oui
Row {
    width: fill

    // First takes remaining space
    Container {
        flex: 1
        background: "#3B82F6"
        height: 40
    }

    // Fixed width
    Container {
        width: 100
        background: "#8B5CF6"
        height: 40
    }
}
```

### Proportional Sizing

```oui
Row {
    width: fill
    gap: 16

    Container { flex: 1 background: "#3B82F6" }  // 1/4
    Container { flex: 2 background: "#8B5CF6" }  // 2/4 (half)
    Container { flex: 1 background: "#EC4899" }  // 1/4
}
```

## Wrapping

Enable wrapping when children overflow:

```oui
Row {
    width: 300
    gap: 8
    wrap: true

    Container { width: 100 height: 40 background: "#3B82F6" radius: 4 }
    Container { width: 100 height: 40 background: "#8B5CF6" radius: 4 }
    Container { width: 100 height: 40 background: "#EC4899" radius: 4 }
    Container { width: 100 height: 40 background: "#F59E0B" radius: 4 }
}
```

### Wrap Direction

```oui
Row {
    wrap: true              // wrap to next row
    // wrap: wrap_reverse   // wrap to previous row
}
```

## Nested Layouts

Combine Row and Column for complex layouts:

```oui
// Two-column layout
Row {
    width: fill
    gap: 24

    // Sidebar
    Column {
        width: 250
        gap: 16

        Text { content: "Sidebar" }
    }

    // Main content
    Column {
        flex: 1
        gap: 16

        Text { content: "Main Content" }
    }
}
```

### Header-Content-Footer Layout

```oui
Column {
    width: fill
    height: fill

    // Header
    Container {
        width: fill
        height: 64
        background: "#1E293B"
    }

    // Content (grows to fill)
    Container {
        flex: 1
        width: fill
        background: "#0F172A"
    }

    // Footer
    Container {
        width: fill
        height: 48
        background: "#1E293B"
    }
}
```

## Common Patterns

### Navigation Bar

```oui
Row {
    width: fill
    height: 64
    padding_x: 24
    align: center
    justify: space_between
    background: "#1E293B"

    // Logo
    Text { content: "Logo" size: 20 color: "#FFFFFF" }

    // Nav links
    Row {
        gap: 32

        Text { content: "Home" color: "#FFFFFF" }
        Text { content: "About" color: "#94A3B8" }
        Text { content: "Contact" color: "#94A3B8" }
    }

    // Actions
    Button { label: "Sign In" variant: "primary" }
}
```

### Card Grid

```oui
Row {
    gap: 16
    wrap: true

    @for item in items {
        Container {
            width: 300
            padding: 24
            background: "#1E293B"
            radius: 12

            Column {
                gap: 12

                Text { content: item.title color: "#FFFFFF" }
                Text { content: item.description color: "#94A3B8" }
            }
        }
    }
}
```

### Form Layout

```oui
Column {
    gap: 24
    width: fill
    max_width: 400

    // Two-column input row
    Row {
        gap: 16

        Column {
            flex: 1
            gap: 8

            Text { content: "First Name" size: 14 color: "#FFFFFF" }
            Input { placeholder: "John" }
        }

        Column {
            flex: 1
            gap: 8

            Text { content: "Last Name" size: 14 color: "#FFFFFF" }
            Input { placeholder: "Doe" }
        }
    }

    // Full-width input
    Column {
        gap: 8

        Text { content: "Email" size: 14 color: "#FFFFFF" }
        Input { placeholder: "john@example.com" type: "email" }
    }

    // Actions
    Row {
        gap: 12
        justify: end

        Button { label: "Cancel" variant: "ghost" }
        Button { label: "Submit" variant: "primary" }
    }
}
```

## Responsive Layout

```oui
Row {
    width: fill
    gap: 24

    // Stack on mobile
    @media (max-width: 768px) {
        direction: column
    }

    Column { flex: 1 }
    Column { flex: 1 }
}
```

## Using Design Tokens

```oui
Column {
    gap: tokens.space.lg
    padding: tokens.space.xl

    Row {
        gap: tokens.space.md
    }
}
```

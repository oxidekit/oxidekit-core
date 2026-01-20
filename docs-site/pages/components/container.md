# Container Component

The Container is a fundamental layout primitive that provides a box model for grouping content with padding, margins, borders, and backgrounds.

## Preview

```oui-preview
Row {
    gap: 16

    Container {
        padding: 24
        background: "#1E293B"
        radius: 8

        Text { content: "Basic Container" color: "#FFFFFF" }
    }

    Container {
        padding: 24
        background: "#1E293B"
        radius: 12
        border: 2
        border_color: "#3B82F6"

        Text { content: "With Border" color: "#FFFFFF" }
    }
}
```

## Basic Usage

```oui
Container {
    padding: 16
    background: "#1E293B"
    radius: 8

    Text { content: "Hello" }
}
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `width` | Size | auto | Container width |
| `height` | Size | auto | Container height |
| `min_width` | Number | - | Minimum width |
| `max_width` | Number | - | Maximum width |
| `min_height` | Number | - | Minimum height |
| `max_height` | Number | - | Maximum height |
| `padding` | Number | 0 | Padding on all sides |
| `padding_x` | Number | - | Horizontal padding |
| `padding_y` | Number | - | Vertical padding |
| `padding_top` | Number | - | Top padding |
| `padding_right` | Number | - | Right padding |
| `padding_bottom` | Number | - | Bottom padding |
| `padding_left` | Number | - | Left padding |
| `margin` | Number | 0 | Margin on all sides |
| `margin_x` | Number | - | Horizontal margin |
| `margin_y` | Number | - | Vertical margin |
| `margin_top` | Number | - | Top margin |
| `margin_right` | Number | - | Right margin |
| `margin_bottom` | Number | - | Bottom margin |
| `margin_left` | Number | - | Left margin |
| `background` | Color | transparent | Background color |
| `radius` | Number | 0 | Border radius |
| `border` | Number | 0 | Border width |
| `border_color` | Color | - | Border color |
| `shadow` | Shadow | none | Box shadow |
| `overflow` | Enum | visible | Overflow behavior |

## Sizing

### Fixed Size

```oui
Container {
    width: 200
    height: 100
    background: "#1E293B"
}
```

### Fill Parent

```oui
Container {
    width: fill
    height: fill
    background: "#1E293B"
}
```

### Min/Max Constraints

```oui
Container {
    width: fill
    max_width: 800
    min_height: 200
    background: "#1E293B"
}
```

### Flex

```oui
Row {
    width: fill

    Container { flex: 1 background: "#1E293B" }  // Takes 1/3
    Container { flex: 2 background: "#334155" }  // Takes 2/3
}
```

## Padding

### Uniform Padding

```oui
Container {
    padding: 24
    background: "#1E293B"

    Text { content: "24px padding all around" }
}
```

### Axis-based Padding

```oui
Container {
    padding_x: 24  // Left and right
    padding_y: 16  // Top and bottom
    background: "#1E293B"
}
```

### Individual Sides

```oui
Container {
    padding_top: 24
    padding_right: 16
    padding_bottom: 24
    padding_left: 16
    background: "#1E293B"
}
```

## Margins

Same pattern as padding:

```oui
Container {
    margin: 16           // All sides
    margin_x: 24         // Horizontal
    margin_y: 16         // Vertical
    margin_top: 32       // Individual
    background: "#1E293B"
}
```

### Center with Margins

```oui
Container {
    width: 600
    margin_x: auto    // Center horizontally
    background: "#1E293B"
}
```

## Borders

### Simple Border

```oui
Container {
    padding: 16
    border: 1
    border_color: "#334155"
    radius: 8
}
```

### Individual Borders

```oui
Container {
    padding: 16
    border_bottom: 1
    border_color: "#334155"
}

Container {
    padding: 16
    border_left: 4
    border_color: "#3B82F6"
}
```

## Border Radius

### Uniform Radius

```oui
Container {
    padding: 16
    background: "#1E293B"
    radius: 8
}
```

### Individual Corners

```oui
Container {
    padding: 16
    background: "#1E293B"
    radius_top_left: 16
    radius_top_right: 16
    radius_bottom_left: 0
    radius_bottom_right: 0
}
```

### Pill Shape

```oui
Container {
    padding_x: 24
    padding_y: 12
    background: "#3B82F6"
    radius: 9999  // Large value creates pill

    Text { content: "Badge" color: "#FFFFFF" }
}
```

## Backgrounds

### Solid Color

```oui
Container {
    background: "#1E293B"
}
```

### Gradient

```oui
Container {
    style {
        background: "linear-gradient(135deg, #3B82F6, #8B5CF6)"
    }
}
```

### Transparent with Opacity

```oui
Container {
    style {
        background: "rgba(59, 130, 246, 0.1)"
    }
}
```

## Shadows

### Preset Shadows

```oui
Container {
    padding: 24
    background: "#1E293B"
    radius: 12
    shadow: tokens.shadow.md
}
```

### Custom Shadow

```oui
Container {
    padding: 24
    background: "#1E293B"
    radius: 12

    style {
        shadow: "0 10px 25px rgba(0, 0, 0, 0.2)"
    }
}
```

## Overflow

```oui
// Hidden overflow
Container {
    width: 200
    height: 100
    overflow: hidden

    Text { content: "Long text that gets clipped..." }
}

// Scroll overflow
Container {
    width: 200
    height: 100
    overflow: scroll

    Text { content: "Long text that can be scrolled..." }
}
```

## Card Pattern

Common card pattern using Container:

```oui
Container {
    padding: 24
    background: "#1E293B"
    radius: 12
    border: 1
    border_color: "#334155"
    shadow: tokens.shadow.sm

    Column {
        gap: 16

        Text {
            content: "Card Title"
            size: 18
            color: "#FFFFFF"
            weight: semibold
        }

        Text {
            content: "Card description text goes here."
            size: 14
            color: "#94A3B8"
        }

        Row {
            gap: 12
            justify: end

            Button { label: "Cancel" variant: "ghost" }
            Button { label: "Save" variant: "primary" }
        }
    }
}
```

## Centered Content Pattern

```oui
// Center content vertically and horizontally
Container {
    width: fill
    height: fill

    Column {
        align: center
        justify: center
        width: fill
        height: fill

        Text { content: "Centered Content" }
    }
}
```

## Interactive Container

Make a container clickable:

```oui
Container {
    padding: 16
    background: "#1E293B"
    radius: 8
    cursor: pointer

    style:hover {
        background: "#334155"
    }

    style:active {
        background: "#475569"
    }

    on click => handle_click

    Text { content: "Click me" color: "#FFFFFF" }
}
```

## Using Design Tokens

```oui
Container {
    padding: tokens.space.lg
    background: tokens.color.surface
    radius: tokens.radius.lg
    border: 1
    border_color: tokens.color.border
    shadow: tokens.shadow.md

    Text {
        content: "Themed Container"
        color: tokens.color.text.primary
    }
}
```

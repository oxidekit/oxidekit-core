# Oxide UI Language Specification

**Version:** 0.1.0
**Status:** Draft (Phase 0)

## Overview

Oxide UI (`.oui`) is a declarative UI language for OxideKit applications. It compiles to an intermediate representation (IR) that the runtime renders natively.

## Design Principles

1. **No arbitrary code execution** - UI files are data declarations, not programs
2. **Deterministic compilation** - Same input always produces same output
3. **Stable component IDs** - Enable state-preserving hot reload
4. **Binding-friendly** - Easy to express data bindings to Rust state
5. **Human-readable** - Clear, minimal syntax

## File Extension

All UI files use the `.oui` extension.

## Basic Syntax

### Comments

```oui
// Single-line comment

/*
   Multi-line
   comment
*/
```

### Component Usage

```oui
ComponentName {
    property: value
    another_property: "string value"

    // Child components
    ChildComponent {
        prop: value
    }
}
```

### App Declaration

Every application has a root `app` declaration:

```oui
app MyApp {
    // Root component tree
    Column {
        Text { content: "Hello" }
    }
}
```

### Component Definition

Custom components are defined with `component`:

```oui
component Button {
    // Properties with types and defaults
    prop label: String
    prop variant: String = "primary"
    prop disabled: bool = false

    // Component body
    Container {
        style {
            padding: 12
            radius: 8
            background: "#3B82F6"
        }

        Text {
            content: label
            color: "#FFFFFF"
        }
    }
}
```

## Property Types

| Type | Example | Description |
|------|---------|-------------|
| `String` | `"Hello"` | Text string |
| `i32` | `42` | 32-bit integer |
| `f32` | `3.14` | 32-bit float |
| `bool` | `true`, `false` | Boolean |
| `Color` | `"#FF5500"` | Hex color |

## Built-in Components

### Layout Components

#### `Column`
Vertical flex container.

```oui
Column {
    align: center      // start, center, end, stretch
    justify: center    // start, center, end, space-between, space-around
    gap: 16            // Space between children
    padding: 24        // Inner padding
    width: fill        // fill, fit, or number
    height: fill

    // children...
}
```

#### `Row`
Horizontal flex container.

```oui
Row {
    align: center
    justify: space-between
    gap: 8

    // children...
}
```

#### `Container`
Generic box container.

```oui
Container {
    width: 200
    height: 100
    padding: 16
    margin: 8

    // children...
}
```

#### `Scroll`
Scrollable region.

```oui
Scroll {
    direction: vertical  // vertical, horizontal, both

    // children...
}
```

### Display Components

#### `Text`
Text display.

```oui
Text {
    content: "Hello World"
    size: 16              // Font size
    color: "#E5E7EB"      // Text color
    font: "system"        // Font family
    weight: "normal"      // normal, bold, light
    align: left           // left, center, right
}
```

#### `Image`
Image display.

```oui
Image {
    src: "assets/logo.png"
    width: 100
    height: 100
    fit: contain          // contain, cover, fill
}
```

## Styling

### Inline Style Block

```oui
Container {
    style {
        background: "#1F2937"
        border: 1
        border_color: "#374151"
        radius: 12
        shadow: 4
        opacity: 0.9
    }

    // children...
}
```

### Style Properties

| Property | Type | Description |
|----------|------|-------------|
| `background` | Color | Background color |
| `border` | Number | Border width |
| `border_color` | Color | Border color |
| `radius` | Number | Corner radius |
| `shadow` | Number | Shadow blur |
| `opacity` | Number | Opacity (0-1) |
| `padding` | Number | Inner padding |
| `margin` | Number | Outer margin |

## Token References

Design tokens are referenced via the `tokens` namespace:

```oui
Text {
    size: tokens.font.size.lg
    color: tokens.colors.text.primary
}

Container {
    style {
        padding: tokens.space.md
        radius: tokens.radius.lg
        background: tokens.colors.surface
    }
}
```

## Theme References

Active theme values via the `theme` namespace:

```oui
Container {
    style {
        background: theme.card.bg
        border_color: theme.card.border
    }
}

Text {
    color: theme.text.primary
}
```

## Event Handlers

Events bind to Rust functions:

```oui
Button {
    label: "Submit"

    on click => app.handle_submit
    on hover => app.handle_hover
}
```

## State Bindings (Phase 2+)

```oui
app CounterApp {
    // State declaration
    state {
        count: i32 = 0
    }

    Column {
        Text {
            content: "Count: {state.count}"
        }

        Button {
            label: "Increment"
            on click => state.count += 1
        }
    }
}
```

## Conditionals (Phase 2+)

```oui
Container {
    @if state.loading {
        Spinner {}
    } @else {
        Content {}
    }
}
```

## Lists (Phase 2+)

```oui
Column {
    @for item in state.items {
        ListItem {
            key: item.id
            title: item.name
        }
    }
}
```

## Imports

```oui
// Import from another file
import { Button, Card } from "./components/ui.oui"

// Import theme
import theme from "theme.wallet.dark"

app MyApp {
    Card {
        Button { label: "Click me" }
    }
}
```

## Phase 0 Scope

For Phase 0, only these features are implemented:

- [x] Basic parsing
- [x] App declaration
- [x] Component usage (built-ins only)
- [x] Simple properties (String, Number, Color)
- [x] Text component
- [x] Column/Row layout
- [x] Container
- [ ] Custom components (Phase 1)
- [ ] Event handlers (Phase 1)
- [ ] State bindings (Phase 2)
- [ ] Conditionals (Phase 2)
- [ ] Lists (Phase 2)
- [ ] Imports (Phase 3)

## Grammar (EBNF)

```ebnf
program     = app_decl | component_decl* app_decl ;
app_decl    = "app" IDENT "{" element* "}" ;
component_decl = "component" IDENT "{" prop_decl* element* "}" ;
prop_decl   = "prop" IDENT ":" type ("=" literal)? ;
element     = IDENT "{" (property | element | style_block)* "}" ;
property    = IDENT ":" value ;
style_block = "style" "{" style_prop* "}" ;
style_prop  = IDENT ":" value ;
value       = literal | token_ref | theme_ref | IDENT ;
literal     = STRING | NUMBER | BOOL | COLOR ;
token_ref   = "tokens" "." path ;
theme_ref   = "theme" "." path ;
path        = IDENT ("." IDENT)* ;
type        = "String" | "i32" | "f32" | "bool" ;

IDENT       = [a-zA-Z_][a-zA-Z0-9_]* ;
STRING      = '"' [^"]* '"' ;
NUMBER      = [0-9]+ ("." [0-9]+)? ;
BOOL        = "true" | "false" ;
COLOR       = '"#' [0-9A-Fa-f]{6} '"' ;
```

## Example: Complete App

```oui
// hello.oui - A complete example

app HelloApp {
    Column {
        align: center
        justify: center
        width: fill
        height: fill
        gap: 24

        style {
            background: "#0B0F14"
        }

        Text {
            content: "Hello OxideKit!"
            size: 48
            color: "#E5E7EB"
        }

        Text {
            content: "A Rust-native application platform"
            size: 20
            color: "#9CA3AF"
        }

        Row {
            gap: 16

            Container {
                style {
                    padding: 12
                    radius: 8
                    background: "#3B82F6"
                }

                Text {
                    content: "Get Started"
                    color: "#FFFFFF"
                }
            }

            Container {
                style {
                    padding: 12
                    radius: 8
                    border: 1
                    border_color: "#374151"
                }

                Text {
                    content: "Learn More"
                    color: "#9CA3AF"
                }
            }
        }
    }
}
```

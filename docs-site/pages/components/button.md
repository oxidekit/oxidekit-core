# Button Component

A clickable button component for triggering actions. The Button component supports multiple variants, sizes, icons, and states.

## Preview

```oui-preview
Row {
    gap: 16
    justify: center

    Button { label: "Primary" variant: "primary" }
    Button { label: "Secondary" variant: "secondary" }
    Button { label: "Outline" variant: "outline" }
    Button { label: "Ghost" variant: "ghost" }
    Button { label: "Danger" variant: "danger" }
}
```

## Basic Usage

```oui
Button {
    label: "Click me"
    on_click: handle_click
}
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `label` | String | required | Button text content |
| `variant` | Enum | `"primary"` | Visual style variant |
| `size` | Enum | `"md"` | Button size (sm, md, lg) |
| `disabled` | bool | `false` | Disable the button |
| `loading` | bool | `false` | Show loading spinner |
| `icon` | String | - | Icon name to display |
| `icon_position` | Enum | `"start"` | Position of icon (start, end) |
| `width` | Size | auto | Button width |
| `on_click` | Callback | - | Click handler |

## Variants

### Primary

Use for main actions and CTAs. This is the most prominent button style.

```oui
Button {
    label: "Save Changes"
    variant: "primary"
    on_click: save_changes
}
```

### Secondary

For secondary actions alongside primary buttons.

```oui
Button {
    label: "Cancel"
    variant: "secondary"
    on_click: cancel
}
```

### Outline

Bordered button for less prominent actions.

```oui
Button {
    label: "Learn More"
    variant: "outline"
    on_click: learn_more
}
```

### Ghost

Minimal button for tertiary actions.

```oui
Button {
    label: "Skip"
    variant: "ghost"
    on_click: skip
}
```

### Danger

For destructive actions like delete.

```oui
Button {
    label: "Delete Account"
    variant: "danger"
    on_click: delete_account
}
```

## Sizes

```oui
Row {
    gap: 16
    align: center

    Button { label: "Small" size: "sm" }
    Button { label: "Medium" size: "md" }
    Button { label: "Large" size: "lg" }
}
```

## With Icons

```oui
// Icon at start (default)
Button {
    label: "Download"
    icon: "download"
}

// Icon at end
Button {
    label: "Next"
    icon: "arrow-right"
    icon_position: "end"
}

// Icon only
Button {
    icon: "plus"
    aria_label: "Add item"
}
```

## States

### Disabled

```oui
Button {
    label: "Submit"
    disabled: !state.form_valid
}
```

### Loading

```oui
Button {
    label: state.submitting ? "Saving..." : "Save"
    loading: state.submitting
    on_click: submit_form
}
```

## Full Width Button

```oui
Button {
    label: "Sign In"
    width: fill
}
```

## Button Group

Create connected buttons for related actions:

```oui
Row {
    gap: 0

    Button {
        label: "Left"
        variant: "outline"
        style {
            border_radius_right: 0
        }
    }

    Button {
        label: "Center"
        variant: "outline"
        style {
            border_radius: 0
            border_left: 0
        }
    }

    Button {
        label: "Right"
        variant: "outline"
        style {
            border_radius_left: 0
            border_left: 0
        }
    }
}
```

## Form Actions Pattern

```oui
Row {
    gap: 12
    justify: end

    Button { label: "Cancel" variant: "ghost" on_click: cancel }
    Button { label: "Save Draft" variant: "outline" on_click: save_draft }
    Button { label: "Publish" variant: "primary" on_click: publish }
}
```

## Events

| Event | Payload | Description |
|-------|---------|-------------|
| `on_click` | - | Fired when button is clicked |
| `on_focus` | - | Fired when button receives focus |
| `on_blur` | - | Fired when button loses focus |

## Accessibility

- **Role**: button
- **Keyboard**: Enter and Space activate the button
- **Focus**: Visible focus ring on keyboard navigation
- **Disabled**: aria-disabled is set automatically

For icon-only buttons, always provide an aria_label:

```oui
Button {
    icon: "close"
    aria_label: "Close dialog"
    on_click: close_dialog
}
```

## Custom Styling

Override default styles with the style block:

```oui
Button {
    label: "Custom Button"

    style {
        background: "#8B5CF6"
        color: "#FFFFFF"
        radius: 20
        padding_x: 24
        padding_y: 12
    }

    style:hover {
        background: "#7C3AED"
    }

    style:active {
        background: "#6D28D9"
    }
}
```

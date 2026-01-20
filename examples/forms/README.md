# Forms Showcase

A comprehensive demonstration of all OxideKit form components with validation examples and different sizes/variants.

## Features

- Text inputs (text, email, password)
- Textarea with auto-resize
- Select/dropdown menus
- Checkboxes and radio buttons
- Toggle switches
- Sliders (single and range)
- Buttons (all variants and sizes)
- Form validation
- Complete form example

## What This Demo Shows

- **Form Components**: All built-in form elements
- **Validation**: Email and password validation with error messages
- **Sizes**: Small, medium, and large variants
- **States**: Normal, disabled, loading, error states
- **Accessibility**: Labels, helper text, required indicators
- **Reusable Components**: FormField and Section wrapper components

## Project Structure

```
forms/
  oxide.toml      # Project manifest
  ui/
    app.oui       # Forms showcase UI
  README.md       # This file
```

## Running

```bash
cd forms
oxide dev          # Development with hot reload
oxide build --target static  # Build for static deployment
```

## Form Components

### TextInput

```oui
TextInput {
    type: "email"
    placeholder: "john@example.com"
    value: state.email
    on_change: state.email = value
    size: "medium"
    error: state.has_error
}
```

### Select

```oui
Select {
    value: state.country
    on_change: state.country = value
    placeholder: "Select a country"

    Option { value: "us" label: "United States" }
    Option { value: "uk" label: "United Kingdom" }
}
```

### Checkbox

```oui
Checkbox {
    label: "Accept terms"
    checked: state.accepted
    on_change: state.accepted = value
}
```

### RadioGroup

```oui
RadioGroup {
    value: state.selection
    on_change: state.selection = value

    Radio { value: "opt1" label: "Option 1" }
    Radio { value: "opt2" label: "Option 2" }
}
```

### Switch

```oui
Switch {
    checked: state.enabled
    on_change: state.enabled = value
}
```

### Slider

```oui
Slider {
    value: state.volume
    min: 0
    max: 100
    on_change: state.volume = value
}
```

### Button Variants

```oui
Button { text: "Primary" variant: "primary" }
Button { text: "Secondary" variant: "secondary" }
Button { text: "Outline" variant: "outline" }
Button { text: "Ghost" variant: "ghost" }
Button { text: "Destructive" variant: "destructive" }
Button { text: "Link" variant: "link" }
```

### Button with Icon

```oui
Button {
    text: "Save"
    icon: "save"
    variant: "primary"
}
```

## Validation Example

```oui
state {
    email_value: String = ""
    email_error: String = ""
}

FormField {
    label: "Email"
    error: state.email_error
    required: true

    TextInput {
        type: "email"
        value: state.email_value
        on_change: app.validate_email
        error: state.email_error != ""
    }
}
```

## Reusable Components

### FormField

Wraps form inputs with label, helper text, and error display.

```oui
FormField {
    label: "Email"
    helper: "We'll never share your email"
    error: state.email_error
    required: true

    TextInput { ... }
}
```

### Section

Groups related form elements with a title.

```oui
Section {
    title: "Personal Information"

    // Form fields here
}
```

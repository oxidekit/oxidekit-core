# AI Assistant Philosophy: How AI Helps Without Taking Over

OxideKit is designed to be AI-friendly, but with a clear philosophy: **AI assists developers, it doesn't replace them.** This guide explains how AI integration works and why certain design decisions were made.

## The Core Principle

```
+----------------------------------+
|        AI AS ASSISTANT           |
+----------------------------------+
|                                  |
|  AI suggests  ->  Human decides  |
|  AI validates ->  Human reviews  |
|  AI generates ->  Human owns     |
|                                  |
+----------------------------------+
```

AI should amplify developer capabilities, not create black boxes that developers don't understand.

---

## Why AI-Native Design Matters

### The Problem with Traditional Approaches

Traditional UI frameworks are built for human comprehension:

```
// Human reads this and understands it
<Button variant="primary" onClick={save}>Save</Button>
```

But AI can hallucinate:

```
// AI might generate this (wrong!)
<Button type="save" primary onSave={handleSave}>Save</Button>
```

The syntax looks plausible, but `type="save"`, `primary` (as a prop), and `onSave` don't exist.

### The OxideKit Solution

OxideKit provides machine-readable specifications that AI can query:

```json
{
  "id": "ui.Button",
  "props": [
    { "name": "variant", "type": "enum", "values": ["primary", "secondary", "outline"] },
    { "name": "disabled", "type": "bool" }
  ],
  "events": [
    { "name": "on_click", "payload": null }
  ]
}
```

Now AI knows exactly what's valid:

```
// AI generates this (correct!)
Button {
    variant: "primary"
    on_click: { save() }
    label: "Save"
}
```

---

## The AI Catalog (oxide.ai.json)

OxideKit exports a complete, machine-readable catalog:

```bash
oxide export ai-schema
```

### Catalog Structure

```json
{
  "schema_version": "1.0",
  "oxidekit_version": "0.5.0",
  "generated_at": "2024-01-15T10:00:00Z",

  "components": {
    "ui.Button": {
      "id": "ui.Button",
      "pack": "ui.core",
      "props": [...],
      "events": [...],
      "slots": [...],
      "examples": [...]
    }
  },

  "tokens": {
    "color.primary": { "type": "color", "theme": "both" },
    "spacing.md": { "type": "spacing", "value": 16 }
  },

  "extensions": {
    "ui.tables": { "components": ["DataTable", "Column"] }
  },

  "starters": {
    "admin-panel": { "description": "..." }
  }
}
```

### What AI Can Query

| Query | Answer |
|-------|--------|
| "What props does Button have?" | `variant`, `disabled`, `label`, `loading` |
| "What events does DataTable emit?" | `on_row_click`, `on_sort`, `on_filter` |
| "What are the valid color tokens?" | `color.primary`, `color.danger`, etc. |
| "How do I create a dashboard?" | Use `admin-panel` starter |

---

## MCP Server Integration

OxideKit provides a Model Context Protocol (MCP) server for AI tools:

```bash
oxide mcp serve --port 9090
```

### Available MCP Methods

| Method | Description |
|--------|-------------|
| `oxide.list_components` | Get all available components |
| `oxide.get_component` | Get full spec for a component |
| `oxide.validate_oui` | Validate OUI code |
| `oxide.get_tokens` | Get theme tokens |
| `oxide.get_recipes` | Get code recipes |

### Example MCP Interaction

```json
// AI Request
{
  "method": "oxide.get_component",
  "params": { "id": "ui.Button" }
}

// Response
{
  "result": {
    "id": "ui.Button",
    "props": [
      { "name": "variant", "type": "enum", "values": ["primary", "secondary"] },
      { "name": "label", "type": "string", "required": true }
    ],
    "events": [{ "name": "on_click" }],
    "examples": [
      {
        "title": "Primary Button",
        "code": "Button { variant: \"primary\", label: \"Click me\", on_click: { do_action() } }"
      }
    ]
  }
}
```

---

## Validation: Catching AI Mistakes

OxideKit validates all generated code:

```
AI generates code
       |
       v
+------------------+
|  Parse & Validate |
+------------------+
       |
       v
+---------+--------+
|  Valid  | Invalid |
+---------+--------+
    |          |
    v          v
  Accept    Error with
             fix suggestions
```

### Validation Examples

**Invalid Prop:**
```oui
Button {
    type: "submit"  // Error: Unknown prop "type"
}
```
```
Error: UI-200 - Unknown prop "type" on component "Button"
       Suggestion: Did you mean "variant"?
       Valid props: variant, label, disabled, loading
```

**Invalid Value:**
```oui
Button {
    variant: "blue"  // Error: Invalid value
}
```
```
Error: UI-201 - Invalid value "blue" for prop "variant"
       Valid values: primary, secondary, outline, ghost
```

**Type Mismatch:**
```oui
Button {
    disabled: "yes"  // Error: Type mismatch
}
```
```
Error: UI-202 - Type mismatch for prop "disabled"
       Expected: bool
       Got: string
       Suggestion: Use `disabled: true` or `disabled: false`
```

---

## Recipes: Guided Code Generation

Recipes provide step-by-step patterns for common tasks:

```json
{
  "recipe": "add-data-table",
  "description": "Add a sortable data table with pagination",
  "steps": [
    {
      "action": "add_extension",
      "extension": "ui.tables",
      "reason": "Provides DataTable component"
    },
    {
      "action": "insert_code",
      "location": "user-specified",
      "code": "DataTable {\n  columns: [...]\n  data: $data\n}"
    },
    {
      "action": "add_state",
      "name": "data",
      "type": "array",
      "initial": "[]"
    }
  ]
}
```

### Recipe vs Freeform Generation

| Approach | Risk | Control |
|----------|------|---------|
| Freeform AI | High (hallucination) | Low |
| Recipe-guided | Low (validated steps) | High |

---

## Design Packs: AI-Extractable Templates

Design packs include tagged, extractable parts:

```oui
// admin-shell/sidebar.oui

// @ai-tag: sidebar-nav
// @description: Navigation sidebar with collapsible sections
Sidebar {
    slot items {
        // @ai-tag: nav-item
        NavItem {
            icon: "dashboard"
            label: "Dashboard"
            href: "/"
        }
        // @ai-tag: nav-section
        NavSection {
            label: "Users"
            items: [...]
        }
    }
}
```

AI can extract these patterns:

```json
{
  "method": "oxide.extract_template_part",
  "params": {
    "pack": "design.admin-shell",
    "tag": "sidebar-nav"
  }
}
```

---

## The Human-AI Workflow

### 1. Discovery

```
Human: "I need a data table"
   |
   v
AI: [Queries oxide.ai.json]
   |
   v
AI: "I found ui.tables extension with DataTable component.
     It supports sorting, filtering, and pagination.
     Would you like me to add it?"
```

### 2. Generation

```
Human: "Yes, add a table showing users"
   |
   v
AI: [Uses recipe: add-data-table]
   |
   v
AI generates:
   - Adds ui.tables extension
   - Creates DataTable with user columns
   - Sets up data binding
   |
   v
Human: Reviews and adjusts
```

### 3. Validation

```
AI-generated code
   |
   v
[OxideKit validates]
   |
   v
If errors: AI shows errors and suggests fixes
If valid: Code is accepted
   |
   v
Human: Final review and commit
```

### 4. Iteration

```
Human: "Make the email column sortable"
   |
   v
AI: [Knows DataTable API]
   |
   v
AI: "I'll add sortable: true to the email column"
   |
   v
Human: Confirms or adjusts
```

---

## What AI Should NOT Do

### 1. Generate Unknown Components

```
BAD:  AI invents "FancyButton" that doesn't exist
GOOD: AI uses only components from oxide.ai.json
```

### 2. Use Invalid Props

```
BAD:  AI uses "onClick" (React-style)
GOOD: AI uses "on_click" (OxideKit style)
```

### 3. Hardcode Values

```
BAD:  AI generates: background: "#3B82F6"
GOOD: AI generates: background: $color.primary
```

### 4. Skip Validation

```
BAD:  AI generates code and assumes it's correct
GOOD: AI generates code, validates, reports any issues
```

### 5. Create Magic Black Boxes

```
BAD:  AI generates complex code the human can't understand
GOOD: AI explains what it's generating and why
```

---

## Prompt Guidance for AI

When AI generates OxideKit code, it should:

### Always

1. Query the catalog first
2. Use exact prop names and types
3. Use token references for styling
4. Include error handling
5. Generate readable, commented code

### Never

1. Invent components or props
2. Use hardcoded color values
3. Generate without validation
4. Create overly complex one-liners
5. Skip accessibility considerations

### Template for AI Prompts

```
When generating OxideKit code:

1. Check oxide.ai.json for component specs
2. Use only props listed in the spec
3. Use $token references for colors, spacing, etc.
4. Follow the examples in the spec
5. Validate output before presenting
6. Explain what you're generating

Do not:
- Invent components or props
- Use hardcoded style values
- Generate without validation
```

---

## Practical Examples

### Good AI Interaction

```
User: "Add a button that saves the form"

AI: I'll add a Button component. Looking at the spec...

    Button supports:
    - variant: "primary" | "secondary" | "outline"
    - label: string (required)
    - disabled: bool
    - loading: bool
    - on_click: event

    Here's the code:

    Button {
        variant: "primary"
        label: "Save"
        disabled: $form.is_invalid
        loading: $form.is_saving
        on_click: { save_form() }
    }

    This button:
    - Uses primary variant for main actions
    - Disables when form is invalid
    - Shows loading state during save
    - Calls save_form() on click

    Would you like me to add this?
```

### Bad AI Interaction

```
User: "Add a button that saves the form"

AI: Here's a button:

    <Button type="submit" onClick={saveForm} className="btn-primary">
        Save
    </Button>

    [Wrong framework! Wrong syntax! No validation!]
```

---

## The OxideKit Promise

1. **Every component is documented** in machine-readable format
2. **Every prop is validated** at compile time
3. **Every error has a code** and suggestion
4. **Every pattern has a recipe** for guided generation
5. **AI is a tool**, not the author

---

## Next Steps

- [Quick Start](./05-quick-start.md) - Get building in 5 minutes
- [Component Reference](../reference/components.md) - Full component specs
- [Token Reference](../reference/tokens.md) - All available tokens

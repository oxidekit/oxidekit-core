# Counter Demo

A simple +/- counter application demonstrating OxideKit's state management basics.

## Features

- Increment counter with + button
- Decrement counter with - button
- Reset counter to zero
- Clean, minimal UI design

## What This Demo Shows

- **State Management**: Using `state` block to declare reactive state
- **Event Handling**: Click handlers with `on click =>` syntax
- **State Updates**: Inline state mutations (`state.count += 1`)
- **Layout System**: Column and Row layouts with flexbox properties
- **Styling**: Container styling with background, radius, and padding

## Project Structure

```
counter/
  oxide.toml      # Project manifest
  ui/
    app.oui       # Main UI component
  README.md       # This file
```

## Running

```bash
cd counter
oxide dev          # Development with hot reload
oxide build --target static  # Build for static deployment
```

## Code Highlights

### State Declaration

```oui
state {
    count: i32 = 0
}
```

### Event Handling

```oui
Container {
    on click => state.count += 1
    // ...
}
```

### Data Binding

```oui
Text {
    content: "{state.count}"
}
```

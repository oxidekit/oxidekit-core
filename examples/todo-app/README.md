# Todo App Demo

A full-featured todo list application demonstrating OxideKit's forms, lists, and filtering capabilities.

## Features

- Add new todos with text input
- Mark todos as complete/incomplete
- Delete individual todos
- Filter todos (All, Active, Completed)
- Clear all completed todos
- Display remaining item count

## What This Demo Shows

- **Forms**: Text input with two-way binding
- **Lists**: Dynamic list rendering with `@for`
- **Conditionals**: Show/hide elements with `@if`
- **Components**: Reusable `TodoItem` component with props
- **State Management**: Complex state with arrays and filters
- **Events**: Custom event handlers for toggle, delete, clear

## Project Structure

```
todo-app/
  oxide.toml      # Project manifest
  ui/
    app.oui       # Main UI with todo logic
  README.md       # This file
```

## Running

```bash
cd todo-app
oxide dev          # Development with hot reload
oxide build --target static  # Build for static deployment
```

## Code Highlights

### List State

```oui
state {
    todos: List<Todo> = []
    filter: String = "all"
}
```

### Dynamic List Rendering

```oui
@for todo in state.filtered_todos {
    TodoItem {
        key: todo.id
        todo: todo
    }
}
```

### Conditional Rendering

```oui
@if state.completed_count > 0 {
    Container {
        on click => app.clear_completed
        Text { content: "Clear completed" }
    }
}
```

### Two-Way Binding

```oui
TextInput {
    value: state.new_todo_text
    on_change: state.new_todo_text = value
}
```

### Reusable Components

```oui
component TodoItem {
    prop todo: Todo
    prop on_toggle: Function
    prop on_delete: Function
    // ...
}
```

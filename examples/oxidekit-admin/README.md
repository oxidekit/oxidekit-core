# OxideKit Admin - Golden Reference App

This is the **Golden Reference App** that validates the OxideKit platform before public launch.

## Purpose

This application exercises all major OxideKit platform capabilities:

- **UI Rendering** - Sidebar, data tables, forms, charts
- **Backend Integration** - Contract-first API client generation
- **Plugin System** - UI data, forms, native filesystem plugins
- **Permissions** - Network allowlist, filesystem access control
- **Diagnostics** - Crash reporting, performance metrics
- **Theming** - Light/dark mode toggle
- **Hot Reload** - State preservation across code changes

## Structure

```
oxidekit-admin/
├── oxide.toml              # App manifest with permissions
├── admin.contract.toml     # Backend API contract
├── src/
│   └── main.rs            # App entry point and state
├── ui/
│   ├── app.oui            # Root app component
│   ├── layouts/
│   │   └── admin_shell.oui    # Sidebar + topbar layout
│   ├── pages/
│   │   ├── dashboard.oui      # Stats and charts
│   │   ├── users.oui          # Data table + CRUD
│   │   └── settings.oui       # Forms and preferences
│   └── components/
│       ├── stat_card.oui      # Statistics card
│       └── nav_item.oui       # Navigation item
└── assets/
    ├── fonts/
    └── icons/
```

## Running

```bash
# Development mode with hot reload
oxide dev

# Production build
oxide build --release

# Run tests
cargo test
```

## Features Demonstrated

### UI Components
- Sidebar navigation (collapsible)
- Top bar with search and user menu
- Stats cards with change indicators
- Data tables with sorting, filtering, pagination
- Forms with validation
- Modal dialogs
- Theme toggle

### Backend Contract
The `admin.contract.toml` defines:
- User CRUD endpoints (list, get, create, update, delete)
- Type-safe request/response types
- Generated client code

### Permissions
```toml
[permissions]
network = ["https://api.example.com", "https://localhost:8080"]
filesystem = { read = ["./data"], write = ["./exports"] }
```

### Diagnostics
- Crash reporting enabled
- Performance metrics collection
- Devtools inspector

## Validation Checklist

- [x] App builds and runs
- [x] Tests pass (6 tests)
- [x] UI files parse correctly
- [x] Permissions configured
- [x] Backend contract defined
- [x] Theme toggle works
- [x] All major features exercised

This app validates that OxideKit is ready for public launch.

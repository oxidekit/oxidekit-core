# Dashboard Demo

A complete admin dashboard demonstrating OxideKit's layout capabilities, navigation, stat cards, data tables, and chart placeholders.

## Features

- Collapsible sidebar navigation
- Top bar with user menu
- Stat cards with metrics
- Revenue chart (placeholder)
- Traffic sources pie chart (placeholder)
- Data table with recent orders
- Responsive layout

## What This Demo Shows

- **Complex Layouts**: Sidebar + main content with nested Row/Column
- **State Management**: Sidebar collapse state, current page
- **Navigation**: Page switching with active state
- **Components**: Reusable StatCard, NavItem, TableRow, LegendItem
- **Conditionals**: Show/hide elements based on sidebar state
- **Scrolling**: Scrollable content area
- **Tables**: Data table with header and rows

## Project Structure

```
dashboard/
  oxide.toml      # Project manifest
  ui/
    app.oui       # Main dashboard UI
  README.md       # This file
```

## Running

```bash
cd dashboard
oxide dev          # Development with hot reload
oxide build --target static  # Build for static deployment
```

## Components

### StatCard

Displays a metric with title, value, change indicator, and icon.

```oui
StatCard {
    title: "Total Users"
    value: "12,847"
    change: "+12.5%"
    change_type: "positive"
    icon: "users"
    color: "#3B82F6"
}
```

### NavItem

Navigation item with icon, label, and active state.

```oui
NavItem {
    icon: "home"
    label: "Overview"
    active: state.current_page == "overview"
    collapsed: state.sidebar_collapsed
    on_click: state.current_page = "overview"
}
```

### TableRow

Data table row with order information.

```oui
TableRow {
    order_id: "#ORD-001"
    customer: "Alice Johnson"
    product: "Pro Subscription"
    amount: "$99.00"
    status: "completed"
}
```

## Layout Structure

```
Row
  Column (Sidebar - 240px or 64px)
    Logo
    NavItems
    Collapse Toggle
  Column (Main - flex: 1)
    TopBar (64px)
    Scroll (Content)
      Stats Row
      Charts Row
      Data Table
```

## Extending

To add real charts, install the charts extension:

```bash
oxide add ui.charts
```

Then replace the chart placeholders with actual chart components:

```oui
LineChart {
    data: state.revenue_data
    x_axis: "date"
    y_axis: "revenue"
}
```

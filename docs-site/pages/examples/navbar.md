# Navbar Example

A responsive navigation bar with logo, links, and action buttons. This pattern is fundamental to most websites and applications.

## Preview

```oui-preview
Row {
    width: fill
    height: 64
    padding_x: 24
    align: center
    justify: space_between
    background: "#0F172A"

    // Logo
    Row {
        gap: 12
        align: center

        Container {
            width: 32
            height: 32
            background: "#3B82F6"
            radius: 8

            Column {
                align: center
                justify: center
                width: fill
                height: fill

                Text { content: "O" size: 18 color: "#FFFFFF" }
            }
        }

        Text { content: "OxideKit" size: 18 color: "#FFFFFF" weight: bold }
    }

    // Nav Links
    Row {
        gap: 32

        Text { content: "Home" color: "#FFFFFF" }
        Text { content: "Features" color: "#94A3B8" }
        Text { content: "Pricing" color: "#94A3B8" }
        Text { content: "Docs" color: "#94A3B8" }
    }

    // CTA
    Container {
        padding_x: 16
        padding_y: 8
        background: "#3B82F6"
        radius: 6

        Text { content: "Get Started" size: 14 color: "#FFFFFF" }
    }
}
```

## Basic Navbar

A simple navigation bar with logo and links:

```oui
Row {
    width: fill
    height: 64
    padding_x: 24
    align: center
    justify: space_between
    background: "#0F172A"

    // Logo
    Row {
        gap: 12
        align: center

        Container {
            width: 32
            height: 32
            background: "#3B82F6"
            radius: 8

            Column {
                align: center
                justify: center
                width: fill
                height: fill

                Text {
                    content: "O"
                    size: 18
                    color: "#FFFFFF"
                    weight: bold
                }
            }
        }

        Text {
            content: "OxideKit"
            size: 18
            color: "#FFFFFF"
            weight: bold
        }
    }

    // Navigation Links
    Row {
        gap: 32

        NavLink { text: "Home" active: true }
        NavLink { text: "Features" }
        NavLink { text: "Pricing" }
        NavLink { text: "Docs" }
    }

    // Call to Action
    Button {
        label: "Get Started"
        variant: "primary"
    }
}

// NavLink component
component NavLink {
    prop text: String
    prop active: bool = false
    prop href: String = ""

    Container {
        cursor: pointer

        on click => navigate(href)

        Text {
            content: text
            size: 14
            color: active ? "#FFFFFF" : "#94A3B8"
            weight: active ? medium : normal
        }

        // Hover state
        style:hover {
            Text {
                color: "#FFFFFF"
            }
        }
    }
}
```

## Navbar with Dropdown

Add dropdown menus for nested navigation:

```oui
component NavDropdown {
    prop title: String
    prop items: Vec<DropdownItem>

    state {
        open: bool = false
    }

    Container {
        on hover_start => state.open = true
        on hover_end => state.open = false

        Column {
            // Trigger
            Row {
                gap: 6
                align: center
                cursor: pointer

                Text {
                    content: title
                    size: 14
                    color: "#94A3B8"
                }

                Text {
                    content: "chevron-down"
                    size: 12
                    color: "#94A3B8"
                }
            }

            // Dropdown
            @if state.open {
                Container {
                    position: absolute
                    top: 100%
                    left: 0
                    margin_top: 8
                    padding: 8
                    min_width: 200
                    background: "#1E293B"
                    radius: 8
                    border: 1
                    border_color: "#334155"
                    shadow: tokens.shadow.lg

                    Column {
                        gap: 4

                        @for item in items {
                            Container {
                                padding: 12
                                radius: 6
                                cursor: pointer

                                style:hover {
                                    background: "#334155"
                                }

                                on click => navigate(item.href)

                                Column {
                                    gap: 4

                                    Text {
                                        content: item.label
                                        size: 14
                                        color: "#FFFFFF"
                                    }

                                    @if item.description != "" {
                                        Text {
                                            content: item.description
                                            size: 12
                                            color: "#64748B"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

## Responsive Navbar

Handle mobile and desktop layouts:

```oui
app MyApp {
    state {
        mobile_menu_open: bool = false
    }

    Column {
        width: fill
        height: fill

        // Navbar
        Row {
            width: fill
            height: 64
            padding_x: 24
            align: center
            justify: space_between
            background: "#0F172A"

            // Logo
            Row {
                gap: 12
                align: center

                Container {
                    width: 32
                    height: 32
                    background: "#3B82F6"
                    radius: 8

                    Text { content: "O" size: 18 color: "#FFFFFF" align: center }
                }

                Text { content: "OxideKit" size: 18 color: "#FFFFFF" weight: bold }
            }

            // Desktop Navigation (hidden on mobile)
            @media (min-width: 768px) {
                Row {
                    gap: 32

                    NavLink { text: "Home" active: true }
                    NavLink { text: "Features" }
                    NavLink { text: "Pricing" }
                    NavLink { text: "Docs" }
                }

                Button { label: "Get Started" variant: "primary" }
            }

            // Mobile Menu Button (hidden on desktop)
            @media (max-width: 767px) {
                Container {
                    padding: 8
                    cursor: pointer

                    on click => state.mobile_menu_open = !state.mobile_menu_open

                    Text {
                        content: state.mobile_menu_open ? "x" : "menu"
                        size: 24
                        color: "#FFFFFF"
                    }
                }
            }
        }

        // Mobile Menu
        @if state.mobile_menu_open {
            @media (max-width: 767px) {
                Container {
                    width: fill
                    padding: 24
                    background: "#1E293B"
                    border_bottom: 1
                    border_color: "#334155"

                    Column {
                        gap: 16

                        NavLink { text: "Home" active: true }
                        NavLink { text: "Features" }
                        NavLink { text: "Pricing" }
                        NavLink { text: "Docs" }

                        Button {
                            label: "Get Started"
                            variant: "primary"
                            width: fill
                        }
                    }
                }
            }
        }

        // Main content
        Container { flex: 1 }
    }
}
```

## Sticky Navbar

Keep the navbar visible while scrolling:

```oui
Container {
    position: sticky
    top: 0
    z_index: 100

    Row {
        width: fill
        height: 64
        padding_x: 24
        align: center
        justify: space_between
        background: "rgba(15, 23, 42, 0.95)"

        style {
            backdrop_filter: "blur(10px)"
            border_bottom: 1
            border_color: "#1E293B"
        }

        // ... navbar content
    }
}
```

## Navbar with Search

Include a search input:

```oui
Row {
    width: fill
    height: 64
    padding_x: 24
    align: center
    justify: space_between
    background: "#0F172A"

    // Logo
    Row {
        gap: 12
        align: center

        Text { content: "Logo" size: 18 color: "#FFFFFF" weight: bold }
    }

    // Search
    Container {
        width: 300
        height: 40
        padding_x: 16
        background: "#1E293B"
        radius: 8

        Row {
            gap: 12
            align: center

            Text {
                content: "search"
                size: 16
                color: "#64748B"
            }

            Input {
                placeholder: "Search..."
                flex: 1

                style {
                    background: transparent
                    border: none
                    color: "#FFFFFF"
                }
            }

            // Keyboard shortcut hint
            Container {
                padding_x: 6
                padding_y: 2
                background: "#334155"
                radius: 4

                Text { content: "/" size: 11 color: "#64748B" }
            }
        }
    }

    // Actions
    Row {
        gap: 16
        align: center

        Container {
            padding: 8
            cursor: pointer

            Text { content: "bell" size: 20 color: "#94A3B8" }
        }

        // User avatar
        Container {
            width: 36
            height: 36
            background: "#3B82F6"
            radius: 18
            cursor: pointer

            Text { content: "JD" size: 14 color: "#FFFFFF" align: center }
        }
    }
}
```

## Complete Navbar Component

A reusable navbar component:

```oui
component Navbar {
    prop logo: String = "Logo"
    prop links: Vec<NavLinkItem>
    prop cta_label: String = "Get Started"
    prop cta_href: String = "/"
    prop theme: String = "dark"

    let colors = {
        bg: theme == "dark" ? "#0F172A" : "#FFFFFF",
        text: theme == "dark" ? "#FFFFFF" : "#0F172A",
        text_muted: theme == "dark" ? "#94A3B8" : "#64748B",
        border: theme == "dark" ? "#1E293B" : "#E2E8F0"
    }

    Row {
        width: fill
        height: 64
        padding_x: 24
        align: center
        justify: space_between
        background: colors.bg

        style {
            border_bottom: 1
            border_color: colors.border
        }

        // Logo
        Container {
            cursor: pointer
            on click => navigate("/")

            Text {
                content: logo
                size: 20
                color: colors.text
                weight: bold
            }
        }

        // Links
        Row {
            gap: 32

            @for link in links {
                Container {
                    cursor: pointer
                    on click => navigate(link.href)

                    Text {
                        content: link.label
                        size: 14
                        color: link.active ? colors.text : colors.text_muted
                        weight: link.active ? medium : normal
                    }
                }
            }
        }

        // CTA
        Button {
            label: cta_label
            variant: "primary"
            on_click: navigate(cta_href)
        }
    }
}

// Usage
Navbar {
    logo: "MyApp"
    links: [
        { label: "Home", href: "/", active: true },
        { label: "Features", href: "/features" },
        { label: "Pricing", href: "/pricing" },
        { label: "Docs", href: "/docs" }
    ]
    cta_label: "Sign Up"
    cta_href: "/signup"
}
```

# Landing Page Example

A complete SaaS landing page with hero section, features, pricing, and footer.

## Complete Landing Page

```oui
app LandingPage {
    state {
        is_annual: bool = true
    }

    Column {
        width: fill
        background: "#030712"

        Scroll {
            width: fill

            Column {
                width: fill

                // Navigation
                Navbar {}

                // Hero Section
                HeroSection {}

                // Features Section
                FeaturesSection {}

                // Pricing Section
                PricingSection {
                    is_annual: state.is_annual
                    on_toggle: state.is_annual = value
                }

                // CTA Section
                CTASection {}

                // Footer
                Footer {}
            }
        }
    }
}
```

## Hero Section

The main headline and call-to-action:

```oui
component HeroSection {
    Container {
        width: fill
        padding: 80
        padding_top: 120

        Column {
            align: center
            gap: 32
            max_width: 900
            align_self: center

            // Badge
            Container {
                padding_x: 16
                padding_y: 8
                background: "#1E293B"
                radius: 20
                border: 1
                border_color: "#334155"

                Row {
                    gap: 8
                    align: center

                    Container {
                        padding_x: 8
                        padding_y: 4
                        background: "#10B981"
                        radius: 4

                        Text {
                            content: "New"
                            size: 10
                            color: "#FFFFFF"
                            weight: bold
                        }
                    }

                    Text {
                        content: "Introducing AI-powered features"
                        size: 12
                        color: "#94A3B8"
                    }
                }
            }

            // Headline
            Column {
                gap: 16
                align: center

                Text {
                    content: "Build Amazing Products"
                    size: 64
                    color: "#FFFFFF"
                    weight: bold
                    align: center
                }

                Text {
                    content: "Faster Than Ever"
                    size: 64
                    color: "#3B82F6"
                    weight: bold
                    align: center
                }
            }

            // Subheadline
            Text {
                content: "The all-in-one platform for teams to build, ship, and scale their products. Trusted by over 10,000 companies worldwide."
                size: 20
                color: "#94A3B8"
                align: center
                max_width: 700
            }

            // CTA Buttons
            Row {
                gap: 16
                margin_top: 16

                Button {
                    label: "Start Free Trial"
                    variant: "primary"
                    size: "lg"
                }

                Button {
                    label: "Watch Demo"
                    variant: "outline"
                    size: "lg"
                    icon: "play"
                }
            }

            // Social Proof
            Column {
                gap: 16
                align: center
                margin_top: 48

                Text {
                    content: "Trusted by leading companies"
                    size: 14
                    color: "#64748B"
                }

                Row {
                    gap: 48

                    @for company in ["Google", "Microsoft", "Amazon", "Netflix", "Spotify"] {
                        Text {
                            content: company
                            size: 14
                            color: "#475569"
                        }
                    }
                }
            }
        }
    }
}
```

## Features Section

Highlight key features with cards:

```oui
component FeaturesSection {
    Container {
        width: fill
        padding: 80
        background: "#0F172A"

        Column {
            gap: 64
            max_width: 1200
            align_self: center
            width: fill

            // Section Header
            Column {
                gap: 16
                align: center

                Text {
                    content: "Everything you need"
                    size: 14
                    color: "#3B82F6"
                    weight: medium
                }

                Text {
                    content: "Powerful Features"
                    size: 40
                    color: "#FFFFFF"
                    weight: bold
                }

                Text {
                    content: "Tools designed to help you build better products, faster."
                    size: 18
                    color: "#64748B"
                }
            }

            // Feature Grid
            Row {
                gap: 24
                wrap: true
                justify: center

                FeatureCard {
                    icon: "zap"
                    color: "#F59E0B"
                    title: "Lightning Fast"
                    description: "Built for speed. Our platform is optimized for performance at every level."
                }

                FeatureCard {
                    icon: "shield"
                    color: "#10B981"
                    title: "Secure by Default"
                    description: "Enterprise-grade security with end-to-end encryption."
                }

                FeatureCard {
                    icon: "code"
                    color: "#8B5CF6"
                    title: "Developer First"
                    description: "Comprehensive APIs and SDKs to integrate with your tools."
                }

                FeatureCard {
                    icon: "chart"
                    color: "#EC4899"
                    title: "Analytics"
                    description: "Real-time insights to track your growth and performance."
                }

                FeatureCard {
                    icon: "users"
                    color: "#06B6D4"
                    title: "Team Collaboration"
                    description: "Work together with real-time collaboration features."
                }

                FeatureCard {
                    icon: "globe"
                    color: "#3B82F6"
                    title: "Global Scale"
                    description: "Deploy worldwide with our global edge network."
                }
            }
        }
    }
}

component FeatureCard {
    prop icon: String
    prop color: String
    prop title: String
    prop description: String

    Container {
        width: 350
        padding: 32
        background: "#1E293B"
        radius: 16

        Column {
            gap: 16

            // Icon
            Container {
                width: 48
                height: 48
                radius: 12

                style {
                    background: "rgba({color}, 0.2)"
                }

                Column {
                    align: center
                    justify: center
                    width: fill
                    height: fill

                    Text {
                        content: icon
                        size: 24
                        color: color
                    }
                }
            }

            Text {
                content: title
                size: 20
                color: "#FFFFFF"
                weight: semibold
            }

            Text {
                content: description
                size: 14
                color: "#94A3B8"
                line_height: 1.6
            }
        }
    }
}
```

## Pricing Section

Pricing cards with billing toggle:

```oui
component PricingSection {
    prop is_annual: bool
    prop on_toggle: Callback

    Container {
        width: fill
        padding: 80

        Column {
            gap: 48
            max_width: 1200
            align_self: center

            // Header
            Column {
                gap: 16
                align: center

                Text {
                    content: "Simple, transparent pricing"
                    size: 14
                    color: "#3B82F6"
                }

                Text {
                    content: "Choose Your Plan"
                    size: 40
                    color: "#FFFFFF"
                    weight: bold
                }

                // Billing Toggle
                Container {
                    padding: 4
                    background: "#1E293B"
                    radius: 12
                    margin_top: 16

                    Row {
                        Container {
                            padding_x: 16
                            padding_y: 10
                            radius: 8
                            background: !is_annual ? "#3B82F6" : "transparent"
                            cursor: pointer

                            on click => on_toggle(false)

                            Text {
                                content: "Monthly"
                                size: 14
                                color: "#FFFFFF"
                            }
                        }

                        Container {
                            padding_x: 16
                            padding_y: 10
                            radius: 8
                            background: is_annual ? "#3B82F6" : "transparent"
                            cursor: pointer

                            on click => on_toggle(true)

                            Row {
                                gap: 8

                                Text {
                                    content: "Annual"
                                    size: 14
                                    color: "#FFFFFF"
                                }

                                Container {
                                    padding_x: 6
                                    padding_y: 2
                                    background: "#10B981"
                                    radius: 4

                                    Text {
                                        content: "Save 20%"
                                        size: 10
                                        color: "#FFFFFF"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Pricing Cards
            Row {
                gap: 24
                justify: center

                PricingCard {
                    name: "Starter"
                    description: "For individuals"
                    price: "0"
                    period: "Free forever"
                    features: ["Up to 3 projects", "Basic analytics", "Community support"]
                    cta: "Get Started"
                }

                PricingCard {
                    name: "Pro"
                    description: "For growing teams"
                    price: is_annual ? "29" : "39"
                    period: is_annual ? "/month, billed annually" : "/month"
                    features: ["Unlimited projects", "Advanced analytics", "Priority support", "Team collaboration"]
                    cta: "Start Free Trial"
                    popular: true
                }

                PricingCard {
                    name: "Enterprise"
                    description: "For large orgs"
                    price: is_annual ? "99" : "129"
                    period: is_annual ? "/month, billed annually" : "/month"
                    features: ["Everything in Pro", "Unlimited storage", "Dedicated support", "Custom contracts"]
                    cta: "Contact Sales"
                }
            }
        }
    }
}

component PricingCard {
    prop name: String
    prop description: String
    prop price: String
    prop period: String
    prop features: Vec<String>
    prop cta: String
    prop popular: bool = false

    Container {
        width: 320
        padding: 32
        background: popular ? "#1E293B" : "#0F172A"
        radius: 16
        border: popular ? 2 : 1
        border_color: popular ? "#3B82F6" : "#1E293B"

        Column {
            gap: 24

            // Header
            Column {
                gap: 8

                @if popular {
                    Container {
                        padding_x: 8
                        padding_y: 4
                        background: "#3B82F6"
                        radius: 4
                        align_self: start

                        Text {
                            content: "Most Popular"
                            size: 12
                            color: "#FFFFFF"
                        }
                    }
                }

                Text {
                    content: name
                    size: 24
                    color: "#FFFFFF"
                    weight: bold
                }

                Text {
                    content: description
                    size: 14
                    color: "#64748B"
                }
            }

            // Price
            Row {
                align: baseline
                gap: 4

                Text {
                    content: "$"
                    size: 24
                    color: "#FFFFFF"
                }

                Text {
                    content: price
                    size: 48
                    color: "#FFFFFF"
                    weight: bold
                }

                Text {
                    content: period
                    size: 14
                    color: "#64748B"
                }
            }

            // CTA Button
            Button {
                label: cta
                variant: popular ? "primary" : "outline"
                width: fill
            }

            // Features
            Column {
                gap: 12

                @for feature in features {
                    Row {
                        gap: 12
                        align: center

                        Container {
                            width: 20
                            height: 20
                            radius: 10
                            background: "rgba(16, 185, 129, 0.2)"

                            Text {
                                content: "check"
                                size: 12
                                color: "#10B981"
                                align: center
                            }
                        }

                        Text {
                            content: feature
                            size: 14
                            color: "#94A3B8"
                        }
                    }
                }
            }
        }
    }
}
```

## CTA Section

Final call-to-action:

```oui
component CTASection {
    Container {
        width: fill
        padding: 80
        background: "#0F172A"

        Container {
            max_width: 800
            align_self: center
            padding: 64
            background: "#1E293B"
            radius: 24
            border: 1
            border_color: "#334155"

            Column {
                gap: 24
                align: center

                Text {
                    content: "Ready to get started?"
                    size: 36
                    color: "#FFFFFF"
                    weight: bold
                }

                Text {
                    content: "Join thousands of teams already using our platform."
                    size: 18
                    color: "#94A3B8"
                }

                Row {
                    gap: 16
                    margin_top: 16

                    Button {
                        label: "Start Free Trial"
                        variant: "primary"
                        size: "lg"
                    }

                    Button {
                        label: "Schedule Demo"
                        variant: "outline"
                        size: "lg"
                    }
                }
            }
        }
    }
}
```

## Footer

Site footer with links:

```oui
component Footer {
    Container {
        width: fill
        padding: 64
        background: "#030712"
        border_top: 1
        border_color: "#1E293B"

        Column {
            gap: 48
            max_width: 1200
            align_self: center
            width: fill

            Row {
                gap: 64

                // Company Info
                Column {
                    gap: 24
                    flex: 2

                    Row {
                        gap: 12
                        align: center

                        Container {
                            width: 40
                            height: 40
                            background: "#3B82F6"
                            radius: 10

                            Text {
                                content: "A"
                                size: 20
                                color: "#FFFFFF"
                                align: center
                            }
                        }

                        Text {
                            content: "Acme"
                            size: 24
                            color: "#FFFFFF"
                            weight: bold
                        }
                    }

                    Text {
                        content: "Building the future of product development."
                        size: 14
                        color: "#64748B"
                        max_width: 300
                    }

                    Row {
                        gap: 16

                        @for social in ["twitter", "github", "linkedin", "discord"] {
                            Container {
                                width: 40
                                height: 40
                                background: "#1E293B"
                                radius: 20
                                cursor: pointer

                                Text {
                                    content: social
                                    size: 12
                                    color: "#64748B"
                                    align: center
                                }
                            }
                        }
                    }
                }

                // Link columns
                FooterLinks {
                    title: "Product"
                    links: ["Features", "Pricing", "Integrations", "Changelog"]
                }

                FooterLinks {
                    title: "Company"
                    links: ["About", "Blog", "Careers", "Press"]
                }

                FooterLinks {
                    title: "Resources"
                    links: ["Documentation", "API Reference", "Community", "Support"]
                }

                FooterLinks {
                    title: "Legal"
                    links: ["Privacy", "Terms", "Security"]
                }
            }

            // Bottom bar
            Container {
                padding_top: 24
                border_top: 1
                border_color: "#1E293B"
                width: fill

                Row {
                    justify: space_between

                    Text {
                        content: "2024 Acme Inc. All rights reserved."
                        size: 14
                        color: "#64748B"
                    }

                    Row {
                        gap: 24

                        Text { content: "Privacy Policy" size: 14 color: "#64748B" }
                        Text { content: "Terms of Service" size: 14 color: "#64748B" }
                    }
                }
            }
        }
    }
}

component FooterLinks {
    prop title: String
    prop links: Vec<String>

    Column {
        gap: 16
        flex: 1

        Text {
            content: title
            size: 14
            color: "#FFFFFF"
            weight: medium
        }

        Column {
            gap: 12

            @for link in links {
                Text {
                    content: link
                    size: 14
                    color: "#64748B"
                    cursor: pointer
                }
            }
        }
    }
}
```

## Creating a Normal Website

To create a full website with OxideKit:

1. **Create project structure**:
```
my-website/
  oxide.toml
  ui/
    app.oui
    pages/
      home.oui
      about.oui
      contact.oui
    components/
      navbar.oui
      footer.oui
```

2. **Configure routing** in `app.oui`:
```oui
app MyWebsite {
    Router {
        Route { path: "/" page: "./pages/home.oui" }
        Route { path: "/about" page: "./pages/about.oui" }
        Route { path: "/contact" page: "./pages/contact.oui" }
    }
}
```

3. **Build for static HTML**:
```bash
oxide build --target static --release
```

4. **Deploy** to any static hosting (Vercel, Netlify, GitHub Pages).

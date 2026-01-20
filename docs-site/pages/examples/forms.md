# Forms Example

Complete form examples with validation, different input types, and common patterns.

## Basic Form

A simple contact form:

```oui
app ContactForm {
    state {
        name: String = ""
        email: String = ""
        message: String = ""
        submitting: bool = false
    }

    Container {
        padding: 32
        max_width: 500
        background: "#1E293B"
        radius: 16

        Column {
            gap: 24

            Text {
                content: "Contact Us"
                size: 24
                color: "#FFFFFF"
                weight: bold
            }

            // Name input
            Column {
                gap: 8

                Text {
                    content: "Name"
                    size: 14
                    color: "#FFFFFF"
                }

                Input {
                    placeholder: "Your name"
                    value: state.name
                    on_change: state.name = value
                }
            }

            // Email input
            Column {
                gap: 8

                Text {
                    content: "Email"
                    size: 14
                    color: "#FFFFFF"
                }

                Input {
                    type: "email"
                    placeholder: "you@example.com"
                    value: state.email
                    on_change: state.email = value
                }
            }

            // Message textarea
            Column {
                gap: 8

                Text {
                    content: "Message"
                    size: 14
                    color: "#FFFFFF"
                }

                Textarea {
                    placeholder: "Your message..."
                    value: state.message
                    on_change: state.message = value
                    rows: 4
                }
            }

            // Submit button
            Button {
                label: state.submitting ? "Sending..." : "Send Message"
                variant: "primary"
                width: fill
                loading: state.submitting
                disabled: state.name == "" || state.email == "" || state.message == ""
                on_click: submit_form
            }
        }
    }

    fn submit_form() {
        state.submitting = true
        // API call here
        state.submitting = false
    }
}
```

## Form with Validation

Real-time validation with error messages:

```oui
app SignupForm {
    state {
        email: String = ""
        password: String = ""
        confirm_password: String = ""

        email_error: String = ""
        password_error: String = ""
        confirm_error: String = ""

        touched: {
            email: bool = false,
            password: bool = false,
            confirm: bool = false
        }
    }

    Container {
        padding: 32
        max_width: 400
        background: "#1E293B"
        radius: 16

        Column {
            gap: 24

            Text {
                content: "Create Account"
                size: 24
                color: "#FFFFFF"
                weight: bold
            }

            // Email field
            FormField {
                label: "Email"
                error: state.touched.email ? state.email_error : ""
                required: true

                Input {
                    type: "email"
                    placeholder: "you@example.com"
                    value: state.email
                    error: state.touched.email && state.email_error != ""
                    on_change: {
                        state.email = value
                        validate_email()
                    }
                    on_blur: state.touched.email = true
                }
            }

            // Password field
            FormField {
                label: "Password"
                error: state.touched.password ? state.password_error : ""
                required: true
                helper: "At least 8 characters with one number"

                Input {
                    type: "password"
                    placeholder: "Enter password"
                    value: state.password
                    error: state.touched.password && state.password_error != ""
                    on_change: {
                        state.password = value
                        validate_password()
                    }
                    on_blur: state.touched.password = true
                }
            }

            // Confirm password field
            FormField {
                label: "Confirm Password"
                error: state.touched.confirm ? state.confirm_error : ""
                required: true

                Input {
                    type: "password"
                    placeholder: "Confirm password"
                    value: state.confirm_password
                    error: state.touched.confirm && state.confirm_error != ""
                    on_change: {
                        state.confirm_password = value
                        validate_confirm()
                    }
                    on_blur: state.touched.confirm = true
                }
            }

            // Password strength indicator
            @if state.password != "" {
                PasswordStrength { password: state.password }
            }

            // Submit
            Button {
                label: "Create Account"
                variant: "primary"
                width: fill
                disabled: !is_form_valid()
            }
        }
    }

    fn validate_email() {
        if state.email == "" {
            state.email_error = "Email is required"
        } else if !is_valid_email(state.email) {
            state.email_error = "Please enter a valid email"
        } else {
            state.email_error = ""
        }
    }

    fn validate_password() {
        if state.password == "" {
            state.password_error = "Password is required"
        } else if state.password.len() < 8 {
            state.password_error = "Password must be at least 8 characters"
        } else if !contains_number(state.password) {
            state.password_error = "Password must contain a number"
        } else {
            state.password_error = ""
        }
    }

    fn validate_confirm() {
        if state.confirm_password != state.password {
            state.confirm_error = "Passwords do not match"
        } else {
            state.confirm_error = ""
        }
    }

    fn is_form_valid() -> bool {
        state.email_error == "" &&
        state.password_error == "" &&
        state.confirm_error == "" &&
        state.email != "" &&
        state.password != "" &&
        state.confirm_password != ""
    }
}

// Reusable FormField component
component FormField {
    prop label: String
    prop error: String = ""
    prop helper: String = ""
    prop required: bool = false

    Column {
        gap: 8
        width: fill

        Row {
            gap: 4

            Text {
                content: label
                size: 14
                color: "#FFFFFF"
            }

            @if required {
                Text {
                    content: "*"
                    size: 14
                    color: "#EF4444"
                }
            }
        }

        @slot default

        @if error != "" {
            Text {
                content: error
                size: 12
                color: "#EF4444"
            }
        } @else if helper != "" {
            Text {
                content: helper
                size: 12
                color: "#64748B"
            }
        }
    }
}

// Password strength indicator
component PasswordStrength {
    prop password: String

    let strength = calculate_strength(password)
    let color = strength < 2 ? "#EF4444" : strength < 4 ? "#F59E0B" : "#22C55E"
    let label = strength < 2 ? "Weak" : strength < 4 ? "Medium" : "Strong"

    Column {
        gap: 8

        Row {
            gap: 4

            @for i in 0..5 {
                Container {
                    flex: 1
                    height: 4
                    radius: 2
                    background: i < strength ? color : "#334155"
                }
            }
        }

        Text {
            content: label
            size: 12
            color: color
        }
    }

    fn calculate_strength(password: String) -> i32 {
        let score = 0
        if password.len() >= 8 { score += 1 }
        if password.len() >= 12 { score += 1 }
        if contains_uppercase(password) { score += 1 }
        if contains_number(password) { score += 1 }
        if contains_special(password) { score += 1 }
        score
    }
}
```

## Select and Checkbox Form

Form with various input types:

```oui
app SettingsForm {
    state {
        country: String = "us"
        language: String = "en"
        notifications_email: bool = true
        notifications_push: bool = false
        theme: String = "dark"
        privacy: String = "friends"
    }

    Container {
        padding: 32
        max_width: 600
        background: "#1E293B"
        radius: 16

        Column {
            gap: 32

            Text {
                content: "Settings"
                size: 24
                color: "#FFFFFF"
                weight: bold
            }

            // Location
            Section {
                title: "Location"

                Row {
                    gap: 16

                    Column {
                        flex: 1
                        gap: 8

                        Text { content: "Country" size: 14 color: "#FFFFFF" }

                        Select {
                            value: state.country
                            on_change: state.country = value

                            Option { value: "us" label: "United States" }
                            Option { value: "uk" label: "United Kingdom" }
                            Option { value: "ca" label: "Canada" }
                            Option { value: "de" label: "Germany" }
                        }
                    }

                    Column {
                        flex: 1
                        gap: 8

                        Text { content: "Language" size: 14 color: "#FFFFFF" }

                        Select {
                            value: state.language
                            on_change: state.language = value

                            Option { value: "en" label: "English" }
                            Option { value: "es" label: "Spanish" }
                            Option { value: "fr" label: "French" }
                            Option { value: "de" label: "German" }
                        }
                    }
                }
            }

            // Notifications
            Section {
                title: "Notifications"

                Column {
                    gap: 16

                    Row {
                        justify: space_between
                        align: center

                        Column {
                            gap: 4

                            Text { content: "Email notifications" size: 14 color: "#FFFFFF" }
                            Text { content: "Receive updates via email" size: 12 color: "#64748B" }
                        }

                        Switch {
                            checked: state.notifications_email
                            on_change: state.notifications_email = value
                        }
                    }

                    Row {
                        justify: space_between
                        align: center

                        Column {
                            gap: 4

                            Text { content: "Push notifications" size: 14 color: "#FFFFFF" }
                            Text { content: "Receive push notifications" size: 12 color: "#64748B" }
                        }

                        Switch {
                            checked: state.notifications_push
                            on_change: state.notifications_push = value
                        }
                    }
                }
            }

            // Appearance
            Section {
                title: "Appearance"

                Column {
                    gap: 8

                    Text { content: "Theme" size: 14 color: "#FFFFFF" }

                    RadioGroup {
                        value: state.theme
                        on_change: state.theme = value

                        Row {
                            gap: 24

                            Radio { value: "light" label: "Light" }
                            Radio { value: "dark" label: "Dark" }
                            Radio { value: "system" label: "System" }
                        }
                    }
                }
            }

            // Privacy
            Section {
                title: "Privacy"

                Column {
                    gap: 8

                    Text { content: "Who can see your profile?" size: 14 color: "#FFFFFF" }

                    RadioGroup {
                        value: state.privacy
                        on_change: state.privacy = value

                        Column {
                            gap: 12

                            Radio { value: "public" label: "Everyone" }
                            Radio { value: "friends" label: "Friends only" }
                            Radio { value: "private" label: "Only me" }
                        }
                    }
                }
            }

            // Actions
            Row {
                gap: 12
                justify: end

                Button { label: "Cancel" variant: "ghost" }
                Button { label: "Save Changes" variant: "primary" }
            }
        }
    }
}

component Section {
    prop title: String

    Column {
        gap: 16

        Text {
            content: title
            size: 16
            color: "#FFFFFF"
            weight: medium
        }

        Container {
            padding: 16
            background: "#0F172A"
            radius: 8

            @slot default
        }
    }
}
```

## Multi-Step Form

Form wizard with steps:

```oui
app OnboardingForm {
    state {
        step: i32 = 1
        total_steps: i32 = 3

        // Step 1
        name: String = ""
        email: String = ""

        // Step 2
        company: String = ""
        role: String = ""

        // Step 3
        plan: String = "pro"
    }

    Container {
        padding: 32
        max_width: 500
        background: "#1E293B"
        radius: 16

        Column {
            gap: 32

            // Progress indicator
            Column {
                gap: 16

                Row {
                    justify: space_between

                    Text {
                        content: "Step {state.step} of {state.total_steps}"
                        size: 14
                        color: "#94A3B8"
                    }

                    Text {
                        content: "{(state.step / state.total_steps * 100) as i32}%"
                        size: 14
                        color: "#3B82F6"
                    }
                }

                // Progress bar
                Container {
                    width: fill
                    height: 4
                    background: "#334155"
                    radius: 2

                    Container {
                        width: "{state.step / state.total_steps * 100}%"
                        height: fill
                        background: "#3B82F6"
                        radius: 2
                    }
                }
            }

            // Step content
            @match state.step {
                1 => {
                    Column {
                        gap: 24

                        Text {
                            content: "Tell us about yourself"
                            size: 20
                            color: "#FFFFFF"
                            weight: bold
                        }

                        FormField {
                            label: "Full Name"

                            Input {
                                placeholder: "John Doe"
                                value: state.name
                                on_change: state.name = value
                            }
                        }

                        FormField {
                            label: "Email"

                            Input {
                                type: "email"
                                placeholder: "john@example.com"
                                value: state.email
                                on_change: state.email = value
                            }
                        }
                    }
                }

                2 => {
                    Column {
                        gap: 24

                        Text {
                            content: "About your company"
                            size: 20
                            color: "#FFFFFF"
                            weight: bold
                        }

                        FormField {
                            label: "Company Name"

                            Input {
                                placeholder: "Acme Inc."
                                value: state.company
                                on_change: state.company = value
                            }
                        }

                        FormField {
                            label: "Your Role"

                            Select {
                                value: state.role
                                on_change: state.role = value
                                placeholder: "Select role"

                                Option { value: "developer" label: "Developer" }
                                Option { value: "designer" label: "Designer" }
                                Option { value: "manager" label: "Manager" }
                                Option { value: "other" label: "Other" }
                            }
                        }
                    }
                }

                3 => {
                    Column {
                        gap: 24

                        Text {
                            content: "Choose your plan"
                            size: 20
                            color: "#FFFFFF"
                            weight: bold
                        }

                        Column {
                            gap: 12

                            PlanOption {
                                name: "starter"
                                title: "Starter"
                                price: "Free"
                                features: ["5 projects", "Basic support"]
                                selected: state.plan == "starter"
                                on_select: state.plan = "starter"
                            }

                            PlanOption {
                                name: "pro"
                                title: "Pro"
                                price: "$29/mo"
                                features: ["Unlimited projects", "Priority support", "Analytics"]
                                selected: state.plan == "pro"
                                on_select: state.plan = "pro"
                            }

                            PlanOption {
                                name: "enterprise"
                                title: "Enterprise"
                                price: "Custom"
                                features: ["Everything in Pro", "Dedicated support", "SLA"]
                                selected: state.plan == "enterprise"
                                on_select: state.plan = "enterprise"
                            }
                        }
                    }
                }
            }

            // Navigation buttons
            Row {
                justify: space_between

                @if state.step > 1 {
                    Button {
                        label: "Back"
                        variant: "ghost"
                        on_click: state.step -= 1
                    }
                } @else {
                    Container {}  // Spacer
                }

                @if state.step < state.total_steps {
                    Button {
                        label: "Continue"
                        variant: "primary"
                        on_click: state.step += 1
                    }
                } @else {
                    Button {
                        label: "Complete Setup"
                        variant: "primary"
                        on_click: complete_setup
                    }
                }
            }
        }
    }
}

component PlanOption {
    prop name: String
    prop title: String
    prop price: String
    prop features: Vec<String>
    prop selected: bool = false
    prop on_select: Callback

    Container {
        padding: 16
        radius: 8
        cursor: pointer
        border: 2
        border_color: selected ? "#3B82F6" : "#334155"
        background: selected ? "rgba(59, 130, 246, 0.1)" : "transparent"

        on click => on_select

        Row {
            justify: space_between
            align: start

            Column {
                gap: 8

                Row {
                    gap: 12
                    align: center

                    Text {
                        content: title
                        size: 16
                        color: "#FFFFFF"
                        weight: medium
                    }

                    Text {
                        content: price
                        size: 14
                        color: "#3B82F6"
                    }
                }

                Column {
                    gap: 4

                    @for feature in features {
                        Text {
                            content: "- {feature}"
                            size: 12
                            color: "#94A3B8"
                        }
                    }
                }
            }

            // Radio indicator
            Container {
                width: 20
                height: 20
                radius: 10
                border: 2
                border_color: selected ? "#3B82F6" : "#334155"
                background: selected ? "#3B82F6" : "transparent"

                @if selected {
                    Container {
                        width: 8
                        height: 8
                        radius: 4
                        background: "#FFFFFF"
                        align_self: center
                    }
                }
            }
        }
    }
}
```

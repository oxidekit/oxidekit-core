//! `oxide learn` command - interactive tutorials and learning system
//!
//! Provides guided tutorials for learning OxideKit concepts.

use anyhow::Result;
use std::io::{self, Write};

/// Tutorial categories
#[derive(Debug, Clone, Copy)]
pub enum TutorialCategory {
    CoreConcepts,
    PluginCategories,
    BuildModes,
    AiPhilosophy,
    QuickStart,
}

impl TutorialCategory {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "core" | "concepts" | "core-concepts" | "1" => Some(Self::CoreConcepts),
            "plugins" | "plugin-categories" | "2" => Some(Self::PluginCategories),
            "build" | "build-modes" | "modes" | "3" => Some(Self::BuildModes),
            "ai" | "ai-philosophy" | "4" => Some(Self::AiPhilosophy),
            "quick" | "quickstart" | "quick-start" | "5" => Some(Self::QuickStart),
            _ => None,
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::CoreConcepts => "Core Concepts",
            Self::PluginCategories => "Plugin Categories",
            Self::BuildModes => "Build Modes",
            Self::AiPhilosophy => "AI Philosophy",
            Self::QuickStart => "Quick Start",
        }
    }

    fn description(&self) -> &str {
        match self {
            Self::CoreConcepts => "Tokens -> Components -> Packs -> Design -> Starters",
            Self::PluginCategories => "UI / Service / Native / Tooling plugins",
            Self::BuildModes => "Dev vs Release vs Diagnostics",
            Self::AiPhilosophy => "How AI assists without taking over",
            Self::QuickStart => "Get building in 5 minutes",
        }
    }
}

/// Run the learn command - main entry point
pub fn run(topic: Option<&str>, list: bool) -> Result<()> {
    if list {
        return list_tutorials();
    }

    match topic {
        Some(t) => {
            if let Some(category) = TutorialCategory::from_str(t) {
                run_tutorial(category)
            } else {
                println!();
                println!("  Unknown topic: {}", t);
                println!();
                list_tutorials()
            }
        }
        None => run_interactive_menu(),
    }
}

/// List all available tutorials
fn list_tutorials() -> Result<()> {
    println!();
    println!("  OxideKit Learning Tutorials");
    println!("  ===========================");
    println!();
    println!("  Available tutorials:");
    println!();
    println!(
        "    1. core-concepts     {}",
        TutorialCategory::CoreConcepts.description()
    );
    println!(
        "    2. plugin-categories {}",
        TutorialCategory::PluginCategories.description()
    );
    println!(
        "    3. build-modes       {}",
        TutorialCategory::BuildModes.description()
    );
    println!(
        "    4. ai-philosophy     {}",
        TutorialCategory::AiPhilosophy.description()
    );
    println!(
        "    5. quick-start       {}",
        TutorialCategory::QuickStart.description()
    );
    println!();
    println!("  Usage:");
    println!("    oxide learn                    # Interactive menu");
    println!("    oxide learn core-concepts      # Start specific tutorial");
    println!("    oxide learn --list             # Show this list");
    println!();

    Ok(())
}

/// Run interactive tutorial menu
fn run_interactive_menu() -> Result<()> {
    clear_screen();
    print_banner();

    println!();
    println!("  Welcome to OxideKit Learning!");
    println!();
    println!("  Select a tutorial to get started:");
    println!();
    println!("    [1] Core Concepts      - The OxideKit mental model");
    println!("    [2] Plugin Categories  - Understanding plugins");
    println!("    [3] Build Modes        - Dev, Release, Diagnostics");
    println!("    [4] AI Philosophy      - AI-assisted development");
    println!("    [5] Quick Start        - Build your first app");
    println!();
    println!("    [q] Quit");
    println!();

    print!("  Enter selection: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim() {
        "1" => run_tutorial(TutorialCategory::CoreConcepts),
        "2" => run_tutorial(TutorialCategory::PluginCategories),
        "3" => run_tutorial(TutorialCategory::BuildModes),
        "4" => run_tutorial(TutorialCategory::AiPhilosophy),
        "5" => run_tutorial(TutorialCategory::QuickStart),
        "q" | "Q" => {
            println!();
            println!("  Happy building with OxideKit!");
            println!();
            Ok(())
        }
        _ => {
            println!();
            println!("  Invalid selection. Please try again.");
            run_interactive_menu()
        }
    }
}

/// Run a specific tutorial
fn run_tutorial(category: TutorialCategory) -> Result<()> {
    clear_screen();

    match category {
        TutorialCategory::CoreConcepts => tutorial_core_concepts(),
        TutorialCategory::PluginCategories => tutorial_plugin_categories(),
        TutorialCategory::BuildModes => tutorial_build_modes(),
        TutorialCategory::AiPhilosophy => tutorial_ai_philosophy(),
        TutorialCategory::QuickStart => tutorial_quick_start(),
    }
}

fn tutorial_core_concepts() -> Result<()> {
    print_lesson_header("Core Concepts", 1, 5);

    println!(
        r#"
  THE OXIDEKIT CONCEPTUAL HIERARCHY
  =================================

  OxideKit organizes everything into a clear progression:

      +-------------------+
      |    STARTERS       |  <- Complete project templates
      +-------------------+
               |
      +-------------------+
      |     DESIGN        |  <- Visual systems & layouts
      +-------------------+
               |
      +-------------------+
      |      PACKS        |  <- Grouped functionality
      +-------------------+
               |
      +-------------------+
      |   COMPONENTS      |  <- UI building blocks
      +-------------------+
               |
      +-------------------+
      |     TOKENS        |  <- Design primitives
      +-------------------+

  Each layer builds on the one below it.
  Think of it like LEGO: small pieces combine into larger structures.
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Core Concepts", 2, 5);

    println!(r##"
  TOKENS: THE FOUNDATION
  ======================

  Tokens are named, semantic values for design properties:

      $color.primary      -> "#3B82F6"
      $spacing.md         -> 16
      $radius.button      -> 6
      $shadow.card        -> "0 4px 6px rgba(0,0,0,0.1)"

  WHY TOKENS MATTER:

      // BAD: Magic numbers
      Button {{
          background: "#3B82F6"
          padding: 12
      }}

      // GOOD: Semantic tokens
      Button {{
          background: $color.primary
          padding: $spacing.md
      }}

  When the theme changes, all token references update automatically!
"##);

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Core Concepts", 3, 5);

    println!(
        r#"
  COMPONENTS: THE BUILDING BLOCKS
  ===============================

  Components are reusable UI elements with defined contracts.

  Every component has:
    - Props (configurable inputs)
    - Events (outputs/callbacks)
    - Slots (places for children)

  EXAMPLE:

      Button {{
          variant: "primary"       // Prop: styling variant
          label: "Save"            // Prop: button text
          disabled: $form.invalid  // Prop: reactive binding
          on_click: {{ save() }}     // Event: click handler
      }}

  Components guarantee:
    - Consistent API across all uses
    - Theme integration via tokens
    - Built-in accessibility support
    - Compile-time validation
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Core Concepts", 4, 5);

    println!(
        r#"
  PACKS, DESIGN, AND STARTERS
  ===========================

  PACKS bundle related components together:

      oxide add ui.tables    # DataTable, Column, Cell, etc.
      oxide add ui.charts    # BarChart, LineChart, etc.

  DESIGN provides visual systems:

      - Themes (dark, light, brand)
      - Typography (font scales, roles)
      - Layout patterns

  STARTERS are complete project templates:

      oxide new my-app --starter admin-panel

      Available starters:
        - admin-panel       Dashboard with tables, charts
        - docs-site         Documentation site
        - desktop-wallet    Secure wallet UI
        - website-single    Marketing landing page
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Core Concepts", 5, 5);

    println!(
        r#"
  KEY PRINCIPLES
  ==============

  1. TOKENS ARE SEMANTIC
     Name by meaning, not by value.
     "$color.primary" not "$color.blue"

  2. COMPONENTS ARE CONTRACTS
     Explicit props, events, slots.
     No magic or hidden behavior.

  3. PACKS ARE COHESIVE
     Group related functionality.
     Install packs, not individual components.

  4. DESIGN IS SYSTEMATIC
     Themes, not ad-hoc styling.
     Consistency across the app.

  5. STARTERS ARE FOUNDATIONS
     Production-ready, not demos.
     Real code you can build on.

  For the full guide, see: docs/guides/01-core-concepts.md
"#
    );

    end_tutorial()?;
    Ok(())
}

fn tutorial_plugin_categories() -> Result<()> {
    print_lesson_header("Plugin Categories", 1, 4);

    println!(
        r#"
  THE SIX PLUGIN CATEGORIES
  =========================

      +------------------+-------------------+------------------+
      |       UI         |     SERVICE       |     NATIVE       |
      |  (No Permissions)|  (App-Level)      |  (OS Access)     |
      +------------------+-------------------+------------------+
      |     TOOLING      |      THEME        |     DESIGN       |
      |  (Build-Time)    |  (Token Packs)    |  (Layout Kits)   |
      +------------------+-------------------+------------------+

  Each category has different:
    - Capabilities
    - Trust requirements
    - Permission models
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Plugin Categories", 2, 4);

    println!(
        r#"
  UI & SERVICE PLUGINS
  ====================

  UI PLUGINS (safest):
    - Pure UI components
    - No filesystem access
    - No network access
    - Zero permissions required

    Examples: ui.tables, ui.charts, ui.forms

  SERVICE PLUGINS (app-level):
    - Business logic
    - May require permissions
    - Can manage state

    Examples: service.auth, service.db, service.sync

  Usage:
    oxide add ui.tables
    oxide add service.auth
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Plugin Categories", 3, 4);

    println!(
        r#"
  NATIVE & TOOLING PLUGINS
  ========================

  NATIVE PLUGINS (highest trust):
    - OS API access
    - Explicit user consent
    - Capability-based security

    Examples: native.fs, native.keychain, native.clipboard

    Permission prompt:
    +------------------------------------------------+
    |  "MyApp" wants to access your Documents        |
    |                                                |
    |         [Deny]            [Allow]              |
    +------------------------------------------------+

  TOOLING PLUGINS (dev-only):
    - Build-time tools
    - Never ship to users
    - Full dev system access

    Examples: tool.codegen, tool.lint, tool.figma

    Usage:
      oxide add tool.codegen --dev
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Plugin Categories", 4, 4);

    println!(
        r#"
  TRUST LEVELS
  ============

      OFFICIAL    -> Maintained by OxideKit, full trust
      VERIFIED    -> Identity verified, signed releases
      COMMUNITY   -> Sandboxed by default, warnings shown

  BEST PRACTICES:

    1. Minimize native plugin usage
    2. Audit community plugins before use
    3. Pin versions in production
    4. Separate dev and production dependencies

  Example oxide.toml:

    [extensions]
    ui.tables = "^1.0"        # Production

    [extensions.dev]
    tool.codegen = "^2.0"     # Dev only

  For the full guide, see: docs/guides/02-plugin-categories.md
"#
    );

    end_tutorial()?;
    Ok(())
}

fn tutorial_build_modes() -> Result<()> {
    print_lesson_header("Build Modes", 1, 3);

    println!(
        r#"
  THE THREE BUILD MODES
  =====================

      +----------------+   +----------------+   +----------------+
      |      DEV       |   |    RELEASE     |   |  DIAGNOSTICS   |
      +----------------+   +----------------+   +----------------+
      |                |   |                |   |                |
      |  Hot Reload    |   |  Optimized     |   |  Optimized     |
      |  Dev Tools     |   |  No Dev Tools  |   |  No Dev Tools  |
      |  Full Logging  |   |  Minimal Logs  |   |  Error Reports |
      |                |   |                |   |                |
      +----------------+   +----------------+   +----------------+
           Debug             Production        Production+Support

  Commands:
    oxide dev                              # Dev mode
    oxide build --release                  # Release mode
    oxide build --release --features diagnostics  # Diagnostics
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Build Modes", 2, 3);

    println!(
        r#"
  DEV MODE FEATURES
  =================

  Hot Reload:
    File changed -> Recompile (50-100ms) -> Update UI
    State preserved (form data, scroll position)

  Dev Tools (Ctrl+Shift+D):
    - Component inspector
    - Performance profiler
    - State viewer
    - Network monitor
    - Log viewer

  Layout Overlay (Ctrl+Shift+L):
    Shows flex layout boundaries

  BINARY SIZE COMPARISON:

    Dev Build:     ~45 MB
    Release Build: ~12 MB  (73% smaller)

  Dev tools are completely stripped from release builds.
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Build Modes", 3, 3);

    println!(
        r#"
  DIAGNOSTICS MODE
  ================

  For production apps that need support tooling.

  User can export a diagnostics bundle:
    Help > Export Diagnostics...

  Bundle contains:
    - Error messages and codes
    - Component state at error time
    - Performance metrics
    - Redacted logs (no sensitive data)

  ERROR CODES:

    UI-100  -> Component not found
    UI-200  -> Invalid prop type
    RT-100  -> Out of memory
    EXT-300 -> Permission denied

  PRIVACY:

    Before: "User john@example.com logged in"
    After:  "User [EMAIL] logged in"

    All PII is automatically redacted.

  For the full guide, see: docs/guides/03-build-modes.md
"#
    );

    end_tutorial()?;
    Ok(())
}

fn tutorial_ai_philosophy() -> Result<()> {
    print_lesson_header("AI Philosophy", 1, 3);

    println!(
        r#"
  AI AS ASSISTANT, NOT AUTHOR
  ===========================

  The core principle:

      +----------------------------------+
      |        AI AS ASSISTANT           |
      +----------------------------------+
      |                                  |
      |  AI suggests  ->  Human decides  |
      |  AI validates ->  Human reviews  |
      |  AI generates ->  Human owns     |
      |                                  |
      +----------------------------------+

  AI should amplify developer capabilities,
  not create black boxes developers don't understand.
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("AI Philosophy", 2, 3);

    println!(
        r#"
  THE HALLUCINATION PROBLEM
  =========================

  Traditional UI (human-readable):
    <Button variant="primary" onClick={{save}}>Save</Button>

  AI might hallucinate:
    <Button type="save" primary onSave={{handleSave}}>Save</Button>

    (Wrong! These props don't exist)

  OXIDEKIT SOLUTION:

  Machine-readable specs (oxide.ai.json):
    {{
      "id": "ui.Button",
      "props": [
        {{ "name": "variant", "values": ["primary", "secondary"] }},
        {{ "name": "disabled", "type": "bool" }}
      ],
      "events": [{{ "name": "on_click" }}]
    }}

  AI queries the spec -> generates valid code:
    Button {{
        variant: "primary"
        on_click: {{ save() }}
        label: "Save"
    }}
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("AI Philosophy", 3, 3);

    println!(
        r#"
  VALIDATION & GUIDANCE
  =====================

  All generated code is validated:

    AI-generated code
           |
           v
    +------------------+
    |  Parse & Validate |
    +------------------+
           |
    +------+------+
    |             |
  Valid       Invalid -> Error with fix suggestions

  Example error:
    Error: UI-200 - Unknown prop "type" on "Button"
    Suggestion: Did you mean "variant"?

  AI SHOULD:
    - Query the catalog first
    - Use exact prop names
    - Use token references for styling
    - Validate before presenting

  AI SHOULD NOT:
    - Invent components
    - Hardcode style values
    - Create magic black boxes

  For the full guide, see: docs/guides/04-ai-philosophy.md
"#
    );

    end_tutorial()?;
    Ok(())
}

fn tutorial_quick_start() -> Result<()> {
    print_lesson_header("Quick Start", 1, 3);

    println!(
        r#"
  GETTING STARTED IN 5 MINUTES
  ============================

  1. INSTALL THE CLI:

     cargo install oxide-cli

  2. CREATE A PROJECT:

     oxide new my-app

     Or with a starter:
     oxide new my-dashboard --starter admin-panel

  3. RUN IN DEVELOPMENT:

     cd my-app
     oxide dev

     Your app opens at http://localhost:3000
     Changes hot reload automatically.
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Quick Start", 2, 3);

    println!(
        r#"
  PROJECT STRUCTURE
  =================

  my-app/
    oxide.toml           # Project configuration
    src/
      app.oui            # Root component
      pages/
        home.oui         # Home page
      components/
        Header.oui       # Custom components
    theme.toml           # Theme customization (optional)
    extensions.lock      # Locked dependencies

  KEY FILES:

  oxide.toml:
    [package]
    name = "my-app"
    version = "0.1.0"

    [extensions]
    ui.core = "^1.0"

  ui/app.oui:
    app MyApp {{
        Header {{ title: "My App" }}
        Router {{
            Route {{ path: "/", component: HomePage }}
        }}
    }}
"#
    );

    wait_for_continue()?;
    clear_screen();
    print_lesson_header("Quick Start", 3, 3);

    println!(
        r#"
  COMMON COMMANDS
  ===============

  Development:
    oxide dev                    # Start dev server
    oxide build                  # Dev build

  Production:
    oxide build --release        # Release build
    oxide run --release          # Run release build

  Plugins:
    oxide add ui.tables          # Add plugin
    oxide starters list          # List starters
    oxide doctor                 # Diagnose issues

  Export:
    oxide export theme dark      # Export theme
    oxide export ai-schema       # Export AI catalog

  YOU'RE READY!

  Start building:
    oxide new my-first-app
    cd my-first-app
    oxide dev

  Read the guides in docs/guides/ for deeper learning.
"#
    );

    end_tutorial()?;
    Ok(())
}

// Helper functions

fn clear_screen() {
    // ANSI escape code to clear screen (works on most terminals)
    print!("\x1B[2J\x1B[1;1H");
    let _ = io::stdout().flush();
}

fn print_banner() {
    println!(
        r#"
   ___       _     _      _  ___ _
  / _ \__  _(_) __| | ___| |/ (_) |_
 | | | \ \/ / |/ _` |/ _ \ ' /| | __|
 | |_| |>  <| | (_| |  __/ . \| | |_
  \___//_/\_\_|\__,_|\___|_|\_\_|\__|

          Learning Tutorials
"#
    );
}

fn print_lesson_header(tutorial: &str, page: usize, total: usize) {
    println!();
    println!("  ┌─────────────────────────────────────────────────────────────┐");
    println!(
        "  │  {} Tutorial                            ({}/{})",
        tutorial, page, total
    );
    println!("  └─────────────────────────────────────────────────────────────┘");
}

fn wait_for_continue() -> Result<()> {
    println!();
    println!("  ─────────────────────────────────────────────────────────────");
    print!("  Press Enter to continue...");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(())
}

fn end_tutorial() -> Result<()> {
    println!();
    println!("  ═════════════════════════════════════════════════════════════");
    println!("  Tutorial complete!");
    println!();
    print!("  Return to menu? [Y/n]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "n" | "no" => {
            println!();
            println!("  Happy building with OxideKit!");
            println!();
            Ok(())
        }
        _ => run_interactive_menu(),
    }
}

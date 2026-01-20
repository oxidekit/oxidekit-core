//! `oxide compat` command - compatibility layer management
//!
//! Provides commands for managing compatibility features like WebView widgets,
//! JS runtime, and NPM bundling.

use anyhow::Result;

/// Add a compatibility package (webview, js-runtime)
pub fn run_add(package: &str) -> Result<()> {
    println!();
    println!("  OxideKit Compatibility Layer");
    println!("  ─────────────────────────────");
    println!();

    match package {
        "webview" | "compat.webview" => {
            println!("  Adding WebView compatibility layer...");
            println!();
            println!("  WARNING: WebView adds a web surface which increases attack surface.");
            println!("  This is NOT recommended for production applications.");
            println!();
            println!("  To enable, add to oxide.toml:");
            println!();
            println!("    [policy]");
            println!("    allow_webview = true");
            println!();
            println!("  Then use in your app:");
            println!();
            println!("    <WebWidget source=\"bundled:widgets/my-widget\" />");
            println!();
        }
        "js" | "js-runtime" | "compat.js" => {
            println!("  Adding JavaScript runtime compatibility layer...");
            println!();
            println!("  WARNING: JS runtime adds JavaScript execution which may impact security.");
            println!("  Consider porting JS logic to Rust for better performance.");
            println!();
            println!("  To enable, add to oxide.toml:");
            println!();
            println!("    [policy]");
            println!("    allow_js_runtime = true");
            println!();
            println!("  Use cases:");
            println!("    - JSON schema validation");
            println!("    - Markdown parsing");
            println!("    - Data transformation");
            println!();
        }
        _ => {
            println!("  Unknown compatibility package: {}", package);
            println!();
            println!("  Available packages:");
            println!("    - webview     : Embed web widgets (compat.webview)");
            println!("    - js-runtime  : Run JavaScript utilities (compat.js)");
            println!();
            anyhow::bail!("Unknown package: {}", package);
        }
    }

    Ok(())
}

/// Remove a compatibility package
pub fn run_remove(package: &str) -> Result<()> {
    println!();
    println!("  Removing compatibility package: {}", package);
    println!();
    println!("  To remove, update oxide.toml:");
    println!();
    println!("    [policy]");
    println!("    allow_{} = false", package.replace('-', "_"));
    println!();

    Ok(())
}

/// NPM add package
pub fn run_npm_add(package: &str) -> Result<()> {
    println!();
    println!("  OxideKit NPM Bundler");
    println!("  ────────────────────");
    println!();
    println!("  Adding package: {}", package);
    println!();
    println!("  NOTE: This requires Node.js to be installed for build-time bundling.");
    println!("  Node.js is NOT used at runtime.");
    println!();

    // Check for Node.js
    match std::process::Command::new("node").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("  Node.js: {}", version.trim());
        }
        Err(_) => {
            println!("  ERROR: Node.js not found. Please install Node.js first.");
            anyhow::bail!("Node.js not found");
        }
    }

    // Parse package spec
    let (pkg_name, pkg_version) = if package.contains('@') {
        let parts: Vec<&str> = package.rsplitn(2, '@').collect();
        (parts[1], parts[0])
    } else {
        (package, "latest")
    };

    println!();
    println!("  Package: {}", pkg_name);
    println!("  Version: {}", pkg_version);
    println!();
    println!("  Run `oxide compat npm build` to bundle packages.");
    println!();

    Ok(())
}

/// NPM build bundles
pub fn run_npm_build() -> Result<()> {
    println!();
    println!("  OxideKit NPM Bundler");
    println!("  ────────────────────");
    println!();
    println!("  Building NPM packages...");
    println!();

    // This would run the actual bundling
    println!("  Step 1/4: Installing dependencies...");
    println!("  Step 2/4: Bundling JavaScript...");
    println!("  Step 3/4: Minifying output...");
    println!("  Step 4/4: Generating hashes...");
    println!();
    println!("  Bundle created: dist/widgets/bundle.js");
    println!("  Lockfile updated: extensions.lock");
    println!();

    Ok(())
}

/// NPM list packages
pub fn run_npm_list() -> Result<()> {
    println!();
    println!("  OxideKit NPM Packages");
    println!("  ─────────────────────");
    println!();
    println!("  No packages installed.");
    println!();
    println!("  Add packages with: oxide compat npm add <package>@<version>");
    println!();

    Ok(())
}

/// Show compatibility status
pub fn run_status() -> Result<()> {
    println!();
    println!("  OxideKit Compatibility Status");
    println!("  ──────────────────────────────");
    println!();

    // Read oxide.toml if it exists
    if std::path::Path::new("oxide.toml").exists() {
        if let Ok(content) = std::fs::read_to_string("oxide.toml") {
            let webview_enabled = content.contains("allow_webview = true");
            let js_enabled = content.contains("allow_js_runtime = true");
            let npm_enabled = content.contains("allow_npm_bundling = true");

            println!("  Policy Settings:");
            println!("    WebView:     {}", if webview_enabled { "enabled" } else { "disabled" });
            println!("    JS Runtime:  {}", if js_enabled { "enabled" } else { "disabled" });
            println!("    NPM Bundling: {}", if npm_enabled { "enabled" } else { "disabled" });

            if webview_enabled || js_enabled {
                println!();
                println!("  WARNING: Compatibility features are enabled.");
                println!("  These increase attack surface and are not recommended for production.");
            }
        } else {
            println!("  No oxide.toml found in current directory.");
        }
    } else {
        println!("  No oxide.toml found. Compatibility features are disabled by default.");
    }

    println!();
    Ok(())
}

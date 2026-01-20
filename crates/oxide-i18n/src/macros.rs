//! Translation macros
//!
//! Provides the `t!` macro for convenient translation lookups.

/// Translate a key using the global i18n instance
///
/// # Examples
///
/// Simple translation:
/// ```rust,ignore
/// let text = t!("auth.login.title");
/// ```
///
/// With interpolation:
/// ```rust,ignore
/// let text = t!("welcome.message", name = "Alice");
/// ```
///
/// With pluralization:
/// ```rust,ignore
/// let text = t!("cart.items", count = 5);
/// ```
///
/// With both:
/// ```rust,ignore
/// let text = t!("files.selected", count = 3, folder = "Downloads");
/// ```
#[macro_export]
macro_rules! t {
    // Simple key lookup
    ($key:expr) => {{
        $crate::runtime::global()
            .map(|i18n| i18n.t($key))
            .unwrap_or_else(|| $key.to_string())
    }};

    // With count for pluralization
    ($key:expr, count = $count:expr) => {{
        $crate::runtime::global()
            .map(|i18n| i18n.t_plural($key, $count as i64))
            .unwrap_or_else(|| $key.to_string())
    }};

    // With count and other parameters
    ($key:expr, count = $count:expr, $($name:ident = $value:expr),+ $(,)?) => {{
        let mut params = std::collections::HashMap::new();
        params.insert("count".to_string(), $count.to_string());
        $(
            params.insert(stringify!($name).to_string(), $value.to_string());
        )+

        $crate::runtime::global()
            .map(|i18n| i18n.t_plural_with_params($key, $count as i64, &params))
            .unwrap_or_else(|| $key.to_string())
    }};

    // With other parameters and count at end
    ($key:expr, $($name:ident = $value:expr),+, count = $count:expr $(,)?) => {{
        let mut params = std::collections::HashMap::new();
        $(
            params.insert(stringify!($name).to_string(), $value.to_string());
        )+
        params.insert("count".to_string(), $count.to_string());

        $crate::runtime::global()
            .map(|i18n| i18n.t_plural_with_params($key, $count as i64, &params))
            .unwrap_or_else(|| $key.to_string())
    }};

    // With named parameters (no count)
    ($key:expr, $($name:ident = $value:expr),+ $(,)?) => {{
        let mut params = std::collections::HashMap::new();
        $(
            params.insert(stringify!($name).to_string(), $value.to_string());
        )+

        $crate::runtime::global()
            .map(|i18n| i18n.t_with_params($key, &params))
            .unwrap_or_else(|| $key.to_string())
    }};
}

/// Translate a key with a specific i18n instance
///
/// # Examples
///
/// ```rust,ignore
/// let text = t_with!(i18n, "auth.login.title");
/// let text = t_with!(i18n, "welcome.message", name = "Alice");
/// ```
#[macro_export]
macro_rules! t_with {
    // Simple key lookup
    ($i18n:expr, $key:expr) => {{
        $i18n.t($key)
    }};

    // With count for pluralization
    ($i18n:expr, $key:expr, count = $count:expr) => {{
        $i18n.t_plural($key, $count as i64)
    }};

    // With named parameters
    ($i18n:expr, $key:expr, $($name:ident = $value:expr),+ $(,)?) => {{
        let mut params = std::collections::HashMap::new();
        $(
            params.insert(stringify!($name).to_string(), $value.to_string());
        )+

        // Check if we have a count parameter for pluralization
        let count: Option<i64> = None;
        $(
            let count = if stringify!($name) == "count" {
                Some($value as i64)
            } else {
                count
            };
        )+

        if let Some(n) = count {
            $i18n.t_plural_with_params($key, n, &params)
        } else {
            $i18n.t_with_params($key, &params)
        }
    }};
}

/// Check if a translation key exists
///
/// # Examples
///
/// ```rust,ignore
/// if has_key!("auth.login.title") {
///     // Key exists
/// }
/// ```
#[macro_export]
macro_rules! has_key {
    ($key:expr) => {{
        $crate::runtime::global()
            .map(|i18n| i18n.has_key($key))
            .unwrap_or(false)
    }};
}

/// Get the current locale
///
/// # Examples
///
/// ```rust,ignore
/// let locale = current_locale!();
/// println!("Current locale: {}", locale);
/// ```
#[macro_export]
macro_rules! current_locale {
    () => {{
        $crate::runtime::global()
            .map(|i18n| i18n.locale())
            .unwrap_or_else(|| "en".to_string())
    }};
}

/// Check if current locale is RTL
///
/// # Examples
///
/// ```rust,ignore
/// if is_rtl!() {
///     // Apply RTL-specific styles
/// }
/// ```
#[macro_export]
macro_rules! is_rtl {
    () => {{
        $crate::runtime::global()
            .map(|i18n| i18n.is_rtl())
            .unwrap_or(false)
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{init_global, I18n};
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_global() -> TempDir {
        let dir = TempDir::new().unwrap();
        let i18n_dir = dir.path().join("i18n");
        fs::create_dir(&i18n_dir).unwrap();

        let en_content = r#"
[greeting]
hello = "Hello"
welcome = "Welcome, {name}!"

[cart]
items = { one = "{count} item", other = "{count} items" }
"#;
        let mut en_file = fs::File::create(i18n_dir.join("en.toml")).unwrap();
        en_file.write_all(en_content.as_bytes()).unwrap();

        let i18n = I18n::load(&i18n_dir).unwrap();
        init_global(i18n);

        dir
    }

    #[test]
    fn test_t_macro_simple() {
        let _dir = setup_global();
        assert_eq!(t!("greeting.hello"), "Hello");
    }

    #[test]
    fn test_t_macro_with_params() {
        let _dir = setup_global();
        assert_eq!(t!("greeting.welcome", name = "Alice"), "Welcome, Alice!");
    }

    #[test]
    fn test_t_macro_plural() {
        let _dir = setup_global();
        assert_eq!(t!("cart.items", count = 1), "1 item");
        assert_eq!(t!("cart.items", count = 5), "5 items");
    }

    #[test]
    fn test_has_key_macro() {
        let _dir = setup_global();
        assert!(has_key!("greeting.hello"));
        assert!(!has_key!("nonexistent.key"));
    }

    #[test]
    fn test_current_locale_macro() {
        let _dir = setup_global();
        assert_eq!(current_locale!(), "en");
    }
}

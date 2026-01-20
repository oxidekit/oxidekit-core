//! Route Definition
//!
//! This module provides the core route definition types for OxideKit's routing system.
//! Routes define the mapping between URL paths and application pages/components.

use crate::params::{ParsedPath, PathParams};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

/// A unique identifier for a route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RouteId(Uuid);

impl RouteId {
    /// Creates a new random route ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a route ID from raw bytes.
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(Uuid::from_bytes(bytes))
    }
}

impl Default for RouteId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RouteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A segment in a route path pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// A literal path segment that must match exactly.
    Literal(String),
    /// A parameter segment that captures a value (e.g., `:id`).
    Param(String),
    /// A wildcard that matches any remaining path segments.
    Wildcard,
    /// An optional segment (ends with `?`).
    Optional(String),
}

/// A compiled route pattern that can be matched against paths.
#[derive(Debug, Clone)]
pub struct RoutePattern {
    /// The original pattern string.
    pub pattern: String,
    /// The parsed segments.
    pub segments: Vec<PathSegment>,
    /// Compiled regex for matching (cached).
    regex: Regex,
    /// Parameter names in order.
    param_names: Vec<String>,
    /// Whether this pattern has a wildcard.
    has_wildcard: bool,
}

impl RoutePattern {
    /// Creates a new route pattern from a path string.
    ///
    /// # Pattern Syntax
    ///
    /// - `/literal` - Matches exactly
    /// - `/:param` - Captures a single segment as a named parameter
    /// - `/:param?` - Optional parameter segment
    /// - `/*` or `/**` - Wildcard matching any remaining segments
    /// - `/users/:id/posts/:postId` - Multiple parameters
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxide_router::route::RoutePattern;
    ///
    /// let pattern = RoutePattern::new("/users/:id/posts/:postId").unwrap();
    /// let params = pattern.match_path("/users/123/posts/456").unwrap();
    /// assert_eq!(params.get("id"), Some("123"));
    /// assert_eq!(params.get("postId"), Some("456"));
    /// ```
    pub fn new(pattern: &str) -> Result<Self, RoutePatternError> {
        let pattern = pattern.to_string();
        let mut segments = Vec::new();
        let mut param_names = Vec::new();
        let mut regex_parts = vec!["^".to_string()];
        let mut has_wildcard = false;

        let path_segments: Vec<&str> = pattern
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        for segment in path_segments {
            if segment == "*" || segment == "**" {
                segments.push(PathSegment::Wildcard);
                regex_parts.push("(?:/(.*))?".to_string());
                param_names.push("*".to_string());
                has_wildcard = true;
            } else if let Some(name) = segment.strip_prefix(':') {
                if let Some(name) = name.strip_suffix('?') {
                    // Optional parameter
                    if name.is_empty() {
                        return Err(RoutePatternError::InvalidPattern(
                            "empty optional parameter name".to_string(),
                        ));
                    }
                    segments.push(PathSegment::Optional(name.to_string()));
                    regex_parts.push(format!("(?:/([^/]+))?"));
                    param_names.push(name.to_string());
                } else {
                    // Required parameter
                    if name.is_empty() {
                        return Err(RoutePatternError::InvalidPattern(
                            "empty parameter name".to_string(),
                        ));
                    }
                    segments.push(PathSegment::Param(name.to_string()));
                    regex_parts.push("/([^/]+)".to_string());
                    param_names.push(name.to_string());
                }
            } else {
                // Literal segment
                segments.push(PathSegment::Literal(segment.to_string()));
                regex_parts.push(format!("/{}", regex::escape(segment)));
            }
        }

        if !has_wildcard {
            regex_parts.push("/?$".to_string());
        }

        let regex_str = regex_parts.join("");
        let regex = Regex::new(&regex_str).map_err(|e| {
            RoutePatternError::InvalidPattern(format!("invalid regex: {}", e))
        })?;

        Ok(Self {
            pattern,
            segments,
            regex,
            param_names,
            has_wildcard,
        })
    }

    /// Attempts to match a path against this pattern, returning extracted parameters.
    pub fn match_path(&self, path: &str) -> Option<PathParams> {
        let path = if path.is_empty() { "/" } else { path };

        // For root pattern, handle specially
        if self.pattern == "/" {
            return if path == "/" || path.is_empty() {
                Some(PathParams::new())
            } else {
                None
            };
        }

        let captures = self.regex.captures(path)?;
        let mut params = PathParams::new();

        for (i, name) in self.param_names.iter().enumerate() {
            if let Some(matched) = captures.get(i + 1) {
                let value = matched.as_str();
                if !value.is_empty() || name == "*" {
                    params.insert(name.clone(), value.to_string());
                }
            }
        }

        Some(params)
    }

    /// Generates a path string from parameters.
    ///
    /// # Example
    ///
    /// ```rust
    /// use oxide_router::route::RoutePattern;
    /// use oxide_router::params::PathParams;
    ///
    /// let pattern = RoutePattern::new("/users/:id/posts/:postId").unwrap();
    /// let mut params = PathParams::new();
    /// params.insert("id", "123");
    /// params.insert("postId", "456");
    ///
    /// assert_eq!(pattern.generate(&params).unwrap(), "/users/123/posts/456");
    /// ```
    pub fn generate(&self, params: &PathParams) -> Result<String, RoutePatternError> {
        let mut path = String::new();

        for segment in &self.segments {
            match segment {
                PathSegment::Literal(s) => {
                    path.push('/');
                    path.push_str(s);
                }
                PathSegment::Param(name) => {
                    let value = params.get(name).ok_or_else(|| {
                        RoutePatternError::MissingParam(name.clone())
                    })?;
                    path.push('/');
                    path.push_str(value);
                }
                PathSegment::Optional(name) => {
                    if let Some(value) = params.get(name) {
                        path.push('/');
                        path.push_str(value);
                    }
                }
                PathSegment::Wildcard => {
                    if let Some(value) = params.get("*") {
                        if !value.is_empty() {
                            if !path.is_empty() {
                                path.push('/');
                            }
                            path.push_str(value);
                        }
                    }
                }
            }
        }

        if path.is_empty() {
            path.push('/');
        }

        Ok(path)
    }

    /// Returns the number of literal segments (used for priority calculation).
    pub fn specificity(&self) -> usize {
        self.segments
            .iter()
            .filter(|s| matches!(s, PathSegment::Literal(_)))
            .count()
    }

    /// Returns true if this pattern has a wildcard.
    pub fn is_wildcard(&self) -> bool {
        self.has_wildcard
    }

    /// Returns the parameter names in this pattern.
    pub fn param_names(&self) -> &[String] {
        &self.param_names
    }
}

impl PartialEq for RoutePattern {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern
    }
}

/// Errors that can occur when working with route patterns.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RoutePatternError {
    /// The pattern syntax is invalid.
    #[error("invalid route pattern: {0}")]
    InvalidPattern(String),

    /// A required parameter was not provided when generating a path.
    #[error("missing parameter: {0}")]
    MissingParam(String),
}

/// A trait for types that can be used as route components/pages.
pub trait RouteComponent: Send + Sync + 'static {
    /// Returns a type identifier for this component.
    fn type_id(&self) -> std::any::TypeId;

    /// Returns the component name for debugging.
    fn name(&self) -> &str;

    /// Clones this component as a boxed trait object.
    fn clone_box(&self) -> Box<dyn RouteComponent>;

    /// Returns this component as Any for downcasting.
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn RouteComponent> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl fmt::Debug for Box<dyn RouteComponent> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RouteComponent({})", self.name())
    }
}

/// A simple component reference using a string identifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentRef {
    /// The component identifier/name.
    pub name: String,
    /// Optional props to pass to the component.
    pub props: Option<serde_json::Value>,
}

impl ComponentRef {
    /// Creates a new component reference.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            props: None,
        }
    }

    /// Creates a component reference with props.
    pub fn with_props(name: impl Into<String>, props: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            props: Some(props),
        }
    }
}

impl RouteComponent for ComponentRef {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<ComponentRef>()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn clone_box(&self) -> Box<dyn RouteComponent> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Metadata associated with a route.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteMeta {
    /// Custom metadata key-value pairs.
    pub data: HashMap<String, serde_json::Value>,
}

impl RouteMeta {
    /// Creates new empty metadata.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Inserts a metadata value.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Serialize) {
        self.data.insert(
            key.into(),
            serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        );
    }

    /// Gets a metadata value.
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Checks if a key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns true if the route requires authentication (based on meta).
    pub fn requires_auth(&self) -> bool {
        self.get::<bool>("requiresAuth").unwrap_or(false)
    }

    /// Returns the page title if set.
    pub fn title(&self) -> Option<String> {
        self.get("title")
    }
}

/// A route definition mapping a URL pattern to a component.
pub struct Route {
    /// Unique identifier for this route.
    pub id: RouteId,
    /// Optional name for the route (for named navigation).
    pub name: Option<String>,
    /// The URL pattern to match.
    pub pattern: RoutePattern,
    /// The component to render when this route matches.
    pub component: Arc<dyn RouteComponent>,
    /// Child routes (for nested routing).
    pub children: Vec<Route>,
    /// Route metadata (title, auth requirements, etc.).
    pub meta: RouteMeta,
    /// The priority of this route (higher = matched first).
    pub priority: i32,
}

impl Route {
    /// Creates a new route with the given pattern and component.
    pub fn new<C: RouteComponent>(path: &str, component: C) -> Result<Self, RoutePatternError> {
        let pattern = RoutePattern::new(path)?;
        let priority = Self::calculate_priority(&pattern);

        Ok(Self {
            id: RouteId::new(),
            name: None,
            pattern,
            component: Arc::new(component),
            children: Vec::new(),
            meta: RouteMeta::new(),
            priority,
        })
    }

    /// Creates a new route with a component reference.
    pub fn with_ref(path: &str, component_name: &str) -> Result<Self, RoutePatternError> {
        Self::new(path, ComponentRef::new(component_name))
    }

    /// Sets the route name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Adds a child route.
    pub fn child(mut self, route: Route) -> Self {
        self.children.push(route);
        self
    }

    /// Adds multiple child routes.
    pub fn children(mut self, routes: impl IntoIterator<Item = Route>) -> Self {
        self.children.extend(routes);
        self
    }

    /// Sets route metadata.
    pub fn meta(mut self, meta: RouteMeta) -> Self {
        self.meta = meta;
        self
    }

    /// Sets a single metadata value.
    pub fn meta_value(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.meta.insert(key, value);
        self
    }

    /// Sets the route title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.meta.insert("title", title.into());
        self
    }

    /// Marks this route as requiring authentication.
    pub fn requires_auth(mut self) -> Self {
        self.meta.insert("requiresAuth", true);
        self
    }

    /// Sets the route priority (higher = matched first).
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Calculates the default priority based on pattern specificity.
    fn calculate_priority(pattern: &RoutePattern) -> i32 {
        let mut priority = 0i32;

        for (i, segment) in pattern.segments.iter().enumerate() {
            let position_weight = 100 - (i as i32 * 10);
            match segment {
                PathSegment::Literal(_) => priority += position_weight * 3,
                PathSegment::Param(_) => priority += position_weight * 2,
                PathSegment::Optional(_) => priority += position_weight,
                PathSegment::Wildcard => priority -= 50,
            }
        }

        priority
    }

    /// Returns the path pattern string.
    pub fn path(&self) -> &str {
        &self.pattern.pattern
    }

    /// Attempts to match a path against this route.
    pub fn match_path(&self, path: &str) -> Option<RouteMatch> {
        self.pattern.match_path(path).map(|params| RouteMatch {
            route: self.clone(),
            params,
            matched_path: path.to_string(),
        })
    }

    /// Attempts to match a path, including child routes.
    pub fn match_path_recursive(&self, path: &str) -> Option<RouteMatch> {
        // Try to match this route
        if let Some(params) = self.pattern.match_path(path) {
            // If this route has children, try to match them
            if !self.children.is_empty() {
                // Get the remaining path after this route's pattern
                let remaining = self.get_remaining_path(path);

                // Try each child route
                for child in &self.children {
                    if let Some(mut child_match) = child.match_path_recursive(&remaining) {
                        // Merge parent params with child params
                        for (k, v) in params.iter() {
                            child_match.params.insert(k.to_string(), v.to_string());
                        }
                        return Some(child_match);
                    }
                }
            }

            // Return this route if no children matched or no children exist
            return Some(RouteMatch {
                route: self.clone(),
                params,
                matched_path: path.to_string(),
            });
        }

        None
    }

    /// Gets the remaining path after this route's pattern matches.
    fn get_remaining_path(&self, path: &str) -> String {
        let parsed_pattern = ParsedPath::parse(&self.pattern.pattern);
        let parsed_path = ParsedPath::parse(path);

        let remaining_segments: Vec<&str> = parsed_path
            .segments
            .iter()
            .skip(parsed_pattern.segments.len())
            .map(|s| s.as_str())
            .collect();

        if remaining_segments.is_empty() {
            "/".to_string()
        } else {
            format!("/{}", remaining_segments.join("/"))
        }
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("pattern", &self.pattern.pattern)
            .field("children", &self.children.len())
            .field("meta", &self.meta)
            .field("priority", &self.priority)
            .finish()
    }
}

impl Clone for Route {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            pattern: self.pattern.clone(),
            component: self.component.clone(),
            children: self.children.clone(),
            meta: self.meta.clone(),
            priority: self.priority,
        }
    }
}

/// A successful route match containing the route and extracted parameters.
#[derive(Debug, Clone)]
pub struct RouteMatch {
    /// The matched route.
    pub route: Route,
    /// The extracted path parameters.
    pub params: PathParams,
    /// The actual path that was matched.
    pub matched_path: String,
}

impl RouteMatch {
    /// Returns the route ID.
    pub fn id(&self) -> RouteId {
        self.route.id
    }

    /// Returns the route name if set.
    pub fn name(&self) -> Option<&str> {
        self.route.name.as_deref()
    }

    /// Returns the route metadata.
    pub fn meta(&self) -> &RouteMeta {
        &self.route.meta
    }
}

/// A redirect definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Redirect {
    /// The source pattern to match.
    pub from: String,
    /// The destination path (can include parameters).
    pub to: String,
    /// Whether this is a permanent redirect (301) or temporary (302).
    pub permanent: bool,
}

impl Redirect {
    /// Creates a new temporary redirect.
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            permanent: false,
        }
    }

    /// Creates a permanent redirect.
    pub fn permanent(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            permanent: true,
        }
    }

    /// Attempts to match and transform a path.
    pub fn apply(&self, path: &str) -> Option<String> {
        let pattern = RoutePattern::new(&self.from).ok()?;
        let params = pattern.match_path(path)?;

        // Simple parameter substitution in the target
        let mut result = self.to.clone();
        for (name, value) in params.iter() {
            result = result.replace(&format!(":{}", name), value);
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod route_pattern {
        use super::*;

        #[test]
        fn test_literal_pattern() {
            let pattern = RoutePattern::new("/users").unwrap();
            assert!(pattern.match_path("/users").is_some());
            assert!(pattern.match_path("/users/").is_some());
            assert!(pattern.match_path("/posts").is_none());
        }

        #[test]
        fn test_root_pattern() {
            let pattern = RoutePattern::new("/").unwrap();
            assert!(pattern.match_path("/").is_some());
            assert!(pattern.match_path("").is_some());
            assert!(pattern.match_path("/users").is_none());
        }

        #[test]
        fn test_param_pattern() {
            let pattern = RoutePattern::new("/users/:id").unwrap();
            let params = pattern.match_path("/users/123").unwrap();
            assert_eq!(params.get("id"), Some("123"));

            let params = pattern.match_path("/users/abc").unwrap();
            assert_eq!(params.get("id"), Some("abc"));

            assert!(pattern.match_path("/users").is_none());
        }

        #[test]
        fn test_multiple_params() {
            let pattern = RoutePattern::new("/users/:userId/posts/:postId").unwrap();
            let params = pattern.match_path("/users/123/posts/456").unwrap();

            assert_eq!(params.get("userId"), Some("123"));
            assert_eq!(params.get("postId"), Some("456"));
        }

        #[test]
        fn test_wildcard_pattern() {
            let pattern = RoutePattern::new("/files/*").unwrap();

            let params = pattern.match_path("/files/a/b/c").unwrap();
            assert_eq!(params.get("*"), Some("a/b/c"));

            assert!(pattern.match_path("/files").is_some());
            assert!(pattern.match_path("/other").is_none());
        }

        #[test]
        fn test_optional_param() {
            let pattern = RoutePattern::new("/users/:id?").unwrap();

            assert!(pattern.match_path("/users").is_some());

            let params = pattern.match_path("/users/123").unwrap();
            assert_eq!(params.get("id"), Some("123"));
        }

        #[test]
        fn test_generate_path() {
            let pattern = RoutePattern::new("/users/:id/posts/:postId").unwrap();
            let mut params = PathParams::new();
            params.insert("id", "123");
            params.insert("postId", "456");

            assert_eq!(pattern.generate(&params).unwrap(), "/users/123/posts/456");
        }

        #[test]
        fn test_generate_missing_param() {
            let pattern = RoutePattern::new("/users/:id").unwrap();
            let params = PathParams::new();

            assert!(matches!(
                pattern.generate(&params),
                Err(RoutePatternError::MissingParam(_))
            ));
        }

        #[test]
        fn test_specificity() {
            let wild = RoutePattern::new("/files/*").unwrap();
            let param = RoutePattern::new("/users/:id").unwrap();
            let literal = RoutePattern::new("/users/list").unwrap();

            assert!(literal.specificity() > param.specificity());
            assert!(param.specificity() > wild.specificity());
        }
    }

    mod route {
        use super::*;

        #[test]
        fn test_route_creation() {
            let route = Route::with_ref("/users/:id", "UserPage").unwrap();
            assert_eq!(route.path(), "/users/:id");
        }

        #[test]
        fn test_route_with_name() {
            let route = Route::with_ref("/users", "UsersPage")
                .unwrap()
                .name("users-list");

            assert_eq!(route.name, Some("users-list".to_string()));
        }

        #[test]
        fn test_route_with_meta() {
            let route = Route::with_ref("/admin", "AdminPage")
                .unwrap()
                .title("Admin Dashboard")
                .requires_auth();

            assert_eq!(route.meta.title(), Some("Admin Dashboard".to_string()));
            assert!(route.meta.requires_auth());
        }

        #[test]
        fn test_route_matching() {
            let route = Route::with_ref("/users/:id", "UserPage").unwrap();
            let matched = route.match_path("/users/123").unwrap();

            assert_eq!(matched.params.get("id"), Some("123"));
        }

        #[test]
        fn test_nested_routes() {
            let parent = Route::with_ref("/users", "UsersLayout")
                .unwrap()
                .child(Route::with_ref(":id", "UserDetail").unwrap())
                .child(Route::with_ref("new", "NewUser").unwrap());

            assert_eq!(parent.children.len(), 2);
        }
    }

    mod redirect {
        use super::*;

        #[test]
        fn test_simple_redirect() {
            let redirect = Redirect::new("/old", "/new");
            assert_eq!(redirect.apply("/old"), Some("/new".to_string()));
            assert_eq!(redirect.apply("/other"), None);
        }

        #[test]
        fn test_redirect_with_params() {
            let redirect = Redirect::new("/users/:id/profile", "/profile/:id");
            assert_eq!(redirect.apply("/users/123/profile"), Some("/profile/123".to_string()));
        }

        #[test]
        fn test_permanent_redirect() {
            let redirect = Redirect::permanent("/old", "/new");
            assert!(redirect.permanent);
        }
    }
}

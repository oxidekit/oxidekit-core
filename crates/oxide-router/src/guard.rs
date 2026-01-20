//! Navigation Guards
//!
//! This module provides a comprehensive guard system for controlling navigation.
//! Guards can be used to implement authentication checks, unsaved changes warnings,
//! data preloading, and other navigation-related logic.

use crate::params::ParsedUrl;
use crate::route::{RouteId, RouteMatch};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;

/// The result of a navigation guard check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GuardResult {
    /// Allow the navigation to proceed.
    Allow,
    /// Deny the navigation (stay on current route).
    Deny,
    /// Redirect to a different path.
    Redirect(String),
    /// Deny with an error message (for user feedback).
    DenyWithMessage(String),
}

impl GuardResult {
    /// Returns true if navigation is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, GuardResult::Allow)
    }

    /// Returns true if navigation is denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, GuardResult::Deny | GuardResult::DenyWithMessage(_))
    }

    /// Returns the redirect path if this is a redirect result.
    pub fn redirect_path(&self) -> Option<&str> {
        match self {
            GuardResult::Redirect(path) => Some(path),
            _ => None,
        }
    }

    /// Returns the denial message if present.
    pub fn denial_message(&self) -> Option<&str> {
        match self {
            GuardResult::DenyWithMessage(msg) => Some(msg),
            _ => None,
        }
    }
}

impl Default for GuardResult {
    fn default() -> Self {
        GuardResult::Allow
    }
}

/// Context provided to navigation guards.
#[derive(Debug, Clone)]
pub struct GuardContext {
    /// The source URL (current location).
    pub from: Option<ParsedUrl>,
    /// The destination URL.
    pub to: ParsedUrl,
    /// The matched route for the destination (if any).
    pub route_match: Option<RouteMatch>,
    /// Custom data passed through the navigation.
    pub data: Option<serde_json::Value>,
}

impl GuardContext {
    /// Creates a new guard context.
    pub fn new(to: ParsedUrl) -> Self {
        Self {
            from: None,
            to,
            route_match: None,
            data: None,
        }
    }

    /// Sets the source URL.
    pub fn with_from(mut self, from: ParsedUrl) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the route match.
    pub fn with_route_match(mut self, route_match: RouteMatch) -> Self {
        self.route_match = Some(route_match);
        self
    }

    /// Sets custom navigation data.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Returns the destination path.
    pub fn path(&self) -> &str {
        &self.to.path.raw
    }

    /// Returns a path parameter from the matched route.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.route_match.as_ref()?.params.get(name)
    }

    /// Returns a query parameter from the destination URL.
    pub fn query(&self, name: &str) -> Option<&str> {
        self.to.query.get(name)
    }
}

/// A trait for synchronous navigation guards.
///
/// Guards are invoked before navigation occurs and can allow, deny, or redirect.
///
/// # Example
///
/// ```rust
/// use oxide_router::guard::{NavigationGuard, GuardContext, GuardResult};
///
/// struct AuthGuard {
///     is_authenticated: bool,
/// }
///
/// impl NavigationGuard for AuthGuard {
///     fn check(&self, _ctx: &GuardContext) -> GuardResult {
///         if self.is_authenticated {
///             GuardResult::Allow
///         } else {
///             GuardResult::Redirect("/login".to_string())
///         }
///     }
///
///     fn name(&self) -> &str {
///         "AuthGuard"
///     }
/// }
/// ```
pub trait NavigationGuard: Send + Sync {
    /// Checks if navigation should be allowed.
    fn check(&self, ctx: &GuardContext) -> GuardResult;

    /// Returns the guard name for debugging.
    fn name(&self) -> &str;

    /// Returns the priority of this guard (higher = runs first).
    /// Default is 0.
    fn priority(&self) -> i32 {
        0
    }
}

impl fmt::Debug for dyn NavigationGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NavigationGuard({})", self.name())
    }
}

/// A trait for asynchronous navigation guards.
///
/// Use this when your guard needs to perform async operations like
/// server-side validation or data fetching.
#[cfg(feature = "async-guards")]
#[async_trait::async_trait]
pub trait AsyncNavigationGuard: Send + Sync {
    /// Asynchronously checks if navigation should be allowed.
    async fn check(&self, ctx: &GuardContext) -> GuardResult;

    /// Returns the guard name for debugging.
    fn name(&self) -> &str;

    /// Returns the priority of this guard (higher = runs first).
    fn priority(&self) -> i32 {
        0
    }
}

#[cfg(feature = "async-guards")]
impl fmt::Debug for dyn AsyncNavigationGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AsyncNavigationGuard({})", self.name())
    }
}

/// When a guard should run in the navigation lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GuardPhase {
    /// Before entering a new route (most common).
    BeforeEnter,
    /// Before leaving the current route.
    BeforeLeave,
    /// After the route has been entered (for analytics, etc.).
    AfterEnter,
    /// Before resolving route data.
    BeforeResolve,
}

/// A guard registration that associates a guard with routes.
pub struct GuardRegistration {
    /// The guard instance.
    pub guard: Arc<dyn NavigationGuard>,
    /// The phase when this guard runs.
    pub phase: GuardPhase,
    /// Optional pattern to match routes (None = applies to all routes).
    pub pattern: Option<String>,
    /// Specific route IDs this guard applies to.
    pub route_ids: Vec<RouteId>,
}

impl GuardRegistration {
    /// Creates a new guard registration for all routes.
    pub fn global(guard: impl NavigationGuard + 'static, phase: GuardPhase) -> Self {
        Self {
            guard: Arc::new(guard),
            phase,
            pattern: None,
            route_ids: Vec::new(),
        }
    }

    /// Creates a guard registration for routes matching a pattern.
    pub fn pattern(
        guard: impl NavigationGuard + 'static,
        phase: GuardPhase,
        pattern: impl Into<String>,
    ) -> Self {
        Self {
            guard: Arc::new(guard),
            phase,
            pattern: Some(pattern.into()),
            route_ids: Vec::new(),
        }
    }

    /// Creates a guard registration for specific routes.
    pub fn routes(
        guard: impl NavigationGuard + 'static,
        phase: GuardPhase,
        route_ids: impl IntoIterator<Item = RouteId>,
    ) -> Self {
        Self {
            guard: Arc::new(guard),
            phase,
            pattern: None,
            route_ids: route_ids.into_iter().collect(),
        }
    }

    /// Checks if this guard applies to the given path and route ID.
    pub fn applies_to(&self, path: &str, route_id: Option<RouteId>) -> bool {
        // If specific route IDs are set, check those
        if !self.route_ids.is_empty() {
            return route_id.map(|id| self.route_ids.contains(&id)).unwrap_or(false);
        }

        // If a pattern is set, check that
        if let Some(ref pattern) = self.pattern {
            return self.path_matches_pattern(path, pattern);
        }

        // Otherwise, applies to all routes
        true
    }

    /// Simple pattern matching (supports * wildcard at end).
    fn path_matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.ends_with("/*") || pattern.ends_with("/**") {
            let prefix = pattern.trim_end_matches("*").trim_end_matches("/");
            path.starts_with(prefix)
        } else if pattern.ends_with('*') {
            let prefix = pattern.trim_end_matches('*');
            path.starts_with(prefix)
        } else {
            path == pattern
        }
    }
}

impl fmt::Debug for GuardRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GuardRegistration")
            .field("guard", &self.guard.name())
            .field("phase", &self.phase)
            .field("pattern", &self.pattern)
            .field("route_ids", &self.route_ids)
            .finish()
    }
}

/// Manages guard registrations and execution.
#[derive(Default)]
pub struct GuardManager {
    guards: Vec<GuardRegistration>,
}

impl GuardManager {
    /// Creates a new guard manager.
    pub fn new() -> Self {
        Self { guards: Vec::new() }
    }

    /// Registers a guard.
    pub fn register(&mut self, registration: GuardRegistration) {
        self.guards.push(registration);
        // Sort by priority (higher first)
        self.guards.sort_by(|a, b| {
            b.guard.priority().cmp(&a.guard.priority())
        });
    }

    /// Registers a global guard for a specific phase.
    pub fn add_global(&mut self, guard: impl NavigationGuard + 'static, phase: GuardPhase) {
        self.register(GuardRegistration::global(guard, phase));
    }

    /// Registers a guard for routes matching a pattern.
    pub fn add_pattern(
        &mut self,
        guard: impl NavigationGuard + 'static,
        phase: GuardPhase,
        pattern: impl Into<String>,
    ) {
        self.register(GuardRegistration::pattern(guard, phase, pattern));
    }

    /// Executes all guards for the given phase and context.
    ///
    /// Returns the first non-Allow result, or Allow if all guards pass.
    pub fn run(&self, phase: GuardPhase, ctx: &GuardContext) -> GuardResult {
        let route_id = ctx.route_match.as_ref().map(|m| m.id());

        for reg in &self.guards {
            if reg.phase != phase {
                continue;
            }

            if !reg.applies_to(ctx.path(), route_id) {
                continue;
            }

            tracing::debug!(
                guard = reg.guard.name(),
                phase = ?phase,
                path = ctx.path(),
                "Running navigation guard"
            );

            let result = reg.guard.check(ctx);

            if !result.is_allowed() {
                tracing::debug!(
                    guard = reg.guard.name(),
                    result = ?result,
                    "Guard blocked navigation"
                );
                return result;
            }
        }

        GuardResult::Allow
    }

    /// Returns the number of registered guards.
    pub fn len(&self) -> usize {
        self.guards.len()
    }

    /// Returns true if no guards are registered.
    pub fn is_empty(&self) -> bool {
        self.guards.is_empty()
    }

    /// Clears all registered guards.
    pub fn clear(&mut self) {
        self.guards.clear();
    }
}

impl fmt::Debug for GuardManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GuardManager")
            .field("guards", &self.guards.len())
            .finish()
    }
}

// Common guard implementations

/// A guard that always allows navigation (useful for testing).
#[derive(Debug, Clone, Copy, Default)]
pub struct AllowAllGuard;

impl NavigationGuard for AllowAllGuard {
    fn check(&self, _ctx: &GuardContext) -> GuardResult {
        GuardResult::Allow
    }

    fn name(&self) -> &str {
        "AllowAllGuard"
    }
}

/// A guard that always denies navigation (useful for testing).
#[derive(Debug, Clone, Default)]
pub struct DenyAllGuard {
    message: Option<String>,
}

impl DenyAllGuard {
    /// Creates a new deny guard with a message.
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
        }
    }
}

impl NavigationGuard for DenyAllGuard {
    fn check(&self, _ctx: &GuardContext) -> GuardResult {
        match &self.message {
            Some(msg) => GuardResult::DenyWithMessage(msg.clone()),
            None => GuardResult::Deny,
        }
    }

    fn name(&self) -> &str {
        "DenyAllGuard"
    }
}

/// A guard that redirects to a specific path.
#[derive(Debug, Clone)]
pub struct RedirectGuard {
    target: String,
}

impl RedirectGuard {
    /// Creates a new redirect guard.
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
        }
    }
}

impl NavigationGuard for RedirectGuard {
    fn check(&self, _ctx: &GuardContext) -> GuardResult {
        GuardResult::Redirect(self.target.clone())
    }

    fn name(&self) -> &str {
        "RedirectGuard"
    }
}

/// A guard that checks a condition function.
#[derive(Clone)]
pub struct ConditionalGuard<F>
where
    F: Fn(&GuardContext) -> bool + Send + Sync,
{
    condition: F,
    name: String,
    on_fail: GuardResult,
}

impl<F> ConditionalGuard<F>
where
    F: Fn(&GuardContext) -> bool + Send + Sync,
{
    /// Creates a new conditional guard.
    pub fn new(name: impl Into<String>, condition: F) -> Self {
        Self {
            condition,
            name: name.into(),
            on_fail: GuardResult::Deny,
        }
    }

    /// Sets the result when the condition fails.
    pub fn on_fail(mut self, result: GuardResult) -> Self {
        self.on_fail = result;
        self
    }
}

impl<F> NavigationGuard for ConditionalGuard<F>
where
    F: Fn(&GuardContext) -> bool + Send + Sync,
{
    fn check(&self, ctx: &GuardContext) -> GuardResult {
        if (self.condition)(ctx) {
            GuardResult::Allow
        } else {
            self.on_fail.clone()
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl<F> fmt::Debug for ConditionalGuard<F>
where
    F: Fn(&GuardContext) -> bool + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConditionalGuard")
            .field("name", &self.name)
            .field("on_fail", &self.on_fail)
            .finish()
    }
}

/// A guard that checks route metadata for authentication requirements.
#[derive(Debug, Clone, Default)]
pub struct MetaAuthGuard {
    login_path: String,
    is_authenticated: bool,
}

impl MetaAuthGuard {
    /// Creates a new meta auth guard.
    pub fn new(login_path: impl Into<String>) -> Self {
        Self {
            login_path: login_path.into(),
            is_authenticated: false,
        }
    }

    /// Sets the authentication status.
    pub fn set_authenticated(&mut self, authenticated: bool) {
        self.is_authenticated = authenticated;
    }
}

impl NavigationGuard for MetaAuthGuard {
    fn check(&self, ctx: &GuardContext) -> GuardResult {
        // Check if the route requires authentication
        let requires_auth = ctx
            .route_match
            .as_ref()
            .map(|m| m.meta().requires_auth())
            .unwrap_or(false);

        if requires_auth && !self.is_authenticated {
            // Add return URL as query param
            let return_url = ctx.path();
            let login_with_return = format!("{}?returnUrl={}", self.login_path, return_url);
            GuardResult::Redirect(login_with_return)
        } else {
            GuardResult::Allow
        }
    }

    fn name(&self) -> &str {
        "MetaAuthGuard"
    }

    fn priority(&self) -> i32 {
        100 // Run early
    }
}

/// A guard that prompts for confirmation before leaving (e.g., unsaved changes).
#[derive(Clone)]
pub struct ConfirmLeaveGuard {
    message: String,
    should_confirm: Arc<dyn Fn() -> bool + Send + Sync>,
}

impl std::fmt::Debug for ConfirmLeaveGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfirmLeaveGuard")
            .field("message", &self.message)
            .field("should_confirm", &"<function>")
            .finish()
    }
}

impl ConfirmLeaveGuard {
    /// Creates a new confirm leave guard with a static condition.
    pub fn new(message: impl Into<String>, should_confirm: impl Fn() -> bool + Send + Sync + 'static) -> Self {
        Self {
            message: message.into(),
            should_confirm: Arc::new(should_confirm),
        }
    }

    /// Creates a guard that always asks for confirmation.
    pub fn always(message: impl Into<String>) -> Self {
        Self::new(message, || true)
    }
}

impl NavigationGuard for ConfirmLeaveGuard {
    fn check(&self, _ctx: &GuardContext) -> GuardResult {
        if (self.should_confirm)() {
            GuardResult::DenyWithMessage(self.message.clone())
        } else {
            GuardResult::Allow
        }
    }

    fn name(&self) -> &str {
        "ConfirmLeaveGuard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ParsedUrl;

    fn make_context(path: &str) -> GuardContext {
        GuardContext::new(ParsedUrl::parse(path).unwrap())
    }

    #[test]
    fn test_guard_result_allow() {
        let result = GuardResult::Allow;
        assert!(result.is_allowed());
        assert!(!result.is_denied());
        assert!(result.redirect_path().is_none());
    }

    #[test]
    fn test_guard_result_deny() {
        let result = GuardResult::Deny;
        assert!(!result.is_allowed());
        assert!(result.is_denied());
    }

    #[test]
    fn test_guard_result_redirect() {
        let result = GuardResult::Redirect("/login".to_string());
        assert!(!result.is_allowed());
        assert!(!result.is_denied());
        assert_eq!(result.redirect_path(), Some("/login"));
    }

    #[test]
    fn test_guard_result_deny_with_message() {
        let result = GuardResult::DenyWithMessage("Unsaved changes".to_string());
        assert!(result.is_denied());
        assert_eq!(result.denial_message(), Some("Unsaved changes"));
    }

    #[test]
    fn test_allow_all_guard() {
        let guard = AllowAllGuard;
        let ctx = make_context("/any/path");
        assert!(guard.check(&ctx).is_allowed());
    }

    #[test]
    fn test_deny_all_guard() {
        let guard = DenyAllGuard::default();
        let ctx = make_context("/any/path");
        assert!(guard.check(&ctx).is_denied());
    }

    #[test]
    fn test_deny_all_guard_with_message() {
        let guard = DenyAllGuard::with_message("Not allowed");
        let ctx = make_context("/any/path");
        let result = guard.check(&ctx);
        assert_eq!(result.denial_message(), Some("Not allowed"));
    }

    #[test]
    fn test_redirect_guard() {
        let guard = RedirectGuard::new("/login");
        let ctx = make_context("/protected");
        let result = guard.check(&ctx);
        assert_eq!(result.redirect_path(), Some("/login"));
    }

    #[test]
    fn test_conditional_guard_allows() {
        let guard = ConditionalGuard::new("TestGuard", |_| true);
        let ctx = make_context("/any");
        assert!(guard.check(&ctx).is_allowed());
    }

    #[test]
    fn test_conditional_guard_denies() {
        let guard = ConditionalGuard::new("TestGuard", |_| false);
        let ctx = make_context("/any");
        assert!(guard.check(&ctx).is_denied());
    }

    #[test]
    fn test_conditional_guard_with_redirect() {
        let guard = ConditionalGuard::new("TestGuard", |_| false)
            .on_fail(GuardResult::Redirect("/home".to_string()));
        let ctx = make_context("/any");
        assert_eq!(guard.check(&ctx).redirect_path(), Some("/home"));
    }

    #[test]
    fn test_guard_manager_run() {
        let mut manager = GuardManager::new();
        manager.add_global(AllowAllGuard, GuardPhase::BeforeEnter);

        let ctx = make_context("/any");
        assert!(manager.run(GuardPhase::BeforeEnter, &ctx).is_allowed());
    }

    #[test]
    fn test_guard_manager_blocks() {
        let mut manager = GuardManager::new();
        manager.add_global(DenyAllGuard::default(), GuardPhase::BeforeEnter);

        let ctx = make_context("/any");
        assert!(manager.run(GuardPhase::BeforeEnter, &ctx).is_denied());
    }

    #[test]
    fn test_guard_manager_pattern_matching() {
        let mut manager = GuardManager::new();
        manager.add_pattern(DenyAllGuard::default(), GuardPhase::BeforeEnter, "/admin/*");

        let admin_ctx = make_context("/admin/users");
        assert!(manager.run(GuardPhase::BeforeEnter, &admin_ctx).is_denied());

        let public_ctx = make_context("/public/page");
        assert!(manager.run(GuardPhase::BeforeEnter, &public_ctx).is_allowed());
    }

    #[test]
    fn test_guard_manager_phase_filtering() {
        let mut manager = GuardManager::new();
        manager.add_global(DenyAllGuard::default(), GuardPhase::BeforeLeave);

        let ctx = make_context("/any");

        // Different phase should not trigger the guard
        assert!(manager.run(GuardPhase::BeforeEnter, &ctx).is_allowed());

        // Same phase should trigger
        assert!(manager.run(GuardPhase::BeforeLeave, &ctx).is_denied());
    }

    #[test]
    fn test_guard_registration_applies_to() {
        let reg = GuardRegistration::pattern(AllowAllGuard, GuardPhase::BeforeEnter, "/admin/*");

        assert!(reg.applies_to("/admin/dashboard", None));
        assert!(reg.applies_to("/admin/users/123", None));
        assert!(!reg.applies_to("/public", None));
    }
}

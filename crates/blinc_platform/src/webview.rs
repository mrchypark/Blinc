//! WebView abstraction for embedded web content.

use crate::error::Result;

/// WebView bounds in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WebViewBounds {
    /// Left position relative to the host window.
    pub x: f32,
    /// Top position relative to the host window.
    pub y: f32,
    /// Width in logical pixels.
    pub width: f32,
    /// Height in logical pixels.
    pub height: f32,
}

impl WebViewBounds {
    /// Create a new set of bounds.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Why a navigation request was blocked by policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebViewNavigationBlockReason {
    /// URL could not be parsed into a valid origin.
    MalformedUrl,
    /// URL origin is not in the allowlist.
    OriginNotAllowed,
}

/// Policy decision for a navigation request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebViewNavigationDecision {
    /// Navigation is allowed.
    Allowed {
        /// Original input URL.
        url: String,
        /// Canonical origin used for allowlist matching.
        origin: String,
    },
    /// Navigation is blocked.
    Blocked {
        /// Original input URL.
        url: String,
        /// Parsed origin when available.
        origin: Option<String>,
        /// Why policy blocked the request.
        reason: WebViewNavigationBlockReason,
    },
}

/// Restrictive navigation policy for WebView URL loading.
///
/// Defaults to an explicit allowlist with no entries, which blocks all
/// navigation until the host app opts in specific origins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebViewNavigationPolicy {
    /// Canonical allowed origins (for example, `https://example.com`).
    pub allowed_origins: Vec<String>,
}

impl Default for WebViewNavigationPolicy {
    fn default() -> Self {
        Self {
            allowed_origins: Vec::new(),
        }
    }
}

impl WebViewNavigationPolicy {
    /// Create a new restrictive policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an allowed origin.
    ///
    /// Invalid origins are ignored to keep configuration deterministic.
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        let raw = origin.into();
        if let Some(canonical) = canonical_origin(&raw) {
            if !self
                .allowed_origins
                .iter()
                .any(|existing| existing == &canonical)
            {
                self.allowed_origins.push(canonical);
            }
        }
        self
    }

    /// Add multiple allowed origins.
    pub fn allow_origins<I, S>(mut self, origins: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for origin in origins {
            self = self.allow_origin(origin);
        }
        self
    }

    /// Evaluate whether a URL should be allowed.
    pub fn evaluate(&self, url: &str) -> WebViewNavigationDecision {
        let input = url.trim().to_string();
        let Some(origin) = canonical_origin(&input) else {
            return WebViewNavigationDecision::Blocked {
                url: input,
                origin: None,
                reason: WebViewNavigationBlockReason::MalformedUrl,
            };
        };

        if self
            .allowed_origins
            .iter()
            .any(|allowed| allowed == &origin)
        {
            WebViewNavigationDecision::Allowed { url: input, origin }
        } else {
            WebViewNavigationDecision::Blocked {
                url: input,
                origin: Some(origin),
                reason: WebViewNavigationBlockReason::OriginNotAllowed,
            }
        }
    }
}

fn canonical_origin(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (scheme, remainder) = trimmed.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }

    let authority = remainder
        .split(['/', '?', '#'])
        .next()
        .map(str::trim)
        .filter(|segment| !segment.is_empty())?;
    if authority.contains('@') || authority.contains(' ') {
        return None;
    }

    let (host, port) = parse_host_and_port(authority)?;
    Some(match port {
        Some(port) => format!("{scheme}://{host}:{port}"),
        None => format!("{scheme}://{host}"),
    })
}

fn parse_host_and_port(authority: &str) -> Option<(String, Option<u16>)> {
    if authority.starts_with('[') {
        let end = authority.find(']')?;
        let host = authority[..=end].to_ascii_lowercase();
        if host.len() <= 2 {
            return None;
        }

        let remainder = &authority[end + 1..];
        if remainder.is_empty() {
            return Some((host, None));
        }

        let port = remainder.strip_prefix(':')?.parse::<u16>().ok()?;
        return Some((host, Some(port)));
    }

    let mut parts = authority.splitn(2, ':');
    let host = parts.next()?.trim().to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }

    let port = match parts.next() {
        Some(raw_port) => {
            if raw_port.is_empty() {
                return None;
            }
            Some(raw_port.parse::<u16>().ok()?)
        }
        None => None,
    };

    Some((host, port))
}

/// WebView creation configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct WebViewConfig {
    /// Initial navigation target.
    pub initial_url: Option<String>,
    /// Initial embedded bounds.
    pub bounds: WebViewBounds,
    /// Navigation allowlist policy.
    pub navigation_policy: WebViewNavigationPolicy,
    /// Whether local file access is permitted.
    pub allow_file_access: bool,
    /// Whether remote debugging is enabled.
    pub enable_remote_debugging: bool,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            initial_url: None,
            bounds: WebViewBounds::new(0.0, 0.0, 0.0, 0.0),
            navigation_policy: WebViewNavigationPolicy::default(),
            allow_file_access: false,
            enable_remote_debugging: false,
        }
    }
}

impl WebViewConfig {
    /// Create a new WebView configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the initial URL to navigate after creation.
    pub fn initial_url(mut self, initial_url: impl Into<String>) -> Self {
        self.initial_url = Some(initial_url.into());
        self
    }

    /// Set the initial WebView bounds.
    pub fn bounds(mut self, bounds: WebViewBounds) -> Self {
        self.bounds = bounds;
        self
    }

    /// Set navigation allowlist policy.
    pub fn navigation_policy(mut self, navigation_policy: WebViewNavigationPolicy) -> Self {
        self.navigation_policy = navigation_policy;
        self
    }

    /// Set file-access capability.
    pub fn allow_file_access(mut self, allow_file_access: bool) -> Self {
        self.allow_file_access = allow_file_access;
        self
    }

    /// Set remote debugging capability.
    pub fn enable_remote_debugging(mut self, enable_remote_debugging: bool) -> Self {
        self.enable_remote_debugging = enable_remote_debugging;
        self
    }
}

/// Stable identifier for a WebView instance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WebViewId(pub u64);

/// WebView error details for asynchronous callback/reporting paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebViewError {
    /// WebView failed to initialize.
    CreationFailed(String),
    /// A navigation request failed.
    NavigationFailed(String),
    /// Message bridge failed.
    MessageFailed(String),
    /// Any backend-specific error.
    Other(String),
}

/// WebView lifecycle and bridge events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebViewEvent {
    /// WebView is initialized and ready.
    Ready,
    /// Navigation started for a URL.
    NavigationStarted(String),
    /// Navigation finished for a URL.
    NavigationFinished(String),
    /// Message received from web content.
    Message(String),
    /// Error emitted by the backend.
    Error(WebViewError),
}

/// Callback invoked for WebView events.
pub type WebViewEventHandler = Box<dyn Fn(WebViewEvent) + Send + Sync + 'static>;

/// WebView abstraction used by platform backends.
pub trait WebView: Send {
    /// Get the stable WebView identifier.
    fn id(&self) -> WebViewId;

    /// Destroy this WebView and release platform resources.
    fn destroy(&mut self) -> Result<()>;

    /// Update WebView bounds within the host window.
    fn set_bounds(&self, bounds: WebViewBounds) -> Result<()>;

    /// Navigate WebView to the provided URL.
    fn navigate(&self, url: &str) -> Result<()>;

    /// Post a message to web content.
    fn post_message(&self, message: &str) -> Result<()>;

    /// Register or clear the event callback.
    fn set_event_handler(&mut self, handler: Option<WebViewEventHandler>) -> Result<()>;
}

/// Host capability for creating WebViews.
pub trait WebViewHost: Send + Sync {
    /// Concrete WebView type used by this host.
    type WebView: WebView;

    /// Create a new WebView instance.
    fn create_webview(&self, config: WebViewConfig) -> Result<Self::WebView>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webview_bounds_constructor() {
        let bounds = WebViewBounds::new(10.0, 20.0, 300.0, 200.0);

        assert_eq!(bounds.x, 10.0);
        assert_eq!(bounds.y, 20.0);
        assert_eq!(bounds.width, 300.0);
        assert_eq!(bounds.height, 200.0);
    }

    #[test]
    fn test_webview_config_builder() {
        let bounds = WebViewBounds::new(0.0, 0.0, 640.0, 480.0);
        let config = WebViewConfig::new()
            .initial_url("https://example.com")
            .bounds(bounds)
            .navigation_policy(WebViewNavigationPolicy::new().allow_origin("https://example.com"));

        assert_eq!(config.initial_url.as_deref(), Some("https://example.com"));
        assert_eq!(config.bounds, bounds);
        assert_eq!(
            config.navigation_policy.allowed_origins,
            vec!["https://example.com"]
        );
        assert!(!config.allow_file_access);
        assert!(!config.enable_remote_debugging);
    }

    #[test]
    fn test_navigation_policy_allows_origin() {
        let policy = WebViewNavigationPolicy::new().allow_origin("https://example.com");

        let decision = policy.evaluate("https://example.com/docs");

        assert_eq!(
            decision,
            WebViewNavigationDecision::Allowed {
                url: "https://example.com/docs".to_string(),
                origin: "https://example.com".to_string(),
            }
        );
    }

    #[test]
    fn test_navigation_policy_blocks_disallowed_origin() {
        let policy = WebViewNavigationPolicy::new().allow_origin("https://example.com");

        let decision = policy.evaluate("https://not-allowed.example/path");

        assert_eq!(
            decision,
            WebViewNavigationDecision::Blocked {
                url: "https://not-allowed.example/path".to_string(),
                origin: Some("https://not-allowed.example".to_string()),
                reason: WebViewNavigationBlockReason::OriginNotAllowed,
            }
        );
    }

    #[test]
    fn test_navigation_policy_blocks_malformed_url() {
        let policy = WebViewNavigationPolicy::new().allow_origin("https://example.com");

        let decision = policy.evaluate("not-a-valid-url");

        assert_eq!(
            decision,
            WebViewNavigationDecision::Blocked {
                url: "not-a-valid-url".to_string(),
                origin: None,
                reason: WebViewNavigationBlockReason::MalformedUrl,
            }
        );
    }
}

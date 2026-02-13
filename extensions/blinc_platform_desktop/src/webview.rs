//! Desktop WebView adapter surface.

use blinc_platform::{
    PlatformError, Result, WebView, WebViewBounds, WebViewConfig, WebViewEventHandler, WebViewHost,
    WebViewId,
};
use std::sync::{Arc, Mutex, MutexGuard};
use winit::window::Window as WinitWindow;

const BACKEND_UNAVAILABLE_REASON: &str =
    "desktop webview backend adapter is not linked in this build";

#[derive(Default)]
struct DesktopWebViewHostState {
    cleaned_up: bool,
}

/// Desktop-side host for creating and managing embedded WebViews.
#[derive(Clone)]
pub struct DesktopWebViewHost {
    window: Arc<WinitWindow>,
    state: Arc<Mutex<DesktopWebViewHostState>>,
}

impl DesktopWebViewHost {
    pub(crate) fn new(window: Arc<WinitWindow>) -> Self {
        Self {
            window,
            state: Arc::new(Mutex::new(DesktopWebViewHostState::default())),
        }
    }

    /// Lifecycle hook for window-size/scale updates.
    pub fn sync_bounds_with_window(&self) -> Result<()> {
        let state = self.lock_state()?;
        if state.cleaned_up {
            return Err(PlatformError::Unavailable(
                "desktop webview host already cleaned up".to_string(),
            ));
        }

        // Keep deterministic behavior until a concrete backend is linked.
        let _ = self.window.scale_factor();
        Ok(())
    }

    /// Lifecycle hook for explicit cleanup on window teardown.
    pub fn cleanup(&self) -> Result<()> {
        let mut state = self.lock_state()?;
        state.cleaned_up = true;
        Ok(())
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, DesktopWebViewHostState>> {
        self.state.lock().map_err(|_| {
            PlatformError::Other("desktop webview host state lock poisoned".to_string())
        })
    }

    fn unavailable_error(&self, action: &str) -> PlatformError {
        PlatformError::Unavailable(format!(
            "desktop webview {action} unavailable for window {:?}: {BACKEND_UNAVAILABLE_REASON}",
            self.window.id()
        ))
    }
}

impl WebViewHost for DesktopWebViewHost {
    type WebView = DesktopWebView;

    fn create_webview(&self, _config: WebViewConfig) -> Result<Self::WebView> {
        let state = self.lock_state()?;
        if state.cleaned_up {
            return Err(PlatformError::Unavailable(
                "desktop webview host already cleaned up".to_string(),
            ));
        }
        drop(state);

        Err(self.unavailable_error("creation"))
    }
}

/// Desktop-side WebView handle.
pub struct DesktopWebView {
    id: WebViewId,
}

impl WebView for DesktopWebView {
    fn id(&self) -> WebViewId {
        self.id
    }

    fn destroy(&mut self) -> Result<()> {
        Err(PlatformError::Unavailable(format!(
            "desktop webview destroy unavailable: {BACKEND_UNAVAILABLE_REASON}"
        )))
    }

    fn set_bounds(&self, _bounds: WebViewBounds) -> Result<()> {
        Err(PlatformError::Unavailable(format!(
            "desktop webview bounds update unavailable: {BACKEND_UNAVAILABLE_REASON}"
        )))
    }

    fn navigate(&self, _url: &str) -> Result<()> {
        Err(PlatformError::Unavailable(format!(
            "desktop webview navigation unavailable: {BACKEND_UNAVAILABLE_REASON}"
        )))
    }

    fn post_message(&self, _message: &str) -> Result<()> {
        Err(PlatformError::Unavailable(format!(
            "desktop webview messaging unavailable: {BACKEND_UNAVAILABLE_REASON}"
        )))
    }

    fn set_event_handler(&mut self, _handler: Option<WebViewEventHandler>) -> Result<()> {
        Err(PlatformError::Unavailable(format!(
            "desktop webview event handler unavailable: {BACKEND_UNAVAILABLE_REASON}"
        )))
    }
}

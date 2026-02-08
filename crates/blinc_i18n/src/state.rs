use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, RwLock};

use tracing::debug;

use crate::label::{Label, Message};
use crate::locale::{locale_fallback_chain, normalize_locale};
use crate::simple::SimpleCatalog;
use crate::I18nError;

#[cfg(feature = "fluent")]
use crate::fluent::FluentStore;

/// Global i18n singleton.
static I18N_STATE: OnceLock<I18nState> = OnceLock::new();

/// Global redraw callback - set by the app layer to trigger UI updates
static REDRAW_CALLBACK: Mutex<Option<fn()>> = Mutex::new(None);

/// Set the redraw callback function.
///
/// The app should set this to something like `request_full_rebuild()`.
pub fn set_redraw_callback(callback: fn()) {
    *REDRAW_CALLBACK.lock().unwrap() = Some(callback);
}

fn trigger_redraw() {
    if let Some(cb) = *REDRAW_CALLBACK.lock().unwrap() {
        cb();
    }
}

/// Runtime i18n state.
pub struct I18nState {
    locale: RwLock<String>,
    simple: RwLock<HashMap<String, SimpleCatalog>>,

    #[cfg(feature = "fluent")]
    fluent: RwLock<FluentStore>,
}

impl I18nState {
    /// Initialize the global i18n state.
    ///
    /// Safe to call multiple times; the first call wins.
    pub fn init(locale: impl Into<String>) {
        let loc = normalize_locale(&locale.into());
        let st = I18nState {
            locale: RwLock::new(if loc.is_empty() {
                "en-US".to_string()
            } else {
                loc
            }),
            simple: RwLock::new(HashMap::new()),
            #[cfg(feature = "fluent")]
            fluent: RwLock::new(FluentStore::new()),
        };

        let _ = I18N_STATE.set(st);
    }

    pub fn get() -> &'static I18nState {
        I18N_STATE
            .get()
            .expect("I18nState not initialized. Call I18nState::init() at app startup.")
    }

    pub fn try_get() -> Option<&'static I18nState> {
        I18N_STATE.get()
    }

    pub fn locale(&self) -> String {
        self.locale.read().unwrap().clone()
    }

    pub fn set_locale(&self, locale: impl Into<String>) {
        let loc = normalize_locale(&locale.into());
        if loc.is_empty() {
            return;
        }

        let mut cur = self.locale.write().unwrap();
        if *cur == loc {
            return;
        }
        debug!("I18nState::set_locale: {} -> {}", *cur, loc);
        *cur = loc;
        drop(cur);

        trigger_redraw();
    }

    /// Load a simple (key=value) catalog for a locale.
    pub fn load_simple_catalog(&self, locale: &str, catalog: SimpleCatalog) {
        let loc = normalize_locale(locale);
        self.simple.write().unwrap().insert(loc, catalog);
    }

    /// Parse and load a simple (key=value) catalog for a locale.
    pub fn load_simple_catalog_str(&self, locale: &str, src: &str) -> Result<(), I18nError> {
        let cat = SimpleCatalog::parse(src)?;
        self.load_simple_catalog(locale, cat);
        Ok(())
    }

    /// Load a Fluent (.ftl) catalog for a locale.
    #[cfg(feature = "fluent")]
    pub fn load_fluent_ftl(&self, locale: &str, ftl: &str) -> Result<(), I18nError> {
        self.fluent
            .write()
            .unwrap()
            .load_from_str(locale, ftl)
            .map_err(I18nError::Fluent)
    }

    /// Translate a message using the locale fallback chain.
    pub fn tr(&self, msg: &Message) -> String {
        let loc = self.locale();
        let chain = locale_fallback_chain(&loc);

        // 1) Fluent (if enabled)
        #[cfg(feature = "fluent")]
        {
            let fluent = self.fluent.read().unwrap();
            for l in &chain {
                if let Some(s) = fluent.format_message(l, msg) {
                    return s;
                }
            }
        }

        // 2) Simple catalog
        {
            let simple = self.simple.read().unwrap();
            for l in &chain {
                if let Some(cat) = simple.get(l) {
                    if let Some(s) = cat.format_message(msg) {
                        return s;
                    }
                }
            }
        }

        // Fallback: show the key id.
        msg.id.to_string()
    }

    pub fn resolve_label(&self, label: Label) -> String {
        match label {
            Label::Raw(s) => s,
            Label::Msg(m) => self.tr(&m),
        }
    }
}

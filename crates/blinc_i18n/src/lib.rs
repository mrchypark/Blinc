//! Blinc internationalization (i18n)
//!
//! Goals:
//! - Framework-level `Label` type (`text(label)`, `button(..., label)`, etc.)
//! - Runtime locale switching with an app-provided redraw callback
//! - Multiple translation backends behind a stable API:
//!   - `simple`: YAML mapping catalogs (default, with legacy key=value fallback)
//!   - `fluent`: Fluent (.ftl) catalogs (optional feature)

mod error;
mod label;
mod locale;
mod simple;
mod state;

#[cfg(feature = "fluent")]
mod fluent;

pub use error::I18nError;
pub use label::{ArgValue, Label, Message};
pub use locale::{locale_fallback_chain, normalize_locale};
pub use simple::{SimpleCatalog, SimpleParseError};
pub use state::{set_redraw_callback, I18nState};

/// Translate a label to a displayable string using the global [`I18nState`] (borrowed).
///
/// Prefer this overload in hot paths to avoid cloning `Label` values.
pub fn resolve_label_ref(label: &Label) -> String {
    if let Some(st) = I18nState::try_get() {
        st.resolve_label(label)
    } else {
        match label {
            Label::Raw(s) => s.clone(),
            Label::Msg(m) => m.id.to_string(),
        }
    }
}

/// Translate a label to a displayable string using the global [`I18nState`].
///
/// If the state isn't initialized, this degrades gracefully:
/// - `Label::Raw` returns its raw text
/// - `Label::Msg` returns the key id
pub fn resolve_label(label: Label) -> String {
    resolve_label_ref(&label)
}

/// Convenience macro for building a translation key + args as a [`Label`].
///
/// Examples:
/// - `t!("app.title")`
/// - `t!("greeting", { name: user_name, count: 3 })`
#[macro_export]
macro_rules! t {
    ($id:literal) => {
        $crate::Label::msg($crate::Message::new($id))
    };
    ($id:literal, { $($name:ident : $value:expr),* $(,)? }) => {{
        let mut m = $crate::Message::new($id);
        $(
            m = m.arg(stringify!($name), $value);
        )*
        $crate::Label::msg(m)
    }};
}

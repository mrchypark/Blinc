use std::collections::HashMap;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};

use unic_langid::LanguageIdentifier;

use crate::label::{ArgValue, Message};

use crate::locale::normalize_locale;

/// A Fluent bundle wrapper keyed by locale.
///
/// Note: we intentionally keep this thin; the stable API lives on `I18nState`.
#[derive(Default)]
pub struct FluentStore {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
}

impl FluentStore {
    pub fn new() -> Self {
        Self {
            bundles: HashMap::new(),
        }
    }

    pub fn load_from_str(&mut self, locale: &str, ftl: &str) -> Result<(), String> {
        let loc = normalize_locale(locale);
        let langid: LanguageIdentifier = loc
            .parse()
            .map_err(|e| format!("invalid locale `{}`: {}", loc, e))?;

        let res = FluentResource::try_new(ftl.to_string())
            .map_err(|(_res, errs)| format!("ftl parse error: {:?}", errs))?;

        let mut bundle = FluentBundle::new_concurrent(vec![langid]);
        bundle
            .add_resource(res)
            .map_err(|errs| format!("ftl add_resource error: {:?}", errs))?;

        self.bundles.insert(loc, bundle);
        Ok(())
    }

    pub fn format_message(&self, locale: &str, msg: &Message) -> Option<String> {
        let bundle = self.bundles.get(locale)?;
        let pattern = bundle.get_message(&msg.id)?.value()?;

        let mut args = FluentArgs::new();
        for (k, v) in &msg.args {
            match v {
                ArgValue::Str(s) => {
                    args.set(k.as_ref(), FluentValue::from(s.as_str()));
                }
                ArgValue::Int(i) => {
                    args.set(k.as_ref(), FluentValue::from(*i));
                }
                ArgValue::Float(f) => {
                    args.set(k.as_ref(), FluentValue::from(*f));
                }
                ArgValue::Bool(b) => {
                    // Fluent has no native bool; pass through as a string.
                    args.set(k.as_ref(), FluentValue::from(b.to_string()));
                }
            }
        }

        let mut errs = Vec::new();
        let s = bundle
            .format_pattern(pattern, Some(&args), &mut errs)
            .to_string();
        if !errs.is_empty() {
            tracing::warn!(
                locale = %locale,
                message_id = %msg.id,
                errors = ?errs,
                "Fluent formatting errors"
            );
        }
        Some(s)
    }
}

// Intentionally no shared conversion helper: `FluentValue` carries lifetimes.

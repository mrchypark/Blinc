use thiserror::Error;

use crate::simple::SimpleParseError;

#[derive(Debug, Error)]
pub enum I18nError {
    #[error(transparent)]
    SimpleParse(#[from] SimpleParseError),

    #[error("fluent error: {0}")]
    Fluent(String),
}

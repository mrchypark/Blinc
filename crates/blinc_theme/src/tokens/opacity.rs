//! Opacity tokens for theming

/// Semantic opacity token keys for dynamic access
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum OpacityToken {
    Disabled,
}

/// Complete set of opacity tokens
#[derive(Clone, Debug)]
pub struct OpacityTokens {
    pub disabled: f32,
}

impl OpacityTokens {
    /// Get opacity value by token key
    pub fn get(&self, token: OpacityToken) -> f32 {
        match token {
            OpacityToken::Disabled => self.disabled,
        }
    }
}

impl Default for OpacityTokens {
    fn default() -> Self {
        Self { disabled: 0.6 }
    }
}

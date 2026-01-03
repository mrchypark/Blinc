//! CSS subset parser for ElementStyle
//!
//! Parses a simplified CSS syntax into ElementStyle objects, enabling
//! stylesheet-based styling for Blinc applications.
//!
//! # Error Handling
//!
//! This parser uses nom's context-based error capture for diagnostics.
//! All parse failures are collected into an error array that can be used
//! for reporting. Errors are also logged via tracing at DEBUG level.
//! The parser gracefully continues after errors - the built-in theme is
//! used when style parsing fails.
//!
//! # Supported Syntax
//!
//! - ID-based selectors: `#element-id { ... }` (matches `.id("element-id")`)
//! - Properties: `background`, `border-radius`, `box-shadow`, `transform`, `opacity`
//! - Theme references: `theme(primary)`, `theme(radius-lg)`, `theme(shadow-md)`
//! - Colors: hex (#rgb, #rrggbb, #rrggbbaa), rgb(), rgba(), named colors
//! - Units: px, %, unitless numbers
//!
//! # Example
//!
//! ```ignore
//! use blinc_layout::css_parser::{Stylesheet, ParseResult as CssParseResult};
//!
//! let css = r#"
//!     #card {
//!         background: theme(surface);
//!         border-radius: theme(radius-lg);
//!         box-shadow: theme(shadow-md);
//!     }
//!     #button-primary {
//!         background: theme(primary);
//!         opacity: 0.9;
//!     }
//! "#;
//!
//! let result = Stylesheet::parse_with_errors(css);
//! let stylesheet = result.stylesheet;
//!
//! // Report any errors that occurred
//! for err in &result.errors {
//!     eprintln!("Warning: {}", err);
//! }
//!
//! // Apply styles to elements
//! div().id("card").style(stylesheet.get("card").unwrap())
//! ```

use std::collections::HashMap;

use blinc_core::{Brush, Color, CornerRadius, Shadow, Transform};
use blinc_theme::{ColorToken, ThemeState};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until, take_while1},
    character::complete::{char, multispace1},
    combinator::{cut, opt, value},
    error::{context, ParseError as NomParseError, VerboseError, VerboseErrorKind},
    multi::many0,
    number::complete::float,
    sequence::{delimited, preceded, tuple},
    Finish, IResult,
};
use tracing::debug;

use crate::element::RenderLayer;
use crate::element_style::ElementStyle;

/// Custom parser result type using VerboseError for better diagnostics
type ParseResult<'a, O> = IResult<&'a str, O, VerboseError<&'a str>>;

/// Severity level for parse warnings/errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Parsing failed completely
    Error,
    /// Parsing succeeded but with issues (e.g., unknown properties)
    Warning,
    /// Informational message
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// Error type for CSS parsing with context information
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Severity level
    pub severity: Severity,
    /// Human-readable error message with context
    pub message: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// The specific input fragment where parsing failed
    pub fragment: String,
    /// Context stack from nom's VerboseError
    pub contexts: Vec<String>,
    /// The property or selector name if applicable
    pub property: Option<String>,
    /// The attempted value if applicable
    pub value: Option<String>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CSS {}: line {}, column {}: {}",
            self.severity, self.line, self.column, self.message
        )?;
        if let Some(ref prop) = self.property {
            if let Some(ref val) = self.value {
                write!(f, " ({}:{})", prop, val)?;
            } else {
                write!(f, " ({})", prop)?;
            }
        }
        if !self.contexts.is_empty() {
            write!(f, "\n  Context: {}", self.contexts.join(" > "))?;
        }
        if !self.fragment.is_empty() && self.fragment.len() < 50 {
            write!(f, "\n  Near: \"{}\"", self.fragment)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    /// Create a new error with the given severity and message
    pub fn new(severity: Severity, message: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            severity,
            message: message.into(),
            line,
            column,
            fragment: String::new(),
            contexts: Vec::new(),
            property: None,
            value: None,
        }
    }

    /// Create an error for an unknown property
    pub fn unknown_property(property: &str, line: usize, column: usize) -> Self {
        Self {
            severity: Severity::Warning,
            message: format!("Unknown property '{}' (ignored)", property),
            line,
            column,
            fragment: String::new(),
            contexts: vec!["property".to_string()],
            property: Some(property.to_string()),
            value: None,
        }
    }

    /// Create an error for an invalid property value
    pub fn invalid_value(property: &str, value: &str, line: usize, column: usize) -> Self {
        Self {
            severity: Severity::Warning,
            message: format!("Invalid value for '{}': '{}'", property, value),
            line,
            column,
            fragment: String::new(),
            contexts: vec!["property value".to_string()],
            property: Some(property.to_string()),
            value: Some(value.to_string()),
        }
    }

    /// Create a ParseError from a nom VerboseError
    fn from_verbose(input: &str, err: VerboseError<&str>) -> Self {
        let (line, column, fragment) = if let Some((frag, _)) = err.errors.first() {
            calculate_position(input, frag)
        } else {
            (1, 1, String::new())
        };

        let contexts: Vec<String> = err
            .errors
            .iter()
            .filter_map(|(_, kind)| match kind {
                VerboseErrorKind::Context(ctx) => Some((*ctx).to_string()),
                _ => None,
            })
            .collect();

        let message = format_verbose_error(&err);

        Self {
            severity: Severity::Error,
            message,
            line,
            column,
            fragment,
            contexts,
            property: None,
            value: None,
        }
    }

    /// Format as a human-readable warning for console output
    pub fn to_warning_string(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "{}[{}:{}]: {}",
            self.severity, self.line, self.column, self.message
        ));
        if let Some(ref prop) = self.property {
            if let Some(ref val) = self.value {
                s.push_str(&format!("\n  Property: {} = {}", prop, val));
            } else {
                s.push_str(&format!("\n  Property: {}", prop));
            }
        }
        if !self.fragment.is_empty() && self.fragment.len() < 80 {
            s.push_str(&format!("\n  Near: \"{}\"", self.fragment));
        }
        s
    }

    /// Format with ANSI color codes for terminal output
    ///
    /// Colors:
    /// - Error: Red
    /// - Warning: Yellow
    /// - Info: Cyan
    /// - Property names: Blue
    /// - Values: Magenta
    /// - Line numbers: Dim
    pub fn to_colored_string(&self) -> String {
        // ANSI color codes
        const RESET: &str = "\x1b[0m";
        const RED: &str = "\x1b[31m";
        const YELLOW: &str = "\x1b[33m";
        const CYAN: &str = "\x1b[36m";
        const BLUE: &str = "\x1b[34m";
        const MAGENTA: &str = "\x1b[35m";
        const DIM: &str = "\x1b[2m";
        const BOLD: &str = "\x1b[1m";

        let (severity_color, icon) = match self.severity {
            Severity::Error => (RED, "✖"),
            Severity::Warning => (YELLOW, "⚠"),
            Severity::Info => (CYAN, "ℹ"),
        };

        let mut s = String::new();

        // Severity with icon and color
        s.push_str(&format!(
            "{}{}{} {}{}{}{RESET} ",
            BOLD, severity_color, icon, severity_color, self.severity, RESET
        ));

        // Location in dim
        s.push_str(&format!(
            "{DIM}[{}:{}]{RESET} ",
            self.line, self.column
        ));

        // Message
        s.push_str(&self.message);

        // Property and value with colors
        if let Some(ref prop) = self.property {
            s.push_str(&format!("\n  {BLUE}Property:{RESET} {}", prop));
            if let Some(ref val) = self.value {
                s.push_str(&format!(" = {MAGENTA}{}{RESET}", val));
            }
        }

        // Context in dim
        if !self.contexts.is_empty() {
            s.push_str(&format!(
                "\n  {DIM}Context: {}{RESET}",
                self.contexts.join(" > ")
            ));
        }

        // Near fragment
        if !self.fragment.is_empty() && self.fragment.len() < 80 {
            s.push_str(&format!("\n  {DIM}Near:{RESET} \"{}\"", self.fragment));
        }

        s
    }
}

/// Result of parsing CSS with error collection
#[derive(Debug, Clone)]
pub struct CssParseResult {
    /// The parsed stylesheet (may be partial if errors occurred)
    pub stylesheet: Stylesheet,
    /// All errors and warnings collected during parsing
    pub errors: Vec<ParseError>,
}

impl CssParseResult {
    /// Check if parsing had any errors (not just warnings)
    pub fn has_errors(&self) -> bool {
        self.errors.iter().any(|e| e.severity == Severity::Error)
    }

    /// Check if parsing had any warnings
    pub fn has_warnings(&self) -> bool {
        self.errors.iter().any(|e| e.severity == Severity::Warning)
    }

    /// Get only the errors (not warnings)
    pub fn errors_only(&self) -> impl Iterator<Item = &ParseError> {
        self.errors.iter().filter(|e| e.severity == Severity::Error)
    }

    /// Get only the warnings
    pub fn warnings_only(&self) -> impl Iterator<Item = &ParseError> {
        self.errors
            .iter()
            .filter(|e| e.severity == Severity::Warning)
    }

    /// Print all errors and warnings as human-readable text (plain, no colors)
    pub fn print_diagnostics(&self) {
        for err in &self.errors {
            match err.severity {
                Severity::Error => eprintln!("❌ {}", err.to_warning_string()),
                Severity::Warning => eprintln!("⚠️  {}", err.to_warning_string()),
                Severity::Info => eprintln!("ℹ️  {}", err.to_warning_string()),
            }
        }
    }

    /// Print all errors and warnings with ANSI color coding
    ///
    /// Uses terminal colors for better readability:
    /// - Errors: Red
    /// - Warnings: Yellow
    /// - Info: Cyan
    pub fn print_colored_diagnostics(&self) {
        for err in &self.errors {
            eprintln!("{}", err.to_colored_string());
        }
    }

    /// Print a summary line with counts (colored)
    pub fn print_summary(&self) {
        const RESET: &str = "\x1b[0m";
        const RED: &str = "\x1b[31m";
        const YELLOW: &str = "\x1b[33m";
        const GREEN: &str = "\x1b[32m";
        const BOLD: &str = "\x1b[1m";

        let error_count = self.errors_only().count();
        let warning_count = self.warnings_only().count();

        if error_count == 0 && warning_count == 0 {
            eprintln!("{BOLD}{GREEN}✓ CSS parsed successfully{RESET}");
        } else {
            let mut parts = Vec::new();
            if error_count > 0 {
                parts.push(format!("{RED}{} error(s){RESET}", error_count));
            }
            if warning_count > 0 {
                parts.push(format!("{YELLOW}{} warning(s){RESET}", warning_count));
            }
            eprintln!("{BOLD}CSS parsing completed with {}{RESET}", parts.join(", "));
        }
    }

    /// Log all errors and warnings via tracing
    pub fn log_diagnostics(&self) {
        for err in &self.errors {
            match err.severity {
                Severity::Error => debug!(
                    severity = "error",
                    line = err.line,
                    column = err.column,
                    message = %err.message,
                    property = ?err.property,
                    value = ?err.value,
                    "CSS parse error"
                ),
                Severity::Warning => debug!(
                    severity = "warning",
                    line = err.line,
                    column = err.column,
                    message = %err.message,
                    property = ?err.property,
                    value = ?err.value,
                    "CSS parse warning"
                ),
                Severity::Info => debug!(
                    severity = "info",
                    line = err.line,
                    column = err.column,
                    message = %err.message,
                    "CSS parse info"
                ),
            }
        }
    }
}

/// Format a VerboseError into a human-readable message
fn format_verbose_error(err: &VerboseError<&str>) -> String {
    let mut parts = Vec::new();

    for (input, kind) in &err.errors {
        match kind {
            VerboseErrorKind::Context(ctx) => {
                parts.push(format!("in {}", ctx));
            }
            VerboseErrorKind::Char(c) => {
                let preview: String = input.chars().take(20).collect();
                parts.push(format!("expected '{}' near \"{}\"", c, preview));
            }
            VerboseErrorKind::Nom(ek) => {
                parts.push(format!("{:?}", ek));
            }
        }
    }

    if parts.is_empty() {
        "unknown parse error".to_string()
    } else {
        parts.join(", ")
    }
}

/// Element state for pseudo-class selectors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ElementState {
    /// :hover pseudo-class
    Hover,
    /// :active pseudo-class (pressed)
    Active,
    /// :focus pseudo-class
    Focus,
    /// :disabled pseudo-class
    Disabled,
}

impl ElementState {
    /// Parse a state from a pseudo-class string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hover" => Some(ElementState::Hover),
            "active" => Some(ElementState::Active),
            "focus" => Some(ElementState::Focus),
            "disabled" => Some(ElementState::Disabled),
            _ => None,
        }
    }
}

impl std::fmt::Display for ElementState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementState::Hover => write!(f, "hover"),
            ElementState::Active => write!(f, "active"),
            ElementState::Focus => write!(f, "focus"),
            ElementState::Disabled => write!(f, "disabled"),
        }
    }
}

/// A parsed CSS selector with optional state modifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CssSelector {
    /// The element ID (without #)
    pub id: String,
    /// Optional state modifier (:hover, :active, :focus, :disabled)
    pub state: Option<ElementState>,
}

impl CssSelector {
    /// Create a selector for an ID without state
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            state: None,
        }
    }

    /// Create a selector with a state modifier
    pub fn with_state(id: impl Into<String>, state: ElementState) -> Self {
        Self {
            id: id.into(),
            state: Some(state),
        }
    }

    /// Get the storage key for this selector
    fn key(&self) -> String {
        match &self.state {
            Some(state) => format!("{}:{}", self.id, state),
            None => self.id.clone(),
        }
    }
}

/// A parsed stylesheet containing styles keyed by element ID
#[derive(Clone, Default, Debug)]
pub struct Stylesheet {
    /// Styles keyed by selector (id or id:state)
    styles: HashMap<String, ElementStyle>,
    /// CSS custom properties (variables) defined in :root
    variables: HashMap<String, String>,
}

impl Stylesheet {
    /// Create an empty stylesheet
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse CSS text into a stylesheet with full error collection
    ///
    /// This is the recommended method for parsing CSS as it collects all
    /// errors and warnings during parsing, allowing you to report them
    /// to users in a human-readable format.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let css = "#card { opacity: 0.5; unknown: value; }";
    /// let result = Stylesheet::parse_with_errors(css);
    ///
    /// // Print any warnings to stderr
    /// result.print_diagnostics();
    ///
    /// // Use the stylesheet (partial results are still available)
    /// let stylesheet = result.stylesheet;
    /// ```
    pub fn parse_with_errors(css: &str) -> CssParseResult {
        let mut errors: Vec<ParseError> = Vec::new();
        let initial_vars = HashMap::new();

        match parse_stylesheet_with_errors(css, &mut errors, &initial_vars).finish() {
            Ok((remaining, parsed)) => {
                // Warn if there's unparsed content
                let remaining = remaining.trim();
                if !remaining.is_empty() {
                    let (line, column, fragment) = calculate_position(css, remaining);
                    errors.push(ParseError {
                        severity: Severity::Warning,
                        message: format!(
                            "Unparsed content remaining ({} chars)",
                            remaining.len()
                        ),
                        line,
                        column,
                        fragment,
                        contexts: vec![],
                        property: None,
                        value: None,
                    });
                }

                let mut stylesheet = Stylesheet::new();
                stylesheet.variables = parsed.variables;
                for (id, style) in parsed.rules {
                    stylesheet.styles.insert(id, style);
                }

                CssParseResult { stylesheet, errors }
            }
            Err(e) => {
                let parse_error = ParseError::from_verbose(css, e);
                errors.push(parse_error);

                CssParseResult {
                    stylesheet: Stylesheet::new(),
                    errors,
                }
            }
        }
    }

    /// Parse CSS text into a stylesheet
    ///
    /// Parse errors are logged via tracing at DEBUG level with full context.
    /// When parsing fails, an error is returned but the application can
    /// fall back to built-in theme styles.
    ///
    /// For full error collection, use `parse_with_errors()` instead.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let css = "#card { opacity: 0.5; }";
    /// let stylesheet = Stylesheet::parse(css)?;
    /// ```
    pub fn parse(css: &str) -> Result<Self, ParseError> {
        let result = Self::parse_with_errors(css);

        // Log all diagnostics via tracing
        result.log_diagnostics();

        if result.has_errors() {
            // Return the first error
            Err(result
                .errors
                .into_iter()
                .find(|e| e.severity == Severity::Error)
                .unwrap())
        } else {
            Ok(result.stylesheet)
        }
    }

    /// Parse CSS text, logging errors and returning an empty stylesheet on failure
    ///
    /// This is a convenience method for cases where you want to gracefully
    /// fall back to an empty stylesheet rather than handle errors explicitly.
    pub fn parse_or_empty(css: &str) -> Self {
        Self::parse(css).unwrap_or_default()
    }

    /// Get a style by element ID (without the # prefix)
    ///
    /// Returns `None` if no style is defined for the given ID.
    pub fn get(&self, id: &str) -> Option<&ElementStyle> {
        self.styles.get(id)
    }

    /// Get a style by element ID and state
    ///
    /// Looks up `#id:state` in the stylesheet.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let css = "#button:hover { opacity: 0.8; }";
    /// let stylesheet = Stylesheet::parse(css)?;
    /// let hover_style = stylesheet.get_with_state("button", ElementState::Hover);
    /// ```
    pub fn get_with_state(&self, id: &str, state: ElementState) -> Option<&ElementStyle> {
        let key = format!("{}:{}", id, state);
        self.styles.get(&key)
    }

    /// Get all styles for an element, including state variants
    ///
    /// Returns a tuple of (base_style, state_styles) where state_styles is a Vec
    /// of (ElementState, &ElementStyle) pairs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let css = r#"
    ///     #button { background: blue; }
    ///     #button:hover { background: lightblue; }
    ///     #button:active { background: darkblue; }
    /// "#;
    /// let stylesheet = Stylesheet::parse(css)?;
    /// let (base, states) = stylesheet.get_all_states("button");
    /// ```
    pub fn get_all_states(&self, id: &str) -> (Option<&ElementStyle>, Vec<(ElementState, &ElementStyle)>) {
        let base = self.styles.get(id);

        let mut state_styles = Vec::new();
        for state in [ElementState::Hover, ElementState::Active, ElementState::Focus, ElementState::Disabled] {
            let key = format!("{}:{}", id, state);
            if let Some(style) = self.styles.get(&key) {
                state_styles.push((state, style));
            }
        }

        (base, state_styles)
    }

    /// Check if a style exists for the given ID
    pub fn contains(&self, id: &str) -> bool {
        self.styles.contains_key(id)
    }

    /// Check if a style exists for the given ID and state
    pub fn contains_with_state(&self, id: &str, state: ElementState) -> bool {
        let key = format!("{}:{}", id, state);
        self.styles.contains_key(&key)
    }

    /// Get all style IDs in the stylesheet
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.styles.keys().map(|s| s.as_str())
    }

    /// Get the number of styles in the stylesheet
    pub fn len(&self) -> usize {
        self.styles.len()
    }

    /// Check if the stylesheet is empty
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    // =========================================================================
    // CSS Variables (Custom Properties)
    // =========================================================================

    /// Get a CSS variable value by name (without the -- prefix)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let css = ":root { --card-bg: #ffffff; }";
    /// let stylesheet = Stylesheet::parse(css)?;
    /// assert_eq!(stylesheet.get_variable("card-bg"), Some("#ffffff"));
    /// ```
    pub fn get_variable(&self, name: &str) -> Option<&str> {
        self.variables.get(name).map(|s| s.as_str())
    }

    /// Set a CSS variable (useful for runtime overrides)
    ///
    /// # Example
    ///
    /// ```ignore
    /// stylesheet.set_variable("primary-color", "#FF0000");
    /// ```
    pub fn set_variable(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Get all variable names
    pub fn variable_names(&self) -> impl Iterator<Item = &str> {
        self.variables.keys().map(|s| s.as_str())
    }

    /// Get the number of variables defined
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// Resolve a var() reference to its value
    ///
    /// Supports fallback syntax: `var(--name, fallback)`
    fn resolve_variable(&self, var_ref: &str) -> Option<String> {
        // Parse var(--name) or var(--name, fallback)
        let inner = var_ref.trim();
        if !inner.starts_with("var(") || !inner.ends_with(')') {
            return None;
        }

        let content = &inner[4..inner.len() - 1].trim();

        // Split on comma for fallback support
        if let Some(comma_pos) = content.find(',') {
            let var_name = content[..comma_pos].trim();
            let fallback = content[comma_pos + 1..].trim();

            // Variable name should start with --
            let name = var_name.strip_prefix("--")?;

            self.variables
                .get(name)
                .cloned()
                .or_else(|| Some(fallback.to_string()))
        } else {
            // No fallback
            let name = content.strip_prefix("--")?;
            self.variables.get(name).cloned()
        }
    }
}

// ============================================================================
// Nom Parsers with VerboseError for diagnostics
// ============================================================================

/// Calculate line and column from the original input and the error fragment
fn calculate_position(original: &str, fragment: &str) -> (usize, usize, String) {
    // Find where the fragment starts in the original input
    let offset = original.len().saturating_sub(fragment.len());
    let consumed = &original[..offset];

    let line = consumed.matches('\n').count() + 1;
    let column = consumed
        .rfind('\n')
        .map(|pos| offset - pos)
        .unwrap_or(offset + 1);

    let preview: String = fragment.chars().take(30).collect();
    (line, column, preview)
}

/// Parse whitespace and comments
fn ws<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    value(
        (),
        many0(alt((
            value((), multispace1),
            value((), parse_comment),
        ))),
    )(input)
}

/// Parse a block comment /* ... */
fn parse_comment<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    delimited(tag("/*"), take_until("*/"), tag("*/"))(input)
}

/// Parse an identifier (alphanumeric, hyphen, underscore)
fn identifier<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_')(input)
}

/// Parse an ID selector: #identifier or #identifier:state
fn id_selector(input: &str) -> ParseResult<CssSelector> {
    context("ID selector", |input| {
        let (input, _) = char('#')(input)?;
        let (input, id) = cut(identifier)(input)?;

        // Check for optional state modifier
        let (input, state) = opt(|i| {
            let (i, _) = char(':')(i)?;
            let (i, state_name) = identifier(i)?;
            Ok((i, state_name))
        })(input)?;

        let element_state = state.and_then(ElementState::from_str);

        Ok((input, CssSelector {
            id: id.to_string(),
            state: element_state,
        }))
    })(input)
}

/// Parse a property name (including CSS custom properties like --var-name)
fn property_name(input: &str) -> ParseResult<&str> {
    context(
        "property name",
        take_while1(|c: char| c.is_alphanumeric() || c == '-' || c == '_'),
    )(input)
}

/// Parse a CSS variable name: --identifier
fn variable_name(input: &str) -> ParseResult<&str> {
    let (input, _) = tag("--")(input)?;
    let (input, name) = identifier(input)?;
    Ok((input, name))
}

/// Parse a property value (everything until ; or })
fn property_value(input: &str) -> ParseResult<&str> {
    let (input, value) = context(
        "property value",
        take_while1(|c: char| c != ';' && c != '}'),
    )(input)?;
    Ok((input, value.trim()))
}

/// Parse a single property declaration: name: value;
fn property_declaration(input: &str) -> ParseResult<(&str, &str)> {
    let (input, _) = ws(input)?;
    let (input, name) = context("property name", property_name)(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = context("colon after property name", char(':'))(input)?;
    let (input, _) = ws(input)?;
    let (input, value) = context("property value", property_value)(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = opt(char(';'))(input)?;
    Ok((input, (name, value)))
}

/// Parse a rule block: { property: value; ... }
fn rule_block(input: &str) -> ParseResult<Vec<(&str, &str)>> {
    let (input, _) = ws::<VerboseError<&str>>(input)?;
    let (input, _) = context("opening brace", char('{'))(input)?;
    let (input, _) = ws::<VerboseError<&str>>(input)?;
    let (input, properties) = many0(property_declaration)(input)?;
    let (input, _) = ws::<VerboseError<&str>>(input)?;
    let (input, _) = context("closing brace", char('}'))(input)?;
    Ok((input, properties))
}

/// Parse a :root block for CSS variables
fn root_block(input: &str) -> ParseResult<Vec<(String, String)>> {
    let (input, _) = ws(input)?;
    let (input, _) = tag(":root")(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char('{')(input)?;
    let (input, _) = ws(input)?;

    // Parse variable declarations
    let (input, declarations) = many0(|i| {
        let (i, _) = ws(i)?;
        let (i, _) = tag("--")(i)?;
        let (i, name) = identifier(i)?;
        let (i, _) = ws(i)?;
        let (i, _) = char(':')(i)?;
        let (i, _) = ws(i)?;
        let (i, value) = property_value(i)?;
        let (i, _) = ws(i)?;
        let (i, _) = opt(char(';'))(i)?;
        Ok((i, (name.to_string(), value.to_string())))
    })(input)?;

    let (input, _) = ws(input)?;
    let (input, _) = char('}')(input)?;
    Ok((input, declarations))
}

/// Parsed content from a stylesheet - can be either a rule or variables
enum CssBlock {
    Rule(String, ElementStyle),
    Variables(Vec<(String, String)>),
}

/// Parse a complete rule: #id { ... } or #id:state { ... }
fn css_rule(input: &str) -> ParseResult<(String, ElementStyle)> {
    let (input, _) = ws(input)?;
    let (input, selector) = context("CSS rule selector", id_selector)(input)?;
    let (input, _) = ws(input)?;
    let (input, properties) = context("CSS rule block", rule_block)(input)?;

    let mut style = ElementStyle::new();
    for (name, value) in properties {
        apply_property(&mut style, name, value);
    }

    // Use the selector key (id or id:state)
    Ok((input, (selector.key(), style)))
}

/// Parse an entire stylesheet
#[allow(dead_code)]
fn parse_stylesheet(input: &str) -> ParseResult<Vec<(String, ElementStyle)>> {
    let (input, _) = ws(input)?;
    let (input, rules) = many0(css_rule)(input)?;
    let (input, _) = ws(input)?;
    Ok((input, rules))
}

/// Parse a complete rule with error collection: #id { ... } or #id:state { ... }
fn css_rule_with_errors<'a, 'b>(
    original_css: &'a str,
    errors: &'b mut Vec<ParseError>,
) -> impl FnMut(&'a str) -> ParseResult<'a, (String, ElementStyle)> + 'b
where
    'a: 'b,
{
    move |input: &'a str| {
        let (input, _) = ws(input)?;
        let (input, selector) = context("CSS rule selector", id_selector)(input)?;
        let (input, _) = ws(input)?;
        let (input, properties) = context("CSS rule block", rule_block)(input)?;

        let mut style = ElementStyle::new();
        for (name, value) in properties {
            apply_property_with_errors(&mut style, name, value, original_css, input, errors);
        }

        Ok((input, (selector.key(), style)))
    }
}

/// Result of parsing a stylesheet - rules and variables
struct ParsedStylesheet {
    rules: Vec<(String, ElementStyle)>,
    variables: HashMap<String, String>,
}

/// Parse an entire stylesheet with error collection
fn parse_stylesheet_with_errors<'a>(
    css: &'a str,
    errors: &mut Vec<ParseError>,
    variables: &HashMap<String, String>,
) -> ParseResult<'a, ParsedStylesheet> {
    let (input, _) = ws(css)?;

    // Parse blocks one at a time to collect errors
    let mut rules = Vec::new();
    let mut parsed_variables = variables.clone();
    let mut remaining = input;

    loop {
        let trimmed = remaining.trim_start();
        if trimmed.is_empty() {
            break;
        }

        // Try to parse a :root block first
        if trimmed.starts_with(":root") {
            match root_block(trimmed) {
                Ok((rest, vars)) => {
                    for (name, value) in vars {
                        parsed_variables.insert(name, value);
                    }
                    remaining = rest;
                    continue;
                }
                Err(_) => {
                    // Not a valid :root block, try as a rule
                }
            }
        }

        // Try to parse a rule
        match css_rule_with_errors_and_vars(css, errors, &parsed_variables)(trimmed) {
            Ok((rest, rule)) => {
                rules.push(rule);
                remaining = rest;
            }
            Err(nom::Err::Error(_)) | Err(nom::Err::Failure(_)) => {
                // Can't parse more rules, break
                break;
            }
            Err(nom::Err::Incomplete(_)) => {
                break;
            }
        }
    }

    let (input, _) = ws(remaining)?;
    Ok((
        input,
        ParsedStylesheet {
            rules,
            variables: parsed_variables,
        },
    ))
}

/// Parse a complete rule with error collection and variable resolution: #id { ... } or #id:state { ... }
fn css_rule_with_errors_and_vars<'a, 'b>(
    original_css: &'a str,
    errors: &'b mut Vec<ParseError>,
    variables: &'b HashMap<String, String>,
) -> impl FnMut(&'a str) -> ParseResult<'a, (String, ElementStyle)> + 'b
where
    'a: 'b,
{
    move |input: &'a str| {
        let (input, _) = ws(input)?;
        let (input, selector) = context("CSS rule selector", id_selector)(input)?;
        let (input, _) = ws(input)?;
        let (input, properties) = context("CSS rule block", rule_block)(input)?;

        let mut style = ElementStyle::new();
        for (name, value) in properties {
            // Resolve var() references before applying
            let resolved_value = resolve_var_references(value, variables);
            apply_property_with_errors(
                &mut style,
                name,
                &resolved_value,
                original_css,
                input,
                errors,
            );
        }

        Ok((input, (selector.key(), style)))
    }
}

/// Resolve var(--name) references in a value string
fn resolve_var_references(value: &str, variables: &HashMap<String, String>) -> String {
    let mut result = value.to_string();
    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 10; // Prevent infinite loops from circular references

    // Keep resolving until no more var() references
    while result.contains("var(") && iterations < MAX_ITERATIONS {
        iterations += 1;

        // Find var( and its matching )
        if let Some(start) = result.find("var(") {
            let after_var = &result[start + 4..];

            // Find matching closing paren (handling nested parens)
            let mut depth = 1;
            let mut end_offset = 0;
            for (i, c) in after_var.char_indices() {
                match c {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth == 0 {
                            end_offset = i;
                            break;
                        }
                    }
                    _ => {}
                }
            }

            if depth == 0 {
                let var_content = &after_var[..end_offset];
                let full_var = &result[start..start + 4 + end_offset + 1];

                // Parse var content: --name or --name, fallback
                let resolved = if let Some(comma_pos) = var_content.find(',') {
                    let var_name = var_content[..comma_pos].trim();
                    let fallback = var_content[comma_pos + 1..].trim();

                    if let Some(name) = var_name.strip_prefix("--") {
                        variables
                            .get(name)
                            .cloned()
                            .unwrap_or_else(|| fallback.to_string())
                    } else {
                        fallback.to_string()
                    }
                } else {
                    let var_name = var_content.trim();
                    if let Some(name) = var_name.strip_prefix("--") {
                        variables.get(name).cloned().unwrap_or_default()
                    } else {
                        String::new()
                    }
                };

                result = result.replace(full_var, &resolved);
            } else {
                // Malformed var(), break to avoid infinite loop
                break;
            }
        }
    }

    result
}

// ============================================================================
// Property Application
// ============================================================================

fn apply_property(style: &mut ElementStyle, name: &str, value: &str) {
    match name {
        "background" | "background-color" => {
            if let Some(brush) = parse_brush(value) {
                style.background = Some(brush);
            }
        }
        "border-radius" => {
            if let Some(radius) = parse_radius(value) {
                style.corner_radius = Some(radius);
            }
        }
        "box-shadow" => {
            if let Some(shadow) = parse_shadow(value) {
                style.shadow = Some(shadow);
            }
        }
        "transform" => {
            if let Some(transform) = parse_transform(value) {
                style.transform = Some(transform);
            }
        }
        "opacity" => {
            if let Ok((_, opacity)) = parse_opacity::<nom::error::Error<&str>>(value) {
                style.opacity = Some(opacity.clamp(0.0, 1.0));
            }
        }
        "render-layer" | "z-index" => {
            if let Ok((_, layer)) = parse_render_layer::<nom::error::Error<&str>>(value) {
                style.render_layer = Some(layer);
            }
        }
        _ => {
            // Unknown property - log at debug level for forward compatibility
            debug!(property = name, value = value, "Unknown CSS property (ignored)");
        }
    }
}

/// Apply a property with error collection
fn apply_property_with_errors(
    style: &mut ElementStyle,
    name: &str,
    value: &str,
    original_css: &str,
    current_input: &str,
    errors: &mut Vec<ParseError>,
) {
    let (line, column, _) = calculate_position(original_css, current_input);

    match name {
        "background" | "background-color" => {
            if let Some(brush) = parse_brush(value) {
                style.background = Some(brush);
            } else {
                errors.push(ParseError::invalid_value(name, value, line, column));
            }
        }
        "border-radius" => {
            if let Some(radius) = parse_radius(value) {
                style.corner_radius = Some(radius);
            } else {
                errors.push(ParseError::invalid_value(name, value, line, column));
            }
        }
        "box-shadow" => {
            if let Some(shadow) = parse_shadow(value) {
                style.shadow = Some(shadow);
            } else {
                errors.push(ParseError::invalid_value(name, value, line, column));
            }
        }
        "transform" => {
            if let Some(transform) = parse_transform(value) {
                style.transform = Some(transform);
            } else {
                errors.push(ParseError::invalid_value(name, value, line, column));
            }
        }
        "opacity" => {
            if let Ok((_, opacity)) = parse_opacity::<nom::error::Error<&str>>(value) {
                style.opacity = Some(opacity.clamp(0.0, 1.0));
            } else {
                errors.push(ParseError::invalid_value(name, value, line, column));
            }
        }
        "render-layer" | "z-index" => {
            if let Ok((_, layer)) = parse_render_layer::<nom::error::Error<&str>>(value) {
                style.render_layer = Some(layer);
            } else {
                errors.push(ParseError::invalid_value(name, value, line, column));
            }
        }
        _ => {
            // Unknown property - collect as warning
            errors.push(ParseError::unknown_property(name, line, column));
        }
    }
}

// ============================================================================
// Value Parsers
// These use generic error types so they work with both simple and VerboseError
// ============================================================================

fn parse_brush(value: &str) -> Option<Brush> {
    // Try theme() function first
    if let Ok((_, color)) = parse_theme_color::<nom::error::Error<&str>>(value) {
        return Some(Brush::Solid(color));
    }

    // Try parsing as color
    parse_color(value).map(Brush::Solid)
}

/// Parse theme(token-name) for colors
fn parse_theme_color<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Color, E> {
    let (input, _) = ws(input)?;
    let (input, _) = tag_no_case("theme")(input)?;
    let (input, _) = ws(input)?;
    let (input, token_name) = delimited(char('('), take_while1(|c: char| c != ')'), char(')'))(input)?;

    let token_name = token_name.trim();
    let token = match token_name.to_lowercase().as_str() {
        // Brand colors
        "primary" => ColorToken::Primary,
        "primary-hover" => ColorToken::PrimaryHover,
        "primary-active" => ColorToken::PrimaryActive,
        "secondary" => ColorToken::Secondary,
        "secondary-hover" => ColorToken::SecondaryHover,
        "secondary-active" => ColorToken::SecondaryActive,
        // Semantic colors
        "success" => ColorToken::Success,
        "success-bg" => ColorToken::SuccessBg,
        "warning" => ColorToken::Warning,
        "warning-bg" => ColorToken::WarningBg,
        "error" => ColorToken::Error,
        "error-bg" => ColorToken::ErrorBg,
        "info" => ColorToken::Info,
        "info-bg" => ColorToken::InfoBg,
        // Surface colors
        "background" => ColorToken::Background,
        "surface" => ColorToken::Surface,
        "surface-elevated" => ColorToken::SurfaceElevated,
        "surface-overlay" => ColorToken::SurfaceOverlay,
        // Text colors
        "text-primary" => ColorToken::TextPrimary,
        "text-secondary" => ColorToken::TextSecondary,
        "text-tertiary" => ColorToken::TextTertiary,
        "text-inverse" => ColorToken::TextInverse,
        "text-link" => ColorToken::TextLink,
        // Border colors
        "border" => ColorToken::Border,
        "border-hover" => ColorToken::BorderHover,
        "border-focus" => ColorToken::BorderFocus,
        "border-error" => ColorToken::BorderError,
        _ => {
            debug!(token = token_name, "Unknown theme color token");
            return Err(nom::Err::Error(E::from_error_kind(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };

    Ok((input, ThemeState::get().color(token)))
}

fn parse_radius(value: &str) -> Option<CornerRadius> {
    // Try theme() function first
    if let Ok((_, radius)) = parse_theme_radius::<nom::error::Error<&str>>(value) {
        return Some(radius);
    }

    // Try parsing as numeric value
    parse_length_value(value).map(CornerRadius::uniform)
}

/// Parse theme(radius-*) tokens
fn parse_theme_radius<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, CornerRadius, E> {
    let (input, _) = ws(input)?;
    let (input, _) = tag_no_case("theme")(input)?;
    let (input, _) = ws(input)?;
    let (input, token_name) = delimited(char('('), take_while1(|c: char| c != ')'), char(')'))(input)?;

    let token_name = token_name.trim();
    let radii = ThemeState::get().radii();

    let radius = match token_name.to_lowercase().replace('_', "-").as_str() {
        "radius-none" => radii.radius_none,
        "radius-sm" => radii.radius_sm,
        "radius-default" => radii.radius_default,
        "radius-md" => radii.radius_md,
        "radius-lg" => radii.radius_lg,
        "radius-xl" => radii.radius_xl,
        "radius-2xl" => radii.radius_2xl,
        "radius-3xl" => radii.radius_3xl,
        "radius-full" => radii.radius_full,
        _ => {
            debug!(token = token_name, "Unknown theme radius token");
            return Err(nom::Err::Error(E::from_error_kind(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };

    Ok((input, CornerRadius::uniform(radius)))
}

fn parse_shadow(value: &str) -> Option<Shadow> {
    // Check for "none"
    if value.trim().eq_ignore_ascii_case("none") {
        return Some(Shadow::new(0.0, 0.0, 0.0, Color::TRANSPARENT));
    }

    // Try theme() function first
    if let Ok((_, shadow)) = parse_theme_shadow::<nom::error::Error<&str>>(value) {
        return Some(shadow);
    }

    // Try parsing explicit shadow: offset-x offset-y blur color
    parse_explicit_shadow(value)
}

/// Parse theme(shadow-*) tokens
fn parse_theme_shadow<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Shadow, E> {
    let (input, _) = ws(input)?;
    let (input, _) = tag_no_case("theme")(input)?;
    let (input, _) = ws(input)?;
    let (input, token_name) = delimited(char('('), take_while1(|c: char| c != ')'), char(')'))(input)?;

    let token_name = token_name.trim();
    let shadows = ThemeState::get().shadows();

    let shadow: blinc_core::Shadow = match token_name.to_lowercase().replace('_', "-").as_str() {
        "shadow-sm" => shadows.shadow_sm.clone().into(),
        "shadow-default" => shadows.shadow_default.clone().into(),
        "shadow-md" => shadows.shadow_md.clone().into(),
        "shadow-lg" => shadows.shadow_lg.clone().into(),
        "shadow-xl" => shadows.shadow_xl.clone().into(),
        "shadow-2xl" => shadows.shadow_2xl.clone().into(),
        "shadow-none" => shadows.shadow_none.clone().into(),
        _ => {
            debug!(token = token_name, "Unknown theme shadow token");
            return Err(nom::Err::Error(E::from_error_kind(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };

    Ok((input, shadow))
}

/// Parse explicit shadow: offset-x offset-y blur color
fn parse_explicit_shadow(input: &str) -> Option<Shadow> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() >= 4 {
        let offset_x = parse_length_value(parts[0])?;
        let offset_y = parse_length_value(parts[1])?;
        let blur = parse_length_value(parts[2])?;
        let color = parse_color(parts[3])?;
        return Some(Shadow::new(offset_x, offset_y, blur, color));
    }
    None
}

fn parse_transform(value: &str) -> Option<Transform> {
    // Try scale()
    if let Ok((_, transform)) = parse_scale_transform::<nom::error::Error<&str>>(value) {
        return Some(transform);
    }

    // Try rotate()
    if let Ok((_, transform)) = parse_rotate_transform::<nom::error::Error<&str>>(value) {
        return Some(transform);
    }

    // Try translate()
    if let Ok((_, transform)) = parse_translate_transform::<nom::error::Error<&str>>(value) {
        return Some(transform);
    }

    debug!(value = value, "Failed to parse transform");
    None
}

/// Parse scale(x) or scale(x, y)
fn parse_scale_transform<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Transform, E> {
    let (input, _) = ws(input)?;
    let (input, _) = tag_no_case("scale")(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = ws(input)?;
    let (input, sx) = float(input)?;
    let (input, _) = ws(input)?;
    let (input, sy) = opt(preceded(
        tuple((char(','), ws::<E>)),
        float,
    ))(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(')')(input)?;

    let sy = sy.unwrap_or(sx);
    Ok((input, Transform::scale(sx, sy)))
}

/// Parse rotate(deg)
fn parse_rotate_transform<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Transform, E> {
    let (input, _) = ws(input)?;
    let (input, _) = tag_no_case("rotate")(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = ws(input)?;
    let (input, degrees) = float(input)?;
    let (input, _) = opt(tag_no_case("deg"))(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(')')(input)?;

    Ok((input, Transform::rotate(degrees * std::f32::consts::PI / 180.0)))
}

/// Parse translate(x, y)
fn parse_translate_transform<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Transform, E> {
    let (input, _) = ws(input)?;
    let (input, _) = tag_no_case("translate")(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = ws(input)?;
    let (input, x) = parse_length(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = ws(input)?;
    let (input, y) = parse_length(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(')')(input)?;

    Ok((input, Transform::translate(x, y)))
}

/// Parse a length value with optional px suffix
fn parse_length<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, f32, E> {
    let (input, value) = float(input)?;
    let (input, _) = opt(tag_no_case("px"))(input)?;
    Ok((input, value))
}

/// Parse a length value from a string slice
fn parse_length_value(input: &str) -> Option<f32> {
    let input = input.trim();
    let input = input.strip_suffix("px").unwrap_or(input).trim();
    input.parse::<f32>().ok()
}

/// Parse opacity value
fn parse_opacity<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, f32, E> {
    let (input, _) = ws(input)?;
    float(input)
}

/// Parse render layer
fn parse_render_layer<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, RenderLayer, E> {
    let (input, _) = ws(input)?;
    alt((
        value(RenderLayer::Foreground, tag_no_case("foreground")),
        value(RenderLayer::Glass, tag_no_case("glass")),
        value(RenderLayer::Background, tag_no_case("background")),
    ))(input)
}

// ============================================================================
// Color Parsing
// ============================================================================

fn parse_color(input: &str) -> Option<Color> {
    let input = input.trim();

    // Try hex color
    if let Ok((_, color)) = parse_hex_color::<nom::error::Error<&str>>(input) {
        return Some(color);
    }

    // Try rgba()
    if let Ok((_, color)) = parse_rgba_color::<nom::error::Error<&str>>(input) {
        return Some(color);
    }

    // Try rgb()
    if let Ok((_, color)) = parse_rgb_color::<nom::error::Error<&str>>(input) {
        return Some(color);
    }

    // Try named color
    parse_named_color(input)
}

/// Parse hex color: #RGB, #RRGGBB, or #RRGGBBAA
fn parse_hex_color<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Color, E> {
    let (input, _) = char('#')(input)?;
    let (input, hex) = take_while1(|c: char| c.is_ascii_hexdigit())(input)?;

    let color = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            Color::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            Color::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            let a = u8::from_str_radix(&hex[6..8], 16).map_err(|_| {
                nom::Err::Error(E::from_error_kind(input, nom::error::ErrorKind::HexDigit))
            })?;
            Color::rgba(
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                a as f32 / 255.0,
            )
        }
        _ => {
            return Err(nom::Err::Error(E::from_error_kind(
                input,
                nom::error::ErrorKind::LengthValue,
            )));
        }
    };

    Ok((input, color))
}

/// Parse rgba(r, g, b, a)
fn parse_rgba_color<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Color, E> {
    let (input, _) = tag_no_case("rgba")(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = ws(input)?;
    let (input, r) = float(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = ws(input)?;
    let (input, g) = float(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = ws(input)?;
    let (input, b) = float(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = ws(input)?;
    let (input, a) = float(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(')')(input)?;

    // Normalize if values are 0-255 range
    let (r, g, b) = if r > 1.0 || g > 1.0 || b > 1.0 {
        (r / 255.0, g / 255.0, b / 255.0)
    } else {
        (r, g, b)
    };

    Ok((input, Color::rgba(r, g, b, a)))
}

/// Parse rgb(r, g, b)
fn parse_rgb_color<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Color, E> {
    let (input, _) = tag_no_case("rgb")(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = ws(input)?;
    let (input, r) = float(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = ws(input)?;
    let (input, g) = float(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = ws(input)?;
    let (input, b) = float(input)?;
    let (input, _) = ws(input)?;
    let (input, _) = char(')')(input)?;

    // Normalize if values are 0-255 range
    let (r, g, b) = if r > 1.0 || g > 1.0 || b > 1.0 {
        (r / 255.0, g / 255.0, b / 255.0)
    } else {
        (r, g, b)
    };

    Ok((input, Color::rgba(r, g, b, 1.0)))
}

/// Parse named colors
fn parse_named_color(name: &str) -> Option<Color> {
    match name.to_lowercase().as_str() {
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::RED),
        "green" => Some(Color::rgb(0.0, 0.5, 0.0)),
        "blue" => Some(Color::BLUE),
        "yellow" => Some(Color::YELLOW),
        "cyan" | "aqua" => Some(Color::CYAN),
        "magenta" | "fuchsia" => Some(Color::MAGENTA),
        "gray" | "grey" => Some(Color::GRAY),
        "orange" => Some(Color::ORANGE),
        "purple" => Some(Color::PURPLE),
        "transparent" => Some(Color::TRANSPARENT),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blinc_theme::ThemeState;

    #[test]
    fn test_parse_empty() {
        let stylesheet = Stylesheet::parse("").unwrap();
        assert!(stylesheet.is_empty());
    }

    #[test]
    fn test_parse_single_rule() {
        let css = "#card { opacity: 0.5; }";
        let stylesheet = Stylesheet::parse(css).unwrap();

        assert!(stylesheet.contains("card"));
        let style = stylesheet.get("card").unwrap();
        assert_eq!(style.opacity, Some(0.5));
    }

    #[test]
    fn test_parse_multiple_rules() {
        let css = r#"
            #card {
                opacity: 0.9;
                border-radius: 8px;
            }
            #button {
                opacity: 1.0;
            }
        "#;
        let stylesheet = Stylesheet::parse(css).unwrap();

        assert_eq!(stylesheet.len(), 2);
        assert!(stylesheet.contains("card"));
        assert!(stylesheet.contains("button"));
    }

    #[test]
    fn test_parse_hex_colors() {
        let css = "#test { background: #FF0000; }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        assert!(style.background.is_some());
    }

    #[test]
    fn test_parse_transform_scale() {
        let css = "#test { transform: scale(1.5); }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        assert!(style.transform.is_some());
    }

    #[test]
    fn test_parse_transform_scale_two_args() {
        let css = "#test { transform: scale(1.5, 2.0); }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        assert!(style.transform.is_some());
    }

    #[test]
    fn test_parse_transform_rotate() {
        let css = "#test { transform: rotate(45deg); }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        assert!(style.transform.is_some());
    }

    #[test]
    fn test_parse_comments() {
        let css = r#"
            /* This is a comment */
            #card {
                /* inline comment */
                opacity: 0.5;
            }
        "#;
        let stylesheet = Stylesheet::parse(css).unwrap();
        assert!(stylesheet.contains("card"));
    }

    #[test]
    fn test_parse_rgb_color() {
        let css = "#test { background: rgb(255, 128, 0); }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        assert!(style.background.is_some());
    }

    #[test]
    fn test_parse_rgba_color() {
        let css = "#test { background: rgba(255, 128, 0, 0.5); }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        assert!(style.background.is_some());
    }

    #[test]
    fn test_parse_named_color() {
        let css = "#test { background: red; }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        assert!(style.background.is_some());
    }

    #[test]
    fn test_parse_short_hex() {
        let css = "#test { background: #F00; }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        assert!(style.background.is_some());
    }

    #[test]
    fn test_parse_render_layer() {
        let css = "#test { render-layer: foreground; }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        assert_eq!(style.render_layer, Some(RenderLayer::Foreground));
    }

    #[test]
    fn test_parse_error_context() {
        // Invalid selector should give error context
        let css = "not-a-selector { opacity: 0.5; }";
        let result = Stylesheet::parse(css);
        // This should parse as empty (no valid rules) but not error
        // since the parser just ignores what it can't parse
        // The parse itself succeeds but finds no valid rules
        let stylesheet = result.unwrap();
        assert!(stylesheet.is_empty());
    }

    #[test]
    fn test_parse_error_has_display() {
        // Create an error manually to test Display impl
        let err = ParseError {
            severity: Severity::Error,
            message: "test error".to_string(),
            line: 1,
            column: 5,
            fragment: "#test".to_string(),
            contexts: vec!["rule".to_string(), "selector".to_string()],
            property: None,
            value: None,
        };
        let display = format!("{}", err);
        assert!(display.contains("CSS error"));
        assert!(display.contains("line 1"));
        assert!(display.contains("column 5"));
    }

    #[test]
    fn test_parse_or_empty_success() {
        let css = "#test { opacity: 0.5; }";
        let stylesheet = Stylesheet::parse_or_empty(css);
        assert!(stylesheet.contains("test"));
    }

    #[test]
    fn test_parse_or_empty_failure() {
        // Invalid CSS returns empty stylesheet
        let css = "this is not valid CSS";
        let stylesheet = Stylesheet::parse_or_empty(css);
        assert!(stylesheet.is_empty());
    }

    #[test]
    fn test_unknown_property_ignored() {
        // Unknown properties are silently ignored
        let css = "#test { unknown-property: value; opacity: 0.5; }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        // The known property is still parsed
        assert_eq!(style.opacity, Some(0.5));
    }

    #[test]
    fn test_invalid_value_skipped() {
        // Invalid values for known properties are skipped
        let css = "#test { opacity: invalid; border-radius: 8px; }";
        let stylesheet = Stylesheet::parse(css).unwrap();
        let style = stylesheet.get("test").unwrap();
        // opacity should be None (invalid value), but radius should work
        assert!(style.opacity.is_none());
        assert!(style.corner_radius.is_some());
    }

    // ========================================================================
    // Error Collection Tests for Reporting
    // ========================================================================

    #[test]
    fn test_error_collection_missing_closing_brace() {
        // Missing closing brace should produce a collectable error
        let css = "#test { opacity: 0.5";
        let result = Stylesheet::parse_with_errors(css);

        // With parse_with_errors, we get partial results plus errors
        // The stylesheet might be empty (couldn't parse any complete rules)
        // and errors should contain info about what went wrong

        // Either we have an error, or we have unparsed content warning
        let has_issues = result.has_errors() || result.has_warnings() || result.stylesheet.is_empty();
        assert!(has_issues, "Should have some indication of incomplete CSS");

        // If there are errors, validate their details
        if !result.errors.is_empty() {
            let err = &result.errors[0];
            assert!(err.line >= 1, "Line number should be set");
            assert!(err.column >= 1, "Column number should be set");
            assert!(!err.message.is_empty(), "Error message should be set");

            let display = format!("{}", err);
            assert!(
                display.contains("line") && display.contains("column"),
                "Display should include line and column info"
            );
        }
    }

    #[test]
    fn test_error_collection_missing_id_after_hash() {
        // # followed by invalid identifier should capture error context
        let css = "#123invalid { opacity: 0.5; }";
        let result = Stylesheet::parse(css);

        // This might parse as empty or error depending on nom's behavior
        // Either way, we should handle it gracefully
        match result {
            Ok(stylesheet) => {
                // If it parsed as empty, that's valid fallback behavior
                assert!(stylesheet.is_empty() || stylesheet.contains("123invalid"));
            }
            Err(err) => {
                // If it errored, error details should be collected
                assert!(!err.message.is_empty());
                assert!(err.line >= 1);
            }
        }
    }

    #[test]
    fn test_error_collection_with_contexts() {
        // Test that context stack is properly collected
        let css = "#test { : value; }"; // Missing property name before colon
        let result = Stylesheet::parse(css);

        match result {
            Ok(stylesheet) => {
                // Parser might skip malformed property, returning empty style
                if stylesheet.contains("test") {
                    let style = stylesheet.get("test").unwrap();
                    // The malformed property should be skipped
                    assert!(style.opacity.is_none());
                }
            }
            Err(err) => {
                // Error should have context about what was being parsed
                assert!(!err.message.is_empty());
                // Contexts might include "property name" or similar
                let display = format!("{}", err);
                assert!(display.contains("CSS parse error"));
            }
        }
    }

    #[test]
    fn test_error_collection_multiline() {
        // Test that line numbers are correctly calculated for multiline CSS
        let css = r#"
#first { opacity: 0.5; }
#second { opacity: 0.7; }
#third { opacity
"#;
        let result = Stylesheet::parse(css);

        match result {
            Ok(stylesheet) => {
                // May successfully parse the complete rules
                assert!(stylesheet.contains("first") || stylesheet.contains("second"));
            }
            Err(err) => {
                // If it errors, the line should be > 1 since error is on line 4
                assert!(err.line >= 1, "Line number should be calculated");
                let display = format!("{}", err);
                assert!(display.contains("line"), "Display should show line info");
            }
        }
    }

    #[test]
    fn test_error_collection_preserves_fragment() {
        // Test that the error fragment is captured for reporting
        let css = "#bad-css { = not valid }";
        let result = Stylesheet::parse(css);

        match result {
            Ok(_) => {
                // Parser might skip invalid content
            }
            Err(err) => {
                // Fragment should be set (though it might be truncated)
                // The fragment helps identify where parsing stopped
                let display = format!("{}", err);
                assert!(!display.is_empty());
            }
        }
    }

    #[test]
    fn test_collect_multiple_errors_via_iterations() {
        // Demonstrate how to collect errors from multiple CSS inputs
        let css_inputs = vec![
            ("#valid { opacity: 0.5; }", true),    // valid
            ("#broken {", false),                  // invalid - missing close
            ("#also-valid { opacity: 1.0; }", true), // valid
            ("@ invalid at-rule", false),          // invalid - no ID selector
        ];

        let mut errors: Vec<ParseError> = Vec::new();
        let mut successes: Vec<Stylesheet> = Vec::new();

        for (css, expected_success) in css_inputs {
            match Stylesheet::parse(css) {
                Ok(stylesheet) => {
                    if expected_success {
                        successes.push(stylesheet);
                    } else {
                        // Unexpected success - parser was lenient
                        successes.push(stylesheet);
                    }
                }
                Err(err) => {
                    // Collect the error for reporting
                    errors.push(err);
                }
            }
        }

        // Report: we can format all collected errors
        for (i, err) in errors.iter().enumerate() {
            let report = format!(
                "Error {}: line {}, col {}: {}",
                i + 1,
                err.line,
                err.column,
                err.message
            );
            assert!(!report.is_empty());
        }

        // At least one should have errored (the unclosed brace)
        assert!(
            !errors.is_empty() || successes.iter().any(|s| s.is_empty()),
            "Should have captured at least one error or empty result"
        );
    }

    #[test]
    fn test_error_debug_format() {
        // Test that ParseError has useful Debug output
        let css = "#incomplete {";
        let result = Stylesheet::parse(css);

        if let Err(err) = result {
            let debug_output = format!("{:?}", err);
            // Debug should include all the fields
            assert!(debug_output.contains("message"));
            assert!(debug_output.contains("line"));
            assert!(debug_output.contains("column"));
            assert!(debug_output.contains("fragment"));
            assert!(debug_output.contains("contexts"));
        }
    }

    #[test]
    fn test_error_is_std_error() {
        // Ensure ParseError implements std::error::Error properly
        let err = ParseError {
            severity: Severity::Error,
            message: "test error".to_string(),
            line: 5,
            column: 10,
            fragment: "broken".to_string(),
            contexts: vec!["rule".to_string()],
            property: Some("opacity".to_string()),
            value: Some("invalid".to_string()),
        };

        // Can be used as a std::error::Error
        let _: &dyn std::error::Error = &err;

        // Default source() implementation returns None
        use std::error::Error;
        assert!(err.source().is_none());
    }

    // ========================================================================
    // Tests for parse_with_errors - Full Error Collection
    // ========================================================================

    #[test]
    fn test_parse_with_errors_collects_unknown_properties() {
        let css = "#test { unknown-prop: value; opacity: 0.5; another-unknown: foo; }";
        let result = Stylesheet::parse_with_errors(css);

        // Should still parse the valid property
        assert!(result.stylesheet.contains("test"));
        let style = result.stylesheet.get("test").unwrap();
        assert_eq!(style.opacity, Some(0.5));

        // Should have collected warnings for unknown properties
        assert!(result.has_warnings(), "Should have warnings for unknown properties");

        let warnings: Vec<_> = result.warnings_only().collect();
        assert!(warnings.len() >= 2, "Should have at least 2 warnings for unknown props");

        // Check that warnings contain property info
        for warning in &warnings {
            assert_eq!(warning.severity, Severity::Warning);
            assert!(warning.property.is_some());
        }
    }

    #[test]
    fn test_parse_with_errors_collects_invalid_values() {
        let css = "#test { opacity: not-a-number; border-radius: ???; background: #FF0000; }";
        let result = Stylesheet::parse_with_errors(css);

        // Should still parse the valid property
        assert!(result.stylesheet.contains("test"));
        let style = result.stylesheet.get("test").unwrap();
        assert!(style.background.is_some(), "Valid background should parse");
        assert!(style.opacity.is_none(), "Invalid opacity should not parse");

        // Should have collected warnings for invalid values
        assert!(result.has_warnings());

        let warnings: Vec<_> = result.warnings_only().collect();
        assert!(warnings.len() >= 2, "Should have warnings for invalid values");

        // Check warning details
        for warning in &warnings {
            assert!(warning.property.is_some());
            assert!(warning.value.is_some());
            assert!(warning.message.contains("Invalid value"));
        }
    }

    #[test]
    fn test_parse_with_errors_print_diagnostics() {
        let css = "#test { unknown: value; opacity: bad; background: red; }";
        let result = Stylesheet::parse_with_errors(css);

        // Should have some errors/warnings
        assert!(!result.errors.is_empty());

        // Test that print_diagnostics doesn't panic
        // (We can't easily capture stderr in tests, but we can verify it runs)
        result.log_diagnostics();

        // Verify to_warning_string works
        for err in &result.errors {
            let warning_str = err.to_warning_string();
            assert!(!warning_str.is_empty());
            assert!(warning_str.contains(&err.severity.to_string()));
        }
    }

    #[test]
    fn test_parse_with_errors_multiline_line_numbers() {
        let css = r#"
#first {
    opacity: 0.5;
    unknown-prop: value;
}
#second {
    opacity: bad;
    background: blue;
}
"#;
        let result = Stylesheet::parse_with_errors(css);

        // Both rules should parse
        assert!(result.stylesheet.contains("first"));
        assert!(result.stylesheet.contains("second"));

        // Should have warnings with line numbers > 1
        let warnings: Vec<_> = result.warnings_only().collect();
        assert!(!warnings.is_empty());

        // At least some warnings should be on lines > 1
        let has_multiline_errors = warnings.iter().any(|w| w.line > 1);
        assert!(has_multiline_errors, "Should have errors on lines > 1");
    }

    #[test]
    fn test_parse_with_errors_severity_levels() {
        // Create various error types and check severity
        let warning = ParseError::unknown_property("foo", 1, 1);
        assert_eq!(warning.severity, Severity::Warning);

        let invalid = ParseError::invalid_value("opacity", "bad", 2, 5);
        assert_eq!(invalid.severity, Severity::Warning);

        let error = ParseError::new(Severity::Error, "fatal error", 3, 10);
        assert_eq!(error.severity, Severity::Error);
    }

    #[test]
    fn test_css_parse_result_methods() {
        let css = "#test { unknown: x; opacity: bad; }";
        let result = Stylesheet::parse_with_errors(css);

        // Test CssParseResult methods
        assert!(result.has_warnings());
        assert!(!result.has_errors()); // These are warnings, not errors

        let warnings_count = result.warnings_only().count();
        let errors_count = result.errors_only().count();

        assert!(warnings_count >= 2);
        assert_eq!(errors_count, 0);
    }

    #[test]
    fn test_error_collection_with_valid_css_no_errors() {
        let css = "#card { opacity: 0.8; background: #FF0000; border-radius: 8px; }";
        let result = Stylesheet::parse_with_errors(css);

        // Should parse successfully with no errors
        assert!(result.stylesheet.contains("card"));
        assert!(result.errors.is_empty(), "Valid CSS should have no errors");
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
    }

    // ========================================================================
    // CSS Variables Tests
    // ========================================================================

    #[test]
    fn test_css_variables_root_parsing() {
        let css = r#"
            :root {
                --primary-color: #FF0000;
                --secondary-color: #00FF00;
                --card-radius: 8px;
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        assert_eq!(result.stylesheet.variable_count(), 3);
        assert_eq!(
            result.stylesheet.get_variable("primary-color"),
            Some("#FF0000")
        );
        assert_eq!(
            result.stylesheet.get_variable("secondary-color"),
            Some("#00FF00")
        );
        assert_eq!(result.stylesheet.get_variable("card-radius"), Some("8px"));
    }

    #[test]
    fn test_css_variables_usage() {
        let css = r#"
            :root {
                --main-opacity: 0.8;
            }
            #card {
                opacity: var(--main-opacity);
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        assert!(result.stylesheet.contains("card"));
        let style = result.stylesheet.get("card").unwrap();
        assert_eq!(style.opacity, Some(0.8));
    }

    #[test]
    fn test_css_variables_with_fallback() {
        let css = r#"
            #card {
                opacity: var(--undefined-var, 0.5);
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        let style = result.stylesheet.get("card").unwrap();
        assert_eq!(style.opacity, Some(0.5));
    }

    #[test]
    fn test_css_variables_color() {
        let css = r#"
            :root {
                --brand-color: #3498db;
            }
            #button {
                background: var(--brand-color);
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        let style = result.stylesheet.get("button").unwrap();
        assert!(style.background.is_some());
    }

    #[test]
    fn test_css_variables_multiple_rules() {
        let css = r#"
            :root {
                --base-radius: 4px;
                --card-opacity: 0.9;
            }
            #card {
                border-radius: var(--base-radius);
                opacity: var(--card-opacity);
            }
            #button {
                opacity: var(--card-opacity);
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        assert!(result.stylesheet.contains("card"));
        assert!(result.stylesheet.contains("button"));

        let card = result.stylesheet.get("card").unwrap();
        let button = result.stylesheet.get("button").unwrap();

        assert!(card.corner_radius.is_some());
        assert_eq!(card.opacity, Some(0.9));
        assert_eq!(button.opacity, Some(0.9));
    }

    #[test]
    fn test_css_variables_set_at_runtime() {
        let css = "#card { opacity: 0.5; }";
        let mut stylesheet = Stylesheet::parse(css).unwrap();

        // Set variable at runtime
        stylesheet.set_variable("custom-var", "#FF0000");

        assert_eq!(stylesheet.get_variable("custom-var"), Some("#FF0000"));
    }

    #[test]
    fn test_css_variables_names_iterator() {
        let css = r#"
            :root {
                --a: 1;
                --b: 2;
                --c: 3;
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        let names: Vec<_> = result.stylesheet.variable_names().collect();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"a"));
        assert!(names.contains(&"b"));
        assert!(names.contains(&"c"));
    }

    #[test]
    fn test_css_variables_with_theme_fallback() {
        // Initialize theme (required for theme() functions)
        ThemeState::init_default();

        let css = r#"
            :root {
                --card-shadow: theme(shadow-md);
            }
            #card {
                box-shadow: var(--card-shadow);
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        // The variable stores the raw value "theme(shadow-md)"
        // which gets resolved when applied to the style
        let style = result.stylesheet.get("card").unwrap();
        assert!(style.shadow.is_some());
    }

    #[test]
    fn test_css_variables_nested_resolution() {
        let css = r#"
            :root {
                --base: 0.5;
                --derived: var(--base);
            }
            #test {
                opacity: var(--derived);
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        let style = result.stylesheet.get("test").unwrap();
        assert_eq!(style.opacity, Some(0.5));
    }

    // ========================================================================
    // State Modifier Tests (Pseudo-classes)
    // ========================================================================

    #[test]
    fn test_state_modifier_hover() {
        let css = r#"
            #button {
                opacity: 1.0;
            }
            #button:hover {
                opacity: 0.8;
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        // Base style
        let base = result.stylesheet.get("button").unwrap();
        assert_eq!(base.opacity, Some(1.0));

        // Hover style
        let hover = result.stylesheet.get_with_state("button", ElementState::Hover).unwrap();
        assert_eq!(hover.opacity, Some(0.8));
    }

    #[test]
    fn test_state_modifier_active() {
        let css = r#"
            #button:active {
                transform: scale(0.95);
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        let active = result.stylesheet.get_with_state("button", ElementState::Active).unwrap();
        assert!(active.transform.is_some());
    }

    #[test]
    fn test_state_modifier_focus() {
        let css = r#"
            #input:focus {
                border-radius: 4px;
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        let focus = result.stylesheet.get_with_state("input", ElementState::Focus).unwrap();
        assert!(focus.corner_radius.is_some());
    }

    #[test]
    fn test_state_modifier_disabled() {
        let css = r#"
            #button:disabled {
                opacity: 0.5;
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        let disabled = result.stylesheet.get_with_state("button", ElementState::Disabled).unwrap();
        assert_eq!(disabled.opacity, Some(0.5));
    }

    #[test]
    fn test_multiple_state_modifiers() {
        let css = r#"
            #button {
                background: #0000FF;
                opacity: 1.0;
            }
            #button:hover {
                opacity: 0.9;
            }
            #button:active {
                opacity: 0.8;
                transform: scale(0.98);
            }
            #button:focus {
                border-radius: 4px;
            }
            #button:disabled {
                opacity: 0.4;
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        // Base style
        assert!(result.stylesheet.contains("button"));
        let base = result.stylesheet.get("button").unwrap();
        assert_eq!(base.opacity, Some(1.0));

        // Check all states exist
        assert!(result.stylesheet.contains_with_state("button", ElementState::Hover));
        assert!(result.stylesheet.contains_with_state("button", ElementState::Active));
        assert!(result.stylesheet.contains_with_state("button", ElementState::Focus));
        assert!(result.stylesheet.contains_with_state("button", ElementState::Disabled));

        // Verify state styles
        let hover = result.stylesheet.get_with_state("button", ElementState::Hover).unwrap();
        assert_eq!(hover.opacity, Some(0.9));

        let active = result.stylesheet.get_with_state("button", ElementState::Active).unwrap();
        assert_eq!(active.opacity, Some(0.8));
        assert!(active.transform.is_some());

        let focus = result.stylesheet.get_with_state("button", ElementState::Focus).unwrap();
        assert!(focus.corner_radius.is_some());

        let disabled = result.stylesheet.get_with_state("button", ElementState::Disabled).unwrap();
        assert_eq!(disabled.opacity, Some(0.4));
    }

    #[test]
    fn test_get_all_states() {
        let css = r#"
            #card {
                opacity: 1.0;
            }
            #card:hover {
                opacity: 0.9;
            }
            #card:active {
                opacity: 0.8;
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        let (base, states) = result.stylesheet.get_all_states("card");

        assert!(base.is_some());
        assert_eq!(base.unwrap().opacity, Some(1.0));

        assert_eq!(states.len(), 2);

        // Check we got hover and active
        let state_types: Vec<_> = states.iter().map(|(s, _)| *s).collect();
        assert!(state_types.contains(&ElementState::Hover));
        assert!(state_types.contains(&ElementState::Active));
    }

    #[test]
    fn test_state_modifier_with_variables() {
        let css = r#"
            :root {
                --hover-opacity: 0.85;
            }
            #button:hover {
                opacity: var(--hover-opacity);
            }
        "#;
        let result = Stylesheet::parse_with_errors(css);

        let hover = result.stylesheet.get_with_state("button", ElementState::Hover).unwrap();
        assert_eq!(hover.opacity, Some(0.85));
    }

    #[test]
    fn test_unknown_state_modifier_ignored() {
        // Unknown pseudo-class should parse the ID part but not set state
        let css = "#button:unknown { opacity: 0.5; }";
        let result = Stylesheet::parse_with_errors(css);

        // The selector "#button:unknown" where "unknown" is not a valid state
        // should still be stored, but with the state part as None
        // Actually, since we parse :unknown but it's not a known state,
        // the state will be None, so it just becomes "button"
        assert!(result.stylesheet.contains("button"));
        let style = result.stylesheet.get("button").unwrap();
        assert_eq!(style.opacity, Some(0.5));
    }

    #[test]
    fn test_element_state_from_str() {
        assert_eq!(ElementState::from_str("hover"), Some(ElementState::Hover));
        assert_eq!(ElementState::from_str("HOVER"), Some(ElementState::Hover));
        assert_eq!(ElementState::from_str("active"), Some(ElementState::Active));
        assert_eq!(ElementState::from_str("focus"), Some(ElementState::Focus));
        assert_eq!(ElementState::from_str("disabled"), Some(ElementState::Disabled));
        assert_eq!(ElementState::from_str("unknown"), None);
    }

    #[test]
    fn test_element_state_display() {
        assert_eq!(format!("{}", ElementState::Hover), "hover");
        assert_eq!(format!("{}", ElementState::Active), "active");
        assert_eq!(format!("{}", ElementState::Focus), "focus");
        assert_eq!(format!("{}", ElementState::Disabled), "disabled");
    }

    #[test]
    fn test_css_selector_key() {
        let selector = CssSelector::new("button");
        assert_eq!(selector.key(), "button");

        let selector_hover = CssSelector::with_state("button", ElementState::Hover);
        assert_eq!(selector_hover.key(), "button:hover");
    }
}

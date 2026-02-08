use std::borrow::Cow;

/// A translation argument value.
#[derive(Clone, Debug, PartialEq)]
pub enum ArgValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl From<String> for ArgValue {
    fn from(v: String) -> Self {
        Self::Str(v)
    }
}

impl From<&str> for ArgValue {
    fn from(v: &str) -> Self {
        Self::Str(v.to_string())
    }
}

impl From<i64> for ArgValue {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<i32> for ArgValue {
    fn from(v: i32) -> Self {
        Self::Int(v as i64)
    }
}

impl From<usize> for ArgValue {
    fn from(v: usize) -> Self {
        Self::Int(v as i64)
    }
}

impl From<f64> for ArgValue {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<f32> for ArgValue {
    fn from(v: f32) -> Self {
        Self::Float(v as f64)
    }
}

impl From<bool> for ArgValue {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

/// A message key + arguments (backend-agnostic).
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: Cow<'static, str>,
    pub args: Vec<(Cow<'static, str>, ArgValue)>,
}

impl Message {
    pub fn new(id: impl Into<Cow<'static, str>>) -> Self {
        Self {
            id: id.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, name: impl Into<Cow<'static, str>>, value: impl Into<ArgValue>) -> Self {
        self.args.push((name.into(), value.into()));
        self
    }
}

/// A UI label: either raw text or a translatable message key.
#[derive(Clone, Debug, PartialEq)]
pub enum Label {
    Raw(String),
    Msg(Message),
}

impl Label {
    pub fn raw(s: impl Into<String>) -> Self {
        Self::Raw(s.into())
    }

    pub fn msg(m: Message) -> Self {
        Self::Msg(m)
    }
}

impl From<String> for Label {
    fn from(s: String) -> Self {
        Self::Raw(s)
    }
}

impl From<&str> for Label {
    fn from(s: &str) -> Self {
        Self::Raw(s.to_string())
    }
}

impl From<&String> for Label {
    fn from(s: &String) -> Self {
        Self::Raw(s.clone())
    }
}

impl From<Message> for Label {
    fn from(m: Message) -> Self {
        Self::Msg(m)
    }
}

// Copyright 2023--present rgconf developers

use std::fmt;

/// A parameter failure that names the input it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    pub path: String,
    pub line: u32,
    pub column: u32,
    pub key: String,
    pub message: String,
}

impl Located {
    pub fn new(
        path: impl Into<String>,
        line: u32,
        column: u32,
        key: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            line,
            column,
            key: key.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Located {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}: {}",
            self.path, self.line, self.column, self.key, self.message
        )
    }
}

/// Failures from resolution, validation, and the provenance log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Located(Located),
    /// `rfc9162` does not yet offer this operation. The call site stays a
    /// trait method so rgconf does not grow a private copy of the log.
    UpstreamPatch {
        function: &'static str,
        detail: &'static str,
    },
    Message(String),
}

impl Error {
    pub fn message(text: impl Into<String>) -> Self {
        Self::Message(text.into())
    }

    pub fn located(
        path: impl Into<String>,
        line: u32,
        column: u32,
        key: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Located(Located::new(path, line, column, key, message))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Located(located) => write!(f, "{located}"),
            Self::UpstreamPatch { function, detail } => {
                write!(f, "upstream patch needed for `{function}`: {detail}")
            }
            Self::Message(text) => write!(f, "{text}"),
        }
    }
}

impl std::error::Error for Error {}

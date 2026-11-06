//! How bad a finding is ([`Severity`]) and the loading phase that raised it ([`Phase`]).

use std::fmt;

/// How bad a finding is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Debug output.
    Debug,
    /// Information.
    Info,
    /// A warning: the document loads.
    Warning,
    /// An error: the document (or entry) is dropped.
    Error,
}

/// The content-log level word: `debug`, `info`, `warning`, `error`.
impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        })
    }
}

/// The phases of the game's versioned loaders, plus the crate's own I/O and link phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Phase {
    /// Reading bytes from the host.
    Read,
    /// JSON and typed parsing.
    Parse,
    /// Version upgrades.
    Upgrade,
    /// Validation on the newest struct.
    Validate,
    /// The cross-family link pass.
    Link,
    /// Writing a document back.
    Write,
}

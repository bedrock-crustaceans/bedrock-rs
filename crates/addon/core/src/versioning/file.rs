//! The policy below a chain's floor ([`BelowFloor`]).

/// What a document below the chain's floor is told (`Family::ENVELOPE.below_floor`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum BelowFloor {
    /// The generic floor line: "To use … use json format version … or higher"
    /// ([`USE_FORMAT_VERSION`](crate::diagnostic::codes::USE_FORMAT_VERSION)). The default.
    #[default]
    UseFormatVersion,
    /// The game's versioned loader has no slice below its first one: "no parser available"
    /// ([`NO_PARSER_FOR_VERSION`](crate::diagnostic::codes::NO_PARSER_FOR_VERSION)), the version rendered as the text above the engine
    /// renders it (`versioning::no_parser_version`: the full triple, a pre-release as
    /// written).
    NoParser,
}

//! The `loading_messages` family's codes: the game's content-log lines, byte for byte, each a
//! definition beside the rules that raise it. Each [`was`](CodeDef::was) the legacy code its sites
//! raised, so a pin or a filter on the code's name keeps matching.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// The file is not JSON: the game's assert; the finding carries the reader's message as its syntax
/// detail.
pub static LOADING_NOT_JSON: CodeDef<0> = CodeDef::new(
    "loading_messages/loading_not_json",
    Kind::Syntax,
    Class::Error,
    "loading messages file is not JSON",
)
.game("Failed to read loading messages file")
.was("MalformedInput");

/// The game's assert for a file with neither a non-empty `loading_messages` nor a
/// `beginner_loading_messages`.
pub static BEGINNER_MISSING: CodeDef<0> = CodeDef::new(
    "loading_messages/beginner_missing",
    Kind::MissingRequired,
    Class::Lint,
    "no beginner messages",
)
.game("Beginner messages are missing in ProgressTips::_parseLoadingMessages")
.was("MissingRequiredField");

/// The game's assert when the beginner list is empty.
pub static BEGINNER_EMPTY: CodeDef<0> = CodeDef::new(
    "loading_messages/beginner_empty",
    Kind::InvalidValue,
    Class::Lint,
    "the beginner list is empty",
)
.game("ProgressTips::getProgressTipStage called before loading the tips!")
.was("InvalidValue");

/// The game's assert for a list the file lacks; the argument is the member name.
pub static LIST_MISSING: CodeDef<1> = CodeDef::new(
    "loading_messages/list_missing",
    Kind::MissingRequired,
    Class::Lint,
    "no {} list",
)
.game("JSON passed to ProgressTips::_parseConditionalLoadingMessages does not contain messages of type: {}")
.was("MissingRequiredField");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    LOADING_NOT_JSON.meta(),
    BEGINNER_MISSING.meta(),
    BEGINNER_EMPTY.meta(),
    LIST_MISSING.meta(),
];

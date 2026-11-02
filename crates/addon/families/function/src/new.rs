//! Constructors and derived views of functions and tick lists.

use super::parse::{first_word, trimmed};
use super::{Function, FunctionLine, FunctionState, QueueOrder, TickFunction};
use bedrock_addon_core::id::FunctionPath;
use bedrock_addon_core::version::CURRENT_COMMAND_VERSION;

/// From this command version on (`min_engine_version` 1.18.10), the commands of a nested
/// function call are queued after the caller's remaining commands.
const FUNCTION_EXECUTE_ORDER: u8 = 18;

impl Function {
    /// A function called `name` with no commands, at the current command version.
    #[must_use]
    pub fn new(name: FunctionPath) -> Function {
        Function {
            name,
            command_version: Some(CURRENT_COMMAND_VERSION),
            commands: Vec::new(),
            dropped: Vec::new(),
        }
    }

    /// Whether the function can run: it cannot when its pack has no usable
    /// `min_engine_version`: the game creates such a function in the not-initialised state
    /// when the pack's version is not valid.
    #[must_use]
    pub fn state(&self) -> FunctionState {
        match self.command_version {
            Some(_) => FunctionState::Valid,
            None => FunctionState::EngineVersionNotInitialized,
        }
    }

    /// Whether vanilla offers the name for auto-completion (its `FunctionName` soft enum): only
    /// a function that can run and loaded without a dropped line is listed, because the game
    /// adds the name to the list only when loading the file gave no error.
    #[must_use]
    pub fn is_auto_completed(&self) -> bool {
        self.command_version.is_some() && self.dropped.is_empty()
    }

    /// Whether `/function <name>` can name the function: every byte of the name must be an
    /// ASCII letter or digit or one of `( ) - . / _` (vanilla's
    /// `commands.function.invalidCharacters` otherwise). A function with another character in
    /// its name still loads and can be listed in `tick.json`.
    #[must_use]
    pub fn is_callable(&self) -> bool {
        self.name.as_str().bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'(' | b')' | b'-' | b'.' | b'/' | b'_')
        })
    }
}

impl FunctionLine {
    /// A command line that was not read from a file (line number 0).
    pub fn new(raw: impl Into<String>) -> FunctionLine {
        FunctionLine {
            line: 0,
            raw: raw.into(),
        }
    }

    /// The command: the line trimmed, as vanilla trims it before it looks at the first word.
    #[must_use]
    pub fn command(&self) -> &str {
        trimmed(&self.raw)
    }

    /// The command's name: the first word of the trimmed line, up to its first space.
    #[must_use]
    pub fn name(&self) -> &str {
        first_word(self.command())
    }
}

impl QueueOrder {
    /// The queue order of nested function calls under `command_version`: the version of the
    /// calling command for `/function`, of the listing pack for a tick entry. Command version 18
    /// is the one that introduced the order after the caller (together with a chest slot fix).
    #[must_use]
    pub fn from_command_version(command_version: u8) -> QueueOrder {
        if command_version < FUNCTION_EXECUTE_ORDER {
            QueueOrder::NextToExecute
        } else {
            QueueOrder::AfterCaller
        }
    }
}

impl TickFunction {
    /// An entry running `function`, at the current command version.
    #[must_use]
    pub fn new(function: FunctionPath) -> TickFunction {
        TickFunction {
            function,
            command_version: CURRENT_COMMAND_VERSION,
        }
    }

    /// Where the commands of functions this entry's function calls are queued.
    #[must_use]
    pub fn queue_order(&self) -> QueueOrder {
        QueueOrder::from_command_version(self.command_version)
    }
}

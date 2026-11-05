//! `loading_messages.json`: reading the file and the game's reading of its lists.

use super::{
    ConditionalMessage, LoadingMessage, LoadingMessages, MARKETPLACE_NOT_DISABLED, MessageList,
};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, VanillaText,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;
use bedrock_addon_support::fixed_path::{
    Authored, Object, array, invalid, invalid_text, iterated, key, others, raw_member, to_text,
};

/// The member of the store's progress texts.
const TOOLTIPS: &str = "store_progress_tooltips";
const ELEMENT_KEYS: [&str; 2] = ["message", "condition"];

/// Vanilla's assert for a file with neither a non-empty `loading_messages` nor a
/// `beginner_loading_messages`.
const BEGINNER_MISSING: &str =
    "Beginner messages are missing in ProgressTips::_parseLoadingMessages";
/// Vanilla's assert when the beginner list is empty.
const BEGINNER_EMPTY: &str = "ProgressTips::getProgressTipStage called before loading the tips!";
/// Vanilla's assert for a list the file lacks; the argument is the member name.
const LIST_MISSING: &str = "JSON passed to ProgressTips::_parseConditionalLoadingMessages does not contain messages of type: {}";

impl MessageList {
    /// Every list, in the order the game's reader names them.
    pub const ALL: [MessageList; 8] = [
        MessageList::Legacy,
        MessageList::Beginner,
        MessageList::MidGame,
        MessageList::LateGame,
        MessageList::Realms,
        MessageList::Addons,
        MessageList::Creative,
        MessageList::Editor,
    ];

    /// The member that holds the list.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Legacy => "loading_messages",
            Self::Beginner => "beginner_loading_messages",
            Self::MidGame => "mid_game_loading_messages",
            Self::LateGame => "late_game_loading_messages",
            Self::Realms => "realms_loading_messages",
            Self::Addons => "addons_loading_messages",
            Self::Creative => "creative_loading_messages",
            Self::Editor => "editor_loading_messages",
        }
    }
}

impl LoadingMessage {
    /// The text the game can show for this element: a plain text, or a conditional one
    /// without a condition or with the one condition the game knows. `None` for an element
    /// the game skips or never shows.
    pub fn shown(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            Self::Conditional(message) => match message.condition.as_deref() {
                None | Some(MARKETPLACE_NOT_DISABLED) => Some(&message.message),
                Some(_) => None,
            },
            Self::Other(_) => None,
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Self::Text(text) => Value::from(text.as_str()),
            Self::Conditional(message) => {
                let mut object = Object::default();
                object.put("message", Some(Value::from(message.message.as_str())));
                object.put("condition", message.condition.as_deref().map(Value::from));
                object.extra(&message.extra);
                object.finish()
            }
            Self::Other(written) => written.clone(),
        }
    }
}

/// Reads one element of a list: a string is kept; an object needs a `message` member, and
/// `message` and `condition` are read as strings with an empty default.
fn read_element(element: &Value, at: &JsonPath, diagnostics: &mut Diagnostics) -> LoadingMessage {
    if let Some(text) = element.as_string() {
        return LoadingMessage::Text(text.as_str().to_owned());
    }
    if !element.is_object() {
        diagnostics.push(Diagnostic::info(invalid()).at(at.clone()));
        return LoadingMessage::Other(element.clone());
    }
    let Some(message) = raw_member(element, "message") else {
        diagnostics.push(
            Diagnostic::info(DiagnosticCode::MissingRequiredField { text: None })
                .at(key(at, "message")),
        );
        return LoadingMessage::Other(element.clone());
    };
    let mut wrong_type = |member: &Value, name: &str| {
        if !member.is_string() {
            diagnostics.push(
                Diagnostic::info(DiagnosticCode::InvalidType { text: None }).at(key(at, name)),
            );
        }
    };
    wrong_type(message, "message");
    let condition = raw_member(element, "condition").map(|condition| {
        wrong_type(condition, "condition");
        to_text(condition)
    });
    if condition
        .as_deref()
        .is_some_and(|c| c != MARKETPLACE_NOT_DISABLED)
    {
        // An unknown condition is false; the message is never shown.
        diagnostics.push(Diagnostic::info(invalid()).at(key(at, "condition")));
    }
    let mut extra = Extra::default();
    if let Some(object) = element.as_object() {
        for (name, member) in object {
            if !ELEMENT_KEYS.contains(&name.as_str()) {
                extra.insert(name.as_str().to_owned(), member.clone());
            }
        }
    }
    LoadingMessage::Conditional(ConditionalMessage {
        message: to_text(message),
        condition,
        extra,
    })
}

/// What the game iterates of a list member, and an information of the crate's own when it is no
/// array.
fn iterate<'v>(
    member: &'v Value,
    at: &JsonPath,
    diagnostics: &mut Diagnostics,
) -> Vec<(JsonPath, &'v Value)> {
    if !member.is_array() {
        diagnostics
            .push(Diagnostic::info(DiagnosticCode::InvalidType { text: None }).at(at.clone()));
    }
    iterated(member, at)
}

impl Authored for LoadingMessages {
    /// Reads the file's root. A root that is no object has no list.
    fn read(root: &Value, diagnostics: &mut Diagnostics) -> Self {
        let at = JsonPath::new();
        if !root.is_object() {
            diagnostics
                .push(Diagnostic::info(DiagnosticCode::InvalidType { text: None }).at(at.clone()));
        }
        let mut messages = Self::default();
        for list in MessageList::ALL {
            let Some(member) = raw_member(root, list.key()) else {
                continue;
            };
            let at = key(&at, list.key());
            let elements = iterate(member, &at, diagnostics)
                .into_iter()
                .map(|(at, element)| read_element(element, &at, diagnostics))
                .collect();
            *messages.slot(list) = Some(elements);
        }
        if let Some(member) = raw_member(root, TOOLTIPS) {
            let at = key(&at, TOOLTIPS);
            let mut tooltips = Vec::new();
            for (at, element) in iterate(member, &at, diagnostics) {
                if !element.is_string() {
                    diagnostics.push(Diagnostic::info(invalid()).at(at));
                }
                tooltips.push(element.clone());
            }
            messages.store_progress_tooltips = Some(tooltips);
        }
        let mut read: Vec<&str> = MessageList::ALL.iter().map(|list| list.key()).collect();
        read.push(TOOLTIPS);
        messages.extra = others(root, &read, &at, diagnostics);
        messages.check_lists(diagnostics);
        messages
    }

    /// The lists as the JSON object they are written as.
    fn fields_value(&self) -> Value {
        let mut object = Object::default();
        for list in MessageList::ALL {
            let elements = self.list(list);
            object.put(
                list.key(),
                elements.map(|list| array(list.iter().map(LoadingMessage::to_value))),
            );
        }
        object.put(
            TOOLTIPS,
            self.store_progress_tooltips
                .as_ref()
                .map(|list| array(list.iter().cloned())),
        );
        object.extra(&self.extra);
        object.finish()
    }

    fn written_mut(&mut self) -> &mut Option<Value> {
        &mut self.written
    }
}

impl LoadingMessages {
    /// Checks the lists the game asks for.
    fn check_lists(&self, diagnostics: &mut Diagnostics) {
        let legacy = self.uses_legacy_list();
        let mut asked = Vec::new();
        if !legacy {
            match &self.beginner_loading_messages {
                None => diagnostics.push(Diagnostic::info(DiagnosticCode::MissingRequiredField {
                    text: Some(VanillaText::Fixed(BEGINNER_MISSING)),
                })),
                Some(list) if list.is_empty() => diagnostics.push(
                    Diagnostic::info(invalid_text(BEGINNER_EMPTY))
                        .at(key(&JsonPath::new(), MessageList::Beginner.key())),
                ),
                Some(_) => {}
            }
            asked.extend([MessageList::MidGame, MessageList::LateGame]);
        }
        asked.extend([
            MessageList::Realms,
            MessageList::Addons,
            MessageList::Creative,
            MessageList::Editor,
        ]);
        for list in asked {
            if self.list(list).is_none() {
                diagnostics.push(Diagnostic::info(DiagnosticCode::MissingRequiredField {
                    text: Some(VanillaText::Formatted {
                        template: LIST_MISSING,
                        args: vec![list.key().to_owned()],
                    }),
                }));
            }
        }
    }

    fn slot(&mut self, list: MessageList) -> &mut Option<Vec<LoadingMessage>> {
        match list {
            MessageList::Legacy => &mut self.loading_messages,
            MessageList::Beginner => &mut self.beginner_loading_messages,
            MessageList::MidGame => &mut self.mid_game_loading_messages,
            MessageList::LateGame => &mut self.late_game_loading_messages,
            MessageList::Realms => &mut self.realms_loading_messages,
            MessageList::Addons => &mut self.addons_loading_messages,
            MessageList::Creative => &mut self.creative_loading_messages,
            MessageList::Editor => &mut self.editor_loading_messages,
        }
    }

    /// The elements of `list` as written; `None` when the file has no such member.
    pub fn list(&self, list: MessageList) -> Option<&[LoadingMessage]> {
        match list {
            MessageList::Legacy => &self.loading_messages,
            MessageList::Beginner => &self.beginner_loading_messages,
            MessageList::MidGame => &self.mid_game_loading_messages,
            MessageList::LateGame => &self.late_game_loading_messages,
            MessageList::Realms => &self.realms_loading_messages,
            MessageList::Addons => &self.addons_loading_messages,
            MessageList::Creative => &self.creative_loading_messages,
            MessageList::Editor => &self.editor_loading_messages,
        }
        .as_deref()
    }

    /// Whether the older single list stands for the three stage lists: when
    /// `loading_messages` has an element.
    pub fn uses_legacy_list(&self) -> bool {
        self.loading_messages
            .as_ref()
            .is_some_and(|list| !list.is_empty())
    }

    /// The tips the game can show from the list a screen or a stage asks for. A stage
    /// (`Beginner`, `MidGame`, `LateGame`) is answered from `loading_messages` when
    /// [that list is in use](Self::uses_legacy_list). A message whose condition the game does
    /// not know and an element it skips are left out.
    pub fn tips(&self, list: MessageList) -> Vec<&str> {
        let stage = matches!(
            list,
            MessageList::Beginner | MessageList::MidGame | MessageList::LateGame
        );
        let list = if stage && self.uses_legacy_list() {
            MessageList::Legacy
        } else {
            list
        };
        self.list(list)
            .unwrap_or_default()
            .iter()
            .filter_map(LoadingMessage::shown)
            .collect()
    }

    /// The store's progress texts as the game reads them: a string as written, a boolean as
    /// `true` / `false`, anything else as the empty text.
    // Every element is read as a string with an empty default.
    pub fn tooltips(&self) -> Vec<String> {
        self.store_progress_tooltips
            .iter()
            .flatten()
            .map(to_text)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::test_cost::assert_linear_at;
    use bedrock_addon_support::jsoncpp::read;

    /// A list is read in time linear in its elements.
    #[test]
    fn a_message_list_is_read_in_linear_time() {
        let build = |entries: usize| {
            let elements: Vec<String> = (0..entries)
                .map(|i| match i % 3 {
                    0 => format!("\"tip {i}\""),
                    1 => format!(r#"{{ "message": "m{i}", "condition": "$x" }}"#),
                    _ => format!("{i}"),
                })
                .collect();
            let text = format!(
                r#"{{ "beginner_loading_messages": [{}] }}"#,
                elements.join(",")
            );
            read(text.as_bytes()).expect("JSON")
        };
        assert_linear_at("LoadingMessages::from_value", 25_000, build, |root| {
            let mut diagnostics = Diagnostics::default();
            let messages = LoadingMessages::from_value(root, &mut diagnostics);
            let read = messages.list(MessageList::Beginner).map(<[_]>::len);
            assert_eq!(
                read,
                root.as_object()
                    .and_then(|o| o.values().next())
                    .and_then(Value::as_array)
                    .map(|a| a.iter().count())
            );
        });
    }
}

#[cfg(test)]
mod kept_root_tests {
    use super::*;
    use bedrock_addon_support::fixed_path::testing::{assert_kept_root_written_linearly, list};

    /// A write of a value that keeps its root is linear in its entries, with one entry the
    /// game skips.
    #[test]
    fn a_kept_root_is_written_in_linear_time() {
        assert_kept_root_written_linearly::<LoadingMessages>(
            "LoadingMessages::to_value",
            5000,
            |entries| {
                let messages = list(entries, |i| match i % 2 {
                    0 => format!("\"tip {i}\""),
                    _ => format!(r#"{{ "message": "m{i}", "condition": "$x" }}"#),
                });
                format!(
                    r#"{{ "beginner_loading_messages": [{messages}], "mid_game_loading_messages": 5 }}"#
                )
            },
        );
    }
}

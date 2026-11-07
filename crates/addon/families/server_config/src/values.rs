//! Reading `variables.json` (name → any JSON) and `secrets.json` (name → string).

use crate::codes;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, JsonPath};
use bedrock_addon_core::json::Value;

use super::SecretString;
use super::read::{Sink, Twice, object, span, string, value, with_root};

/// Reads one `variables.json`: every member, with whatever value it has. A root that is not
/// an object gives no variables.
pub(super) fn variables(bytes: &[u8], sink: &mut Sink<'_>) -> IndexMap<String, Value> {
    let mut variables = IndexMap::new();
    with_root(bytes, Twice::Last, sink, |root, sink| {
        let at = JsonPath::new();
        let Some(members) = object(root) else {
            // Silent on the server.
            sink.warn(Code::from(&core_codes::INVALID_TYPE), at, span(root));
            return;
        };
        variables.reserve(members.len());
        for (name, raw) in &members {
            let Some(slice) = root.nested(raw) else {
                continue;
            };
            if let Some(value) = value(slice, &at.child(name), sink) {
                variables.insert(name.clone(), value);
            }
        }
    });
    variables
}

/// Reads one `secrets.json`: the members whose value is a string. Any other member is
/// skipped with the server's warning, which names the secret and never its value; the
/// warnings come in the byte order of the names, as the server prints them.
pub(super) fn secrets(bytes: &[u8], sink: &mut Sink<'_>) -> IndexMap<String, SecretString> {
    let mut secrets = IndexMap::new();
    with_root(bytes, Twice::Last, sink, |root, sink| {
        let at = JsonPath::new();
        let Some(members) = object(root) else {
            // Silent on the server. The sink keeps the span out.
            sink.warn(Code::from(&core_codes::INVALID_TYPE), at, span(root));
            return;
        };
        secrets.reserve(members.len());
        let mut skipped = Vec::new();
        for (name, raw) in &members {
            let Some(slice) = root.nested(raw) else {
                continue;
            };
            match string(slice) {
                Some(secret) => {
                    secrets.insert(name.clone(), SecretString::new(secret));
                }
                None => skipped.push((name.as_str(), span(slice))),
            }
        }
        skipped.sort_unstable_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        for (name, span) in skipped {
            sink.warn_logged(
                codes::SECRET_NOT_A_STRING.with([name, sink.name.as_str()]),
                at.child(name),
                span,
            );
        }
    });
    secrets
}

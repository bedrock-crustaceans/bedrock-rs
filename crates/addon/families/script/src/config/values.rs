//! Reading `variables.json` (name → any JSON) and `secrets.json` (name → string).

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{DiagnosticCode, JsonPath, VanillaText};
use bedrock_addon_core::json::Value;

use super::SecretString;
use super::read::{Sink, Twice, child, object, span, string, value, with_root};

/// The server's text for a secret that is not a string, which it logs as a warning.
pub(super) const SECRET_NOT_A_STRING: &str =
    "Secret '{}' in file '{}' is not a string. Secrets can only be strings.";

/// Reads one `variables.json`: every member, with whatever value it has. A root that is not
/// an object gives no variables.
pub(super) fn variables(bytes: &[u8], sink: &mut Sink<'_>) -> IndexMap<String, Value> {
    let mut variables = IndexMap::new();
    with_root(bytes, Twice::Last, sink, |root, sink| {
        let at = JsonPath::new();
        let Some(members) = object(root) else {
            // Silent on the server.
            sink.warn(DiagnosticCode::InvalidType { text: None }, at, span(root));
            return;
        };
        variables.reserve(members.len());
        for (name, raw) in &members {
            let Some(slice) = root.nested(raw) else {
                continue;
            };
            if let Some(value) = value(slice, &child(&at, name), sink) {
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
            sink.warn(DiagnosticCode::InvalidType { text: None }, at, span(root));
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
            let text = Some(VanillaText::Formatted {
                template: SECRET_NOT_A_STRING,
                args: vec![name.to_owned(), sink.name.clone()],
            });
            sink.warn_logged(DiagnosticCode::InvalidType { text }, child(&at, name), span);
        }
    });
    secrets
}

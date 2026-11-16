//! Reading `variables.json` (name → any JSON) and `secrets.json` (name → string): each an
//! object read as a map under the folder's reading.

use crate::codes;
use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{Diagnostic, Severity};
use bedrock_addon_core::json::kit::{MemberOrder, SpanRule};
use bedrock_addon_core::json::{Value, Winner};
use bedrock_addon_core::schema::{read_document, read_entries};

use super::SecretString;
use super::read::{FOLDER_PROFILE, Sink, folder_read, with_root};

/// Reads one `variables.json`: every member, with whatever value it has. A root that is not
/// an object gives no variables (silent on the server).
pub(super) fn variables(bytes: &[u8], sink: &mut Sink<'_>) -> IndexMap<String, Value> {
    let mut variables = IndexMap::new();
    with_root(bytes, Winner::Last, sink, |root, found| {
        variables = read_document(root, &mut folder_read(found)).unwrap_or_default();
    });
    variables
}

/// Reads one `secrets.json`: the members whose value is a string. Any other member is
/// skipped with the server's warning, which names the secret and never its value; the
/// warnings come in the byte order of the names, as the server prints them, and no finding
/// spans the file's bytes.
pub(super) fn secrets(bytes: &[u8], sink: &mut Sink<'_>) -> IndexMap<String, SecretString> {
    let mut secrets = IndexMap::new();
    let file = sink.name.clone();
    with_root(bytes, Winner::Last, sink, |root, found| {
        let profile = FOLDER_PROFILE
            .with_members(MemberOrder::SortedInPlace)
            .with_spans(SpanRule::Withheld);
        let mut read = folder_read(found).profile(profile);
        let secret = |node: &bedrock_addon_core::json::kit::Node<'_>,
                      read: &mut bedrock_addon_core::json::kit::Read<'_>| {
            let secret = node.value().as_string();
            if secret.is_none() {
                let name = node.path().last_key().unwrap_or_default();
                read.push(
                    Diagnostic::of(codes::SECRET_NOT_A_STRING.with([name, file.as_str()]))
                        .severity(Severity::Warning)
                        .at(node.path().clone()),
                );
            }
            secret.map(|secret| SecretString::new(secret.as_str()))
        };
        // A root that is no object: silent on the server.
        secrets = read_entries(root, &mut read, secret).unwrap_or_default();
        read.finish();
    });
    secrets
}

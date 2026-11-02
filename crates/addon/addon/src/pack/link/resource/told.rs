//! What the resource-stack rules about blocks share: the findings told once per member, and
//! vanilla's texts with their arguments filled in.

use std::collections::HashSet;

use bedrock_addon_core::diagnostic::VanillaText;
use bedrock_addon_core::document::Origin;

/// Vanilla's text `template` with `args`.
pub(super) fn formatted(template: &'static str, args: &[&str]) -> VanillaText {
    VanillaText::Formatted {
        template,
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
    }
}

/// What was told already: one finding per member, however many blocks an entry applies to (a
/// legacy name is split onto every block it once stood for) and however many blocks share a
/// culling document.
#[derive(Debug, Default)]
pub(super) struct Told(HashSet<(Option<usize>, String, String, &'static str, String)>);

impl Told {
    /// Whether this is the first finding about `what` at the member `member` of the entry
    /// `key` of the document of `origin`.
    pub(super) fn first(
        &mut self,
        origin: &Origin,
        key: &str,
        member: &'static str,
        what: &str,
    ) -> bool {
        self.0.insert((
            origin.pack.index(),
            origin.path.as_str().to_owned(),
            key.to_owned(),
            member,
            what.to_owned(),
        ))
    }
}

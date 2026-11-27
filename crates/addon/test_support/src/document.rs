//! The outcome of a document read: loaded or refused, and written back.

use std::fmt::Debug;

use bedrock_addon_core::diagnostic::Diagnostics;
use bedrock_addon_core::document::{Content, Document, Rejected};

use crate::diagnostics::lines;

/// Requires a read to have loaded; the document.
#[track_caller]
pub fn loaded<T: Debug>(result: Result<Document<T>, Rejected>) -> Document<T> {
    match result {
        Ok(document) => document,
        Err(rejected) => panic!("rejected: {}", lines(&rejected.diagnostics).join("\n")),
    }
}

/// Requires a read to have been refused; its findings.
#[track_caller]
pub fn rejected<T: Debug>(result: Result<Document<T>, Rejected>) -> Diagnostics {
    match result {
        Ok(document) => panic!("loaded: {:#?}", document.value()),
        Err(rejected) => rejected.diagnostics,
    }
}

/// Writes `document`, reads the text again with `parse` and requires an equal value and an
/// identical second write; the text written.
#[track_caller]
pub fn round_trip<T: Content + PartialEq + Debug>(
    document: &Document<T>,
    parse: impl Fn(&str) -> Result<Document<T>, Rejected>,
) -> String {
    let text = document.to_json().unwrap();
    let again = loaded(parse(&text));
    assert_eq!(again.value(), document.value(), "{text}");
    assert_eq!(again.to_json().unwrap(), text);
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::diagnostic::{Diagnostic, codes};
    use bedrock_addon_core::document::Origin;
    use bedrock_addon_core::pack_identity::{PackPath, PackRef, PackSide};

    /// A refusal of an empty file of the app-data layer, with its one finding.
    fn refused_empty_file() -> Rejected {
        let mut d = Diagnostics::default();
        d.push(Diagnostic::info(&codes::EMPTY_DOCUMENT));
        let origin = Origin::file(
            PackRef::app_data(),
            PackSide::Resource,
            PackPath::from_static("entity/empty.json"),
        );
        Rejected::new(origin, d)
    }

    #[test]
    fn rejected_returns_the_findings_of_a_refused_document() {
        let d = rejected::<()>(Err(refused_empty_file()));
        assert_eq!(d.len(), 1, "the refusal's findings");
        assert!(
            d.iter().all(|f| f.is(&codes::EMPTY_DOCUMENT)),
            "{:?}",
            d.iter().collect::<Vec<_>>()
        );
    }
}

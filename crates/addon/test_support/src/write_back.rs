//! Write-back as authored: a document that loads is written as the JSON value it was read from.

use bedrock_addon_core::document::Content;
use bedrock_addon_core::load::LoadContext;
use bedrock_addon_core::pack_identity::PackPath;

use crate::json::json;

/// Every text loads and is written back as the same JSON value.
pub fn as_authored<T: Content>(texts: &[&str]) {
    for text in texts {
        let path = PackPath::from_static("x.json");
        let document = T::parse(&path, text.as_bytes(), &LoadContext::detached())
            .unwrap_or_else(|rejected| panic!("{text}: {:?}", rejected.diagnostics));
        let written = document
            .to_json()
            .unwrap_or_else(|e| panic!("{text}: {e:?}"));
        assert_eq!(json(&written), json(text), "{} wrote {written}", T::NAME);
        // What is written is stable, and loads to the same document.
        let again = T::parse(&path, written.as_bytes(), &LoadContext::detached()).unwrap();
        assert_eq!(again.to_json().unwrap(), written, "{text}");
    }
}

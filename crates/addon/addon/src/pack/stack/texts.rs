//! What the stack makes of the text families together: the `languages` list says which language
//! codes a pack lists, and the `lang` family's pack keywords are looked up through it.

use bedrock_addon_core::pack_identity::PackIdVersion;

use super::{PackStack, Side};
use crate::families::lang::{Language, Translations, pack_keyword_value};

impl<S: Side> PackStack<S> {
    /// What a pack shows for a header `name` or `description`: `text` itself unless it is a
    /// pack keyword; for a keyword, its value in the first of `tables` (the current language,
    /// then the base language) that the pack's own `texts/languages.json` lists and whose
    /// table has it; otherwise the keyword itself.
    #[must_use]
    pub fn pack_keyword_value<'a>(
        &self,
        text: &'a str,
        pack: &PackIdVersion,
        tables: &[(&Language, &'a Translations)],
    ) -> &'a str {
        let languages = self.languages();
        let listed = |language: &Language| {
            languages
                .layers()
                .iter()
                .any(|layer| layer.origin().pack.id() == pack && layer.codes.contains(language))
        };
        pack_keyword_value(text, pack, listed, tables)
    }
}

//! The credits (`credits/credits.json`), read from the game's own data and never from a pack.
//! One fixed-path file with its own reader in the client; it carries no `format_version`.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod load;
mod read;

use bedrock_addon_core::document::Extra;
use bedrock_addon_core::json::Value;

#[doc(hidden)]
pub use load::load_credits;
pub use read::check_starter_seeds;

/// The path of the credits, in the game's own data (never in a pack).
pub const CREDITS_PATH: &str = "credits/credits.json";
/// The files of the game's own data that a resource pack carries in vain, credits aside:
/// a resource stack's report names each with an `IgnoredFile` info.
pub const APP_ONLY_FILES: [&str; 4] = [
    "CreativeStarterSeeds.txt",
    "StarterSeeds.txt",
    "manifest_publish.json",
    "texture_backwards_compatibility_mapping.json",
];

/// The credits (`credits/credits.json` of the game's own data): sections of disciplines of
/// titles of names.
///
/// JSON: `[{"section": "Mojang", "disciplines": [{"discipline": "Design", "titles":
/// [{"title": "Game Designer", "names": ["A. Person"]}]}]}]`. The game reads the file from its
/// own data and never from a pack; a pack's copy is ignored.
#[derive(Clone, Debug, Default, PartialEq)]
#[doc(alias = "credits.json")]
pub struct Credits {
    /// The sections the game shows, in order: the root's elements that are objects with a
    /// member.
    pub sections: Vec<CreditsSection>,
    /// The root as the author wrote it, kept when the fields cannot say it (a root that is no
    /// array, an element the game leaves out, a list that is no array, a heading or a name
    /// that is no string).
    /// `to_json` writes it for as long as the fields are still the game's reading of it; a
    /// value that was changed, or built by hand, writes its fields. The entries the game
    /// skipped are in this root alone, so one edit of any field drops every one of them from
    /// the written file, not only those beside the edited member. `PartialEq` and `Debug`
    /// include the root: a loaded value that kept one differs from a hand-built value with
    /// the same fields.
    pub written: Option<Value>,
}

/// One section of the credits.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreditsSection {
    /// JSON `section`: the heading.
    pub section: Option<String>,
    /// JSON `disciplines`.
    pub disciplines: Option<Vec<CreditsDiscipline>>,
    /// The other members.
    pub extra: Extra,
}

/// One discipline of a credits section.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreditsDiscipline {
    /// JSON `discipline`: the heading.
    pub discipline: Option<String>,
    /// JSON `titles`.
    pub titles: Option<Vec<CreditsTitle>>,
    /// The other members.
    pub extra: Extra,
}

/// One title of a credits discipline, with the people who hold it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CreditsTitle {
    /// JSON `title`.
    pub title: Option<String>,
    /// JSON `names`, each read as the game reads any value as a string.
    pub names: Option<Vec<String>>,
    /// The other members.
    pub extra: Extra,
}

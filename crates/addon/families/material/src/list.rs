//! A material list file (`materials/common.json`): which `.material` files a group loads.

use bedrock_addon_core::document::Extra;

/// Which list file a [`MaterialList`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MaterialListKind {
    /// `materials/common.json`: always loaded.
    Common,
    /// `materials/fancy.json`: loaded in the Fancy graphics mode.
    Fancy,
    /// `materials/sad.json`: loaded in every other graphics mode (Simple, Vibrant Visuals, ray
    /// tracing).
    Sad,
    /// `materials/gameface.json`: shipped by vanilla; the client's material reload does not
    /// ask for it.
    Gameface,
}

impl MaterialListKind {
    /// The four list files.
    pub const ALL: [Self; 4] = [Self::Common, Self::Fancy, Self::Sad, Self::Gameface];

    /// The list's path in a pack.
    pub fn path(self) -> &'static str {
        match self {
            Self::Common => "materials/common.json",
            Self::Fancy => "materials/fancy.json",
            Self::Sad => "materials/sad.json",
            Self::Gameface => "materials/gameface.json",
        }
    }

    /// The list at `path`, when it is one of the four.
    pub fn from_path(path: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.path() == path)
    }
}

/// One list file: a JSON array of entries, each naming a `.material` file.
///
/// A pack's list **adds** to the lists of the packs below it. The copies are processed from
/// the highest pack down, and a path one list already loaded is not loaded again, so the first
/// entry for a path, with its defines, is the one that counts.
#[doc(alias = "common.json")]
#[doc(alias = "fancy.json")]
#[doc(alias = "sad.json")]
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialList {
    /// Which list the file is: decides its path.
    pub kind: MaterialListKind,
    /// The entries in file order.
    pub entries: Vec<ListEntry>,
}

/// One entry of a list file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ListEntry {
    /// `path`: the `.material` file (`materials/entity.material`).
    pub path: String,
    /// `defines`: defines every parentless material of the file starts with.
    pub defines: Option<Vec<String>>,
    /// `+defines`: added after `defines` (vanilla's `fancy.json` adds `FANCY` this way).
    pub add_defines: Option<Vec<String>>,
    /// `-defines`: removed after the other two are added.
    pub sub_defines: Option<Vec<String>>,
    /// Every other member, as written. The client rejects `tag` with an error.
    pub extra: Extra,
}

impl ListEntry {
    /// The defines the entry gives its file's parentless materials: `defines`, then
    /// `+defines`, less `-defines`, each name once, in the order first written.
    pub fn base_defines(&self) -> Vec<String> {
        let mut defines = indexmap::IndexSet::new();
        for name in self.defines.iter().chain(&self.add_defines).flatten() {
            defines.insert(name.as_str());
        }
        let removed: std::collections::HashSet<&str> = self
            .sub_defines
            .iter()
            .flatten()
            .map(String::as_str)
            .collect();
        defines
            .into_iter()
            .filter(|name| !removed.contains(name))
            .map(str::to_owned)
            .collect()
    }
}

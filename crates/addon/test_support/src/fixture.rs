//! The fixture folder a package's tests read, found from the caller's own
//! `env!("CARGO_MANIFEST_DIR")`.

use std::path::{Path, PathBuf};

/// The `tests/fixtures` folder the tests of the package at `manifest_dir` read: the package's
/// own when it has one (`bedrock_addon`, core), else `bedrock_addon`'s, which is
/// `../../addon/tests/fixtures` from a family crate's folder (`crates/addon/families/<name>`):
/// a family's tests read the fixtures they share with `bedrock_addon`'s there. A test passes
/// its `env!("CARGO_MANIFEST_DIR")`, so a file that both packages compile resolves in each.
pub fn root(manifest_dir: impl AsRef<Path>) -> PathBuf {
    let manifest = manifest_dir.as_ref();
    let own = manifest.join("tests/fixtures");
    if own.is_dir() {
        own
    } else {
        manifest.join("../../addon/tests/fixtures")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A crate folder beside this one, `crates/addon/<name>`.
    fn sibling(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(name)
    }

    /// The folder `path` names, with `..` resolved; `None` when there is none.
    fn folder(path: &Path) -> Option<PathBuf> {
        std::fs::canonicalize(path).ok().filter(|p| p.is_dir())
    }

    #[test]
    fn the_fixture_root_of_a_family_is_addon_s() {
        let addon_s = folder(&sibling("addon/tests/fixtures"));
        assert!(
            addon_s.is_some(),
            "bedrock_addon has a tests/fixtures folder"
        );
        let family = sibling("families/loot");
        assert!(
            !family.join("tests/fixtures").exists(),
            "loot has none of its own"
        );
        assert_eq!(
            folder(&root(&family)),
            addon_s,
            "a family's root is addon's"
        );
        assert_eq!(
            folder(&root(sibling("addon"))),
            addon_s,
            "addon's root is its own"
        );
        assert_eq!(
            folder(&root(sibling("core"))),
            folder(&sibling("core/tests/fixtures")),
            "a package with its own folder reads its own"
        );
    }
}

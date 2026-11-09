//! One module per game document family, each behind the feature of its name: the behaviour
//! pack's definitions, the resource pack's, and the few families the game reads from elsewhere.
//! A family imports the core vocabulary and the shared crates, never another family; what one
//! family needs to know of another's loaded definitions it asks the stack.
//!
//! The families are registered once, in the list of `list.rs`; the modules here expand from it.

// The exceptions: the glue that names a family by hand rather than through the list, one line
// each, lands with the list's last consumer.

mod list;
pub(crate) use list::with_families;

/// The re-export of each family's module, and under test the feature names in list order.
macro_rules! reexports {
    ($($feature:literal $name:ident = $krate:ident $(:: $module:ident)* { $($body:tt)* })*) => {
        $(
            #[cfg(feature = $feature)]
            #[cfg_attr(docsrs, doc(cfg(feature = $feature)))]
            #[doc(inline)]
            pub use $krate $(:: $module)* as $name;
        )*

        /// The feature of every family in the list, compiled in or not, in list order.
        #[cfg(test)]
        const NAMES: &[&str] = &[$($feature),*];
    };
}

with_families!(reexports);

#[cfg(test)]
mod tests {
    use super::NAMES;

    /// The features of this crate's manifest that turn on a family crate (a
    /// `dep:bedrock_addon_family_*` entry in their list), read by name from `[features]`.
    fn family_features_of_the_manifest() -> Vec<String> {
        let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
        let mut features = Vec::new();
        let mut in_features = false;
        let mut entry: Option<(String, String)> = None;
        for line in manifest.lines() {
            let line = line.split('#').next().unwrap_or_default().trim();
            if line.starts_with('[') && entry.is_none() {
                in_features = line == "[features]";
                continue;
            }
            if !in_features || line.is_empty() {
                continue;
            }
            let (name, value) = if let Some((name, value)) = entry.take() {
                (name, value + line)
            } else {
                let (name, value) = line.split_once('=').expect("a feature line");
                (name.trim().to_owned(), value.trim().to_owned())
            };
            if value.matches('[').count() > value.matches(']').count() {
                entry = Some((name, value));
                continue;
            }
            if value.contains("\"dep:bedrock_addon_family_") {
                features.push(name);
            }
        }
        features
    }

    #[test]
    fn the_family_list_is_the_manifest_s_family_features() {
        let mut listed: Vec<&str> = NAMES.to_vec();
        listed.sort_unstable();
        let before = listed.len();
        listed.dedup();
        assert_eq!(listed.len(), before, "a family is listed twice");
        let mut manifest = family_features_of_the_manifest();
        manifest.sort_unstable();
        assert_eq!(
            listed, manifest,
            "the family list and the manifest's family features differ"
        );
    }
}

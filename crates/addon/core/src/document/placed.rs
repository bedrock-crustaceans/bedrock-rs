//! The helper behind every `Placed::default_path`.

use crate::id::{Id, Kind, Syntax};
use crate::pack_identity::PackPath;

/// The namespace a bare identifier of a kind that defaults to vanilla has.
const VANILLA_NAMESPACE: &str = "minecraft";

/// The longest segment written, in bytes; the pack-path limit is 1023 for the whole path.
const MAX_SEGMENT_BYTES: usize = 255;

/// `<directory>/<namespace>/<name>.json`: where `PackWriter::add` puts a root type of a family
/// whose files live in `directory` (`blocks/demo/ruby_block.json` for `demo:ruby_block`).
///
/// A bare identifier of a kind that defaults to `minecraft:` gets `minecraft`; an identifier
/// without a namespace (`geometry.ruby`) is `<directory>/<text>.json`. The identifier's path
/// may hold `/` and becomes subdirectories. An entity's `<event>` and an item's `:aux` suffix
/// are left out, as the identifier compares without them. A malformed identifier (the loader
/// keeps them as written) still gives a valid path: a `.` or `..` segment becomes `_`, and so
/// does each `:` and ASCII control character.
#[doc(hidden)]
pub fn default_path<K: Kind>(directory: &str, id: &Id<K>) -> PackPath {
    let Parts {
        mut folders,
        namespace,
        name,
    } = parts(directory, id);
    folders.extend(namespace);
    folders.extend(name);
    path_of(&folders)
}

/// [`default_path`] for a family whose walk reads no sub-folder of `directory` in a pack
/// shipped with the game: a `minecraft:` identifier (written or defaulted) is
/// `<directory>/<name>.json`, the `/` of its name becoming `_`, so that every pack's walk
/// reads it back; any other namespace keeps `<directory>/<namespace>/<name>.json`, which the
/// packs that may hold it read.
///
/// The path is not one per identifier: `minecraft:a/b` and `minecraft:a_b` are both
/// `<directory>/a_b.json` (and, as with [`default_path`], two identifiers that differ only
/// in what [`sanitise`] replaces). A writer that places two such documents by their default
/// paths writes one file over the other; the path is a default, and the caller that holds
/// both gives one of them another.
#[doc(hidden)]
pub fn default_path_flat_for_vanilla<K: Kind>(directory: &str, id: &Id<K>) -> PackPath {
    let Parts {
        mut folders,
        namespace,
        name,
    } = parts(directory, id);
    match namespace {
        // A name that, joined into one file name, is longer than a pack path may be keeps the
        // nested `<directory>/<namespace>/<name>.json` instead.
        Some(namespace)
            if namespace.eq_ignore_ascii_case(VANILLA_NAMESPACE)
                && PackPath::new(&name.join("_")).is_ok() =>
        {
            folders.push(name.join("_"));
        }
        namespace => {
            folders.extend(namespace);
            folders.extend(name);
        }
    }
    path_of(&folders)
}

/// [`default_path`] for a family whose walk reads the files directly in `directory` only:
/// `<directory>/<name>.json` for a `minecraft:` identifier and
/// `<directory>/<namespace>_<name>.json` for any other, the `/` of the name becoming `_`.
///
/// The path is not one per identifier: with `_` joining the namespace and every segment of
/// the name, `a:b_c`, `a_b:c` and `a:b/c` are all `<directory>/a_b_c.json`, and `minecraft:x`
/// shares `<directory>/x.json` with nothing of another namespace but does with
/// `minecraft:x` written bare. A writer that places two such documents by their default
/// paths writes one file over the other; the path is a default, and the caller that holds
/// both gives one of them another.
#[doc(hidden)]
pub fn default_path_flat<K: Kind>(directory: &str, id: &Id<K>) -> PackPath {
    let Parts {
        mut folders,
        namespace,
        mut name,
    } = parts(directory, id);
    if let Some(namespace) = namespace
        && !namespace.eq_ignore_ascii_case(VANILLA_NAMESPACE)
    {
        name.insert(0, namespace);
    }
    folders.push(name.join("_"));
    path_of(&folders)
}

/// The sanitised parts of a default path.
struct Parts {
    /// The segments of the directory.
    folders: Vec<String>,
    /// The namespace, written or defaulted.
    namespace: Option<String>,
    /// The segments of the name, the last with `.json`.
    name: Vec<String>,
}

fn parts<K: Kind>(directory: &str, id: &Id<K>) -> Parts {
    let namespace = match (id.namespace(), K::SYNTAX) {
        (Some(namespace), _) => Some(namespace),
        (
            None,
            Syntax::Namespaced {
                default_vanilla: true,
            },
        ) => Some(VANILLA_NAMESPACE),
        (None, _) => None,
    };
    let mut name: Vec<String> = segments(id.path()).map(sanitise).collect();
    if name.is_empty() {
        name.push("_".to_owned());
    }
    if let Some(last) = name.last_mut() {
        last.push_str(".json");
    }
    Parts {
        folders: segments(directory).map(sanitise).collect(),
        namespace: namespace.map(sanitise),
        name,
    }
}

/// The pack path of `segments`.
fn path_of(segments: &[String]) -> PackPath {
    segments
        .iter()
        .map(|segment| PackPath::new(segment).unwrap_or_else(|_| PackPath::from_static("_")))
        .fold(PackPath::ROOT, |path, segment| path.join(&segment))
}

/// The non-empty segments of a `/`- or `\`-separated text.
fn segments(text: &str) -> impl Iterator<Item = &str> {
    text.split(['/', '\\'])
        .filter(|segment| !segment.is_empty())
}

/// One path segment the pack-path rules accept: `.` and `..` become `_`, as do `:` and ASCII
/// control characters; at most [`MAX_SEGMENT_BYTES`] bytes (cut at a character boundary).
fn sanitise(segment: &str) -> String {
    if segment == "." || segment == ".." {
        return "_".to_owned();
    }
    let mut out = String::with_capacity(segment.len());
    for c in segment.chars() {
        let c = if c == ':' || c.is_ascii_control() {
            '_'
        } else {
            c
        };
        if out.len() + c.len_utf8() > MAX_SEGMENT_BYTES {
            break;
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::kind;

    fn flat(id: &str) -> String {
        default_path_flat("dimensions", &Id::<kind::Dimension>::parse(id))
            .as_str()
            .to_owned()
    }

    fn flat_for_vanilla(id: &str) -> String {
        default_path_flat_for_vanilla("biomes", &Id::<kind::Biome>::parse(id))
            .as_str()
            .to_owned()
    }

    /// The flat default paths join with `_`, so identifiers that differ only in where a `_`,
    /// a `:` or a `/` stands share one path. Pinned so the doc comments stay true.
    #[test]
    fn the_flat_default_paths_are_not_one_per_identifier() {
        assert_eq!(flat("a:b_c"), "dimensions/a_b_c.json");
        assert_eq!(flat("a_b:c"), "dimensions/a_b_c.json");
        assert_eq!(flat("a:b/c"), "dimensions/a_b_c.json");
        // The vanilla namespace has no prefix; another namespace always has its own.
        assert_eq!(flat("minecraft:overworld"), "dimensions/overworld.json");
        assert_eq!(flat("demo:overworld"), "dimensions/demo_overworld.json");
        assert_eq!(flat("minecraft:a_b"), "dimensions/a_b.json");
        assert_eq!(flat("a:b"), "dimensions/a_b.json");

        assert_eq!(flat_for_vanilla("minecraft:a/b"), "biomes/a_b.json");
        assert_eq!(flat_for_vanilla("minecraft:a_b"), "biomes/a_b.json");
        // A bare biome identifier is `minecraft:`.
        assert_eq!(flat_for_vanilla("a_b"), "biomes/a_b.json");
        // Another namespace keeps its folder and its sub-folders: no collision with the above.
        assert_eq!(flat_for_vanilla("demo:a/b"), "biomes/demo/a/b.json");
        assert_eq!(flat_for_vanilla("demo:a_b"), "biomes/demo/a_b.json");
    }
}

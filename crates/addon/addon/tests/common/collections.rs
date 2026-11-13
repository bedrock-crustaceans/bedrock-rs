//! What one load of a world came to, as text that two loads can be compared by: the report and
//! every collection of both stacks, collection by collection: two loads of one world give equal
//! `Debug` output of the `StackReport` and of every collection's `iter()`.
//!
//! Used by `crates/addon/addon/tests/determinism.rs` (the five completion orders) and by the driver
//! crate's `full_strategies.rs`, `full_strategies_client_install_a.rs` and
//! `full_strategies_client_install_b.rs` (`Strategy::Sequential` against `Strategy::Rayon`); it
//! names `bedrock_addon` alone.
//!
//! The file is `bedrock_addon`'s alone. Neither crate can hand the other a test helper, so the
//! driver crate's three targets include it by `#[path]`, and its `tests/package_includes.rs`
//! lists each of those includes.
//!
//! The collections are enumerated from the stacks themselves: a `PackStack`'s `Debug` lists
//! every accessor of its side whose cache is filled, as `(Some(<Collection variant>), "<the
//! collection's Debug>")`, in accessor-list order. After `load_all` or a `StackLoader` over
//! `CollectionSet::ALL` that is every accessor compiled in, so nothing here is a list kept by
//! hand.
//!
//! Why the `Debug` of a collection is its iteration order, kind by kind:
//!
//! - `Registry<T>` and `Files<T>` print `entries` as `debug_map().entries(self.iter())`: the
//!   very iterator, in its order (`bedrock_addon_core` `document/by_id.rs`, `by_path.rs`).
//!   `rejected` and `diagnostics` are vectors in walk order; a registry's `beta` is the beta
//!   registry's `iter()`.
//! - `Merged<T, L>` prints `value` (the family's merged struct), `origins` (an `IndexMap`,
//!   insertion order, which is what `Merged::origin` walks) and `layers` (a vector, lowest
//!   pack first: what `layers()` iterates).
//! - The group types (`Jigsaw`, `Recipes`, `FeatureRules`, `Poi`, `Cameras`, `AimAssist`,
//!   `SoundFiles`, `DeferredSettings`, `DataDrivenUi`, `TextureAtlases`, `TextureLists`,
//!   `TextureSets`, `TextureFiles`, `Fonts`, `Materials`) derive
//!   `Debug` (`Recipes` writes the same by hand) over fields that are registries, files,
//!   merged values, vectors and `IndexMap`s: declaration order, each field as above.
//! - A keyed accessor (`translations`) prints its keys as a list of `(key, collection)` in
//!   walk order; a slice accessor (`scripts`) prints its vector.
//!
//! No collection sorts for `Debug` and none prints a hash map, so a difference in iteration
//! order is a difference in the text.
//!
//! What the text does not show. Some values inside the compared collections have a `Debug`
//! that prints less than they hold, so two loads that differ only there have the same text.
//! Five of single families:
//!
//! - a material's `NameSet` prints "N names", the count and not the names
//!   (`families/material/set.rs`);
//! - a geometry's `Inherited` prints its identifier, the number of bones and the chain's depth,
//!   not the bones (`geometry/mod.rs`);
//! - `JigsawMetadata` prints `body_len`, the length of the cache document and not its bytes
//!   (`families/jigsaw/metadata.rs`);
//! - a structure's `Layout` prints `kept`, whether the source is held, and nothing of it
//!   (`structure/mod.rs`);
//! - a UI definition's `Inherited` prints the namespace and name it inherits from, not what it
//!   resolved to (`ui/chain.rs`).
//!
//! And of every registry and path map everything but the effective documents: their `Debug`
//! prints `entries` (the effective document of each identifier or path), `rejected` and
//! `diagnostics`, and of a registry's beta registry the identifiers of its effective documents.
//! It prints no document that is not effective (a shadowed copy kept with
//! `LoadOptions::keep_shadowed`; a copy under a refused or ignored one; a stable definition a
//! beta one replaced, and the reverse), nothing of a beta document, and not the beta
//! registry's rejected files and findings (`bedrock_addon_core` `document/by_id.rs`,
//! `by_path.rs`). `Registry::all_layers` and `Files::all_layers` list every identifier and
//! path anything is kept of with all of its layers; `unprinted.rs` takes the whole table from
//! them.
//!
//! The first two are results of a combine step. Each is the type's own `Debug`, written so
//! for a reason of its own (a chain printed once, a cache not dumped), and stays so. The
//! comparison reads what they leave out through the public accessors instead:
//! `unprinted.rs` gives each such value as an [`Unprinted`], a snapshot carries them
//! ([`Snapshot::with_unprinted`]) and [`differences`] compares them value by value, by name.
//!
//! Outside the collections: a stack's packs are printed with the length of their file
//! listing, and the bytes a pack holds are in no `Debug`. `StackSet::link()` is the public
//! view of both (a rule reads a file no collection holds; another resolves a name by the
//! listing), so a snapshot carries what linking says.

#![allow(dead_code, reason = "each test file uses part of it")]

use bedrock_addon::pack::{StackReport, StackSet};

/// One load, as text.
pub struct Snapshot {
    /// `Debug` of the `StackReport`: every finding in load order (`StackReport::iter`).
    pub report: String,
    /// `Display` of the `StackReport`: one line per finding, sorted by pack, path and offset.
    pub display: String,
    /// `{:#?}` of the behaviour stack: its packs, the hidden ones and its collections.
    pub behavior: String,
    /// `{:#?}` of the resource stack.
    pub resource: String,
    /// `Debug` of what `StackSet::link()` says of the loaded set, the lints included: every
    /// finding of the link pass in its order. The pass reads what no collection holds (a
    /// pack's resident bytes, the file listing of a behaviour stack's packs).
    pub link: String,
    /// What the `Debug` of the collections leaves out, value by value (`unprinted.rs`); empty
    /// until [`with_unprinted`](Self::with_unprinted) gives it.
    pub unprinted: Vec<Unprinted>,
}

/// A value of a loaded collection whose `Debug` prints less than the value holds: the part
/// the text leaves out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unprinted {
    /// The value and the part, as a difference names them (`the body of <path>`).
    pub what: String,
    /// The part, as bytes: a text, or the bytes themselves.
    pub content: Vec<u8>,
}

impl Unprinted {
    /// The part `content` of the value `what`.
    pub fn new(what: String, content: impl Into<Vec<u8>>) -> Unprinted {
        Unprinted {
            what,
            content: content.into(),
        }
    }
}

/// One collection of a stack: an accessor whose cache is filled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry<'a> {
    /// `"behavior"` or `"resource"`.
    pub side: &'static str,
    /// The accessor's `Collection` variant, as its `Debug` prints it.
    pub name: &'a str,
    /// The collection's `Debug`, as the stack's `Debug` holds it (a string literal's inside,
    /// still escaped).
    pub text: &'a str,
}

impl Snapshot {
    /// The texts of `set` and of `report`, the report of that set (`set.load_all()`: the link
    /// pass is run after it, over everything loaded).
    pub fn of(set: &StackSet, report: &StackReport<'_>) -> Snapshot {
        Snapshot {
            report: format!("{report:?}"),
            display: report.to_string(),
            behavior: format!("{:#?}", set.behavior),
            resource: format!("{:#?}", set.resource),
            link: format!("{:?}", set.link()),
            unprinted: Vec::new(),
        }
    }

    /// The snapshot with `unprinted`, what `unprinted::of` reads from the same set.
    pub fn with_unprinted(mut self, unprinted: Vec<Unprinted>) -> Snapshot {
        self.unprinted = unprinted;
        self
    }

    /// Every loaded collection: the behaviour stack's in accessor-list order, then the
    /// resource stack's.
    pub fn collections(&self) -> Vec<Entry<'_>> {
        let mut entries = collections_of("behavior", &self.behavior);
        entries.extend(collections_of("resource", &self.resource));
        entries
    }
}

/// The collections a stack's pretty `Debug` lists.
///
/// The text is `PackStack { side, packs, hidden, collections: [(Some(Variant), "…"), …], .. }`
/// printed with `{:#?}`: the field's line is indented by four spaces, a variant's by sixteen
/// and a collection's text, a string literal on one line (a line break inside it is written
/// `\n`), by twelve. Any other line inside the field is a change of that format and panics.
fn collections_of<'a>(side: &'static str, stack: &'a str) -> Vec<Entry<'a>> {
    let mut lines = stack.lines();
    let mut entries = Vec::new();
    loop {
        match lines.next() {
            Some("    collections: [],") => return entries,
            Some("    collections: [") => break,
            Some(_) => {}
            None => panic!("the {side} stack's Debug has no `collections` field"),
        }
    }
    let mut name = None;
    for line in lines {
        match line {
            "    ]," => return entries,
            "        (" | "            Some(" | "            )," | "        )," => {}
            _ => {
                if let Some(text) = line
                    .strip_prefix("            \"")
                    .and_then(|rest| rest.strip_suffix("\","))
                {
                    let name = name.take().expect("a collection's variant before its text");
                    entries.push(Entry { side, name, text });
                } else if let Some(variant) = line
                    .strip_prefix("                ")
                    .and_then(|rest| rest.strip_suffix(','))
                    .filter(|variant| variant.chars().all(char::is_alphanumeric))
                {
                    name = Some(variant);
                } else {
                    panic!("the {side} stack's `collections` field has the line {line:?}");
                }
            }
        }
    }
    panic!("the {side} stack's `collections` field does not end");
}

/// Where two texts first differ, with what each holds there; `None` when they are equal.
fn difference(found: &str, reference: &str) -> Option<String> {
    difference_of_bytes(found.as_bytes(), reference.as_bytes())
}

/// [`difference`] for bytes: the excerpts are shown as text, a byte that is none escaped.
fn difference_of_bytes(found: &[u8], reference: &[u8]) -> Option<String> {
    if found == reference {
        return None;
    }
    let at = found
        .iter()
        .zip(reference)
        .position(|(a, b)| a != b)
        .unwrap_or(found.len().min(reference.len()));
    let excerpt = |bytes: &[u8]| {
        let from = at.saturating_sub(120);
        let to = (at + 120).min(bytes.len());
        match std::str::from_utf8(bytes) {
            // A text is cut at characters.
            Ok(text) => {
                let (mut from, mut to) = (from, to);
                while !text.is_char_boundary(from) {
                    from -= 1;
                }
                while !text.is_char_boundary(to) {
                    to += 1;
                }
                text[from..to].to_owned()
            }
            Err(_) => bytes[from..to].escape_ascii().to_string(),
        }
    };
    Some(format!(
        "first difference at byte {at} (lengths {} and {}):\n  found:     …{}…\n  reference: …{}…",
        found.len(),
        reference.len(),
        excerpt(found),
        excerpt(reference)
    ))
}

/// Every part of `found` that differs from `reference`, named: the report in load order, its
/// `Display`, each collection of each stack, what else a stack's `Debug` holds (its packs
/// and the hidden ones), what linking says, and each value whose `Debug` leaves a part out, by
/// that part. Empty when the two loads are the same.
pub fn differences(found: &Snapshot, reference: &Snapshot) -> Vec<String> {
    let mut differences = Vec::new();
    let mut compare = |what: String, found: &str, reference: &str| {
        if let Some(difference) = difference(found, reference) {
            differences.push(format!("{what}: {difference}"));
        }
    };
    compare(
        String::from("the report (findings in load order)"),
        &found.report,
        &reference.report,
    );
    compare(
        String::from("the report's Display"),
        &found.display,
        &reference.display,
    );
    let (ours, theirs) = (found.collections(), reference.collections());
    let names = |entries: &[Entry<'_>]| -> Vec<String> {
        entries
            .iter()
            .map(|entry| format!("{} {}", entry.side, entry.name))
            .collect()
    };
    if names(&ours) == names(&theirs) {
        for (ours, theirs) in ours.iter().zip(&theirs) {
            compare(
                format!("collection {} of the {} stack", ours.name, ours.side),
                ours.text,
                theirs.text,
            );
        }
    } else {
        compare(
            String::from("the loaded collections"),
            &names(&ours).join(", "),
            &names(&theirs).join(", "),
        );
    }
    // The packs, the hidden packs and anything a collection's text did not show.
    compare(
        String::from("the behaviour stack"),
        &found.behavior,
        &reference.behavior,
    );
    compare(
        String::from("the resource stack"),
        &found.resource,
        &reference.resource,
    );
    compare(
        String::from("what linking says"),
        &found.link,
        &reference.link,
    );
    // What the texts above leave out, value by value.
    let whats = |snapshot: &Snapshot| -> Vec<String> {
        snapshot
            .unprinted
            .iter()
            .map(|value| value.what.clone())
            .collect()
    };
    if whats(found) == whats(reference) {
        for (ours, theirs) in found.unprinted.iter().zip(&reference.unprinted) {
            if let Some(difference) = difference_of_bytes(&ours.content, &theirs.content) {
                differences.push(format!("{}: {difference}", ours.what));
            }
        }
    } else {
        compare(
            String::from("the values whose Debug leaves a part out"),
            &whats(found).join("\n"),
            &whats(reference).join("\n"),
        );
    }
    differences
}

/// Requires `found` to be the load `reference` is: the same report, in load order and as
/// displayed, the same `Debug` of every collection of both stacks and of the stacks, and the
/// same content where a `Debug` leaves a part out.
#[track_caller]
pub fn assert_same(what: &str, found: &Snapshot, reference: &Snapshot) {
    let differences = differences(found, reference);
    assert!(
        differences.is_empty(),
        "{what}: the loads differ in {} places:\n{}",
        differences.len(),
        differences.join("\n")
    );
}

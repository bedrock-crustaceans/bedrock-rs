//! The registry and files builders and [`Merged::merge`] collect a document's findings with
//! `Diagnostics::append`, which keeps the places an upgrade's moves left
//! (`Diagnostics::place_of`) along with the findings, for a stable, a rejected and a merged
//! copy alike. (Before, they collected with `extend`, which dropped the places.)
//!
//! The places are seen only through the collections' own `Diagnostics`, which no public path
//! hands out: `Registry::diagnostics`, `Files::diagnostics` and `Merged::diagnostics` yield
//! the findings alone, and the link pass's `place_of` calls read each document's own
//! diagnostics (`Document::diagnostics`), which keep their moves either way.

use uuid::Uuid;

use super::{
    CombineRule, Content, Contribution, Definition, Document, FilesBuilder, FilesRule, Merge,
    Merged, NoBeta, Origin, RegistryBuilder, Rejected, VersionInfo,
};
use crate::__private::Sealed;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity, codes,
};
use crate::id::{Id, kind};
use crate::json::{EncodeError, WriteStyle};
use crate::load::{LoadContext, LoadOptions};
use crate::pack_identity::{PackIdVersion, PackPath, PackRef, PackSide, PackType};
use crate::version::{FormatVersion, SemVersion};

#[derive(Clone, Debug, PartialEq)]
struct Moved;

impl Sealed for Moved {}

impl Content for Moved {
    const NAME: &'static str = "moved";
    const SIDES: &'static [PackSide] = &[PackSide::Behavior];

    fn parse(_: &PackPath, _: &[u8], _: &LoadContext) -> Result<Document<Self>, Rejected> {
        Err(Rejected::new(origin(), Diagnostics::default()))
    }

    fn write_json(&self, _: &mut Vec<u8>, _: WriteStyle) -> Result<(), EncodeError> {
        Ok(())
    }

    fn to_json(&self) -> Result<String, EncodeError> {
        Ok(String::new())
    }
}

impl Definition for Moved {
    type Kind = kind::Block;
    type File = Moved;
    type Beta = NoBeta<kind::Block>;
}

fn path(key: &str) -> JsonPath {
    let mut at = JsonPath::new();
    at.push(key);
    at
}

fn origin() -> Origin {
    let pack = PackRef::new(
        0,
        PackIdVersion::new(
            Uuid::from_u128(1),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        ),
        "pack".to_owned(),
    );
    Origin::new(
        pack,
        PackSide::Behavior,
        PackPath::from_static("blocks/moved.json"),
        None,
        Vec::new(),
    )
}

/// Findings after an upgrade moved `old` to `new` and a finding was told at `new`: the
/// finding is named at `old`, and `place_of(new)` is `old`.
fn upgraded() -> Diagnostics {
    let mut diagnostics = Diagnostics::default();
    diagnostics.moved(path("old"), path("new"));
    diagnostics.push(Diagnostic::new(
        DiagnosticCode::from(&codes::INVALID_VALUE),
        Severity::Warning,
        Phase::Upgrade,
        Location::new(None, None, path("new"), None),
    ));
    diagnostics.name_as_the_file_has_it();
    assert_eq!(diagnostics.place_of(&path("new")), path("old"));
    diagnostics
}

fn document() -> Document<Moved> {
    let v = FormatVersion::new(1, 0, 0);
    Document::new(
        Moved,
        origin(),
        VersionInfo::new(Some(v), false, v, v, false),
        upgraded(),
    )
}

/// The finding is kept, named as the file has it, and so is the place.
fn kept(diagnostics: &Diagnostics) {
    let paths: Vec<_> = diagnostics
        .iter()
        .map(|d| d.location.json_path.clone())
        .collect();
    assert_eq!(paths, [path("old")]);
    assert_eq!(
        diagnostics.place_of(&path("new")),
        path("old"),
        "the combine keeps the place of the move"
    );
}

#[test]
fn registry_builder_keeps_the_places_of_a_document() {
    let registry = RegistryBuilder::combine(
        [Contribution::Stable(
            Id::from_static("demo:moved"),
            document(),
        )],
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        &LoadOptions::default(),
    );
    assert_eq!(
        registry
            .get(&Id::from_static("demo:moved"))
            .map(|document| document.diagnostics().place_of(&path("new"))),
        Some(path("old")),
        "the document keeps its own"
    );
    kept(registry.held_diagnostics());
}

#[test]
fn registry_builder_keeps_the_places_of_a_rejected_document() {
    let registry = RegistryBuilder::<Moved>::combine(
        [Contribution::Rejected(
            None,
            Rejected::new(origin(), upgraded()),
        )],
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        &LoadOptions::default(),
    );
    kept(registry.held_diagnostics());
}

#[test]
fn files_builder_keeps_the_places_of_a_copy() {
    for copy in [Ok(document()), Err(Rejected::new(origin(), upgraded()))] {
        let files = FilesBuilder::combine(
            [(PackPath::from_static("blocks/moved.json"), copy)],
            FilesRule::TopWins,
            &LoadOptions::default(),
        );
        kept(files.held_diagnostics());
    }
}

impl Merge for Moved {
    type Key = ();
}

#[test]
fn merged_keeps_the_places_of_a_layer_and_a_failed_copy() {
    for copy in [Ok(document()), Err(Rejected::new(origin(), upgraded()))] {
        let merged = Merged::merge(vec![copy], Moved, |_, _, _| {});
        kept(merged.held_diagnostics());
    }
}

/// A document of pack `pack`, whose upgrade moved `old<pack>-<tag>` to `new<pack>-<tag>`.
fn moved_in(pack: usize, tag: &str) -> Document<Moved> {
    let v = FormatVersion::new(1, 0, 0);
    let mut diagnostics = Diagnostics::default();
    diagnostics.moved(
        path(&format!("old{pack}-{tag}")),
        path(&format!("new{pack}-{tag}")),
    );
    diagnostics.push(Diagnostic::new(
        DiagnosticCode::from(&codes::INVALID_VALUE),
        Severity::Warning,
        Phase::Upgrade,
        Location::new(None, None, path(&format!("new{pack}-{tag}")), None),
    ));
    diagnostics.name_as_the_file_has_it();
    let pack_ref = PackRef::new(
        pack,
        PackIdVersion::new(
            Uuid::from_u128(pack as u128 + 1),
            SemVersion::new(1, 0, 0),
            PackType::Behavior,
        ),
        format!("pack {pack}"),
    );
    let origin = Origin::new(
        pack_ref,
        PackSide::Behavior,
        PackPath::from_static("blocks/moved.json"),
        None,
        Vec::new(),
    );
    Document::new(
        Moved,
        origin,
        VersionInfo::new(Some(v), false, v, v, false),
        diagnostics,
    )
}

/// Characterization (a question for the maintainers): a `FieldMerge` keeps the places of ONE
/// of the two documents it merges, the lower pack's (the first pushed within one pack), and
/// drops those of the other, though it keeps the findings of both. `Diagnostics::append`
/// would union them, and apply the places of one file to the paths of another. No link rule
/// asks a merged document for a place today.
#[test]
fn a_field_merge_keeps_the_places_of_the_lower_document_only() {
    // (pack of the first pushed, pack of the second, the tag of the document whose places stay)
    for (first, second, base) in [(1, 0, "a"), (0, 0, "a"), (0, 1, "b")] {
        let registry = RegistryBuilder::combine(
            [
                Contribution::Stable(Id::from_static("demo:moved"), moved_in(first, "a")),
                Contribution::Stable(Id::from_static("demo:moved"), moved_in(second, "b")),
            ],
            CombineRule::FieldMerge { merge: |_, _| {} },
            CombineRule::FieldMerge { merge: |_, _| {} },
            &LoadOptions::default(),
        );
        let merged = registry
            .get(&Id::from_static("demo:moved"))
            .expect("the merge is effective")
            .diagnostics();
        assert_eq!(merged.len(), 2, "the findings of both are kept");
        let name = |tag: &str, prefix: &str| {
            let pack = if tag == "a" { first } else { second };
            path(&format!("{prefix}{pack}-{tag}"))
        };
        let other = if base == "a" { "b" } else { "a" };
        assert_eq!(
            merged.place_of(&name(base, "new")),
            name(base, "old"),
            "({first}, {second}): the base's place stays"
        );
        assert_eq!(
            merged.place_of(&name(other, "new")),
            name(other, "new"),
            "({first}, {second}): the other document's place is dropped"
        );
    }
}

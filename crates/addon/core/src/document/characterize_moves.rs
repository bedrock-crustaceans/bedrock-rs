//! Characterization: the registry and files builders and [`Merged::merge`] collect a
//! document's findings with `extend` (`iter().cloned()`, or a rejected copy's by value), which
//! keeps the findings and drops the places an upgrade's moves left
//! (`Diagnostics::place_of`). Pinned before the switch to `Diagnostics::append`, the change
//! that deletes these tests and pins the places kept.
//!
//! The drop is seen only through the collections' own `Diagnostics`, which no public path
//! hands out: `Registry::diagnostics`, `Files::diagnostics` and `Merged::diagnostics` yield
//! the findings alone, and the link pass's `place_of` calls read each document's own
//! diagnostics (`Document::diagnostics`), which keep their moves.

use uuid::Uuid;

use super::{
    CombineRule, Content, Contribution, Definition, Document, FilesBuilder, FilesRule, Merge,
    Merged, NoBeta, Origin, RegistryBuilder, Rejected, VersionInfo,
};
use crate::__private::Sealed;
use crate::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
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
        DiagnosticCode::InvalidValue { text: None },
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

/// The finding is kept, named as the file has it; the place is not.
fn dropped(diagnostics: &Diagnostics) {
    let paths: Vec<_> = diagnostics
        .iter()
        .map(|d| d.location.json_path.clone())
        .collect();
    assert_eq!(paths, [path("old")]);
    assert_eq!(
        diagnostics.place_of(&path("new")),
        path("new"),
        "today the combine drops the place of the move"
    );
}

#[test]
fn registry_builder_drops_the_places_of_a_document_today() {
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
    dropped(registry.held_diagnostics());
}

#[test]
fn registry_builder_drops_the_places_of_a_rejected_document_today() {
    let registry = RegistryBuilder::<Moved>::combine(
        [Contribution::Rejected(
            None,
            Rejected::new(origin(), upgraded()),
        )],
        CombineRule::top_wins(),
        CombineRule::top_wins(),
        &LoadOptions::default(),
    );
    dropped(registry.held_diagnostics());
}

#[test]
fn files_builder_drops_the_places_of_a_copy_today() {
    for copy in [Ok(document()), Err(Rejected::new(origin(), upgraded()))] {
        let files = FilesBuilder::combine(
            [(PackPath::from_static("blocks/moved.json"), copy)],
            FilesRule::TopWins,
            &LoadOptions::default(),
        );
        dropped(files.held_diagnostics());
    }
}

impl Merge for Moved {
    type Key = ();
}

#[test]
fn merged_drops_the_places_of_a_layer_and_a_failed_copy_today() {
    for copy in [Ok(document()), Err(Rejected::new(origin(), upgraded()))] {
        let merged = Merged::merge(vec![copy], Moved, |_, _, _| {});
        dropped(merged.held_diagnostics());
    }
}

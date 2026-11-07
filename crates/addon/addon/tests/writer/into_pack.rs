//! `PackWriter::into_pack`: the written pack goes through the loader a pack on disk does and
//! comes out with the identity, the files and the bytes it was written with.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::load::RootId;
use bedrock_addon::manifest::{Manifest, Setting};
use bedrock_addon::pack::{OpenError, Pack, PackOrigin, PackType, PackWriter};
use bedrock_addon::storage::PackPath;

use super::from_pack::as_rewritten;
use super::packs::{Case, plain_packs};
use super::{path, ruby_manifest};

/// `pack` is `case`'s pack again: its identity and type, its file set (the manifest and the
/// files a copy holds) and every file's bytes.
pub fn assert_same_pack(case: &Case, pack: &Pack) {
    let name = &case.name;
    assert_eq!(pack.id(), case.pack.id(), "{name}: the identity");
    assert_eq!(pack.pack_type(), case.pack.pack_type(), "{name}: the type");
    assert_eq!(
        *pack.manifest(),
        as_rewritten(&case.manifest),
        "{name}: the manifest"
    );
    let listed: Vec<&str> = pack.paths().map(PackPath::as_str).collect();
    let mut expected: Vec<&str> = case.files.iter().map(|(at, _)| at.as_str()).collect();
    expected.push("manifest.json");
    expected.sort_unstable();
    assert_eq!(listed, expected, "{name}: the file set");
    for (at, bytes) in &case.files {
        assert!(
            pack.read(&path(at)).unwrap() == bytes,
            "{name}: the bytes of {at}"
        );
    }
    assert!(!pack.report().has_errors(), "{name}: {:?}", pack.report());
}

#[test]
fn into_pack_opens_every_copied_fixture_pack_as_the_same_pack() {
    for case in plain_packs() {
        let writer = PackWriter::from_pack(&case.pack).unwrap();
        let pack = writer
            .into_pack(PackOrigin::World)
            .unwrap_or_else(|error| panic!("{}: {error}", case.name));
        assert_same_pack(&case, &pack);
        assert_eq!(pack.location().root, RootId::MEMORY, "{}", case.name);
        assert!(pack.location().path.is_root(), "{}", case.name);
    }
}

#[test]
fn into_pack_gives_the_pack_the_origin_asked_for_and_every_file_resident() {
    for origin in [PackOrigin::World, PackOrigin::Package] {
        let mut writer = PackWriter::new(ruby_manifest());
        writer
            .put_bytes(path("textures/ruby.png"), b"ruby".to_vec())
            .unwrap();
        let pack = writer.into_pack(origin).unwrap();
        assert_eq!(pack.report().pack_origin, origin);
        assert_eq!(pack.manifest(), &ruby_manifest());
        assert_eq!(pack.pack_type(), PackType::Behavior);
        assert_eq!(pack.id(), &ruby_manifest().identity());
        let listed: Vec<&str> = pack.paths().map(PackPath::as_str).collect();
        assert_eq!(listed, ["manifest.json", "textures/ruby.png"]);
        for at in ["manifest.json", "textures/ruby.png"] {
            assert!(pack.is_resident(&path(at)), "{at}");
        }
        assert_eq!(pack.read(&path("textures/ruby.png")).unwrap(), &b"ruby"[..]);
    }
}

/// A format-2 manifest holding a setting only format 3 can.
fn unwritable_manifest() -> Manifest {
    let mut manifest = ruby_manifest();
    manifest.settings.push(Setting::Label {
        text: "A label, which format 2 holds".to_owned(),
    });
    manifest.settings.push(Setting::Toggle {
        name: "demo:lit".to_owned(),
        text: "Lit".to_owned(),
        default: true,
    });
    manifest
}

/// `into_pack` returns an `OpenError`, which has no variant for a manifest that cannot be
/// written: it is an invalid manifest, the report naming the member its format cannot hold.
#[test]
fn into_pack_of_a_manifest_its_format_cannot_hold_is_an_invalid_manifest() {
    let unwritable = unwritable_manifest();
    assert!(unwritable.to_json().is_err());
    let mut writer = PackWriter::new(unwritable);
    writer
        .put_bytes(path("textures/ruby.png"), b"ruby".to_vec())
        .unwrap();
    let refused = writer.into_pack(PackOrigin::World);
    let Err(OpenError::InvalidManifest { location, report }) = refused else {
        panic!("{refused:?}");
    };
    assert_eq!(location.root, RootId::MEMORY);
    assert_eq!(report.pack_origin, PackOrigin::World);
    let findings: Vec<_> = report.diagnostics().iter().collect();
    assert_eq!(findings.len(), 1, "{findings:?}");
    let finding = findings[0];
    assert!(
        finding.is(&bedrock_addon::diagnostic::codes::INVALID_VALUE),
        "{finding:?}"
    );
    assert_eq!(finding.severity, Severity::Error);
    assert_eq!(
        finding.location.path.as_ref().map(PackPath::as_str),
        Some("manifest.json")
    );
    // The second setting, not the label before it.
    assert_eq!(finding.location.json_path.to_string(), "settings/1");
}

/// The other side: the same settings in a format-3 manifest are written and opened.
#[test]
fn into_pack_of_the_same_settings_in_format_3_opens() {
    let mut manifest = unwritable_manifest();
    manifest.format = bedrock_addon::manifest::ManifestFormat::V3;
    let pack = PackWriter::new(manifest.clone())
        .into_pack(PackOrigin::World)
        .unwrap();
    assert_eq!(pack.manifest().settings, manifest.settings);
}

/// The loader's own refusals stand: a manifest that is written and then not accepted.
#[test]
fn into_pack_of_a_manifest_the_loader_refuses_is_the_loaders_error() {
    let mut manifest = ruby_manifest();
    manifest.header.min_engine_version =
        Some(bedrock_addon::version::MinEngineVersion::new(99, 0, 0));
    assert!(manifest.to_json().is_ok(), "written, though no pack");
    let refused = PackWriter::new(manifest).into_pack(PackOrigin::World);
    let Err(OpenError::InvalidManifest { report, .. }) = refused else {
        panic!("{refused:?}");
    };
    assert!(report.has_errors());
    assert!(
        report
            .diagnostics()
            .iter()
            .all(|finding| finding.location.json_path.to_string() != "settings/1"),
        "{report:?}"
    );
}

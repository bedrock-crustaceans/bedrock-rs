#![allow(clippy::pedantic, missing_docs)]
//! `PackStack::of` and the `Pack` accessors. Packs are opened with `Pack::from_files` from small
//! in-memory manifests.

#[path = "pack/common.rs"]
mod common;

use bedrock_addon::load::{LoadOptions, RootId};
use bedrock_addon::pack::{Behavior, PackStack, PackType, Resource, WrongPackType};
use bedrock_addon::storage::PackPath;

use common::{bp, rp};

#[test]
fn packs_stacks_and_repositories_are_send_and_sync() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<bedrock_addon::pack::Pack>();
    send_sync::<PackStack<Behavior>>();
    send_sync::<PackStack<Resource>>();
    send_sync::<bedrock_addon::pack::StackSet>();
    send_sync::<bedrock_addon::pack::PackRepository>();
}

#[test]
fn an_empty_stack_has_no_packs_and_keeps_its_options() {
    let options = LoadOptions {
        beta_apis: true,
        ..LoadOptions::default()
    };
    let stack = PackStack::<Resource>::of([], options).expect("no pack to refuse");
    assert_eq!(stack.packs().len(), 0);
    assert!(stack.options().beta_apis);
}

#[test]
fn a_stack_of_packs_lists_them_top_first_with_their_index() {
    let (a, b) = (rp(1), rp(2));
    let stack = PackStack::<Resource>::of([a.open(), b.open()], LoadOptions::default())
        .expect("resource packs");
    let packs: Vec<_> = stack
        .packs()
        .map(|pack| (pack.index(), pack.id().clone(), pack.name().to_owned()))
        .collect();
    assert_eq!(
        packs,
        [
            (Some(0), a.id(), "rp 1".to_owned()),
            (Some(1), b.id(), "rp 2".to_owned()),
        ]
    );
}

#[test]
fn a_pack_without_a_module_of_the_stacks_kind_is_refused() {
    let r = rp(1);
    let error = PackStack::<Behavior>::of([bp(2).open(), r.open()], LoadOptions::default())
        .expect_err("a resource pack on the behaviour stack");
    // `WrongPackType` is read-only (`#[non_exhaustive]`): compared field by field.
    let WrongPackType {
        pack,
        expected,
        found,
        ..
    } = error;
    assert_eq!(
        (pack, expected, found),
        (r.id(), PackType::Behavior, PackType::Resource)
    );
}

// A pack with both a data and a resources module cannot come out of the manifest readers
// (`MultipleModules`); `PackStack::of` accepting one on either stack is tested in-crate with a
// hand-built manifest: `pack::repository::tests::a_stack_of_packs_refuses_one_without_a_module_of_its_kind`

/// A 1×1 PNG's first bytes: the signature and the start of `IHDR`.
const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";

#[test]
fn the_pack_accessors_read_the_manifest_and_the_files() {
    let spec = rp(5)
        .file("pack_icon.png", PNG)
        .file("textures/b.png", b"b")
        .file("Textures/a.png", b"a");
    let pack = spec.open();
    assert_eq!(*pack.id(), spec.id());
    assert_eq!(pack.pack_type(), PackType::Resource);
    assert_eq!(pack.location().root, RootId::MEMORY);
    assert_eq!(pack.icon().as_deref(), Some(PNG));
    assert!(pack.is_resident(&PackPath::from_static("textures/b.png")));
    assert!(!pack.is_resident(&PackPath::from_static("textures/c.png")));
    let paths: Vec<&str> = pack.paths().map(PackPath::as_str).collect();
    assert_eq!(
        paths,
        [
            "Textures/a.png",
            "manifest.json",
            "pack_icon.png",
            "textures/b.png"
        ]
    );
}

#[test]
fn an_icon_that_is_not_a_png_is_none() {
    let pack = rp(6).file("pack_icon.png", b"GIF89a").open();
    assert!(pack.icon().is_none());
    assert!(rp(7).open().icon().is_none());
}

/// The header of a 256 by 128 PNG.
const WIDE_PNG: &[u8] =
    b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR\x00\x00\x01\x00\x00\x00\x00\x80\x08\x06\x00\x00\x00";
/// The header of a 256 by 256 PNG.
const SQUARE_PNG: &[u8] =
    b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR\x00\x00\x01\x00\x00\x00\x01\x00\x08\x06\x00\x00\x00";

/// The pack icon findings of a report: code, severity, pack index, path.
fn icon_lints(
    report: &bedrock_addon::pack::StackReport<'_>,
) -> Vec<(
    &'static str,
    bedrock_addon::diagnostic::Severity,
    Option<usize>,
    String,
)> {
    report
        .iter()
        .filter(|f| {
            f.location
                .path
                .as_ref()
                .is_some_and(|path| path.as_str() == "pack_icon.png")
        })
        .map(|f| {
            (
                f.code.name(),
                f.severity,
                f.location.pack.as_ref().and_then(|pack| pack.index()),
                f.location.path.as_ref().unwrap().as_str().to_owned(),
            )
        })
        .collect()
}

/// families/rp_misc validation P1 to P3 run on the packs of a stack: the stack's own
/// findings hold each pack's icon lints, lowest pack first, each with its pack; a pack with a
/// square icon adds nothing. They are found on either side.
#[test]
fn a_stack_reports_the_icon_lints_of_its_packs_lowest_first() {
    use bedrock_addon::diagnostic::Severity;

    let at = "pack_icon.png".to_owned();
    let jpeg: &[u8] = b"\xff\xd8\xff\xe0\x00\x10JFIF";
    let stack = PackStack::<Resource>::of(
        [
            rp(1).open(),
            rp(2).file("pack_icon.png", jpeg).open(),
            rp(3).file("pack_icon.png", WIDE_PNG).open(),
            rp(4).file("pack_icon.png", SQUARE_PNG).open(),
        ],
        LoadOptions::default(),
    )
    .expect("resource packs");
    assert_eq!(
        icon_lints(&stack.report()),
        [
            ("PackIconShape", Severity::Info, Some(2), at.clone()),
            ("InvalidValue", Severity::Info, Some(1), at.clone()),
            ("PackIconMissing", Severity::Info, Some(0), at.clone()),
        ]
    );
    let stack = PackStack::<Behavior>::of(
        [bp(5).file("pack_icon.png", SQUARE_PNG).open(), bp(6).open()],
        LoadOptions::default(),
    )
    .expect("behaviour packs");
    assert_eq!(
        icon_lints(&stack.report()),
        [("PackIconMissing", Severity::Info, Some(1), at)]
    );
}

/// The game's own slices ship no icon and no pack screen lists them: a base-game pack
/// without one is not reported. An icon it does carry is judged like any other.
#[test]
fn a_base_game_pack_without_an_icon_is_not_reported() {
    use bedrock_addon::diagnostic::Severity;
    use bedrock_addon::pack::{OpenOptions, Pack, PackOrigin};

    let shipped = |spec: common::Spec| {
        let mut files = vec![(
            PackPath::from_static("manifest.json"),
            bytes::Bytes::from(spec.manifest()),
        )];
        files.extend(spec.files.iter().map(|(path, bytes)| {
            (
                PackPath::from_static(path),
                bytes::Bytes::from_static(bytes),
            )
        }));
        let options = OpenOptions {
            origin: PackOrigin::Package,
            ..OpenOptions::default()
        };
        Pack::from_files(files, &options).expect("the pack opens")
    };
    let stack = PackStack::<Resource>::of(
        [
            shipped(rp(1)),
            shipped(rp(2).file("pack_icon.png", WIDE_PNG)),
            rp(3).open(),
        ],
        LoadOptions::default(),
    )
    .expect("resource packs");
    assert_eq!(
        icon_lints(&stack.report()),
        [
            (
                "PackIconMissing",
                Severity::Info,
                Some(2),
                "pack_icon.png".to_owned()
            ),
            (
                "PackIconShape",
                Severity::Info,
                Some(1),
                "pack_icon.png".to_owned()
            ),
        ]
    );
}

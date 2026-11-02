//! The rules of the fonts that need a file's bytes on a stack: a glyph sheet's shape, a metrics
//! file's header, a remapping table. Each calls `StackSet::link()` and holds its result to the
//! rule.

use bedrock_addon::diagnostic::{Diagnostics, Severity};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{PackOrigin, PackRepository, WorldPacks};

use super::common::{pack_uuid, pack_with};

/// The link findings of one resource pack with `files`.
fn link(files: &[(&str, &[u8])]) -> Diagnostics {
    let pack = pack_with(0, "resources", &[], files);
    let entries = format!(
        r#"[{{"pack_id": "{}", "version": [1, 0, 0]}}]"#,
        pack_uuid(0)
    );
    let world = WorldPacks::parse(None, Some(entries.as_bytes())).expect("a world list");
    let mut repository = PackRepository::new();
    repository.add_pack(pack, PackOrigin::User);
    repository.stacks(&world, LoadOptions::default()).link()
}

fn found(d: &Diagnostics, code: &str) -> Vec<(Severity, String)> {
    d.iter()
        .filter(|f| f.code.name() == code)
        .map(|f| {
            let path = f.location.path.as_ref().map(ToString::to_string);
            (f.severity, path.unwrap_or_default())
        })
        .collect()
}

/// A PNG header of the given size.
fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR".to_vec();
    bytes.extend(width.to_be_bytes());
    bytes.extend(height.to_be_bytes());
    bytes
}

#[test]
fn f16_f24_f26_the_font_files_of_a_stack_are_checked_from_their_bytes() {
    let mut bad_header = vec![0_u8; 1028];
    bad_header[0] = 1;
    let d = link(&[
        ("font/glyph_E2.png", &png(200, 256)),
        ("font/glyph_E3.png", &png(512, 512)),
        ("font/smooth/smooth_41.png", &png(1024, 1024)),
        ("font/smooth/smooth_41.fontdata", &bad_header),
        // One record, whose glyph is on page 41 (the page the pack has), and a byte too
        // many.
        ("font/smooth/remapping.dat", &[1, 2, 3, 0x41, 5]),
    ]);
    assert_eq!(
        found(&d, "GlyphSheetShape"),
        [(Severity::Info, "font/glyph_E2.png".to_owned())],
        "{d:?}"
    );
    assert_eq!(
        found(&d, "MalformedInput"),
        [
            // F26 and F24 are the game's silence and its assert: informations
            (Severity::Info, "font/smooth/remapping.dat".to_owned()),
            (Severity::Info, "font/smooth/smooth_41.fontdata".to_owned()),
        ],
        "{d:?}"
    );
}

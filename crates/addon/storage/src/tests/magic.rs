//! Magic detection over the copied fixtures.

use super::{fixture, fixture_names};
use crate::ContentIdentity;
use crate::magic::{FileType, PREFIX_LEN, detect};

fn detect_fixture(relative: &str) -> FileType {
    let bytes = fixture(relative);
    detect(&bytes[..bytes.len().min(PREFIX_LEN)])
}

#[test]
fn every_zip_pack_is_a_zip() {
    let names = fixture_names("zip_packs");
    assert_eq!(names.len(), 23);
    for name in names {
        assert_eq!(
            detect_fixture(&format!("zip_packs/{name}")),
            FileType::Zip,
            "{name}"
        );
    }
}

#[test]
fn encrypted_contents_json_is_detected_with_its_identity() {
    assert_eq!(
        detect_fixture("encrypted/testPack/contents.json"),
        FileType::Encrypted(ContentIdentity::new("325FCAEA-29F0-4D51-AB14-A0BE8C66DC97"))
    );
}

#[test]
fn encrypted_pack_manifest_is_plain_text() {
    assert_eq!(
        detect_fixture("encrypted/testPack/manifest.json"),
        FileType::Plain
    );
}

#[test]
fn encrypted_assets_have_no_header() {
    // assets are decrypted at offset 0 with their own key: no magic survives encryption
    for asset in [
        "encrypted/testPack/__brarchive/test.brarchive",
        "encrypted/testPack/__brarchive/test/nested.brarchive",
        "encrypted/testPack/test/test.json",
    ] {
        assert_eq!(detect_fixture(asset), FileType::Plain, "{asset}");
    }
}

#[test]
fn synthetic_containers() {
    assert_eq!(
        detect_fixture("synthetic/valid_reference.zip"),
        FileType::Zip
    );
    // `PK\x03\x04` alone, cut before the rest of the local header
    assert_eq!(
        detect_fixture("synthetic/truncated_4bytes.bin"),
        FileType::Zip
    );
    assert_eq!(
        detect_fixture("synthetic/truncated_near_full.bin"),
        FileType::Zip
    );
    // `PK` is not the four-byte signature
    assert_eq!(
        detect_fixture("synthetic/truncated_2bytes.bin"),
        FileType::Plain
    );
    // an empty archive starts with the end-of-central-directory record, not a local header
    assert_eq!(
        detect_fixture("synthetic/empty_zip_zero_entries.zip"),
        FileType::Plain
    );
    assert_eq!(detect_fixture("synthetic/not_a_zip.zip"), FileType::Plain);
}

#[test]
fn plain_contents_json_is_plain() {
    for name in fixture_names("contents_json") {
        assert_eq!(
            detect_fixture(&format!("contents_json/{name}")),
            FileType::Plain,
            "{name}"
        );
    }
}

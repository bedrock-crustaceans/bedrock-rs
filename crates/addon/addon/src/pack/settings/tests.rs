//! Unit tests of the module's private parts.

use super::history::{PackSettingsFamily, PackSettingsHistory};
use crate::version::FormatVersion;
use crate::version::FormatVersionInput;
use crate::versioning::{RangeModel, Selection, VersionChain, check_chain, select_versions};

fn selects(text: &str) -> Selection {
    let input = FormatVersionInput::parse(text).expect("a version");
    select_versions(
        <PackSettingsHistory as VersionChain<PackSettingsFamily>>::VERSIONS,
        FormatVersion::new(1, 21, 100),
        RangeModel::ClosedToLatest,
        None,
        &input,
    )
}

#[test]
fn the_chain_is_contiguous_from_the_floor_to_latest() {
    check_chain::<PackSettingsFamily>().expect("a well-formed chain");
    assert_eq!(
        <PackSettingsHistory as VersionChain<PackSettingsFamily>>::VERSIONS,
        [
            FormatVersion::new(1, 21, 100),
            FormatVersion::new(1, 26, 40)
        ]
    );
}

#[test]
fn closed_to_latest_selects_the_old_struct_below_1_26_40_and_latest_only_at_it() {
    assert_eq!(selects("1.21.100"), Selection::Struct(0));
    assert_eq!(selects("1.21.110"), Selection::Struct(0));
    assert_eq!(selects("1.26.39"), Selection::Struct(0));
    assert_eq!(selects("1.26.40"), Selection::Struct(1));
    assert_eq!(selects("1.26.41"), Selection::Unsupported);
    assert_eq!(selects("1.26.50"), Selection::Unsupported);
    assert_eq!(selects("1.21.90"), Selection::BelowFloor);
    assert_eq!(selects("1.21.100-beta.1"), Selection::BelowFloor);
    assert_eq!(selects("1.26.40-beta.1"), Selection::Unsupported);
}

/// A file that fails to load closes on the settings' own definition, naming the file.
#[test]
fn a_file_that_fails_to_load_closes_on_its_definition() {
    let rejected = super::SavedSettings::parse_named("world_resource_pack_settings.json", b"[")
        .expect_err("no JSON");
    let closing = rejected
        .diagnostics
        .iter()
        .last()
        .expect("a closing finding");
    assert!(closing.is(&super::codes::LOAD_FAILED), "{closing:?}");
    assert_eq!(
        closing.code.to_string(),
        "Pack Settings failed to load [world_resource_pack_settings.json]"
    );
}

#[test]
fn spans_of_a_renamed_payload_key_move_back_to_the_files_offsets() {
    use crate::diagnostic::Span;
    // The quoted name `"x"` is bytes 9..12.
    let text = r#"{"a": 1, "x": {}}"#;
    let key = super::ForeignKey {
        name: "x".to_owned(),
        start: 9,
        end: 12,
        warns: true,
    };
    let renamed = key.rename(text);
    assert_eq!(renamed, r#"{"a": 1, "minecraft:pack_settings": {}}"#);
    let restored = |offset: usize, len: usize| {
        let mut span = Span {
            offset: u32::try_from(offset).unwrap(),
            len: u32::try_from(len).unwrap(),
        };
        key.restore(&mut span);
        (span.offset as usize, span.len as usize)
    };
    // Before the name: unchanged.
    assert_eq!(restored(1, 3), (1, 3));
    // After the name: the payload `{}`.
    let payload = renamed.find("{}").unwrap();
    assert_eq!(restored(payload, 2), (text.find("{}").unwrap(), 2));
    // Right after the name: the colon.
    let colon = renamed.find(": {}").unwrap();
    assert_eq!(restored(colon, 1), (12, 1));
    // Around the name: the whole document, from its start or from the name's.
    assert_eq!(restored(0, renamed.len()), (0, text.len()));
    assert_eq!(restored(9, renamed.len() - 9), (9, text.len() - 9));
    // The name itself and anything ending inside it keep their offsets.
    assert_eq!(restored(9, 5), (9, 5));
}

#[test]
fn setting_values_convert_from_their_plain_types() {
    use super::SettingValue;

    assert_eq!(SettingValue::from(2.5_f32), SettingValue::Float(2.5));
    assert_eq!(SettingValue::from(true), SettingValue::Bool(true));
    assert_eq!(
        SettingValue::from("high"),
        SettingValue::String("high".to_owned())
    );
    assert_eq!(
        SettingValue::from(String::from("low")),
        SettingValue::String("low".to_owned())
    );
    assert_eq!(
        SettingValue::from(vec![String::from("a")]),
        SettingValue::StringList(vec!["a".to_owned()])
    );
}

#[test]
fn saved_settings_get_finds_the_first_entry_of_a_pack() {
    use super::{SavedSettings, Settings};
    use indexmap::IndexMap;
    use uuid::Uuid;

    let first = Settings::new(Uuid::from_u128(1), IndexMap::new());
    let mut second = Settings::new(Uuid::from_u128(1), IndexMap::new());
    second.last_used = Some(5);
    let saved = SavedSettings {
        settings: vec![first.clone(), second],
    };
    assert_eq!(saved.get(Uuid::from_u128(1)), Some(&first));
    assert_eq!(saved.get(Uuid::from_u128(2)), None);
}

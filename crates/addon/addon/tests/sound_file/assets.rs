//! The audio files of the stack (`rp.sound_files()`): the rows `as01`, `as02`, `as07` to `as10`
//! of the fixtures (the vanilla rows `as12`, `as13` are the smoke test's) and the rules of the
//! listing. Every rule here is ours: vanilla checks nothing at load. The names a sound
//! definition has of these files are judged by the link pass, in `sound_definitions`.

use bedrock_addon::diagnostic::Severity;
use bedrock_addon::families::sound_file::{AudioContainer, AudioExt};
use bedrock_addon::load::LoadOptions;
use bedrock_addon::pack::{Pack, PackOrigin, PackStack, Resource};

use super::common::{
    DEFINITIONS, describe, expect_clean, files, found, fsb, ogg, pack_with_origin, plays, stack_of,
    vorbis, wav,
};

fn stack(packs: Vec<Pack>) -> PackStack<Resource> {
    stack_of(packs, LoadOptions::default())
}

#[test]
fn as01_one_pack_is_probed_fsb_then_ogg_then_wav() {
    let stack = stack(vec![files(
        10,
        &[
            (DEFINITIONS, plays(&["sounds/x"]).into_bytes()),
            ("sounds/x.wav", wav()),
            ("sounds/x.ogg", vorbis()),
            ("sounds/x.fsb", fsb(1)),
        ],
    )]);
    let sounds = stack.sound_files();
    let winner = sounds.get("sounds/x").unwrap();
    assert_eq!(
        (winner.path.as_str(), winner.ext),
        ("sounds/x.fsb", AudioExt::Fsb)
    );
    let hidden: Vec<(&str, &str)> = sounds
        .shadowed
        .iter()
        .map(|(loser, winner)| (loser.path.as_str(), winner.path.as_str()))
        .collect();
    assert_eq!(
        hidden,
        [
            ("sounds/x.ogg", "sounds/x.fsb"),
            ("sounds/x.wav", "sounds/x.fsb")
        ]
    );
    let shadowed = found(sounds, "core/shadowed_definition");
    assert_eq!(shadowed.len(), 2);
    assert!(shadowed.iter().all(|d| d.severity == Severity::Info));
    assert_eq!(sounds.findings().count(), 2, "{:#?}", describe(sounds));
    // Without the `.fsb`, the `.ogg` is first.
    let stack = self::stack(vec![files(
        10,
        &[("sounds/x.wav", wav()), ("sounds/x.ogg", vorbis())],
    )]);
    assert_eq!(
        stack.sound_files().get("sounds/x").unwrap().ext,
        AudioExt::Ogg
    );
    assert_eq!(
        AudioExt::PROBE_ORDER,
        [AudioExt::Fsb, AudioExt::Ogg, AudioExt::Wav]
    );
}

#[test]
fn as02_the_top_pack_beats_the_extension() {
    let stack = stack(vec![
        files(
            10,
            &[
                ("sounds/x.fsb", fsb(1)),
                (DEFINITIONS, plays(&["sounds/x"]).into_bytes()),
            ],
        ),
        files(20, &[("sounds/x.wav", wav())]),
    ]);
    let sounds = stack.sound_files();
    let winner = sounds.get("sounds/x").unwrap();
    assert_eq!(winner.path.as_str(), "sounds/x.wav");
    assert_eq!(winner.pack.index(), Some(0));
    assert!(sounds.shadowed.is_empty());
    assert_eq!(sounds.findings().count(), 0, "{:#?}", describe(sounds));
}

#[test]
fn as07_a_file_of_another_extension_is_never_loaded() {
    let stack = stack(vec![files(
        10,
        &[
            (DEFINITIONS, plays(&["sounds/x"]).into_bytes()),
            ("sounds/x.mp3", b"ID3".to_vec()),
            ("sounds/y.OGG", vorbis()),
            ("sounds/notes.json", b"{}".to_vec()),
        ],
    )]);
    let sounds = stack.sound_files();
    let ignored: Vec<&str> = sounds
        .ignored
        .iter()
        .map(|(_, path)| path.as_str())
        .collect();
    assert_eq!(ignored, ["sounds/x.mp3", "sounds/y.OGG"]);
    let infos = found(sounds, "core/ignored_file");
    assert_eq!(infos.len(), 2);
    assert!(infos.iter().all(|d| d.severity == Severity::Info));
    assert!(sounds.by_name.is_empty());
}

#[test]
fn as08_as09_as10_the_container_is_told_from_the_first_bytes_never_at_load_deviation() {
    // Deviation from the folder: the load reads no audio file, so rule 34 is
    // not raised by it. The sniff is what a host that has the bytes asks (chorus serving a
    // pack, or the link pass once it is handed file heads).
    let cases: Vec<(Vec<u8>, AudioContainer)> = vec![
        (b"ID3\x04\x00".to_vec(), AudioContainer::Mpeg),
        (vec![0xFF, 0xFB, 0x90, 0x00], AudioContainer::Mpeg),
        (vec![0xFF, 0x1B], AudioContainer::Unknown),
        (ogg(b"OpusHead"), AudioContainer::OggOther),
        (vorbis(), AudioContainer::OggVorbis),
        (Vec::new(), AudioContainer::Empty),
        (b"hello".to_vec(), AudioContainer::Unknown),
        (wav(), AudioContainer::RiffWave),
        (
            b"RIFF\x00\x00\x00\x00AVI ".to_vec(),
            AudioContainer::Unknown,
        ),
        (
            fsb(1),
            AudioContainer::Fsb5 {
                version: 1,
                subsounds: 1,
                codec: 16,
            },
        ),
        (
            fsb(2),
            AudioContainer::Fsb5 {
                version: 1,
                subsounds: 2,
                codec: 16,
            },
        ),
    ];
    for (bytes, container) in &cases {
        assert_eq!(AudioContainer::sniff(bytes), *container, "{bytes:?}");
    }
    // Each extension takes its own container and no other.
    let (bank, vorbis, wave) = (
        AudioContainer::sniff(&fsb(1)),
        AudioContainer::OggVorbis,
        AudioContainer::RiffWave,
    );
    for (ext, fitting) in [
        (AudioExt::Fsb, bank),
        (AudioExt::Ogg, vorbis),
        (AudioExt::Wav, wave),
    ] {
        for container in [
            bank,
            vorbis,
            wave,
            AudioContainer::Mpeg,
            AudioContainer::OggOther,
            AudioContainer::Empty,
            AudioContainer::Unknown,
        ] {
            assert_eq!(
                container.fits(ext),
                container == fitting,
                "{ext:?} {container:?}"
            );
        }
    }
    // A stack whose audio is none of that loads without a word about it.
    let stack = stack(vec![files(
        10,
        &[
            ("sounds/mp3.ogg", b"ID3".to_vec()),
            ("sounds/empty.fsb", Vec::new()),
            ("sounds/two.fsb", fsb(2)),
        ],
    )]);
    let sounds = stack.sound_files();
    assert_eq!(sounds.by_name.len(), 3);
    assert_eq!(sounds.findings().count(), 0, "{:#?}", describe(sounds));
}

#[test]
fn a_file_of_a_higher_pack_and_a_lower_packs_are_one_name() {
    let stack = stack(vec![
        files(10, &[("sounds/a/x.ogg", vorbis()), ("sounds/b.wav", wav())]),
        files(20, &[("sounds/a/x.fsb", fsb(1))]),
    ]);
    let sounds = stack.sound_files();
    assert_eq!(sounds.by_name.len(), 2);
    let winner = sounds.get("sounds/a/x").unwrap();
    assert_eq!((winner.pack.index(), winner.ext), (Some(0), AudioExt::Fsb));
    assert!(sounds.shadowed.is_empty());
    assert_eq!(sounds.findings().count(), 0);
}

#[test]
fn the_audio_of_a_base_game_pack_is_listed_and_not_judged() {
    let definitions = plays(&["sounds/x"]);
    let files: &[(&str, &[u8])] = &[
        (DEFINITIONS, definitions.as_bytes()),
        ("sounds/x.ogg", b"ID3"),
        ("sounds/x.wav", b"ID3"),
        ("sounds/y.mp3", b"ID3"),
    ];
    // The client's own pack: its files resolve names and get no finding of their own.
    let stack = PackStack::<Resource>::of(
        vec![pack_with_origin(10, PackOrigin::Package, files)],
        LoadOptions::default(),
    )
    .unwrap();
    let sounds = stack.sound_files();
    assert_eq!(sounds.by_name.len(), 1);
    assert!(sounds.ignored.is_empty());
    assert_eq!(sounds.shadowed.len(), 1);
    expect_clean(sounds);
    // The same files in a served pack get rules 33 and 35.
    let stack = PackStack::<Resource>::of(
        vec![pack_with_origin(10, PackOrigin::User, files)],
        LoadOptions::default(),
    )
    .unwrap();
    let sounds = stack.sound_files();
    assert_eq!(sounds.by_name.len(), 1);
    assert_eq!(sounds.ignored.len(), 1);
    assert_eq!(
        found(sounds, "core/ignored_file").len(),
        1,
        "{:#?}",
        describe(sounds)
    );
    assert_eq!(found(sounds, "core/shadowed_definition").len(), 1);
    assert_eq!(sounds.findings().count(), 2);
}

#[test]
fn no_file_in_the_stack_is_an_empty_listing_without_findings() {
    let stack = super::common::copies("textures/readme.txt", &["x"]);
    let sounds = stack.sound_files();
    expect_clean(sounds);
    assert!(sounds.by_name.is_empty());
}

//! The audio files of the stack's `sounds/` trees: which file a sound entry's `name` resolves
//! to, and the lint rules over them. Vanilla opens audio only when a sound is first played;
//! nothing here changes what is loaded. The game tries the extensions `.fsb`, `.ogg` and `.wav`
//! in that order, for each pack in turn.

use indexmap::IndexMap;

use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Location, Phase, Severity,
};
use bedrock_addon_core::load::{ListedFile, StackPack};
use bedrock_addon_core::pack_identity::{PackPath, PackRef};

/// The kind [`BuiltIns`](bedrock_addon_core::load::BuiltIns) is asked about for an audio file
/// no pack of the stack ships: the file's path with its extension
/// (`sounds/random/click.fsb`), as the client's own packs store it.
pub const SOUND_FILE_KIND: &str = "sound file";

/// An audio file extension the client looks sounds up with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum AudioExt {
    /// `.fsb`: probed first.
    Fsb,
    /// `.ogg`: probed second.
    Ogg,
    /// `.wav`: probed last.
    Wav,
}

impl AudioExt {
    /// The three extensions in the client's probe order.
    pub const PROBE_ORDER: [Self; 3] = [Self::Fsb, Self::Ogg, Self::Wav];

    /// The extension with its dot.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fsb => ".fsb",
            Self::Ogg => ".ogg",
            Self::Wav => ".wav",
        }
    }

    /// The extension of `path`, compared exactly: `.OGG` is none of the three.
    fn of(path: &str) -> Option<(Self, &str)> {
        Self::PROBE_ORDER
            .into_iter()
            .find_map(|ext| Some((ext, path.strip_suffix(ext.as_str())?)))
    }
}

/// What the first bytes of an audio file say it is. The client's audio library decides by
/// content, never by extension. The load never reads an audio file, so nothing here is checked
/// at load; a host that has the bytes can [`sniff`](Self::sniff) them and ask
/// [`fits`](Self::fits).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AudioContainer {
    /// An FMOD sound bank, `FSB5`.
    Fsb5 {
        /// The header's version.
        version: u32,
        /// How many sounds the bank holds; the client plays the first.
        subsounds: u32,
        /// The header's codec mode (16 FADPCM, 2 PCM16, 15 Vorbis).
        codec: u32,
    },
    /// An Ogg stream whose first packet is Vorbis.
    OggVorbis,
    /// An Ogg stream of another codec (Opus).
    OggOther,
    /// A RIFF WAVE file.
    RiffWave,
    /// MPEG audio (an `ID3` tag or a frame sync), which the client rejects.
    Mpeg,
    /// An empty file.
    Empty,
    /// Anything else.
    Unknown,
}

impl AudioContainer {
    /// The container of a file that starts with `head`.
    pub fn sniff(head: &[u8]) -> Self {
        let word = |at: usize| {
            head.get(at..at + 4)
                .and_then(|bytes| bytes.try_into().ok())
                .map_or(0, u32::from_le_bytes)
        };
        if head.is_empty() {
            Self::Empty
        } else if head.starts_with(b"FSB5") {
            Self::Fsb5 {
                version: word(4),
                subsounds: word(8),
                codec: word(24),
            }
        } else if head.starts_with(b"OggS") {
            if head.get(28..35) == Some(b"\x01vorbis") {
                Self::OggVorbis
            } else {
                Self::OggOther
            }
        } else if head.starts_with(b"RIFF") && head.get(8..12) == Some(b"WAVE") {
            Self::RiffWave
        } else if head.starts_with(b"ID3")
            || matches!(head, [0xFF, second, ..] if second & 0xE0 == 0xE0)
        {
            Self::Mpeg
        } else {
            Self::Unknown
        }
    }

    /// Whether the client plays a file with extension `ext` that holds this container: an
    /// `FSB5` bank under `.fsb`, Ogg Vorbis under `.ogg`, RIFF WAVE under `.wav`.
    pub fn fits(self, ext: AudioExt) -> bool {
        matches!(
            (ext, self),
            (AudioExt::Fsb, Self::Fsb5 { .. })
                | (AudioExt::Ogg, Self::OggVorbis)
                | (AudioExt::Wav, Self::RiffWave)
        )
    }
}

/// One audio file of a pack.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct SoundAsset {
    /// The pack that ships the file.
    pub pack: PackRef,
    /// The file's path in the pack, with its extension, in its exact case: the path to ask
    /// the pack for. A file of the active subpack keeps its `subpacks/<folder>/` prefix here;
    /// the name it resolves is without it.
    pub path: PackPath,
    /// The extension.
    pub ext: AudioExt,
}

/// The audio files under `sounds/` of every pack of the stack (`rp.sound_files()`), as the
/// client would resolve a sound entry's `name` to one of them. Built from the packs' file
/// listings: no audio file is read.
///
/// A name resolves to the highest pack that ships `name.fsb`, `name.ogg` or `name.wav`, and
/// inside that pack to the first of the three. Names compare with case.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct SoundFiles {
    /// The file each name resolves to, by the name: the path without its extension, and for
    /// a file of the active subpack without its `subpacks/<folder>/` prefix.
    pub by_name: IndexMap<String, SoundAsset>,
    /// The files another extension of the same pack hides: the hidden file and the one that
    /// is loaded instead.
    pub shadowed: Vec<(SoundAsset, SoundAsset)>,
    /// The files under `sounds/` the client never looks for: another extension than `.fsb`,
    /// `.ogg`, `.wav` and `.json`. Each by its path in the pack.
    pub ignored: Vec<(PackRef, PackPath)>,
    /// The findings of the files.
    pub(crate) diagnostics: Diagnostics,
    /// A base-game pack of the stack lists audio: the client's own files are in view, so a
    /// name that resolves to none is worth a finding.
    #[doc(hidden)]
    pub client_audio: bool,
}

impl SoundFiles {
    /// The file `name` resolves to.
    pub fn get(&self, name: &str) -> Option<&SoundAsset> {
        self.by_name.get(name)
    }

    /// The findings of the audio files.
    pub fn findings(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.diagnostics.iter()
    }
}

/// A finding about a file of a pack.
fn file_finding(
    code: DiagnosticCode,
    severity: Severity,
    pack: &PackRef,
    path: &PackPath,
) -> Diagnostic {
    Diagnostic::new(
        code,
        severity,
        Phase::Validate,
        Location::new(
            Some(pack.clone()),
            Some(path.clone()),
            JsonPath::new(),
            None,
        ),
    )
}

/// Builds the index from the file listings of the stack's packs, top first. The files of a
/// pack that is served get the findings of the ignored and the shadowed files; a base-game pack's
/// are the client's own and are only listed.
pub(crate) fn index(packs: &[StackPack]) -> SoundFiles {
    index_listings(
        packs
            .iter()
            .map(|pack| (&pack.pack, !pack.origin.is_base_game(), pack.files().iter())),
    )
}

/// [`index`] over `(pack, whether it is served, its listing)`, top pack first
/// ([`StackPack::files`]). A file is named by its key. The order of a pack's listing does not
/// matter: inside one pack the probe order of the extensions decides, then the key.
fn index_listings<'a>(
    packs: impl Iterator<Item = (&'a PackRef, bool, impl Iterator<Item = &'a ListedFile>)>,
) -> SoundFiles {
    let mut out = SoundFiles::default();
    for (pack, served, listing) in packs {
        let mut listing: Vec<&ListedFile> = listing.collect();
        listing.sort_by(|a, b| a.key.cmp(&b.key));
        let mut audio: Vec<(&str, SoundAsset)> = Vec::new();
        for file in listing {
            // Named by the key; found, and reported, at the path the pack has it under.
            let path = &file.path;
            let text = file.key.as_str();
            // Exact, as the client's own lookups are: `.JSON` is one more file it never reads.
            #[allow(
                clippy::case_sensitive_file_extension_comparisons,
                reason = "see above"
            )]
            if !text.starts_with("sounds/") || text.ends_with(".json") {
                continue;
            }
            match AudioExt::of(text) {
                Some((ext, name)) => audio.push((
                    name,
                    SoundAsset {
                        pack: pack.clone(),
                        path: path.clone(),
                        ext,
                    },
                )),
                None if served => {
                    // A file below `sounds/` with no audio extension is never read.
                    out.diagnostics.push(file_finding(
                        DiagnosticCode::IgnoredFile { text: None },
                        Severity::Info,
                        pack,
                        path,
                    ));
                    out.ignored.push((pack.clone(), path.clone()));
                }
                None => {}
            }
        }
        out.client_audio |= !served && !audio.is_empty();
        // The probe order inside one pack; the sort is stable, so keys keep their order.
        audio.sort_by_key(|(_, asset)| asset.ext);
        for (name, asset) in audio {
            match out.by_name.get(name) {
                None => {
                    out.by_name.insert(name.to_owned(), asset);
                }
                Some(winner) if winner.pack == asset.pack => {
                    // The same pack ships the name under an earlier extension.
                    if served {
                        out.diagnostics.push(file_finding(
                            DiagnosticCode::ShadowedDefinition { text: None },
                            Severity::Info,
                            &asset.pack,
                            &asset.path,
                        ));
                    }
                    out.shadowed.push((asset, winner.clone()));
                }
                // A lower pack's file under any extension: a normal override.
                Some(_) => {}
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use bedrock_addon_core::pack_identity::{PackIdVersion, PackType};
    use bedrock_addon_core::version::SemVersion;

    fn pack(index: usize) -> PackRef {
        let id = PackIdVersion::new(
            uuid::Uuid::from_u128(index as u128 + 1),
            SemVersion::new(1, 0, 0),
            PackType::Resource,
        );
        PackRef::new(index, id, format!("pack {index}"))
    }

    /// A listing of a pack without a subpack: each file keyed by its own path.
    fn paths(list: &[&str]) -> Vec<ListedFile> {
        list.iter()
            .map(|path| {
                let path = PackPath::new(path).unwrap();
                ListedFile {
                    key: path.clone(),
                    path,
                }
            })
            .collect()
    }

    #[test]
    fn an_unsorted_listing_resolves_by_extension_order_not_listing_order() {
        let top = pack(0);
        let orders = [
            [
                "sounds/x.wav",
                "sounds/x.ogg",
                "sounds/x.fsb",
                "sounds/y.wav",
                "sounds/y.ogg",
                "sounds/a.wav",
            ],
            [
                "sounds/y.ogg",
                "sounds/x.fsb",
                "sounds/a.wav",
                "sounds/x.ogg",
                "sounds/y.wav",
                "sounds/x.wav",
            ],
            [
                "sounds/a.wav",
                "sounds/y.wav",
                "sounds/x.ogg",
                "sounds/x.wav",
                "sounds/y.ogg",
                "sounds/x.fsb",
            ],
        ];
        let indexed: Vec<SoundFiles> = orders
            .iter()
            .map(|order| {
                let listed = paths(order);
                index_listings(std::iter::once((&top, true, listed.iter())))
            })
            .collect();
        for files in &indexed {
            // `.fsb` before `.ogg` before `.wav`, whatever the listing gave first.
            assert_eq!(files.get("sounds/x").unwrap().ext, AudioExt::Fsb);
            assert_eq!(files.get("sounds/y").unwrap().ext, AudioExt::Ogg);
            assert_eq!(files.get("sounds/a").unwrap().ext, AudioExt::Wav);
            let hidden: Vec<&str> = files
                .shadowed
                .iter()
                .map(|(loser, _)| loser.path.as_str())
                .collect();
            assert_eq!(hidden, ["sounds/x.ogg", "sounds/x.wav", "sounds/y.wav"]);
        }
        // The same index, in the same order, for every listing order.
        assert_eq!(indexed[0], indexed[1]);
        assert_eq!(indexed[0], indexed[2]);
        let names: Vec<&str> = indexed[0].by_name.keys().map(String::as_str).collect();
        assert_eq!(names, ["sounds/x", "sounds/y", "sounds/a"]);
    }

    #[test]
    fn only_a_base_game_pack_puts_the_clients_audio_in_view() {
        let (top, base) = (pack(0), pack(1));
        let listed = paths(&["sounds/x.ogg"]);
        let none = paths(&["sounds/notes.txt"]);
        let served = index_listings(std::iter::once((&top, true, listed.iter())));
        assert!(!served.client_audio);
        let client =
            index_listings([(&top, true, none.iter()), (&base, false, listed.iter())].into_iter());
        assert!(client.client_audio);
        // A base-game pack without audio shows nothing of the client's.
        let empty = index_listings(std::iter::once((&base, false, none.iter())));
        assert!(!empty.client_audio && empty.ignored.is_empty());
    }
}

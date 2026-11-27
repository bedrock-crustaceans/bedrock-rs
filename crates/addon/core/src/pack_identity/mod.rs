//! The pack-identity vocabulary that `Origin`, `Location` and `Content::parse` name. Defined
//! here because the core cannot depend on `bedrock_addon`; re-exported once, at
//! `bedrock_addon::storage::PackPath` and `bedrock_addon::pack::*`, which are the documented
//! paths.

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use uuid::Uuid;

use crate::version::SemVersion;

/// A file path inside a pack, relative to the pack root: `/`-separated, case-sensitive, no `.`
/// or `..` segments. Virtual: it never names a host path.
///
/// Built by [`PackPath::new`] under vanilla's storage rules: `/` and `\` both separate segments
/// and repeated, leading or trailing separators collapse; `.` and `..` segments are rejected
/// (vanilla does not resolve them; rejecting them makes a joined host path stay below its root),
/// as are ASCII control characters (U+0000 to U+001F, U+007F) and `:` (a drive prefix or stream
/// name on Windows hosts). At most 64 segments and 1023 bytes once normalised. Ordered bytewise.
#[derive(Clone)]
pub struct PackPath {
    text: PathText,
}

/// The normalised text: a literal or a shared allocation.
#[derive(Clone)]
enum PathText {
    Static(&'static str),
    Shared(Arc<str>),
}

impl PackPath {
    /// The empty path (a root or the pack root).
    pub const ROOT: PackPath = PackPath {
        text: PathText::Static(""),
    };

    /// A path known valid at compile time; panics on a rule violation (as `BlockId::from_static`).
    ///
    /// # Panics
    ///
    /// Panics when `path` breaks a path rule.
    pub fn from_static(path: &'static str) -> PackPath {
        match check_path(path) {
            // Already normalised: keep the literal.
            Ok(_) if normalised(path) => PackPath {
                text: PathText::Static(path),
            },
            Ok(_) => Self::normalise(path),
            Err(fault) => panic!(
                "malformed pack path {path:?}: {}",
                PathRejected::from(fault)
            ),
        }
    }

    /// Validates a path under the storage rules (`/` or `\`, no `.` / `..`, segment and length
    /// limits).
    ///
    /// # Errors
    ///
    /// Returns `PathRejected` when `path` breaks a storage rule.
    pub fn new(path: &str) -> Result<PackPath, PathRejected> {
        check_path(path)?;
        Ok(Self::normalise(path))
    }

    /// `path` (already checked) with its separators normalised.
    fn normalise(path: &str) -> PackPath {
        if normalised(path) {
            return Self::shared(path);
        }
        let joined = path
            .split(['/', '\\'])
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join("/");
        if joined.is_empty() {
            Self::ROOT
        } else {
            Self::shared(&joined)
        }
    }

    fn shared(text: &str) -> PackPath {
        PackPath {
            text: PathText::Shared(Arc::from(text)),
        }
    }

    /// The `/`-separated segments, for a driver that joins them onto a host path.
    pub fn segments(&self) -> impl Iterator<Item = &str> + '_ {
        self.as_str()
            .split('/')
            .filter(|segment| !segment.is_empty())
    }

    /// The normalised text (`blocks/ruby.json`; empty for the root).
    pub fn as_str(&self) -> &str {
        match &self.text {
            PathText::Static(text) => text,
            PathText::Shared(text) => text,
        }
    }

    /// This is the root (the empty path).
    pub fn is_root(&self) -> bool {
        self.as_str().is_empty()
    }

    /// Whether the last segment ends in `.<ext>`, the letter case compared exactly
    /// (`a.json` has `json`; `a.JSON` and `a.json/b` do not).
    pub fn has_extension(&self, ext: &str) -> bool {
        let name = self.as_str().rsplit('/').next().unwrap_or_default();
        name.rsplit_once('.')
            .is_some_and(|(_, extension)| extension == ext)
    }

    /// Whether the file is a `.json` file, the letter case compared exactly
    /// ([`has_extension`](Self::has_extension)`("json")`).
    pub fn is_json(&self) -> bool {
        self.has_extension("json")
    }

    /// `self` followed by `child`'s segments (a subpack folder and a file in it). Both are
    /// already valid; a result beyond the length or segment limits is still returned, and a
    /// driver that reads it reports the miss.
    #[must_use]
    pub fn join(&self, child: &PackPath) -> PackPath {
        match (self.is_root(), child.is_root()) {
            (true, _) => child.clone(),
            (_, true) => self.clone(),
            _ => Self::shared(&format!("{}/{}", self.as_str(), child.as_str())),
        }
    }
}

/// `text` has no `\`, no empty segment and no leading or trailing `/`.
fn normalised(text: &str) -> bool {
    !text.contains('\\') && text.split('/').all(|segment| !segment.is_empty())
}

impl std::str::FromStr for PackPath {
    type Err = PathRejected;

    fn from_str(path: &str) -> Result<Self, Self::Err> {
        Self::new(path)
    }
}

impl TryFrom<&str> for PackPath {
    type Error = PathRejected;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        Self::new(path)
    }
}

impl AsRef<str> for PackPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for PackPath {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for PackPath {}

impl Hash for PackPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for PackPath {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Bytewise on the normalised text.
impl Ord for PackPath {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

/// `PackPath("blocks/ruby.json")`.
impl fmt::Debug for PackPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PackPath").field(&self.as_str()).finish()
    }
}

/// The normalised text, `/`-separated.
impl fmt::Display for PackPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why [`PackPath::new`] rejected a text.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum PathRejected {
    /// A `.` or `..` segment, which could leave the pack root.
    #[error("`.` or `..` segment at byte {index}")]
    DotSegment {
        /// Byte index of the segment's first character.
        index: usize,
    },
    /// An ASCII control character (U+0000 to U+001F, U+007F), or a `:`.
    #[error("invalid character {ch:?} at byte {index}")]
    InvalidChar {
        /// Byte index of the character.
        index: usize,
        /// The character.
        ch: char,
    },
    /// More than 64 segments.
    #[error("more than 64 path segments")]
    TooManySegments,
    /// More than 1023 bytes once normalised.
    #[error("path longer than 1023 bytes")]
    TooLong,
}

impl From<PathFault> for PathRejected {
    fn from(fault: PathFault) -> Self {
        match fault {
            PathFault::DotSegment(index) => Self::DotSegment { index },
            PathFault::InvalidChar(index, ch) => Self::InvalidChar { index, ch },
            PathFault::TooManySegments => Self::TooManySegments,
            PathFault::TooLong => Self::TooLong,
        }
    }
}

/// At most this many segments in a pack path (03: the storage limits).
pub(crate) const MAX_SEGMENTS: usize = 64;
/// At most this many bytes in a pack path, segments joined with `/` (03).
pub(crate) const MAX_PATH_BYTES: usize = 1023;

/// A pack-path rule a text breaks; `PackPath::new` and the path identifier kinds share it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PathFault {
    /// A `.` or `..` segment starting at this byte.
    DotSegment(usize),
    /// An ASCII control character or `:` at this byte.
    InvalidChar(usize, char),
    /// More than [`MAX_SEGMENTS`] segments.
    TooManySegments,
    /// More than [`MAX_PATH_BYTES`] bytes once normalised.
    TooLong,
}

/// Checks `text` under the pack-path rules: `/` and `\` separate segments, empty segments
/// collapse, `.` / `..` segments, ASCII control characters and `:` are rejected, and the
/// normalised path has at most [`MAX_SEGMENTS`] segments and [`MAX_PATH_BYTES`] bytes. Returns
/// the number of segments. `const` so identifier literals are checked at compile time.
pub(crate) const fn check_path(text: &str) -> Result<usize, PathFault> {
    let bytes = text.as_bytes();
    let mut segments = 0;
    let mut length = 0;
    let mut start = 0;
    let mut i = 0;
    while i <= bytes.len() {
        if i == bytes.len() || bytes[i] == b'/' || bytes[i] == b'\\' {
            let len = i - start;
            if len > 0 {
                if (len == 1 && bytes[start] == b'.')
                    || (len == 2 && bytes[start] == b'.' && bytes[start + 1] == b'.')
                {
                    return Err(PathFault::DotSegment(start));
                }
                segments += 1;
                // Segments after the first are joined with one `/`.
                length += len + if segments > 1 { 1 } else { 0 };
            }
            start = i + 1;
        } else if bytes[i] < 0x20 || bytes[i] == 0x7f || bytes[i] == b':' {
            // ASCII, so the byte is its own character.
            return Err(PathFault::InvalidChar(i, bytes[i] as char));
        }
        i += 1;
    }
    if segments > MAX_SEGMENTS {
        return Err(PathFault::TooManySegments);
    }
    if length > MAX_PATH_BYTES {
        return Err(PathFault::TooLong);
    }
    Ok(segments)
}

/// A cheap handle to a pack on a stack: stack index, identity, name.
///
/// Documents read from the app-data layer carry [`PackRef::is_app_data`] instead of a stack
/// index; that layer is ordered below every pack.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PackRef {
    /// The stack index (0 = highest priority); `None` for the app-data layer.
    index: Option<usize>,
    pack: Arc<PackRefInner>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct PackRefInner {
    id: PackIdVersion,
    name: Box<str>,
}

impl PackRef {
    /// The handle of the pack at `index` on a stack, with its identity and display name.
    ///
    /// The fields are private, so this is the only way to build a handle for a pack.
    pub fn new(index: usize, id: PackIdVersion, name: impl Into<Box<str>>) -> Self {
        Self {
            index: Some(index),
            pack: Arc::new(PackRefInner {
                id,
                name: name.into(),
            }),
        }
    }

    /// The synthetic handle of the app-data layer: nil UUID, version 0.0.0, name `app data`
    /// It sorts below every pack on a stack.
    pub fn app_data() -> Self {
        Self {
            index: None,
            pack: Arc::new(PackRefInner {
                id: PackIdVersion::new(Uuid::nil(), SemVersion::new(0, 0, 0), PackType::Invalid),
                name: "app data".into(),
            }),
        }
    }

    /// The pack's display name.
    pub fn name(&self) -> &str {
        &self.pack.name
    }

    /// The pack's `(uuid, version)`.
    pub fn id(&self) -> &PackIdVersion {
        &self.pack.id
    }

    /// The pack's stack index (0 = highest priority); `None` for the app-data layer.
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    /// Whether this is the synthetic pack of the app-data layer (`LoadOptions::app_data`),
    /// ordered below every pack: `true` for that layer, `false` for a pack on the stack.
    pub fn is_app_data(&self) -> bool {
        self.index.is_none()
    }
}

/// `(uuid, version)` of a manifest header: the identity a stack refers to. Equality and hashing
/// ignore `pack_type`, as vanilla's `PackIdVersion` does.
#[derive(Clone, Debug)]
pub struct PackIdVersion {
    /// The header `uuid`.
    pub uuid: Uuid,
    /// The header `version`.
    pub version: SemVersion,
    /// The pack type; not part of the identity.
    pub pack_type: PackType,
}

impl PackIdVersion {
    /// The identity `(uuid, version)` of a pack of `pack_type`.
    pub fn new(uuid: Uuid, version: SemVersion, pack_type: PackType) -> Self {
        Self {
            uuid,
            version,
            pack_type,
        }
    }
}

impl PartialEq for PackIdVersion {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid && self.version == other.version
    }
}

impl Eq for PackIdVersion {}

impl Hash for PackIdVersion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
        self.version.hash(state);
    }
}

/// `<uuid> <version>`.
impl fmt::Display for PackIdVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.uuid, self.version)
    }
}

/// What a pack holds, from the manifest's modules (vanilla `PackType`, with its values).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(i8)]
pub enum PackType {
    /// No recognised module (`invalid`).
    Invalid = 0,
    /// An add-on (`addon`).
    Addon = 1,
    /// A cached pack.
    Cached = 2,
    /// A copy-protected pack.
    CopyProtected = 3,
    /// A behaviour pack (`data`, `script`, … modules).
    Behavior = 4,
    /// A persona piece (`persona_piece`).
    PersonaPiece = 5,
    /// A resource pack (`resources`, `client_script`, … modules); vanilla's `Resources`.
    Resource = 6,
    /// A skin pack (`skin_pack`).
    Skins = 7,
    /// A world template (`world_template`).
    WorldTemplate = 8,
}

/// The side a document came from, as a value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PackSide {
    /// The behaviour-pack stack.
    Behavior,
    /// The resource-pack stack.
    Resource,
}

impl PackSide {
    /// The pack type every pack on this side's stack has.
    pub const fn pack_type(self) -> PackType {
        match self {
            Self::Behavior => PackType::Behavior,
            Self::Resource => PackType::Resource,
        }
    }
}

/// Where a pack comes from (vanilla `PackOrigin`, with its values). The origin decides
/// `is_base_game_pack`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum PackOrigin {
    /// Unknown.
    Unknown = 0,
    /// A Realms pack of unknown kind.
    RealmsUnknown = 1,
    /// The vanilla layer the host ships (app-bundled packs; base-game packs).
    Package = 2,
    /// An A/B treatment pack.
    Treatment = 3,
    /// A development pack (`development_*_packs/`).
    Dev = 4,
    /// A world's own packs.
    World = 5,
    /// A user-installed pack.
    User = 6,
    /// A temporary cached pack.
    TempCache = 7,
    /// A cached premium pack.
    PremiumCache = 8,
    /// A temporary cached premium pack.
    PremiumTempCache = 9,
    /// A pack from the Realms service.
    RealmsService = 10,
    /// A test pack.
    Test = 11,
    /// A system pack.
    System = 12,
}

impl PackOrigin {
    /// Packs of this origin are base-game packs (vanilla's base-game check): exactly
    /// [`PackOrigin::Package`].
    pub const fn is_base_game(self) -> bool {
        matches!(self, Self::Package)
    }
}

/// Where a pack may be used: the manifest header's `pack_scope` (vanilla's `PackScope` flags).
///
/// `World` is `AddOn` and `Level` together, `Any` (the default when the header has none) every
/// scope but `System`. `global`, `system`, `world` and `any` have a JSON spelling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u8)]
pub enum PackScope {
    /// The global pack stack (client settings).
    Global = 1,
    /// An add-on.
    AddOn = 2,
    /// A level's own packs.
    Level = 4,
    /// `"system"`: the system stack.
    System = 8,
    /// `"world"`: an add-on or a level's own pack.
    World = 6,
    /// `"any"`: global, add-on or level.
    #[default]
    Any = 7,
}

impl PackScope {
    /// The flag bits (Global 1, `AddOn` 2, Level 4, System 8).
    pub const fn bits(self) -> u8 {
        self as u8
    }

    /// Every flag of `other` is set in `self` (`World` contains `Level`).
    pub const fn contains(self, other: PackScope) -> bool {
        self.bits() & other.bits() == other.bits()
    }

    /// The scope a manifest names: `"global"`, `"system"`, `"world"` or `"any"`
    /// (case-sensitive); `addon` and `level` have no manifest spelling.
    pub fn from_json_name(name: &str) -> Option<PackScope> {
        match name {
            "global" => Some(Self::Global),
            "system" => Some(Self::System),
            "world" => Some(Self::World),
            "any" => Some(Self::Any),
            _ => None,
        }
    }

    /// The manifest spelling, for the four scopes that have one.
    pub const fn json_name(self) -> Option<&'static str> {
        match self {
            Self::Global => Some("global"),
            Self::System => Some("system"),
            Self::World => Some("world"),
            Self::Any => Some("any"),
            Self::AddOn | Self::Level => None,
        }
    }
}

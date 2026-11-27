//! The virtual locations a loader asks about: a caller-chosen root
//! handle plus a validated path below it, never a host path.

use std::fmt;

use crate::pack_identity::PackPath;

/// A caller-chosen handle for one host location (a directory or a file). The core never sees
/// the host path behind it; the driver keeps the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RootId(pub u32);

impl RootId {
    /// Reserved for packs built from memory (`Pack::from_files`, `Pack::from_archive`,
    /// `PackWriter::into_pack`); a loader never requests it.
    pub const MEMORY: RootId = RootId(u32::MAX);
}

/// A place a loader asks about: a root plus a validated, virtual path below it. `path` is
/// empty for the root itself.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Place {
    /// The host location.
    pub root: RootId,
    /// The virtual path below it.
    pub path: PackPath,
}

impl Place {
    /// The root itself.
    #[must_use]
    pub fn root(root: RootId) -> Place {
        Place {
            root,
            path: PackPath::ROOT,
        }
    }

    /// `self.path` joined with `child`; `child` was already validated as a `PackPath`.
    #[must_use]
    pub fn join(&self, child: &PackPath) -> Place {
        Place {
            root: self.root,
            path: self.path.join(child),
        }
    }
}

/// `#<root>/<path>` (`#3/blocks/ruby.json`); the root alone is `#<root>`. A driver renders the
/// host path itself (`bedrock_addon_std::io::Roots::host_path`).
impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.root.0)?;
        if !self.path.is_root() {
            write!(f, "/{}", self.path)?;
        }
        Ok(())
    }
}

/// How deep a directory listing goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListDepth {
    /// Direct children only (a packs directory).
    Children,
    /// Every entry below, at any depth (a pack folder). Entry names are paths relative to the
    /// listed location.
    Recursive,
}

/// How much of a file to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadRange {
    /// The whole file.
    Whole,
    /// The first `n` bytes, or the whole file if it is shorter (magic detection).
    Prefix(u32),
}

//! The root table and the one function that answers a loader request with blocking `std::fs`
//! calls. Reusable by any loop (chorus uses it).

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};

use bedrock_addon::load::{
    DirEntry, EntryKind, Input, ListDepth, Place, ReadRange, Request, RootId,
};
use bedrock_addon::storage::PackPath;
use bytes::Bytes;

/// `path` below the host directory `root`, one component per segment.
pub fn host_path(root: &Path, path: &PackPath) -> PathBuf {
    path.segments()
        .fold(root.to_path_buf(), |host, segment| host.join(segment))
}

/// The driver's root table: `RootId` → host path.
///
/// A handle is the index of its registration, so a table only grows; the driver keeps one for
/// as long as packs carry `Place`s into it.
#[derive(Clone, Debug, Default)]
pub struct Roots {
    /// Host paths, indexed by `RootId`.
    paths: Vec<PathBuf>,
}

impl Roots {
    /// An empty table.
    pub fn new() -> Roots {
        Roots::default()
    }

    /// Registers a host path and returns its handle. Registering the same path twice gives two
    /// handles.
    ///
    /// # Panics
    ///
    /// When the table already holds `u32::MAX` paths (the next handle would be
    /// `RootId::MEMORY`).
    pub fn add(&mut self, path: impl Into<PathBuf>) -> RootId {
        let index = u32::try_from(self.paths.len())
            .ok()
            .filter(|&index| index != RootId::MEMORY.0);
        let Some(index) = index else {
            panic!("the root table is full");
        };
        self.paths.push(path.into());
        RootId(index)
    }

    /// The host path of a location: the root joined with `location.path.segments()`.
    /// `None` for an unknown root or `RootId::MEMORY`.
    pub fn host_path(&self, location: &Place) -> Option<PathBuf> {
        if location.root == RootId::MEMORY {
            return None;
        }
        let root = self.paths.get(usize::try_from(location.root.0).ok()?)?;
        Some(host_path(root, &location.path))
    }

    /// Answers one request with blocking `std::fs` calls. Never panics and never fails: an
    /// I/O error becomes `Input::Missing` / `Input::Failed`. Listings follow symlinks and skip
    /// entries that are neither file nor directory. A recursive listing remembers the
    /// directories it has entered (by canonical path) and skips a link back into one, so a
    /// symlink cycle ends the walk instead of looping until the path-length limit.
    ///
    /// Listings are sorted by name. In a recursive listing a "link back" is a link to an ancestor
    /// of the entry; of two entries of one directory resolving to the same directory, the real
    /// directory is kept over a link to it, else the first by name.
    ///
    /// The answers in detail:
    /// - a location whose root is unknown, or `RootId::MEMORY`: `Failed` with
    ///   `ErrorKind::InvalidInput`;
    /// - a location that does not exist (also a dangling symlink): `Missing`;
    /// - `ReadFile` of a directory: `Failed` with `ErrorKind::IsADirectory`; of anything else that
    ///   is not a regular file (a FIFO, a socket, a device): `Failed` with
    ///   `ErrorKind::InvalidInput`, without opening it, so a FIFO cannot block the caller;
    /// - `ListDir` of a file: `Failed` with `ErrorKind::NotADirectory`;
    /// - any other error, including one while reading a subdirectory of a recursive listing:
    ///   `Failed` with its kind. An entry whose type cannot be read (a dangling symlink) is
    ///   skipped like a FIFO.
    ///
    /// Entry names are `/`-separated paths relative to the listed location; a name that is not
    /// UTF-8 is converted with replacement characters and marked `lossy`, and so is every entry
    /// below it.
    pub fn respond(&self, request: Request) -> Input {
        match request {
            Request::ReadFile {
                location, range, ..
            } => match self.host_path(&location) {
                None => unknown_root(location),
                Some(path) => match read_file(&path, range) {
                    Ok(bytes) => Input::File { location, bytes },
                    Err(error) => failure(location, error.kind()),
                },
            },
            Request::ListDir { location, depth } => match self.host_path(&location) {
                None => unknown_root(location),
                Some(path) => match walk(&path, depth) {
                    Ok(entries) => Input::Dir {
                        location,
                        entries: entries.into_iter().map(|(_, entry)| entry).collect(),
                    },
                    Err(error) => failure(location, error.kind()),
                },
            },
        }
    }
}

/// Answers one request against `roots`; the same as [`Roots::respond`], for a loop that holds
/// the table by reference.
pub fn respond(roots: &Roots, request: Request) -> Input {
    roots.respond(request)
}

fn unknown_root(location: Place) -> Input {
    Input::Failed {
        location,
        kind: ErrorKind::InvalidInput,
    }
}

fn failure(location: Place, kind: ErrorKind) -> Input {
    if kind == ErrorKind::NotFound {
        Input::Missing { location }
    } else {
        Input::Failed { location, kind }
    }
}

/// Reads `range` of the regular file at `path`, following symlinks. Anything that is not a
/// regular file is refused before it is opened.
fn read_file(path: &Path, range: ReadRange) -> io::Result<Bytes> {
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        return Err(ErrorKind::IsADirectory.into());
    }
    if !metadata.is_file() {
        return Err(ErrorKind::InvalidInput.into());
    }
    let limit = match range {
        ReadRange::Whole => u64::MAX,
        ReadRange::Prefix(n) => u64::from(n),
    };
    // The real size, not the listing's `len_hint`: the file may have changed since.
    let capacity = usize::try_from(metadata.len().min(limit)).unwrap_or(0);
    let mut bytes = Vec::with_capacity(capacity);
    File::open(path)?.take(limit).read_to_end(&mut bytes)?;
    Ok(Bytes::from(bytes))
}

/// Lists `base` (one level, or every level for `Recursive`), following symlinks, sorted by
/// name. Each entry comes with its host path, so a caller in this crate can read what it lists.
///
/// A recursive walk keeps, for each directory it enters, the canonical paths of its ancestors
/// (`base` included) and skips a directory entry that resolves to one of them: a link back up
/// is a cycle. Within one directory, two entries resolving to the same directory keep one: the
/// real directory over a link to it, else the first by name. A link to a directory that is
/// neither an ancestor nor a sibling's target is listed with its contents, so a tree may list one
/// folder twice under two names. Only a cycle ends the walk, so the result never depends on the
/// host's listing order.
pub(crate) fn walk(base: &Path, depth: ListDepth) -> io::Result<Vec<(PathBuf, DirEntry)>> {
    let recursive = depth == ListDepth::Recursive;
    let ancestors = if recursive {
        vec![fs::canonicalize(base)?]
    } else {
        Vec::new()
    };
    let mut listed = Vec::new();
    let mut pending = vec![Pending {
        directory: base.to_path_buf(),
        prefix: String::new(),
        lossy: false,
        ancestors,
    }];
    while let Some(parent) = pending.pop() {
        let mut children = read_children(&parent)?;
        if recursive {
            children = without_cycles_and_twins(children, &parent.ancestors);
        }
        for child in children {
            if let Some(canonical) = child.canonical {
                let mut ancestors = parent.ancestors.clone();
                ancestors.push(canonical);
                pending.push(Pending {
                    directory: child.path.clone(),
                    prefix: format!("{}/", child.entry.name),
                    lossy: child.entry.lossy,
                    ancestors,
                });
            }
            listed.push((child.path, child.entry));
        }
    }
    listed.sort_by(|(_, a), (_, b)| a.name.cmp(&b.name));
    Ok(listed)
}

/// A directory a walk has still to read.
struct Pending {
    directory: PathBuf,
    /// The name prefix of its entries (`blocks/ores/`).
    prefix: String,
    /// The prefix was converted lossily.
    lossy: bool,
    /// Canonical paths of the directory and everything above it, up to the walk's base.
    ancestors: Vec<PathBuf>,
}

/// One entry of a directory, before the cycle and twin checks.
struct Child {
    path: PathBuf,
    entry: DirEntry,
    /// The entry itself is a symlink.
    is_link: bool,
    /// For a directory of a recursive walk: its canonical path, once checked.
    canonical: Option<PathBuf>,
}

/// The files and directories of `parent`, sorted by name; anything else, and an entry whose
/// type cannot be read (a dangling symlink), is skipped.
fn read_children(parent: &Pending) -> io::Result<Vec<Child>> {
    let mut children = Vec::new();
    for entry in fs::read_dir(&parent.directory)? {
        let entry = entry?;
        let path = entry.path();
        // Follows symlinks; a dangling one has no type and is skipped.
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        let kind = if metadata.is_dir() {
            EntryKind::Dir
        } else if metadata.is_file() {
            EntryKind::File {
                len: metadata.len(),
            }
        } else {
            continue;
        };
        let raw = entry.file_name();
        let lossy = parent.lossy || raw.to_str().is_none();
        let name = format!("{}{}", parent.prefix, raw.to_string_lossy());
        children.push(Child {
            is_link: entry.file_type().is_ok_and(|kind| kind.is_symlink()),
            path,
            entry: DirEntry {
                name: name.into(),
                lossy,
                kind,
            },
            canonical: None,
        });
    }
    children.sort_by(|a, b| a.entry.name.cmp(&b.entry.name));
    Ok(children)
}

/// `children` (sorted by name) without the directories that lead back to an ancestor and
/// without the second of two entries resolving to one directory (the real one wins over a
/// link, else the first by name); directories kept get their canonical path.
fn without_cycles_and_twins(children: Vec<Child>, ancestors: &[PathBuf]) -> Vec<Child> {
    let mut kept: Vec<Child> = Vec::with_capacity(children.len());
    let mut by_target: HashMap<PathBuf, usize> = HashMap::new();
    for mut child in children {
        if child.entry.kind == EntryKind::Dir {
            let Ok(canonical) = fs::canonicalize(&child.path) else {
                continue;
            };
            if ancestors.contains(&canonical) {
                continue;
            }
            if let Some(&index) = by_target.get(&canonical) {
                if kept[index].is_link && !child.is_link {
                    child.canonical = Some(canonical);
                    kept[index] = child;
                }
                continue;
            }
            by_target.insert(canonical.clone(), kept.len());
            child.canonical = Some(canonical);
        }
        kept.push(child);
    }
    kept.sort_by(|a, b| a.entry.name.cmp(&b.entry.name));
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_path_joins_each_segment_below_the_root() {
        assert_eq!(
            host_path(Path::new("r"), &PackPath::from_static("a/b.json")),
            Path::new("r").join("a").join("b.json"),
            "every segment of the pack path joins onto the root"
        );
        assert_eq!(
            host_path(Path::new("r"), &PackPath::ROOT),
            Path::new("r"),
            "the pack root is the host root"
        );
    }
}

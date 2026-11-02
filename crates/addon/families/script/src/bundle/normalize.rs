//! The engine's module-name normaliser, reimplemented: the whole of import resolution.
//! Every step is byte-level.

use super::ScriptPath;

/// The specifier is one vanilla's normaliser dies on: its leading run of `./` and `../`
/// reaches the end of the specifier (`.`, `..`, `./`, `../`, `./.`, `../..`). The engine reads
/// one byte past the end, throws an out-of-range exception, and the server terminates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, thiserror::Error)]
#[error("the import specifier terminates the server")]
pub struct EngineCrash;

/// A path of this many bytes or more does not fit the engine's path buffer.
// The engine copies the path into a fixed buffer of 1024 bytes.
const PATH_LIMIT: usize = 1024;

/// How a specifier was read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Form {
    /// Returned as written: empty, `@…`, or the exact name of a native module.
    Verbatim,
    /// `.…`: resolved against the importing file's folder.
    Relative,
    /// `.…` with more leading `../` than the importing file has folders: the whole specifier
    /// is the path.
    Escapes,
    /// Anything else: a path below `scripts/`.
    Bare,
}

/// A normalised specifier with what the lints ask about it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Normalized {
    /// The module name the engine looks up.
    pub(super) name: String,
    /// How the specifier was read.
    pub(super) form: Form,
    /// `.js` was appended.
    pub(super) appended_js: bool,
    /// The path was `PATH_LIMIT` bytes or more: the release build looks up `.js` or the empty
    /// name in its place.
    pub(super) overflow: bool,
}

/// The extension of the last component of `path` with its dot (`.js` of `lib/a.b.js` and of
/// `.hidden.js`); empty without a dot.
// The extension is taken from the last path component, dot included.
pub(super) fn extension(path: &str) -> &str {
    let component = path.rsplit('/').next().unwrap_or(path);
    component.rfind('.').map_or("", |dot| &component[dot..])
}

/// The folders of the file `referrer` is: `["sub", "deep"]` for `sub/deep/d.js`, none for a
/// top-level file.
fn folders(referrer: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = referrer.split('/').collect();
    parts.pop();
    parts
}

/// The length of the leading run of `./` and `../` of `spec` and how many `../` it has.
///
/// The engine reads each byte with a bounds check: a run that needs a byte past the end is
/// [`EngineCrash`].
// The engine throws an out-of-range exception there.
fn leading_run(spec: &[u8]) -> Result<(usize, usize), EngineCrash> {
    let at = |index: usize| spec.get(index).copied().ok_or(EngineCrash);
    let (mut scanned, mut ups) = (0, 0);
    while at(scanned)? == b'.' {
        let next = at(scanned + 1)?;
        if next == b'/' {
            scanned += 2;
        } else if next == b'.' && at(scanned + 2)? == b'/' {
            scanned += 3;
            ups += 1;
        } else {
            break;
        }
    }
    Ok((scanned, ups))
}

/// The module name the engine looks up for the specifier `spec` written in the file
/// `referrer`. `known` says whether a name is a native module of the context.
///
/// - An empty specifier, one that starts with `@`, and the exact name of a native module are
///   returned as written.
/// - One that starts with `.` is read against the folder of `referrer`: a leading run of `./`
///   and `../` only. More `../` than `referrer` has folders leaves the whole specifier as the
///   path; `.` and `..` further in are kept as written.
/// - Anything else is a path below `scripts/`, never one beside `referrer`.
/// - `\` reads as `/`, and `.js` is appended unless the path ends in exactly `.js`: `./b` is
///   `b.js`, `./data.json` is `data.json.js`, `./lib/` is `lib.js`.
/// - A path of 1024 bytes or more is looked up as `.js`, or as the empty name when it is
///   relative.
///
/// # Errors
///
/// [`EngineCrash`] for a specifier whose leading run of `./` and `../` reaches its end.
pub fn normalize(
    referrer: &ScriptPath,
    spec: &str,
    known: &dyn Fn(&str) -> bool,
) -> Result<String, EngineCrash> {
    normalize_full(referrer, spec, known).map(|normalized| normalized.name)
}

/// [`normalize`], with how the specifier was read.
pub(super) fn normalize_full(
    referrer: &ScriptPath,
    spec: &str,
    known: &dyn Fn(&str) -> bool,
) -> Result<Normalized, EngineCrash> {
    // An empty specifier, an `@` name and a native module's name are returned as written.
    if spec.is_empty() || spec.starts_with('@') || known(spec) {
        return Ok(Normalized {
            name: spec.to_owned(),
            form: Form::Verbatim,
            appended_js: false,
            overflow: false,
        });
    }
    let (path, form, overflow) = if spec.starts_with('.') {
        // A relative specifier is resolved against the importing file's folder.
        let (scanned, ups) = leading_run(spec.as_bytes())?;
        let folders = folders(referrer.as_str());
        if ups > folders.len() {
            // The whole original specifier; an overflow empties the buffer, which is the
            // empty path.
            let overflow = spec.len() >= PATH_LIMIT;
            let path = if overflow {
                String::new()
            } else {
                spec.to_owned()
            };
            (path, Form::Escapes, overflow)
        } else {
            let kept = &folders[..folders.len() - ups];
            let mut path = kept.join("/");
            if !kept.is_empty() {
                path.push('/');
            }
            path.push_str(&spec[scanned..]);
            if path.len() >= PATH_LIMIT {
                // The join leaves the empty name, and nothing is appended to it.
                return Ok(Normalized {
                    name: String::new(),
                    form: Form::Relative,
                    appended_js: false,
                    overflow: true,
                });
            }
            (path, Form::Relative, false)
        }
    } else {
        // A bare specifier is a path below `scripts/`; an overflow empties the buffer.
        let overflow = spec.len() >= PATH_LIMIT;
        let path = if overflow {
            String::new()
        } else {
            spec.to_owned()
        };
        (path, Form::Bare, overflow)
    };
    // The engine's path type reads `\` as a separator.
    let mut name = path.replace('\\', "/");
    // `.js` is appended unless the name ends in it; the join drops a trailing `/`.
    let appended_js = extension(&name) != ".js";
    if appended_js {
        let trimmed = name.trim_end_matches('/').len();
        name.truncate(trimmed);
        name.push_str(".js");
    }
    Ok(Normalized {
        name,
        form,
        appended_js,
        overflow,
    })
}

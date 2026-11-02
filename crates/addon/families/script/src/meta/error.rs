//! `MetaError`: why a metadata file, a privilege list or a type descriptor was rejected.

use bedrock_addon_core::diagnostic::JsonPath;
use bedrock_addon_core::json::{SliceError, TooDeep};

/// Why a metadata file was rejected. The files are machine-written, so every deviation from the
/// shapes the generator writes is an error: a regeneration notices a new generator key or shape
/// instead of ignoring it.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MetaError {
    /// The file nests arrays and objects deeper than the JSON depth limit.
    #[error(transparent)]
    TooDeep(#[from] TooDeep),
    /// The file is not UTF-8.
    #[error("the file is not UTF-8")]
    NotUtf8(#[from] std::str::Utf8Error),
    /// The file is not JSON, or a member has a type its field does not take.
    #[error(transparent)]
    Json(#[from] SliceError),
    /// A member no field of the model takes.
    #[error("unknown member `{path}`")]
    UnknownMember {
        /// The member, from the file root.
        path: JsonPath,
    },
    /// A module UUID that does not parse.
    #[error("the module uuid does not parse")]
    Uuid(#[from] uuid::Error),
    /// A stub (a file with `parentModule`) without `wrapper_module`.
    #[error("a stub module has no `wrapper_module`")]
    StubWithoutWrapper,
    /// `wrapper_module` in a file without `parentModule`.
    #[error("`wrapper_module` without `parentModule`")]
    WrapperWithoutParent,
    /// A stub carries a member only native modules have.
    #[error("a stub module has `{member}`")]
    StubMember {
        /// The member's JSON key.
        member: &'static str,
    },
    /// A native module (a file without `parentModule`) lacks one of its nine arrays.
    #[error("a native module has no `{member}`")]
    MissingArray {
        /// The member's JSON key.
        member: &'static str,
    },
    /// A privilege list with `default` and another name.
    #[error("`default` is not the only privilege of its list")]
    DefaultNotAlone,
    /// A privilege list naming `restricted_execution` or `early_execution` twice, or naming
    /// them in the other order.
    #[error("privilege names repeated or out of order")]
    PrivilegeOrder,
    /// An empty privilege list: the generator writes one only for an unregistered privilege
    /// value.
    #[error("an empty privilege list")]
    EmptyPrivilegeList,
    /// A composite descriptor (`optional`, `array`, …) without its payload member.
    #[error("a `{tag}` descriptor has no `{key}`")]
    MissingPayload {
        /// The composite tag.
        tag: &'static str,
        /// The payload member it needs.
        key: &'static str,
    },
    /// A payload member on a descriptor whose name does not take it (on a declared type, a
    /// primitive or another composite).
    #[error("a descriptor carries `{key}`, which its name does not take")]
    UnexpectedPayload {
        /// The payload member.
        key: &'static str,
    },
    /// `error_types` on a descriptor that is not errorable, or an empty `error_types`.
    #[error("`error_types` without `is_errorable`, or empty")]
    ErrorTypes,
}

//! Reading the POI documents: each struct read member by member by core's reader
//! (`versioning::read_payload_with`, every failure told), the family's own rules on its fields
//! (`schema::read_with`): a name is `^\w+:\w+$`, a byte an integer token from 0 to 255.
//!
//! Every finding here is a parse finding: an error fails the file, as a parse error of the
//! game's loader does.

use super::{PoiTag, PoiType};
use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{Code, Diagnostics};
use bedrock_addon_core::id::{Id, Kind};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
use bedrock_addon_core::schema::{FromJson, read_each};
use bedrock_addon_core::version::{FormatVersion, known};
use bedrock_addon_core::versioning::{
    ParseCx, VersionStruct, Versioned, read_payload_with, write_payload,
};

/// The version of both documents' only struct: 1.26.40.
pub(crate) const V1_26_40: FormatVersion = known::V1_26_40;

/// Whether `text` is a POI name, `^\w+:\w+$`: a namespace and a name, each one or more of
/// `A-Z`, `a-z`, `0-9` and `_`, around one `:`.
pub(crate) fn is_poi_name(text: &str) -> bool {
    let word = |part: &str| {
        !part.is_empty()
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    };
    text.split_once(':')
        .is_some_and(|(namespace, name)| word(namespace) && word(name))
}

/// A POI name (`schema::read_with` of an identifier): a string, kept as written (anything else
/// is `InvalidType`); one that is no POI name is `InvalidValue`. Either fails the member.
pub(crate) fn name<K: Kind>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Id<K>> {
    let id = Id::<K>::from_json(node, read)?;
    if is_poi_name(id.as_str()) {
        return Some(id);
    }
    read.refuse(node, Code::from(&core_codes::INVALID_VALUE));
    None
}

/// A list of POI names (`schema::read_with` of `blocks`, `tags`, `children` and `parents`): an
/// array (anything else is `InvalidType`), each element a [`name`]; an element that fails fails
/// the list.
pub(crate) fn names<K: Kind>(node: &Node<'_>, read: &mut Read<'_>) -> Option<Vec<Id<K>>> {
    read_each(node, read, name)
}

/// A byte (`schema::read_with` of `tickets` and `usable_distance`): an integer token from 0 to
/// 255. Any other token (a real, an integral one included, a string, `null`) is `InvalidType`,
/// an integer outside `0..=255` `InvalidValue`: the game's strict reader, with the range told
/// apart.
pub(crate) fn byte(node: &Node<'_>, read: &mut Read<'_>) -> Option<u8> {
    if read.stopped() {
        return None;
    }
    let integer = node
        .value()
        .as_number()
        .filter(|number| number.is_integer())
        .map(|number| number.to_u64().and_then(|n| u8::try_from(n).ok()));
    match integer {
        Some(Some(byte)) => Some(byte),
        Some(None) => {
            read.refuse(node, Code::from(&core_codes::INVALID_VALUE));
            None
        }
        None => {
            read.wrong_type(node);
            None
        }
    }
}

impl Versioned for PoiType {
    const VERSION: FormatVersion = V1_26_40;
}

/// Read under the slice's reading (`Family::reading`: the strict mask, so an unknown member is
/// an error that refuses the file, its value still built for the pipeline to drop), telling
/// every failure: the game's reader checks every member.
impl VersionStruct for PoiType {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read_payload_with(payload, cx, d, |read| read.profile(Profile::EVERY_FAILURE))
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

impl Versioned for PoiTag {
    const VERSION: FormatVersion = V1_26_40;
}

/// Read as [`PoiType`] is.
impl VersionStruct for PoiTag {
    fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
        read_payload_with(payload, cx, d, |read| read.profile(Profile::EVERY_FAILURE))
    }

    fn write(&self) -> Result<Value, EncodeError> {
        write_payload(self)
    }
}

//! The lenient conversion's re-parse loop: a component converted from an older document
//! (`ConvertMode::Lenient`) whose members do not all read is read again member by member, and
//! element by element, each one that fails told and left out so its field takes its default.
//! The slice's `on_member_failure = DefaultAndTell` is implemented here.

use bedrock_addon_core::diagnostic::codes as core_codes;
use bedrock_addon_core::diagnostic::{
    Diagnostic, Diagnostics, JsonPath, JsonPathSegment, Severity,
};
use bedrock_addon_core::json::{JsonSlice, Value};
use bedrock_addon_core::versioning::ConvertMode;
use facet_value::{VArray, VObject};

use super::parse::{Typed, json_text, written};
use crate::typed::{failure_path, finding, read};

/// The lenient conversion's member isolation: each member read alone, a failing one reported
/// (`InvalidValue` warning) and removed so its field takes its default, then the rest read
/// together.
pub(super) fn isolate<T: Typed>(object: &mut VObject, d: &mut Diagnostics) -> Option<T> {
    let keys: Vec<String> = object.keys().map(|k| k.as_str().to_owned()).collect();
    for key in keys {
        let Some(member) = object.get(&key) else {
            continue;
        };
        let mut alone = VObject::new();
        alone.insert(key.as_str(), member.clone());
        let text = written::<T>(&alone, ConvertMode::Lenient, d)?;
        if let Err(error) = read::<T>(
            JsonSlice {
                text: &text,
                offset: 0,
            },
            None,
        ) {
            if isolate_elements::<T>(object, &key, d) {
                continue;
            }
            if T::KEEP_NESTED
                && let Some((kept, removed)) = strip_nested::<T>(&alone)
            {
                for at in removed {
                    d.push(
                        // The lenient isolation's notes are the crate's own (the game writes no
                        // content-log line for this): the legacy readers keep what they read in silence.
                        Diagnostic::of(&core_codes::INVALID_VALUE)
                            .severity(Severity::Info)
                            .at(at.clone()),
                    );
                }
                if let Some(member) = kept.get(&key) {
                    object.insert(key.as_str(), member.clone());
                }
                continue;
            }
            let at = failure_path(&error);
            let at = if at.is_empty() {
                JsonPath::new().child(&key)
            } else {
                at
            };
            let nested = at.len() > 1;
            d.push(
                // The crate's own, as above.
                Diagnostic::of(&core_codes::INVALID_VALUE)
                    .severity(Severity::Info)
                    .at(at.clone()),
            );
            if nested {
                // The member goes whole for the value inside it: told at the member too, so
                // what else it holds is known to be dropped (an information, the warning is
                // the one above).
                d.push(Diagnostic::of(&core_codes::DROPPED_FIELD).at(JsonPath::new().child(&key)));
            }
            object.remove(&key);
        }
    }
    let text = written::<T>(object, ConvertMode::Lenient, d)?;
    match read::<T>(
        JsonSlice {
            text: &text,
            offset: 0,
        },
        None,
    ) {
        Ok(value) => Some(value),
        Err(error) => {
            d.push(finding(
                &error,
                None,
                // The crate's own, as above.
                Severity::Info,
            ));
            None
        }
    }
}

/// [`isolate`] for a member that is a list: each element read alone, a failing one
/// reported (`InvalidValue` warning where it fails, and `DroppedField` INFO at the element
/// when the failure is inside it) and left out, the others kept. `false`, with nothing told
/// or changed, when the member is no list, no element fails alone, or the elements left do
/// not read together: the member then goes whole, as any other.
fn isolate_elements<T: Typed>(object: &mut VObject, key: &str, d: &mut Diagnostics) -> bool {
    let Some(list) = object.get(key).and_then(Value::as_array) else {
        return false;
    };
    let reads = |elements: VArray| {
        let mut alone = VObject::new();
        alone.insert(key, elements);
        let text = json_text::<T>(&alone, ConvertMode::Lenient)?;
        Some(
            read::<T>(
                JsonSlice {
                    text: &text,
                    offset: 0,
                },
                None,
            )
            .map(|_| ()),
        )
    };
    let mut kept = VArray::new();
    let mut told = Vec::new();
    let mut moves = Vec::new();
    for (place, element) in list.iter().enumerate() {
        let mut one = VArray::new();
        one.push(element.clone());
        let at_element = JsonPath::new().child(key).index(place);
        match reads(one) {
            None => return false,
            Some(Ok(())) => {
                if kept.len() != place {
                    moves.push((at_element, JsonPath::new().child(key).index(kept.len())));
                }
                kept.push(element.clone());
            }
            Some(Err(_)) if T::KEEP_NESTED && salvage::<T>(key, element).is_some() => {
                let Some((salvaged, removed)) = salvage::<T>(key, element) else {
                    continue;
                };
                told.extend(removed.iter().map(|at| element_finding(key, place, at)));
                if kept.len() != place {
                    moves.push((at_element, JsonPath::new().child(key).index(kept.len())));
                }
                kept.push(salvaged);
            }
            Some(Err(error)) => {
                // The failure's path names the element as the first of its list of one.
                // Below the element it is followed as far as the file has a value: a
                // failure named deeper (the element `0` of an empty list) is of the value
                // the file has there.
                let mut at = at_element.clone();
                let mut inside = false;
                let mut node = Some(element);
                let path = failure_path(&error);
                for segment in path.segments().skip(2) {
                    node = match segment {
                        JsonPathSegment::Key(name) => node
                            .and_then(Value::as_object)
                            .and_then(|object| object.get(name)),
                        JsonPathSegment::Index(number) => node
                            .and_then(Value::as_array)
                            .and_then(|array| array.get(number)),
                    };
                    if node.is_none() {
                        break;
                    }
                    inside = true;
                    match segment {
                        JsonPathSegment::Key(name) => at.push(name),
                        JsonPathSegment::Index(number) => at.push_index(number),
                    }
                }
                told.push(
                    // The crate's own, as above.
                    Diagnostic::of(&core_codes::INVALID_VALUE)
                        .severity(Severity::Info)
                        .at(at.clone()),
                );
                if inside {
                    told.push(Diagnostic::of(&core_codes::DROPPED_FIELD).at(at_element.clone()));
                }
            }
        }
    }
    if told.is_empty() || !matches!(reads(kept.clone()), Some(Ok(()))) {
        return false;
    }
    for finding in told {
        d.push(finding);
    }
    for (from, to) in moves {
        d.moved(from, to);
    }
    object.insert(key, kept);
    true
}

/// The `InvalidValue` warning at `at`, a path that names the element `place` of the list `key`
/// as the first of its list of one, at the element the file has.
fn element_finding(key: &str, place: usize, at: &JsonPath) -> Diagnostic {
    let mut here = JsonPath::new().child(key);
    here.push_index(place);
    for segment in at.segments().skip(2) {
        match segment {
            JsonPathSegment::Key(name) => here.push(name),
            JsonPathSegment::Index(number) => here.push_index(number),
        }
    }
    // The crate's own, as above.
    Diagnostic::of(&core_codes::INVALID_VALUE)
        .severity(Severity::Info)
        .at(here.clone())
}

/// [`strip_nested`] of the element `element` of the list `key`, read as the one element of its
/// list: the element with the nested members taken out, and their paths (below the list of
/// one), or `None` when it cannot be read so.
fn salvage<T: Typed>(key: &str, element: &Value) -> Option<(Value, Vec<JsonPath>)> {
    let mut one = VArray::new();
    one.push(element.clone());
    let mut alone = VObject::new();
    alone.insert(key, one);
    let (kept, removed) = strip_nested::<T>(&alone)?;
    let element = kept.get(key)?.as_array()?.get(0)?.clone();
    Some((element, removed))
}

/// Takes out of `alone` (one member of a component), one by one, each nested member at which a
/// lone read fails, until the read succeeds: the object left and the paths taken out (below the
/// component). `None` when a failure is not at a member of an object below the top level (the
/// member itself is of the wrong shape), or after 64 tries.
fn strip_nested<T: Typed>(alone: &VObject) -> Option<(VObject, Vec<JsonPath>)> {
    let mut object = alone.clone();
    let mut removed = Vec::new();
    for _ in 0..64 {
        let text = json_text::<T>(&object, ConvertMode::Lenient)?;
        let Err(error) = read::<T>(
            JsonSlice {
                text: &text,
                offset: 0,
            },
            None,
        ) else {
            return (!removed.is_empty()).then_some((object, removed));
        };
        let path = failure_path(&error);
        let segments: Vec<JsonPathSegment<'_>> = path.segments().collect();
        let (JsonPathSegment::Key(name), parents) = segments.split_last()? else {
            return None;
        };
        if parents.is_empty() {
            return None;
        }
        let mut node: Option<&mut Value> = None;
        for (i, segment) in parents.iter().enumerate() {
            node = match (i, segment, node) {
                (0, JsonPathSegment::Key(key), _) => object.get_mut(key),
                (_, JsonPathSegment::Key(key), Some(value)) => {
                    value.as_object_mut().and_then(|o| o.get_mut(key))
                }
                (_, JsonPathSegment::Index(n), Some(value)) => {
                    value.as_array_mut().and_then(|a| a.get_mut(*n))
                }
                _ => None,
            };
        }
        node?.as_object_mut()?.remove(name)?;
        removed.push(path.clone());
    }
    None
}

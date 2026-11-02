//! Writing the UI files back: each value as the JSON it was read from.

use facet_value::{VArray, VObject};

use super::load::listed;
use super::{GlobalVariables, UiDefsList, UiFile};
use bedrock_addon_core::json::Value;

/// A file: `namespace` first when it has one, then the definitions in order.
pub(super) fn file_value(file: &UiFile) -> Value {
    let mut root = VObject::new();
    if let Some(namespace) = &file.namespace {
        root.insert("namespace", namespace.as_str());
    }
    for (key, body) in &file.defs {
        root.insert(key.to_string(), body.clone());
    }
    root.into_value()
}

/// A list: `ui_defs`, then the members the game does not read. No `ui_defs` is written for
/// a list that has none and was read from a file without one.
pub(super) fn defs_list_value(list: &UiDefsList) -> Value {
    let mut root = VObject::new();
    // A `ui_defs` kept as written is written back while the typed list still is its reading.
    let written = list
        .extra
        .get("ui_defs")
        .filter(|written| read_ui_defs(written) == list.ui_defs);
    if let Some(written) = written {
        root.insert("ui_defs", written.clone());
    } else if !list.ui_defs.is_empty() {
        let mut paths = VArray::new();
        for path in &list.ui_defs {
            paths.push(path.as_str());
        }
        root.insert("ui_defs", paths);
    }
    for (key, value) in list.extra.iter() {
        if key != "ui_defs" {
            root.insert(key, value.clone());
        }
    }
    root.into_value()
}

/// The paths the game reads from a `ui_defs` value: the elements of an array.
pub(super) fn read_ui_defs(written: &Value) -> Vec<String> {
    written
        .as_array()
        .map(|paths| listed(paths.as_slice()))
        .unwrap_or_default()
}

/// The variables in order.
pub(super) fn globals_value(globals: &GlobalVariables) -> Value {
    let mut root = VObject::new();
    for (name, value) in &globals.variables {
        root.insert(name.as_str(), value.clone());
    }
    root.into_value()
}

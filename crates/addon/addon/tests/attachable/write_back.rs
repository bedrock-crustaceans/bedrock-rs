//! What `to_json` gives for an attachable built in code: text this loader reads back to the
//! same value, or `NotRepresentable` with the path of what the loader would refuse.

use bedrock_addon::document::Content;
use bedrock_addon::families::attachable::{Attachable, Description, ItemBinding};
use bedrock_addon::id::{AttachableId, MaterialId, RenderControllerId};
use bedrock_addon::json::EncodeError;
use bedrock_addon::molang::{Molang, MolangSource};
use bedrock_addon::version::FormatVersion;
use bedrock_addon::visual::{AnimateEntry, ControllerUse, HeldItemScale, Scripts};
use indexmap::IndexMap;

use super::common::*;

/// `demo:wand`, changed by `change`.
fn built(change: impl FnOnce(&mut Description)) -> Attachable {
    let id = AttachableId::parse("demo:wand");
    let mut description = Description::new(id.clone());
    change(&mut description);
    Attachable {
        description,
        ..Attachable::new(id)
    }
}

/// Requires `to_json` to refuse `attachable` at `path`.
#[track_caller]
fn refused(attachable: &Attachable, path: &str) {
    match attachable.to_json() {
        Err(EncodeError::NotRepresentable { at, .. }) => assert_eq!(at.to_string(), path),
        Err(other) => panic!("{other}"),
        Ok(text) => {
            let outcome = super::common::attachable(&text, &cx());
            panic!(
                "written: {text}\nwhich this loader {}",
                if outcome.is_ok() { "reads" } else { "refuses" }
            );
        }
    }
    // The other writer refuses alike.
    let mut out = Vec::new();
    assert!(
        attachable
            .write_json(&mut out, bedrock_addon::json::WriteStyle::default())
            .is_err()
    );
}

fn condition(text: &str) -> Molang {
    Molang::Expr(MolangSource::string_without_context(text))
}

#[test]
fn a_built_attachable_writes_text_that_reads_back_to_it() {
    let wand = built(|d| {
        d.materials
            .insert("default".to_owned(), MaterialId::parse("entity_alphatest"));
        d.item = Some(ItemBinding::Conditional(IndexMap::from([(
            "demo:wand_item".to_owned(),
            condition("query.is_sneaking"),
        )])));
        d.render_controllers = Some(vec![ControllerUse::Always(RenderControllerId::parse(
            "controller.render.item_default",
        ))]);
        d.held_item_scale = Some(HeldItemScale::Number(0.1));
        d.animations = Some(IndexMap::new());
    });
    let text = wand.to_json().unwrap();
    let again = loaded(attachable(&text, &cx()));
    assert_eq!(again.to_json().unwrap(), text);
    assert_eq!(again.description.materials, wand.description.materials);
    assert_eq!(again.binding().unwrap().0.as_str(), "demo:wand_item");
    // An authored empty `animations` is written; an absent one is not.
    assert!(text.contains("\"animations\": {}"), "{text}");
    assert!(!text.contains("sound_effects"), "{text}");
    // The scale is the shortest number that reads back to the same float.
    assert!(text.contains("\"held_item_scale\": 0.1,\n"), "{text}");
}

#[test]
fn an_item_the_loader_refuses_is_not_written() {
    const ITEM: &str = "minecraft:attachable/description/item";
    // Rule 16: an empty object.
    let empty = built(|d| d.item = Some(ItemBinding::Conditional(IndexMap::new())));
    refused(&empty, ITEM);
    // Rule 20: a key outside the pattern.
    let key = built(|d| {
        d.item = Some(ItemBinding::Conditional(IndexMap::from([(
            "demo:wand item".to_owned(),
            condition("1.0"),
        )])));
    });
    refused(&key, &format!("{ITEM}/demo:wand item"));
    // The string form takes any string (a warning when it is read).
    let odd = built(|d| d.item = Some(ItemBinding::Item("demo:wand item".to_owned())));
    let text = odd.to_json().unwrap();
    assert_eq!(
        loaded(attachable(&text, &cx()))
            .binding()
            .unwrap()
            .0
            .as_str(),
        "demo:wand item"
    );
}

#[test]
fn a_description_the_schema_refuses_is_not_written() {
    const AT: &str = "minecraft:attachable/description";
    refused(
        &Attachable::new(AttachableId::parse("")),
        &format!("{AT}/identifier"),
    );
    let key = built(|d| {
        d.materials
            .insert("my:slot".to_owned(), MaterialId::parse("m"));
    });
    refused(&key, &format!("{AT}/materials/my:slot"));
    let variable = built(|d| {
        d.scripts = Some(Scripts {
            variables: Some(IndexMap::from([(
                "variable.x".to_owned(),
                "private".to_owned(),
            )])),
            ..Scripts::default()
        });
    });
    refused(&variable, &format!("{AT}/scripts/variables/variable.x"));
    let empty_variables = built(|d| {
        d.scripts = Some(Scripts {
            variables: Some(IndexMap::new()),
            ..Scripts::default()
        });
    });
    refused(&empty_variables, &format!("{AT}/scripts/variables"));
}

#[test]
fn an_empty_conditional_object_is_written_and_reads_back() {
    // The loader holds such an entry (with its warning), so the writer writes it.
    let wand = built(|d| {
        d.render_controllers = Some(vec![
            ControllerUse::Conditional(IndexMap::new()),
            ControllerUse::Always(RenderControllerId::parse("controller.render.wand")),
        ]);
        d.scripts = Some(Scripts {
            animate: Some(vec![AnimateEntry::Conditional(IndexMap::new())]),
            ..Scripts::default()
        });
    });
    let text = wand.to_json().unwrap();
    let again = loaded(attachable(&text, &cx()));
    assert_eq!(again.value(), &wand, "{text}");
    assert_eq!(again.to_json().unwrap(), text);
}

#[test]
fn a_version_the_loader_refuses_is_not_written() {
    let future = Attachable {
        format_version: FormatVersion::new(99, 0, 0),
        ..built(|_| {})
    };
    refused(&future, "format_version");
    // The engine's own version is fine, and an older stamp is written as 1.26.0.
    let engine = Attachable {
        format_version: bedrock_addon::version::ENGINE_VERSION,
        ..built(|_| {})
    };
    loaded(attachable(&engine.to_json().unwrap(), &cx()));
}

#[test]
fn a_scale_that_is_no_number_is_not_written() {
    for scale in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let wand = built(|d| d.held_item_scale = Some(HeldItemScale::Number(scale)));
        assert!(matches!(
            wand.to_json(),
            Err(EncodeError::NotRepresentable { .. })
        ));
    }
    // A scale at or below zero is written: the loader reads it, with its warning.
    let wand = built(|d| d.held_item_scale = Some(HeldItemScale::Number(-1.0)));
    loaded(attachable(&wand.to_json().unwrap(), &cx()));
}

#[test]
fn the_stack_s_bindings_are_no_part_of_the_text() {
    let low = ghost("1.10.0", r#""item": "demo:a""#);
    let high = ghost("1.10.0", r#""item": "demo:b""#);
    let set = stack(&[
        pack(&[("attachables/ghost.json", &high)]),
        pack(&[("attachables/ghost.json", &low)]),
    ]);
    let ghost = set
        .resource
        .attachables()
        .get(&AttachableId::parse("demo:ghost"))
        .unwrap();
    assert_eq!(ghost.bound_items().len(), 2);
    let text = ghost.to_json().unwrap();
    assert!(
        !text.contains("demo:a") && !text.contains("stack_bindings"),
        "{text}"
    );
    // Read back alone, the document has its own binding and no stack's.
    let again = loaded(attachable(&text, &cx()));
    assert!(again.bound_items().is_empty());
    assert_eq!(again.description, ghost.description);
}

#[test]
fn a_short_or_array_format_version_is_written_in_full_deviation() {
    // `format_version` is held as the version it
    // reads as, so a document that is not upgraded has its version written in full, as a
    // string. A loss of spelling, not of value: the written text reads back to an equal
    // document, and vanilla reads the three spellings alike.
    let full = loaded(attachable(&ghost("1.26.0", ""), &cx()));
    for (spelled, written) in [
        (r#""1.26""#, "1.26.0"),
        ("[1, 26, 0]", "1.26.0"),
        (r#""1.26.30""#, "1.26.30"),
        ("[1, 26, 30]", "1.26.30"),
    ] {
        let text = format!(
            r#"{{"format_version": {spelled}, "minecraft:attachable": {{"description": {{"identifier": "demo:ghost"}}}}}}"#
        );
        let document = loaded(attachable(&text, &cx()));
        silent(document.diagnostics());
        assert!(!document.version().upgraded, "{spelled}");
        assert_eq!(document.format_version.to_string(), written, "{spelled}");
        let text = round_trip(&document, |text| attachable(text, &cx()));
        let json: bedrock_addon::json::Value = bedrock_addon::json::from_str(&text).unwrap();
        let version = &json.as_object().unwrap()["format_version"];
        assert_eq!(
            version.as_string().map(facet_value::VString::as_str),
            Some(written),
            "{spelled}: {text}"
        );
        if written == "1.26.0" {
            assert_eq!(document.value(), full.value(), "{spelled}");
        }
    }
    // An upgraded document is stamped whatever its spelling: no loss there.
    for spelled in [r#""1.10""#, "[1, 10, 0]", r#""1.8""#, "[1, 8, 0]"] {
        let text = format!(
            r#"{{"format_version": {spelled}, "minecraft:attachable": {{"description": {{"identifier": "demo:ghost"}}}}}}"#
        );
        let document = loaded(attachable(&text, &cx()));
        assert!(document.version().upgraded, "{spelled}");
        assert_eq!(document.format_version.to_string(), "1.26.0", "{spelled}");
    }
}

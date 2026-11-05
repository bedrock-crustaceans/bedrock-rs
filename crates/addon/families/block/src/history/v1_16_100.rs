//! Format 1.16.100: the full legacy description (with `properties`) and every legacy
//! component shape.

use super::legacy::{Legacy, component_path, input_check_failed, misread, number};
use super::ops::{clamp, edit_object, int, number_member, whole};
use bedrock_addon_core::diagnostic::{Diagnostic, DiagnosticCode, Phase};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::versioning::{Upgrade, UpgradeCx, Versioned};

/// A block document of `[1.16.100, 1.18.0)`.
pub type Block = Legacy<1, 16, 100>;

/// Vanilla's 1.18.0 fixers.
impl Upgrade for Block {
    type Next = super::v1_18_0::Block;

    fn upgrade(self, cx: &mut UpgradeCx<'_>) -> Self::Next {
        let mut block: Self::Next = self.retag();
        let to = <Self::Next as Versioned>::VERSION;
        // Input checks on the base: `crafting_table.grid_size` is 3; `block_light_absorption`
        // is a number in 0..=16, a bool, or the object form.
        if let Some(table) = block.components.get("minecraft:crafting_table")
            && let Some(size) = table.as_object().and_then(|o| o.get("grid_size"))
            && number(size) != Some(3.0)
        {
            input_check_failed(
                "minecraft:crafting_table",
                to,
                cx.original_version,
                cx.diagnostics,
            );
        }
        if let Some(absorption) = block.components.get("minecraft:block_light_absorption") {
            let ok = absorption.is_bool()
                || number_member(absorption, "filter_level")
                    .is_some_and(|n| (0.0..=16.0).contains(&n))
                || (absorption.is_object() && number_member(absorption, "filter_level").is_none());
            if !ok {
                input_check_failed(
                    "minecraft:block_light_absorption",
                    to,
                    cx.original_version,
                    cx.diagnostics,
                );
            }
        }
        block.rename_component(
            "minecraft:entity_collision",
            "minecraft:block_collision",
            |v| v,
            cx.diagnostics,
        );
        block.rename_component(
            "minecraft:pick_collision",
            "minecraft:aim_collision",
            |v| v,
            cx.diagnostics,
        );
        // `grid_size` is removed, as the game's fixer does in silence; told where the file
        // has it, an information of either size: the game says nothing, so the finding is the
        // crate's own.
        let mut sizes = Vec::new();
        for (section, map) in block.sections_mut() {
            if let Some(table) = map.get_mut("minecraft:crafting_table") {
                edit_object(table, |t| {
                    if t.shift_remove("grid_size").is_some() {
                        sizes.push(section);
                    }
                });
            }
        }
        for section in sizes {
            let mut at = component_path(section, "minecraft:crafting_table");
            at.push("grid_size");
            cx.diagnostics.push(
                Diagnostic::info(DiagnosticCode::DroppedField { text: None })
                    .at(at.clone())
                    .phase(Phase::Upgrade),
            );
        }
        // `block_light_absorption` → `block_light_filter`: a number truncated and clamped to
        // 0..=15 (the object form's `filter_level` too); a bool is 1 / 0 (the crate's own reading of a bool).
        if !block.has_anywhere("minecraft:block_light_absorption") {
            return block;
        }
        // The crate's own findings: a bool read as 1 / 0 is told as a value of another type, a number
        // with a fraction as a value the level does not hold; the level is what it was.
        for (section, map) in block.sections_mut() {
            let Some(value) = map.get("minecraft:block_light_absorption") else {
                continue;
            };
            let at = component_path(section, "minecraft:block_light_absorption");
            let level = match value.as_object() {
                Some(object) => object.get("filter_level"),
                None => Some(value),
            };
            match level {
                Some(level) if level.is_bool() => misread(&at, cx.diagnostics),
                Some(level) if number(level).is_some_and(|n| n.fract() != 0.0) => {
                    cx.diagnostics.push(
                        Diagnostic::info(DiagnosticCode::InvalidValue { text: None })
                            .at(at.clone())
                            .phase(Phase::Upgrade),
                    );
                }
                _ => {}
            }
        }
        block.rename_component(
            "minecraft:block_light_absorption",
            "minecraft:block_light_filter",
            |v| absorption_to_filter(&v),
            cx.diagnostics,
        );
        for (section, map) in block.sections_mut() {
            if let Some(value) = map.get_mut("minecraft:block_light_filter") {
                clamp_filter(value, section, cx.diagnostics);
            }
        }
        block
    }
}

/// The absorption value with a bool turned into 1 / 0 and numbers truncated.
fn absorption_to_filter(value: &Value) -> Value {
    let truncate = |n: f64| int(whole(n.trunc()));
    if let Some(b) = value.as_bool() {
        return int(i64::from(b));
    }
    if let Some(n) = number(value) {
        return truncate(n);
    }
    let mut value = value.clone();
    edit_object(&mut value, |o| {
        if let Some(n) = o.get("filter_level").and_then(number) {
            o.insert("filter_level".to_owned(), truncate(n));
        }
    });
    value
}

/// Clamps a filter level (bare or `filter_level`) into 0..=15.
fn clamp_filter(
    value: &mut Value,
    section: Option<usize>,
    d: &mut bedrock_addon_core::diagnostic::Diagnostics,
) {
    const NAME: &str = "minecraft:block_light_filter";
    if let Some(n) = number(value) {
        *value = int(whole(clamp(n, 0.0, 15.0, section, NAME, d)));
        return;
    }
    edit_object(value, |o| {
        if let Some(n) = o.get("filter_level").and_then(number) {
            o.insert(
                "filter_level".to_owned(),
                int(whole(clamp(n, 0.0, 15.0, section, NAME, d))),
            );
        }
    });
}

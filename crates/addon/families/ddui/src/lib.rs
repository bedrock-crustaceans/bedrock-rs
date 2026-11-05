//! Data-driven UI: the three document kinds below `ddui/` of a resource pack — roots
//! (`ddui/root/`, `minecraft:ui-root`), compositions (`ddui/composition/`,
//! `minecraft:ui-composition`) and form styles (`ddui/styling/`, `minecraft:ui-style`) — loaded
//! together as the `DataDrivenUi` group. One struct per kind: roots and compositions at
//! `format_version` 1.21.130 exactly, styles at `"beta"` only; the control catalogue is keyed by
//! the engine version.

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod history;
pub use catalogue::{Binding, Catalogue, ChildSet, ControlRule};
pub use component::Component;
pub use style::{
    BorderImageRepeat, BoxSize, FormLabelStyle, FormPanelStyle, FormTextFieldStyle, StyleColor,
    StyleComponents, StyleDescription, UiStyle, UiStyleId,
};

mod catalogue;
mod component;
mod load;
mod new;
mod read;
mod style;
mod validate;
mod write;

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use bedrock_addon_core::diagnostic::Diagnostic;
use bedrock_addon_core::document::Registry;
use bedrock_addon_core::id::{UiCompositionId, UiRootId};
use bedrock_addon_core::json::Value;
use bedrock_addon_core::version::{FormatVersion, known};

#[doc(hidden)]
pub use load::load_group;

/// The one `format_version` roots and compositions are read at.
pub const FORMAT_VERSION: FormatVersion = known::V1_21_130;

/// The data-driven UI of the resource packs (`rp.data_driven_ui()`): three collections loaded
/// and used together.
///
/// Roots and compositions are keyed by their `description.identifier`, compared byte for
/// byte. The client reads the stack from the top pack down and a later document replaces an
/// earlier one of the same identifier, so the **lowest** pack that defines an identifier
/// wins, whole; a document that failed replaces nothing. Every root of the stack is read
/// before the first composition, because a composition's `extension_point` must name an
/// `ExtensionPoint` control of a loaded root.
#[derive(Debug)]
#[non_exhaustive]
#[doc(alias = "ddui")]
#[doc(alias = "DataDrivenUIRepository")]
pub struct DataDrivenUi {
    /// The roots (`ddui/root/**/*.json`), in store order: by the first definition of each
    /// identifier, a replacement keeping the place.
    pub roots: Registry<UiRoot>,
    /// The compositions (`ddui/composition/**/*.json`), in store order.
    pub compositions: Registry<UiComposition>,
    /// The form styles (`ddui/styling/**/*.json`, engine 1.26.60 and later), in store order; the same
    /// combine rule as the roots'.
    pub styles: Registry<UiStyle>,
}

impl DataDrivenUi {
    /// Every finding of the three collections, roots first (the client's load order).
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> + '_ {
        self.roots
            .diagnostics()
            .chain(self.compositions.diagnostics())
            .chain(self.styles.diagnostics())
    }

    /// The controls spliced in at the extension point `name`: the markup of every
    /// composition whose `attribs.extension_point` is `name`, one composition after the other
    /// in store order, which is the order the client splices them in.
    pub fn extension_point_contents<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a Control> + 'a {
        self.compositions
            .iter()
            .filter(move |(_, composition)| {
                composition.attribs.extension_point.as_deref() == Some(name)
            })
            .flat_map(|(_, composition)| composition.layout.markup.iter())
    }
}

/// A data-driven UI root (`minecraft:ui-root`): a named screen, a tree of controls.
///
/// JSON: `{"format_version": "1.21.130", "minecraft:ui-root": {"description": {"identifier":
/// "demo:screen"}, "attribs": {"root_point": "minecraft:screen"}, "layout": {"markup":
/// [{"component": "Panel", "children": []}]}}}`. Every member is required; a member the client
/// does not bind fails the file.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:ui-root")]
#[doc(alias = "UIRoot")]
pub struct UiRoot {
    /// JSON `description`.
    pub description: RootDescription,
    /// JSON `attribs`.
    pub attribs: RootAttribs,
    /// JSON `layout`. The top-level controls of a root are of
    /// [`ChildSet::RootControl`].
    pub layout: Layout,
}

/// The `description` of a root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootDescription {
    /// The root's name (JSON `identifier`): a non-empty namespace, `:` and a non-empty name.
    /// A document of a pack the game ships must have `minecraft` somewhere in it; a document
    /// of any other pack must not.
    pub identifier: UiRootId,
}

/// The `attribs` of a root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootAttribs {
    /// Where the screen is attached (JSON `root_point`), never empty. Every vanilla root
    /// writes `minecraft:screen`; the client checks no other value when it loads the file.
    pub root_point: String,
}

/// The `layout` of a root or a composition.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    /// The top-level controls (JSON `markup`), at least one.
    pub markup: Vec<Control>,
}

/// A data-driven UI composition (`minecraft:ui-composition`): a named list of controls that
/// can be spliced into a root at one of its `ExtensionPoint` controls.
///
/// JSON: `{"format_version": "1.21.130", "minecraft:ui-composition": {"description":
/// {"identifier": "demo:extra"}, "attribs": {}, "layout": {"markup": [{"component":
/// "Form.Divider"}]}}}`. `attribs` is required even when it is empty.
#[derive(Clone, Debug, PartialEq)]
#[doc(alias = "minecraft:ui-composition")]
#[doc(alias = "UIComposition")]
pub struct UiComposition {
    /// JSON `description`.
    pub description: CompositionDescription,
    /// JSON `attribs`.
    pub attribs: CompositionAttribs,
    /// JSON `layout`. The top-level controls of a composition are of
    /// [`ChildSet::PanelContent`].
    pub layout: Layout,
}

/// The `description` of a composition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionDescription {
    /// The composition's name (JSON `identifier`), under the rules of a root's.
    pub identifier: UiCompositionId,
}

/// The `attribs` of a composition.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompositionAttribs {
    /// The extension point the composition is spliced in at (JSON `extension_point`), never
    /// empty when written: the `attribs.name` of an `ExtensionPoint` control of a loaded
    /// root. Only a pack the game ships may write it.
    pub extension_point: Option<String>,
}

/// One control: `{"component": "Form.Button", "attribs": {…}, "children": […]}`.
///
/// Which of `attribs` and `children` a control may or must write, and which control types may
/// stand below it, is the [`Catalogue`]'s of the engine that reads the file. A control drops
/// the controls below it without recursion, so a tree built in memory may nest without bound;
/// for the same reason its members cannot be moved out of it by a pattern
/// ([`std::mem::take`] them). Only the drop is free of recursion: `Clone`, `PartialEq` and
/// `Debug` are derived and take a stack frame per level, so a tree nested deeper than the
/// stack holds (some ten thousand levels) must not be cloned, compared or printed. A loaded
/// tree is never that deep: the reader's depth limit bounds it.
#[derive(Clone, Debug, PartialEq)]
pub struct Control {
    /// The control type (JSON `component`).
    pub component: Component,
    /// JSON `attribs`: opaque to the client's loader, which checks no key and no type; the
    /// screen reads it when it is shown. `None` when the control writes none.
    pub attribs: Option<Value>,
    /// The controls below (JSON `children`); `None` when the control writes none, which a
    /// control type with a child set may not.
    pub children: Option<Vec<Control>>,
}

impl Control {
    /// The controls directly below this one; none when it writes no `children`.
    pub fn children(&self) -> &[Control] {
        self.children.as_deref().unwrap_or_default()
    }

    /// The name an `ExtensionPoint` control declares: its `attribs.name`, when that is a
    /// string. `None` for every other control.
    pub fn extension_point_name(&self) -> Option<&str> {
        if self.component != Component::ExtensionPoint {
            return None;
        }
        let name = self.attribs.as_ref()?.as_object()?.get("name")?;
        Some(name.as_string()?.as_str())
    }
}

/// Drops the controls below without a stack frame per level.
impl Drop for Control {
    fn drop(&mut self) {
        // Every control is taken apart on this one list; a control dropped at the end of a
        // turn of the loop has no control below it left, so its own drop goes no deeper.
        let mut pending = self.children.take().unwrap_or_default();
        while let Some(mut control) = pending.pop() {
            pending.append(&mut control.children.take().unwrap_or_default());
        }
    }
}

/// Every control of `markup` and below, each before the controls below it, in document
/// order; no recursion.
pub(crate) fn controls(markup: &[Control]) -> impl Iterator<Item = &Control> {
    let mut pending: Vec<&Control> = markup.iter().rev().collect();
    std::iter::from_fn(move || {
        let control = pending.pop()?;
        pending.extend(control.children().iter().rev());
        Some(control)
    })
}

impl UiRoot {
    /// The extension points the root declares: the `attribs.name` of every `ExtensionPoint`
    /// control anywhere in its tree, in document order, each name once. An `ExtensionPoint`
    /// without a string `attribs.name` declares nothing.
    pub fn extension_points(&self) -> Vec<&str> {
        let mut seen = HashSet::new();
        controls(&self.layout.markup)
            .filter_map(Control::extension_point_name)
            .filter(|name| seen.insert(*name))
            .collect()
    }
}

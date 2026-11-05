//! The root and composition documents and their controls, with their constructors.

use std::collections::HashSet;

use super::Component;
use bedrock_addon_core::id::{UiCompositionId, UiRootId};
use bedrock_addon_core::json::Value;

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

impl UiRoot {
    /// The root point every vanilla root is attached at.
    pub const SCREEN: &'static str = "minecraft:screen";

    /// A root named `identifier`, attached at `minecraft:screen`, with `markup` as its
    /// top-level controls.
    pub fn new(identifier: UiRootId, markup: Vec<Control>) -> Self {
        Self {
            description: RootDescription { identifier },
            attribs: RootAttribs {
                root_point: Self::SCREEN.to_owned(),
            },
            layout: Layout { markup },
        }
    }
}

impl UiComposition {
    /// A composition named `identifier` with `markup` as its controls and no extension
    /// point.
    pub fn new(identifier: UiCompositionId, markup: Vec<Control>) -> Self {
        Self {
            description: CompositionDescription { identifier },
            attribs: CompositionAttribs::default(),
            layout: Layout { markup },
        }
    }
}

impl Control {
    /// A control of type `component` that writes neither `attribs` nor `children`.
    pub fn new(component: Component) -> Self {
        Self {
            component,
            attribs: None,
            children: None,
        }
    }

    /// A control of type `component` with `children` below it and no `attribs`.
    pub fn with_children(component: Component, children: Vec<Control>) -> Self {
        Self {
            component,
            attribs: None,
            children: Some(children),
        }
    }
}

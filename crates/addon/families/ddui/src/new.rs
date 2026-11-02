//! Constructors.

use super::{
    Component, CompositionAttribs, CompositionDescription, Control, Layout, RootAttribs,
    RootDescription, UiComposition, UiRoot,
};
use bedrock_addon_core::id::{UiCompositionId, UiRootId};

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

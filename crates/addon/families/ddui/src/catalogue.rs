//! The control catalogue: which control types an engine version knows, which members each
//! binds and which types may stand below which. What a type is and binds is the control
//! types' tagged enum's (`component::Bound`: its variants' engines and structs), read here
//! from its `Shape`; which types a child set holds on each engine is this file's table.

use super::Component;
use super::component::Bound;
use bedrock_addon_core::schema::EngineWindow;
use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_core::version::known;
use facet::{Shape, Type, UserType};

/// The control catalogue of an engine version: the closed set of control types, the members
/// each binds and its child set.
///
/// The catalogue changes with the engine while the documents stay at `format_version`
/// 1.21.130: the same file loads on one client and is skipped by another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Catalogue {
    /// Engine 1.26.20: 19 control types.
    V1_26_20,
    /// Engine 1.26.45: 22 control types. `Container.Slot` stands where `Container.Item` stood;
    /// `Container.TextField`, `Form.Image` and `ScrollableGridLayout` are new.
    V1_26_45,
    /// Engine 1.26.60: 27 control types. `Form.HorizontalStack`, `Form.Panel`, `Form.Text`,
    /// `Form.Spacing` and `Form.CloseButton` are new; `Form.Panel` is a root control, and
    /// `ExtensionPoint` may stand below `Context.List` and `ScrollableGridLayout`.
    V1_26_60,
}

/// The engine version of [`Catalogue::V1_26_45`].
const V1_26_45: FormatVersion = FormatVersion::new(1, 26, 45);
/// The engine version of [`Catalogue::V1_26_60`].
const V1_26_60: FormatVersion = known::V1_26_60;

/// Whether a control type binds a member, and whether it must be written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Binding {
    /// The type does not bind the member: writing it is an unknown member, which fails the
    /// file.
    Unbound,
    /// The member may be written.
    Optional,
    /// The member must be written.
    Required,
}

/// What a catalogue says about one control type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct ControlRule {
    /// The `attribs` member.
    pub attribs: Binding,
    /// The set its `children` are of; `None` for a type that binds no `children`. A type with
    /// a child set must write `children` (an empty array is accepted).
    pub children: Option<ChildSet>,
}

/// The control types that may stand in one place: the alternatives of the member's variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ChildSet {
    /// A root's `layout.markup`, and the children of `Context` and `Container.Layout`.
    RootControl,
    /// The children of `Panel`, `Form.ScrollView`, `Visibility`, `Form.Panel` and
    /// `Form.HorizontalStack`, and a composition's `layout.markup`.
    PanelContent,
    /// The children of `Context.List` and `ScrollableGridLayout`.
    ListContent,
    /// The children of `Container.FixedGridLayout`.
    GridContent,
}

impl ChildSet {
    /// The set whose `children` type the reader reads `shape` as.
    fn of(shape: &Shape) -> Option<Self> {
        use super::component::{GridContent, ListContent, PanelContent, RootControl};
        use facet::Facet;
        [
            (RootControl::SHAPE, Self::RootControl),
            (PanelContent::SHAPE, Self::PanelContent),
            (ListContent::SHAPE, Self::ListContent),
            (GridContent::SHAPE, Self::GridContent),
        ]
        .into_iter()
        .find_map(|(set, child_set)| (set == shape).then_some(child_set))
    }
}

use Component as C;

/// The bits of `components`.
const fn bits(components: &[Component]) -> u32 {
    let mut set = 0;
    let mut index = 0;
    while index < components.len() {
        set |= components[index].bit();
        index += 1;
    }
    set
}

/// The root controls up to 1.26.45.
const ROOT_CONTROL_1_26_20: u32 = bits(&[C::ContainerLayout, C::Context, C::Panel]);

/// 4: `Form.Panel` joins the three, as an alternative of `Layout.markup`.
const ROOT_CONTROL_1_26_60: u32 = ROOT_CONTROL_1_26_20 | C::FormPanel.bit();

/// The 15 alternatives of 1.26.20 below a panel.
const PANEL_CONTENT_1_26_20: u32 = bits(&[
    C::ContainerFixedGridLayout,
    C::ExtensionPoint,
    C::FormButton,
    C::FormDivider,
    C::FormDropdown,
    C::FormScrollView,
    C::FormSlider,
    C::FormSwitch,
    C::FormTextField,
    C::PanelCloseButton,
    C::PanelDecoration,
    C::PanelSpacing,
    C::ContextList,
    C::PanelText,
    C::Visibility,
]);

/// The 14 alternatives of 1.26.20 below a list.
const LIST_CONTENT_1_26_20: u32 = bits(&[
    C::ContainerItem,
    C::ContainerFixedGridLayout,
    C::FormButton,
    C::FormDivider,
    C::FormDropdown,
    C::FormScrollView,
    C::FormSlider,
    C::FormSwitch,
    C::FormTextField,
    C::PanelDecoration,
    C::PanelSpacing,
    C::ContextList,
    C::PanelText,
    C::Visibility,
]);

const GRID_CONTENT_1_26_20: u32 = bits(&[C::ContainerItem, C::ContextList]);

/// 18: the 15 of 1.26.20 and the three new types.
const PANEL_CONTENT_1_26_45: u32 =
    PANEL_CONTENT_1_26_20 | bits(&[C::ContainerTextField, C::FormImage, C::ScrollableGridLayout]);

/// 17: `Container.Slot` for `Container.Item`, and the three new types.
const LIST_CONTENT_1_26_45: u32 = (LIST_CONTENT_1_26_20 & !C::ContainerItem.bit())
    | bits(&[
        C::ContainerSlot,
        C::ContainerTextField,
        C::FormImage,
        C::ScrollableGridLayout,
    ]);

/// The alternatives below `Container.FixedGridLayout` from 1.26.45: `Container.Item` is replaced
/// by `Container.Slot`.
const GRID_CONTENT_1_26_45: u32 = bits(&[C::ContainerSlot, C::ContextList]);

/// The four types of 1.26.60 that stand in panels and lists (not `Form.Panel`).
const FORM_CONTENT_1_26_60: u32 = bits(&[
    C::FormCloseButton,
    C::FormSpacing,
    C::FormText,
    C::FormHorizontalStack,
]);

/// 22: the 18 of 1.26.45 and four of the five new types.
const PANEL_CONTENT_1_26_60: u32 = PANEL_CONTENT_1_26_45 | FORM_CONTENT_1_26_60;

/// 22: the 17 of 1.26.45, the same four and `ExtensionPoint`, which from 1.26.60 may stand
/// below `Context.List` and `ScrollableGridLayout`.
const LIST_CONTENT_1_26_60: u32 =
    LIST_CONTENT_1_26_45 | FORM_CONTENT_1_26_60 | C::ExtensionPoint.bit();

impl Catalogue {
    /// Every catalogue, oldest first.
    pub const ALL: [Catalogue; 3] = [Self::V1_26_20, Self::V1_26_45, Self::V1_26_60];

    /// The catalogue an engine of version `engine` reads documents with: 1.26.60's from
    /// 1.26.60, 1.26.45's from 1.26.45, 1.26.20's below it (the versions between at which the
    /// catalogue changed are not known).
    pub fn for_engine(engine: FormatVersion) -> Self {
        if engine >= V1_26_60 {
            Self::V1_26_60
        } else if engine >= V1_26_45 {
            Self::V1_26_45
        } else {
            Self::V1_26_20
        }
    }

    /// The engine version the catalogue is of.
    pub fn engine(self) -> FormatVersion {
        match self {
            Self::V1_26_20 => known::V1_26_20,
            Self::V1_26_45 => V1_26_45,
            Self::V1_26_60 => V1_26_60,
        }
    }

    /// What the catalogue says about `component`; `None` when it has no such control type.
    /// Read from the control types' tagged enum the reader reads by: the variant's engines,
    /// whether its struct has `attribs` (on this engine, and `schema::required`) and of which
    /// child set its `children` are.
    pub fn rule(self, component: Component) -> Option<ControlRule> {
        let variant = Bound::variant(component)?;
        let engine = self.engine();
        if !EngineWindow::of(variant.attributes).admits(engine) {
            return None;
        }
        let Type::User(UserType::Struct(members)) = variant.data.fields.first()?.shape().ty else {
            return None;
        };
        let member = |name: &str| members.fields.iter().find(|field| field.name == name);
        let attribs = match member("attribs") {
            Some(field) if EngineWindow::of(field.attributes).admits(engine) => {
                if field.has_attr(Some("schema"), "required") {
                    Binding::Required
                } else {
                    Binding::Optional
                }
            }
            _ => Binding::Unbound,
        };
        let children = member("children").and_then(|field| ChildSet::of(field.shape()));
        Some(ControlRule { attribs, children })
    }

    /// Whether the catalogue has the control type.
    pub fn has(self, component: Component) -> bool {
        self.rule(component).is_some()
    }

    /// The control types of `set`, as bits.
    const fn set(self, set: ChildSet) -> u32 {
        match (self, set) {
            (Self::V1_26_20 | Self::V1_26_45, ChildSet::RootControl) => ROOT_CONTROL_1_26_20,
            (Self::V1_26_60, ChildSet::RootControl) => ROOT_CONTROL_1_26_60,
            (Self::V1_26_20, ChildSet::PanelContent) => PANEL_CONTENT_1_26_20,
            (Self::V1_26_20, ChildSet::ListContent) => LIST_CONTENT_1_26_20,
            (Self::V1_26_20, ChildSet::GridContent) => GRID_CONTENT_1_26_20,
            (Self::V1_26_45, ChildSet::PanelContent) => PANEL_CONTENT_1_26_45,
            (Self::V1_26_45, ChildSet::ListContent) => LIST_CONTENT_1_26_45,
            (Self::V1_26_45 | Self::V1_26_60, ChildSet::GridContent) => GRID_CONTENT_1_26_45,
            (Self::V1_26_60, ChildSet::PanelContent) => PANEL_CONTENT_1_26_60,
            (Self::V1_26_60, ChildSet::ListContent) => LIST_CONTENT_1_26_60,
        }
    }

    /// Whether `component` may stand where `set` is expected.
    pub fn allows(self, set: ChildSet, component: Component) -> bool {
        self.set(set) & component.bit() != 0
    }

    /// The control types of `set`, in declaration order of [`Component`].
    pub fn members(self, set: ChildSet) -> impl Iterator<Item = Component> {
        Component::ALL
            .into_iter()
            .filter(move |component| self.allows(set, *component))
    }

    /// The first catalogue after this one that has `component`, when this one lacks it.
    pub(crate) fn later_with(self, component: Component) -> Option<Catalogue> {
        Self::ALL
            .into_iter()
            .find(|other| *other > self && other.has(component))
    }

    /// The last catalogue before this one that has `component`, when this one lacks it.
    pub(crate) fn earlier_with(self, component: Component) -> Option<Catalogue> {
        Self::ALL
            .into_iter()
            .rev()
            .find(|other| *other < self && other.has(component))
    }
}

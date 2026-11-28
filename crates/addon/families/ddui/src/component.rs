//! The control types of data-driven UI (`ComponentType`) and their JSON names.

use bedrock_addon_core::json::Value;
use bedrock_addon_core::json::kit::{Expected, Node, Read};
use bedrock_addon_core::schema::{self, FromJson, Schema};

use crate::read;
use crate::{ChildSet, Control};

/// The type of a control: the value of its `component` member.
///
/// The set is closed per engine version ([`Catalogue`](super::Catalogue)): a name outside the
/// catalogue of the engine that reads the document fails the whole file. Names are compared
/// with case (`"panel"` is no control).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, facet::Facet)]
#[non_exhaustive]
#[doc(alias = "ComponentType")]
#[repr(u8)]
pub enum Component {
    /// `ExtensionPoint`: a named place in a root where compositions are spliced in.
    ExtensionPoint,
    /// `Panel`.
    Panel,
    /// `Panel.Text`.
    PanelText,
    /// `Panel.Decoration`.
    PanelDecoration,
    /// `Panel.Spacing`.
    PanelSpacing,
    /// `Panel.CloseButton`.
    PanelCloseButton,
    /// `Context`: binds a data path for the controls below it.
    Context,
    /// `Context.List`: repeats its children per element of a bound list.
    ContextList,
    /// `ScrollableGridLayout` (from engine 1.26.45).
    ScrollableGridLayout,
    /// `Container.FixedGridLayout`.
    ContainerFixedGridLayout,
    /// `Container.Item` (until engine 1.26.45, which has `Container.Slot` in its place).
    ContainerItem,
    /// `Container.Slot` (from engine 1.26.45).
    ContainerSlot,
    /// `Container.Layout`.
    ContainerLayout,
    /// `Container.TextField` (from engine 1.26.45).
    ContainerTextField,
    /// `Form.Button`.
    FormButton,
    /// `Form.Divider`.
    FormDivider,
    /// `Form.Dropdown`.
    FormDropdown,
    /// `Form.Image` (from engine 1.26.45).
    FormImage,
    /// `Form.ScrollView`.
    FormScrollView,
    /// `Form.Slider`.
    FormSlider,
    /// `Form.Switch`.
    FormSwitch,
    /// `Form.TextField`.
    FormTextField,
    /// `Visibility`: shows or hides its children.
    Visibility,
    /// `Form.HorizontalStack` (from engine 1.26.60): lays its children out in a row.
    FormHorizontalStack,
    /// `Form.Panel` (from engine 1.26.60): the form's panel, a root control.
    FormPanel,
    /// `Form.Text` (from engine 1.26.60).
    FormText,
    /// `Form.Spacing` (from engine 1.26.60).
    FormSpacing,
    /// `Form.CloseButton` (from engine 1.26.60).
    FormCloseButton,
}

/// Every control type with its JSON name: the one table both directions read.
const NAMES: [(Component, &str); 28] = [
    (Component::ExtensionPoint, "ExtensionPoint"),
    (Component::Panel, "Panel"),
    (Component::PanelText, "Panel.Text"),
    (Component::PanelDecoration, "Panel.Decoration"),
    (Component::PanelSpacing, "Panel.Spacing"),
    (Component::PanelCloseButton, "Panel.CloseButton"),
    (Component::Context, "Context"),
    (Component::ContextList, "Context.List"),
    (Component::ScrollableGridLayout, "ScrollableGridLayout"),
    (
        Component::ContainerFixedGridLayout,
        "Container.FixedGridLayout",
    ),
    (Component::ContainerItem, "Container.Item"),
    (Component::ContainerSlot, "Container.Slot"),
    (Component::ContainerLayout, "Container.Layout"),
    (Component::ContainerTextField, "Container.TextField"),
    (Component::FormButton, "Form.Button"),
    (Component::FormDivider, "Form.Divider"),
    (Component::FormDropdown, "Form.Dropdown"),
    (Component::FormImage, "Form.Image"),
    (Component::FormScrollView, "Form.ScrollView"),
    (Component::FormSlider, "Form.Slider"),
    (Component::FormSwitch, "Form.Switch"),
    (Component::FormTextField, "Form.TextField"),
    (Component::Visibility, "Visibility"),
    (Component::FormHorizontalStack, "Form.HorizontalStack"),
    (Component::FormPanel, "Form.Panel"),
    (Component::FormText, "Form.Text"),
    (Component::FormSpacing, "Form.Spacing"),
    (Component::FormCloseButton, "Form.CloseButton"),
];

impl Component {
    /// Every control type of every engine version, in declaration order.
    pub const ALL: [Component; 28] = {
        let mut all = [Component::ExtensionPoint; 28];
        let mut index = 0;
        while index < NAMES.len() {
            all[index] = NAMES[index].0;
            index += 1;
        }
        all
    };

    /// The JSON name (`"Form.Button"`).
    pub fn name(self) -> &'static str {
        NAMES[self as usize].1
    }

    /// The control type a JSON name selects; `None` for any other string (`"panel"`
    /// included).
    pub fn from_name(name: &str) -> Option<Self> {
        NAMES
            .iter()
            .find(|(_, known)| *known == name)
            .map(|(component, _)| *component)
    }

    /// The control type's bit in a set of control types.
    pub(crate) const fn bit(self) -> u32 {
        1 << (self as u32)
    }
}

/// The JSON name.
impl std::fmt::Display for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

// ---- the control types as the reader takes them ---------------------------------------------

/// The control types as the reader takes them, in the order of [`Component`] and of the same
/// names: `component` names the variant ([`read::component`]: the name the catalogue of the
/// read's engine has, by each variant's `since_engine` and `until_engine`), and the variant's
/// struct says which members the type binds and which child set its `children` are of.
#[derive(Debug, facet::Facet, Schema)]
#[facet(
    tag = "component",
    schema::tag_with = read::component,
    schema::after_read = read::extension_point
)]
#[repr(u8)]
pub(crate) enum Bound {
    #[facet(rename = "ExtensionPoint")]
    ExtensionPoint(Leaf),
    #[facet(rename = "Panel")]
    Panel(Branch<PanelContent>),
    #[facet(rename = "Panel.Text")]
    PanelText(Text),
    #[facet(rename = "Panel.Decoration")]
    PanelDecoration(Bare),
    #[facet(rename = "Panel.Spacing")]
    PanelSpacing(Bare),
    #[facet(rename = "Panel.CloseButton")]
    PanelCloseButton(Leaf),
    #[facet(rename = "Context")]
    Context(Branch<RootControl>),
    #[facet(rename = "Context.List")]
    ContextList(Branch<ListContent>),
    #[facet(rename = "ScrollableGridLayout", schema::since_engine = "1.26.45")]
    ScrollableGridLayout(Branch<ListContent>),
    #[facet(rename = "Container.FixedGridLayout")]
    ContainerFixedGridLayout(Branch<GridContent>),
    #[facet(rename = "Container.Item", schema::until_engine = "1.26.45")]
    ContainerItem(Leaf),
    #[facet(rename = "Container.Slot", schema::since_engine = "1.26.45")]
    ContainerSlot(Leaf),
    #[facet(rename = "Container.Layout")]
    ContainerLayout(Holder<RootControl>),
    #[facet(rename = "Container.TextField", schema::since_engine = "1.26.45")]
    ContainerTextField(Leaf),
    #[facet(rename = "Form.Button")]
    FormButton(Leaf),
    #[facet(rename = "Form.Divider")]
    FormDivider(Divider),
    #[facet(rename = "Form.Dropdown")]
    FormDropdown(Leaf),
    #[facet(rename = "Form.Image", schema::since_engine = "1.26.45")]
    FormImage(Leaf),
    #[facet(rename = "Form.ScrollView")]
    FormScrollView(Branch<PanelContent>),
    #[facet(rename = "Form.Slider")]
    FormSlider(Leaf),
    #[facet(rename = "Form.Switch")]
    FormSwitch(Leaf),
    #[facet(rename = "Form.TextField")]
    FormTextField(Leaf),
    #[facet(rename = "Visibility")]
    Visibility(Branch<PanelContent>),
    #[facet(rename = "Form.HorizontalStack", schema::since_engine = "1.26.60")]
    FormHorizontalStack(Holder<PanelContent>),
    #[facet(rename = "Form.Panel", schema::since_engine = "1.26.60")]
    FormPanel(Branch<PanelContent>),
    #[facet(rename = "Form.Text", schema::since_engine = "1.26.60")]
    FormText(Text),
    #[facet(rename = "Form.Spacing", schema::since_engine = "1.26.60")]
    FormSpacing(Bare),
    #[facet(rename = "Form.CloseButton", schema::since_engine = "1.26.60")]
    FormCloseButton(Text),
}

impl Bound {
    /// The variant of `component`, from the enum's `Shape`: its variants stand in the order of
    /// [`Component`].
    pub(crate) fn variant(component: Component) -> Option<&'static facet::Variant> {
        use facet::Facet;
        match Self::SHAPE.ty {
            facet::Type::User(facet::UserType::Enum(en)) => en.variants.get(component as usize),
            _ => None,
        }
    }

    /// The control the variant reads as.
    pub(crate) fn into_control(self) -> Control {
        match self {
            Self::ExtensionPoint(payload) => payload.control(Component::ExtensionPoint),
            Self::Panel(payload) => payload.control(Component::Panel),
            Self::PanelText(payload) => payload.control(Component::PanelText),
            Self::PanelDecoration(payload) => payload.control(Component::PanelDecoration),
            Self::PanelSpacing(payload) => payload.control(Component::PanelSpacing),
            Self::PanelCloseButton(payload) => payload.control(Component::PanelCloseButton),
            Self::Context(payload) => payload.control(Component::Context),
            Self::ContextList(payload) => payload.control(Component::ContextList),
            Self::ScrollableGridLayout(payload) => payload.control(Component::ScrollableGridLayout),
            Self::ContainerFixedGridLayout(payload) => {
                payload.control(Component::ContainerFixedGridLayout)
            }
            Self::ContainerItem(payload) => payload.control(Component::ContainerItem),
            Self::ContainerSlot(payload) => payload.control(Component::ContainerSlot),
            Self::ContainerLayout(payload) => payload.control(Component::ContainerLayout),
            Self::ContainerTextField(payload) => payload.control(Component::ContainerTextField),
            Self::FormButton(payload) => payload.control(Component::FormButton),
            Self::FormDivider(payload) => payload.control(Component::FormDivider),
            Self::FormDropdown(payload) => payload.control(Component::FormDropdown),
            Self::FormImage(payload) => payload.control(Component::FormImage),
            Self::FormScrollView(payload) => payload.control(Component::FormScrollView),
            Self::FormSlider(payload) => payload.control(Component::FormSlider),
            Self::FormSwitch(payload) => payload.control(Component::FormSwitch),
            Self::FormTextField(payload) => payload.control(Component::FormTextField),
            Self::Visibility(payload) => payload.control(Component::Visibility),
            Self::FormHorizontalStack(payload) => payload.control(Component::FormHorizontalStack),
            Self::FormPanel(payload) => payload.control(Component::FormPanel),
            Self::FormText(payload) => payload.control(Component::FormText),
            Self::FormSpacing(payload) => payload.control(Component::FormSpacing),
            Self::FormCloseButton(payload) => payload.control(Component::FormCloseButton),
        }
    }
}

/// A type that binds neither `attribs` nor `children`.
#[derive(Debug, PartialEq, facet::Facet, Schema)]
pub(crate) struct Bare {}

/// A type that may write `attribs` and binds no `children`.
#[derive(Debug, PartialEq, facet::Facet, Schema)]
pub(crate) struct Leaf {
    /// JSON `attribs`, any JSON.
    #[facet(schema::reads_null, schema::read_with = read::attribs)]
    pub(crate) attribs: Option<Value>,
}

/// A type that must write `attribs` and binds no `children`.
#[derive(Debug, PartialEq, facet::Facet, Schema)]
pub(crate) struct Text {
    /// JSON `attribs`, any JSON.
    #[facet(schema::required, schema::reads_null, schema::read_with = read::attribs)]
    attribs: Option<Value>,
}

/// `Form.Divider`: `attribs` from engine 1.26.60 (the vanilla roots write `styleId` there);
/// before it the type binds nothing.
#[derive(Debug, PartialEq, facet::Facet, Schema)]
pub(crate) struct Divider {
    /// JSON `attribs`, any JSON.
    #[facet(
        schema::since_engine = "1.26.60",
        schema::reads_null,
        schema::read_with = read::attribs
    )]
    attribs: Option<Value>,
}

/// A type that may write `attribs` and must write `children` of the set `C`.
#[derive(Debug, facet::Facet, Schema)]
pub(crate) struct Branch<C> {
    /// JSON `attribs`, any JSON.
    #[facet(schema::reads_null, schema::read_with = read::attribs)]
    attribs: Option<Value>,
    /// JSON `children` (an empty array is accepted).
    children: C,
}

/// A type that binds no `attribs` and must write `children` of the set `C`.
#[derive(Debug, facet::Facet, Schema)]
pub(crate) struct Holder<C> {
    /// JSON `children` (an empty array is accepted).
    children: C,
}

/// The `children` of a type with a child set: each set its own type, read by its set.
macro_rules! child_sets {
    ($($set:ident),* $(,)?) => {$(
        #[doc = concat!("The `children` of a type whose child set is [`ChildSet::", stringify!($set), "`].")]
        #[derive(Debug, facet::Facet)]
        #[facet(opaque)]
        pub(crate) struct $set(Vec<Control>);

        impl FromJson for $set {
            const EXPECTED: Option<Expected> = Some(Expected::Array);

            fn from_json(node: &Node<'_>, read: &mut Read<'_>) -> Option<Self> {
                read::children(node, read, ChildSet::$set).map(Self)
            }
        }

        impl From<$set> for Vec<Control> {
            fn from(children: $set) -> Self {
                children.0
            }
        }

        bedrock_addon_core::impl_schema_leaf!($set);
        bedrock_addon_core::impl_no_molang!($set);
    )*};
}

child_sets!(RootControl, PanelContent, ListContent, GridContent);

/// What a variant's struct holds, as the members of a control of type `component`.
trait Members {
    fn control(self, component: Component) -> Control;
}

impl Members for Bare {
    fn control(self, component: Component) -> Control {
        Control::new(component)
    }
}

impl Members for Leaf {
    fn control(self, component: Component) -> Control {
        Control {
            component,
            attribs: self.attribs,
            children: None,
        }
    }
}

impl Members for Text {
    fn control(self, component: Component) -> Control {
        Leaf {
            attribs: self.attribs,
        }
        .control(component)
    }
}

impl Members for Divider {
    fn control(self, component: Component) -> Control {
        Leaf {
            attribs: self.attribs,
        }
        .control(component)
    }
}

impl<C: Into<Vec<Control>>> Members for Branch<C> {
    fn control(self, component: Component) -> Control {
        Control {
            component,
            attribs: self.attribs,
            children: Some(self.children.into()),
        }
    }
}

impl<C: Into<Vec<Control>>> Members for Holder<C> {
    fn control(self, component: Component) -> Control {
        Control::with_children(component, self.children.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Catalogue::rule` finds a control type's variant by its index: the variants must stand in
    /// the order of `Component`, each renamed to the type's JSON name.
    #[test]
    fn the_bound_variants_stand_in_the_order_of_component() {
        let names: Vec<&str> = Component::ALL
            .into_iter()
            .map(|component| {
                Bound::variant(component).map_or("", |variant| variant.effective_name())
            })
            .collect();
        let expected: Vec<&str> = Component::ALL.into_iter().map(Component::name).collect();
        assert_eq!(names, expected);
    }
}

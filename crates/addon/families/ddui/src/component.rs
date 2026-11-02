//! The control types of data-driven UI (`ComponentType`) and their JSON names.

/// The type of a control: the value of its `component` member.
///
/// The set is closed per engine version ([`Catalogue`](super::Catalogue)): a name outside the
/// catalogue of the engine that reads the document fails the whole file. Names are compared
/// with case (`"panel"` is no control).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
#[doc(alias = "ComponentType")]
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
    #[must_use]
    pub fn name(self) -> &'static str {
        NAMES[self as usize].1
    }

    /// The control type a JSON name selects; `None` for any other string (`"panel"`
    /// included).
    #[must_use]
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

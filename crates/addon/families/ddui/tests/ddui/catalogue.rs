//! The control types and the catalogue of each engine version.

use bedrock_addon_core::version::FormatVersion;
use bedrock_addon_family_ddui::{Binding, Catalogue, ChildSet, Component};

use Component as C;

/// Every control type with the text the client's files write for it.
const NAMES: [(Component, &str); 28] = [
    (C::ExtensionPoint, "ExtensionPoint"),
    (C::Panel, "Panel"),
    (C::PanelText, "Panel.Text"),
    (C::PanelDecoration, "Panel.Decoration"),
    (C::PanelSpacing, "Panel.Spacing"),
    (C::PanelCloseButton, "Panel.CloseButton"),
    (C::Context, "Context"),
    (C::ContextList, "Context.List"),
    (C::ScrollableGridLayout, "ScrollableGridLayout"),
    (C::ContainerFixedGridLayout, "Container.FixedGridLayout"),
    (C::ContainerItem, "Container.Item"),
    (C::ContainerSlot, "Container.Slot"),
    (C::ContainerLayout, "Container.Layout"),
    (C::ContainerTextField, "Container.TextField"),
    (C::FormButton, "Form.Button"),
    (C::FormDivider, "Form.Divider"),
    (C::FormDropdown, "Form.Dropdown"),
    (C::FormImage, "Form.Image"),
    (C::FormScrollView, "Form.ScrollView"),
    (C::FormSlider, "Form.Slider"),
    (C::FormSwitch, "Form.Switch"),
    (C::FormTextField, "Form.TextField"),
    (C::Visibility, "Visibility"),
    (C::FormHorizontalStack, "Form.HorizontalStack"),
    (C::FormPanel, "Form.Panel"),
    (C::FormText, "Form.Text"),
    (C::FormSpacing, "Form.Spacing"),
    (C::FormCloseButton, "Form.CloseButton"),
];

/// The read and the write of a name share one table: each side is held to the literal text.
#[test]
fn every_control_type_reads_and_writes_its_literal_name() {
    assert_eq!(Component::ALL.len(), NAMES.len());
    for (index, (component, name)) in NAMES.into_iter().enumerate() {
        assert_eq!(Component::ALL[index], component);
        assert_eq!(component.name(), name, "{component:?} writes");
        assert_eq!(component.to_string(), name);
        assert_eq!(Component::from_name(name), Some(component), "{name} reads");
    }
}

#[test]
fn a_name_is_compared_with_case_and_whole() {
    for name in [
        "panel",
        "PANEL",
        "Panel ",
        " Panel",
        "Panel.",
        "Form",
        "form.button",
        "",
    ] {
        assert_eq!(Component::from_name(name), None, "{name:?}");
    }
}

fn names(catalogue: Catalogue, set: ChildSet) -> Vec<&'static str> {
    let mut names: Vec<&str> = catalogue.members(set).map(Component::name).collect();
    names.sort_unstable();
    names
}

fn sorted(mut names: Vec<&'static str>) -> Vec<&'static str> {
    names.sort_unstable();
    names
}

#[test]
fn the_engine_selects_the_catalogue_at_1_26_45() {
    let of = |major, minor, patch| Catalogue::for_engine(FormatVersion::new(major, minor, patch));
    assert_eq!(of(1, 26, 44), Catalogue::V1_26_20);
    assert_eq!(of(1, 26, 45), Catalogue::V1_26_45);
    assert_eq!(of(1, 26, 46), Catalogue::V1_26_45);
    assert_eq!(of(1, 26, 20), Catalogue::V1_26_20);
    // Nothing older than the first catalogue is known: it stands for every engine below.
    assert_eq!(of(1, 21, 130), Catalogue::V1_26_20);
    assert_eq!(of(1, 26, 59), Catalogue::V1_26_45);
    assert_eq!(Catalogue::V1_26_20.engine(), FormatVersion::new(1, 26, 20));
    assert_eq!(Catalogue::V1_26_45.engine(), FormatVersion::new(1, 26, 45));
}

/// The catalogue of 1.26.60 from engine 1.26.60 on; 1.26.59 still
/// reads with 1.26.45's (the version between at which it changed is not known).
#[test]
fn the_engine_selects_the_1_26_60_catalogue_from_1_26_60() {
    let of = |major, minor, patch| Catalogue::for_engine(FormatVersion::new(major, minor, patch));
    assert_eq!(of(1, 26, 59), Catalogue::V1_26_45);
    assert_eq!(of(1, 26, 60), Catalogue::V1_26_60);
    assert_eq!(of(1, 27, 0), Catalogue::V1_26_60);
    assert_eq!(Catalogue::V1_26_60.engine(), FormatVersion::new(1, 26, 60));
    assert_eq!(
        Catalogue::ALL,
        [
            Catalogue::V1_26_20,
            Catalogue::V1_26_45,
            Catalogue::V1_26_60
        ]
    );
}

/// 27 control types: the 22 of 1.26.45 and five more.
#[test]
fn the_1_26_60_catalogue_has_27_control_types() {
    let known: Vec<&str> = Component::ALL
        .into_iter()
        .filter(|c| Catalogue::V1_26_60.has(*c))
        .map(Component::name)
        .collect();
    assert_eq!(known.len(), 27);
    assert!(!known.contains(&"Container.Item"));
    for added in [
        "Form.HorizontalStack",
        "Form.Panel",
        "Form.Text",
        "Form.Spacing",
        "Form.CloseButton",
    ] {
        assert!(known.contains(&added), "{added}");
        let component = Component::from_name(added).unwrap();
        assert!(
            !Catalogue::V1_26_45.has(component),
            "{added} before 1.26.60"
        );
        assert!(!Catalogue::V1_26_20.has(component), "{added} in 1.26.20");
    }
}

/// `attribs` and `children` of the five new types.
#[test]
fn the_member_traits_of_the_five_types_added_by_1_26_60() {
    let rows = [
        (
            C::FormHorizontalStack,
            Binding::Unbound,
            Some(ChildSet::PanelContent),
        ),
        (
            C::FormPanel,
            Binding::Optional,
            Some(ChildSet::PanelContent),
        ),
        (C::FormText, Binding::Required, None),
        (C::FormSpacing, Binding::Unbound, None),
        (C::FormCloseButton, Binding::Required, None),
    ];
    for (component, attribs, children) in rows {
        let rule = Catalogue::V1_26_60
            .rule(component)
            .unwrap_or_else(|| panic!("{component}"));
        assert_eq!(rule.attribs, attribs, "{component} attribs");
        assert_eq!(rule.children, children, "{component} children");
        assert_eq!(Catalogue::V1_26_45.rule(component), None, "{component}");
    }
    // The vanilla `custom_form.json` / `message_box.json` of
    // 1.26.60 (`Form.Divider` with `attribs.styleId`): `Form.Divider` binds `attribs`.
    assert_eq!(
        Catalogue::V1_26_45.rule(C::FormDivider).unwrap().attribs,
        Binding::Unbound
    );
    assert_eq!(
        Catalogue::V1_26_60.rule(C::FormDivider).unwrap().attribs,
        Binding::Optional
    );
    // The other 21 types of 1.26.45 keep their traits.
    for component in Component::ALL {
        if Catalogue::V1_26_45.has(component) && component != C::FormDivider {
            assert_eq!(
                Catalogue::V1_26_60.rule(component),
                Catalogue::V1_26_45.rule(component),
                "{component}"
            );
        }
    }
}

#[test]
fn the_1_26_20_catalogue_has_19_control_types() {
    let known: Vec<&str> = Component::ALL
        .into_iter()
        .filter(|c| Catalogue::V1_26_20.has(*c))
        .map(Component::name)
        .collect();
    assert_eq!(known.len(), 19);
    for absent in [
        "Container.Slot",
        "Container.TextField",
        "Form.Image",
        "ScrollableGridLayout",
    ] {
        assert!(!known.contains(&absent), "{absent}");
    }
    assert!(known.contains(&"Container.Item"));
}

#[test]
fn the_1_26_45_catalogue_has_22_control_types_and_no_container_item() {
    let known: Vec<&str> = Component::ALL
        .into_iter()
        .filter(|c| Catalogue::V1_26_45.has(*c))
        .map(Component::name)
        .collect();
    assert_eq!(known.len(), 22);
    assert!(!known.contains(&"Container.Item"));
    for added in [
        "Container.Slot",
        "Container.TextField",
        "Form.Image",
        "ScrollableGridLayout",
    ] {
        assert!(known.contains(&added), "{added}");
    }
}

/// `attribs` and `children` of each type, row by row.
#[test]
fn the_member_traits_of_1_26_20() {
    use Binding::{Optional, Required, Unbound};
    use ChildSet::{GridContent, ListContent, PanelContent, RootControl};
    let rows = [
        (C::ExtensionPoint, Optional, None),
        (C::Panel, Optional, Some(PanelContent)),
        (C::PanelText, Required, None),
        (C::PanelDecoration, Unbound, None),
        (C::PanelSpacing, Unbound, None),
        (C::PanelCloseButton, Optional, None),
        (C::Context, Optional, Some(RootControl)),
        (C::ContextList, Optional, Some(ListContent)),
        (C::ContainerFixedGridLayout, Optional, Some(GridContent)),
        (C::ContainerItem, Optional, None),
        (C::ContainerLayout, Unbound, Some(RootControl)),
        (C::FormButton, Optional, None),
        (C::FormDivider, Unbound, None),
        (C::FormDropdown, Optional, None),
        (C::FormScrollView, Optional, Some(PanelContent)),
        (C::FormSlider, Optional, None),
        (C::FormSwitch, Optional, None),
        (C::FormTextField, Optional, None),
        (C::Visibility, Optional, Some(PanelContent)),
    ];
    assert_eq!(rows.len(), 19);
    for (component, attribs, children) in rows {
        let rule = Catalogue::V1_26_20
            .rule(component)
            .unwrap_or_else(|| panic!("{component}"));
        assert_eq!(rule.attribs, attribs, "{component} attribs");
        assert_eq!(rule.children, children, "{component} children");
        // The 18 that stay keep their traits in 1.26.45.
        if component != C::ContainerItem {
            assert_eq!(
                Catalogue::V1_26_45.rule(component),
                Some(rule),
                "{component}"
            );
        }
    }
    assert_eq!(Catalogue::V1_26_45.rule(C::ContainerItem), None);
}

#[test]
fn the_member_traits_of_the_four_types_added_by_1_26_45() {
    let rows = [
        (C::ContainerSlot, Binding::Optional, None),
        (C::ContainerTextField, Binding::Optional, None),
        (C::FormImage, Binding::Optional, None),
        (
            C::ScrollableGridLayout,
            Binding::Optional,
            Some(ChildSet::ListContent),
        ),
    ];
    for (component, attribs, children) in rows {
        let rule = Catalogue::V1_26_45
            .rule(component)
            .unwrap_or_else(|| panic!("{component}"));
        assert_eq!(rule.attribs, attribs, "{component} attribs");
        assert_eq!(rule.children, children, "{component} children");
        assert_eq!(Catalogue::V1_26_20.rule(component), None, "{component}");
    }
}

#[test]
fn the_child_sets_of_1_26_20() {
    let c = Catalogue::V1_26_20;
    assert_eq!(
        names(c, ChildSet::RootControl),
        sorted(vec!["Container.Layout", "Context", "Panel"])
    );
    let panel = sorted(vec![
        "Container.FixedGridLayout",
        "ExtensionPoint",
        "Form.Button",
        "Form.Divider",
        "Form.Dropdown",
        "Form.ScrollView",
        "Form.Slider",
        "Form.Switch",
        "Form.TextField",
        "Panel.CloseButton",
        "Panel.Decoration",
        "Panel.Spacing",
        "Context.List",
        "Panel.Text",
        "Visibility",
    ]);
    assert_eq!(panel.len(), 15);
    assert_eq!(names(c, ChildSet::PanelContent), panel);
    let list = sorted(vec![
        "Container.Item",
        "Container.FixedGridLayout",
        "Form.Button",
        "Form.Divider",
        "Form.Dropdown",
        "Form.ScrollView",
        "Form.Slider",
        "Form.Switch",
        "Form.TextField",
        "Panel.Decoration",
        "Panel.Spacing",
        "Context.List",
        "Panel.Text",
        "Visibility",
    ]);
    assert_eq!(list.len(), 14);
    assert_eq!(names(c, ChildSet::ListContent), list);
    assert_eq!(
        names(c, ChildSet::GridContent),
        sorted(vec!["Container.Item", "Context.List"])
    );
}

#[test]
fn the_child_sets_of_1_26_45() {
    let c = Catalogue::V1_26_45;
    assert_eq!(
        names(c, ChildSet::RootControl),
        sorted(vec!["Container.Layout", "Context", "Panel"])
    );
    let panel = sorted(vec![
        "Container.FixedGridLayout",
        "Container.TextField",
        "ExtensionPoint",
        "Form.Button",
        "Form.Divider",
        "Form.Dropdown",
        "Form.Image",
        "Form.ScrollView",
        "Form.Slider",
        "Form.Switch",
        "Form.TextField",
        "Panel.CloseButton",
        "Panel.Decoration",
        "Panel.Spacing",
        "Context.List",
        "ScrollableGridLayout",
        "Panel.Text",
        "Visibility",
    ]);
    assert_eq!(panel.len(), 18);
    assert_eq!(names(c, ChildSet::PanelContent), panel);
    let list = sorted(vec![
        "Container.Slot",
        "Container.FixedGridLayout",
        "Container.TextField",
        "Form.Button",
        "Form.Divider",
        "Form.Dropdown",
        "Form.Image",
        "Form.ScrollView",
        "Form.Slider",
        "Form.Switch",
        "Form.TextField",
        "Panel.Decoration",
        "Panel.Spacing",
        "Context.List",
        "ScrollableGridLayout",
        "Panel.Text",
        "Visibility",
    ]);
    assert_eq!(list.len(), 17);
    assert_eq!(names(c, ChildSet::ListContent), list);
    assert_eq!(
        names(c, ChildSet::GridContent),
        sorted(vec!["Container.Slot", "Context.List"])
    );
}

/// Child sets: `RootControl` gains `Form.Panel`; `PanelContent` the four
/// `Form.CloseButton`, `Form.Spacing`, `Form.Text`, `Form.HorizontalStack`; `ListContent` those
/// four and `ExtensionPoint`; `GridContent` is unchanged.
#[test]
fn the_child_sets_of_1_26_60() {
    let (c, before) = (Catalogue::V1_26_60, Catalogue::V1_26_45);
    let with = |base: Vec<&'static str>, added: &[&'static str]| {
        let mut all = base;
        all.extend_from_slice(added);
        sorted(all)
    };
    assert_eq!(
        names(c, ChildSet::RootControl),
        sorted(vec!["Container.Layout", "Context", "Form.Panel", "Panel"])
    );
    let four = [
        "Form.CloseButton",
        "Form.HorizontalStack",
        "Form.Spacing",
        "Form.Text",
    ];
    let panel = with(names(before, ChildSet::PanelContent), &four);
    assert_eq!(panel.len(), 22);
    assert!(!panel.contains(&"Form.Panel"));
    assert_eq!(names(c, ChildSet::PanelContent), panel);
    let list = with(
        names(before, ChildSet::ListContent),
        &[four.as_slice(), &["ExtensionPoint"]].concat(),
    );
    assert_eq!(list.len(), 22);
    assert!(!list.contains(&"Panel.CloseButton"));
    assert_eq!(names(c, ChildSet::ListContent), list);
    assert_eq!(
        names(c, ChildSet::GridContent),
        sorted(vec!["Container.Slot", "Context.List"])
    );
}

/// `allows` and `members` are one set read two ways.
#[test]
fn allows_agrees_with_members_for_every_set_and_type() {
    for catalogue in Catalogue::ALL {
        for set in [
            ChildSet::RootControl,
            ChildSet::PanelContent,
            ChildSet::ListContent,
            ChildSet::GridContent,
        ] {
            let members: Vec<Component> = catalogue.members(set).collect();
            for component in Component::ALL {
                assert_eq!(
                    catalogue.allows(set, component),
                    members.contains(&component),
                    "{catalogue:?} {set:?} {component}"
                );
                // No set names a type its catalogue lacks.
                assert!(!catalogue.allows(set, component) || catalogue.has(component));
            }
        }
    }
}

//! A read's Molang leaf (`Read::molang_leaf`): a family whose reader takes Molang its own way
//! (particle's legacy parser and its 1.20.80 reader) reads every `Molang` and `[Molang; N]`
//! member of its derived types through its own functions.

use bedrock_addon_core::diagnostic::{Diagnostic, Diagnostics, JsonPath, codes};
use bedrock_addon_core::json::kit::{Node, Profile, Read};
use bedrock_addon_core::json::{JsonSlice, parse};
use bedrock_addon_core::molang::{Molang, MolangSource, RawVersion};
use bedrock_addon_core::schema::{self, MolangLeaf, Schema};
use bedrock_addon_core::versioning::Reading;
use facet::Facet;

/// A string at version 7; anything else no Molang (told, `None`).
fn one(node: &Node<'_>, read: &mut Read<'_>) -> Option<Molang> {
    let Some(text) = node.value().as_string() else {
        read.wrong_type(node);
        return None;
    };
    Some(Molang::Expr(MolangSource::string(
        text.as_str(),
        RawVersion(7),
    )))
}

/// An array of the length, each element by [`one`]; another length is the defaults, with a
/// line at the array.
fn array(node: &Node<'_>, read: &mut Read<'_>, len: usize) -> Option<Vec<Molang>> {
    let elements = node
        .value()
        .as_array()
        .filter(|elements| elements.len() == len);
    let Some(elements) = elements else {
        read.push(Diagnostic::of(&codes::INVALID_VALUE).at(node.path().clone()));
        return Some(vec![Molang::Const(0.0); len]);
    };
    (0..len)
        .map(|index| {
            let mut path = node.path().clone();
            path.push_index(index);
            one(&Node::new(&elements[index], path), read)
        })
        .collect()
}

const LEAF: MolangLeaf = MolangLeaf { one, array };

#[derive(Clone, Debug, PartialEq, Facet, Schema)]
struct Emitter {
    rate: Option<Molang>,
    offset: Option<[Molang; 2]>,
}

fn read(text: &str, leaf: Option<MolangLeaf>) -> (Option<Emitter>, Vec<String>) {
    let parsed = parse::parse_slice(JsonSlice::new(text), parse::Options::default()).unwrap();
    let node = Node::parsed(&parsed, JsonPath::new());
    let mut found = Diagnostics::default();
    let mut read = Read::new(Reading::DOCUMENT, &mut found).profile(Profile::DECLARED_ORDER);
    if let Some(leaf) = leaf {
        read = read.molang_leaf(leaf);
    }
    let value = schema::read_document::<Emitter>(&node, &mut read);
    let told = found
        .iter()
        .map(|f| format!("{:?} {} {}", f.severity, f.location.json_path, f.code.id()))
        .collect();
    (value, told)
}

#[test]
fn a_read_s_molang_leaf_reads_every_molang_member_and_array() {
    let text = r#"{"rate": "q.x", "offset": [1, 2, 3]}"#;
    let (emitter, told) = read(text, Some(LEAF));
    assert_eq!(
        emitter,
        Some(Emitter {
            rate: Some(Molang::Expr(MolangSource::string("q.x", RawVersion(7)))),
            offset: Some([Molang::Const(0.0), Molang::Const(0.0)]),
        })
    );
    assert_eq!(told, ["Error offset core/invalid_value"]);
    // The leaf's own refusal of a number, where core's leaf takes it.
    let (emitter, told) = read(r#"{"rate": 1}"#, Some(LEAF));
    assert_eq!(emitter, None);
    assert_eq!(told, ["Error rate core/invalid_type"]);
    // Without a leaf, core's.
    let (emitter, _) = read(r#"{"rate": 1, "offset": [1, 2]}"#, None);
    assert_eq!(
        emitter,
        Some(Emitter {
            rate: Some(Molang::Const(1.0)),
            offset: Some([Molang::Const(1.0), Molang::Const(2.0)]),
        })
    );
}

/// A fixed array of Molang is a field per element in the walk, at its index.
#[test]
fn a_fixed_array_of_molang_is_walked_element_by_element() {
    let mut emitter = Emitter {
        rate: None,
        offset: Some([Molang::Const(1.0), Molang::Const(2.0)]),
    };
    let mut paths = Vec::new();
    bedrock_addon_core::molang::visit_fields(&mut emitter, &JsonPath::new(), |field| {
        paths.push(field.path.to_string());
    });
    assert_eq!(paths, ["offset/0", "offset/1"]);
}

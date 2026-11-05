//! The `#[component(...)]` attributes of `#[derive(Components)]`; the reference (keys, values,
//! defaults) is the derive's rustdoc in `lib.rs`.

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use syn::meta::ParseNestedMeta;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, Error, Ident, LitStr, Path, Result, Token, Type, braced, bracketed};

/// The keys a component row accepts, in the order the docs list them.
const ROW_KEYS: &str = "`name`, `released`, `deprecated_since`, `deprecated_message`, \
    `supersedes`, `requires`, `area`, `fields`, `parse`, `molang`, `typed`, `shorthand`, `goal`, \
    `triggers`";

/// The keys the enum accepts.
const ENUM_KEYS: &str = "`family`, `custom`, `unknown`, `dynamic`, `from_only`, `write`, \
    `write_with`, `versioned`, `typed_table`, `triggers`";

/// The attributes on the enum.
#[derive(Default)]
pub(crate) struct EnumAttrs {
    /// `family = Type`.
    pub family: Option<Type>,
    /// `custom = Variant`.
    pub custom: Option<Ident>,
    /// `unknown = Variant`.
    pub unknown: Option<Ident>,
    /// `dynamic = Variant`.
    pub dynamic: Option<Ident>,
    /// `from_only`: emit `From` impls and nothing else (an older version's enum); the flag's
    /// span, for errors.
    pub from_only: Option<Span>,
    /// `write`: emit the write dispatch over `write_payload`; the flag's span.
    pub write: Option<Span>,
    /// `write_with = path`: emit the write dispatch over `path` instead.
    pub write_with: Option<Path>,
    /// `versioned = path`: the generic parse of a goal by the document's version; each `goal`
    /// row puts `path::<T>` in the enum's `VERSIONED` table.
    pub versioned: Option<Path>,
    /// `typed_table = path`: `fn(&str) -> Option<Option<FormatVersion>>`, the generated game
    /// table's typed version of a key; a test the derive emits holds every row's `typed` to it.
    pub typed_table: Option<Path>,
    /// `triggers`: emit `triggers()`, the trigger members of every row; the flag's span.
    pub triggers: Option<Span>,
}

impl EnumAttrs {
    /// Reads every `#[component(...)]` on the enum. What could be read is kept; an attribute
    /// that fails adds its first error to `errors`.
    pub fn parse(attrs: &[Attribute], errors: &mut Vec<Error>) -> Self {
        let mut out = EnumAttrs::default();
        for attr in component_attrs(attrs) {
            let result = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("family") {
                    set_once(&meta, &mut out.family, |m| m.value()?.parse())
                } else if meta.path.is_ident("custom") {
                    set_once(&meta, &mut out.custom, |m| m.value()?.parse())
                } else if meta.path.is_ident("unknown") {
                    set_once(&meta, &mut out.unknown, |m| m.value()?.parse())
                } else if meta.path.is_ident("dynamic") {
                    set_once(&meta, &mut out.dynamic, |m| m.value()?.parse())
                } else if meta.path.is_ident("from_only") {
                    set_once(&meta, &mut out.from_only, |m| Ok(m.path.span()))
                } else if meta.path.is_ident("write") {
                    set_once(&meta, &mut out.write, |m| Ok(m.path.span()))
                } else if meta.path.is_ident("write_with") {
                    set_once(&meta, &mut out.write_with, |m| m.value()?.parse())
                } else if meta.path.is_ident("versioned") {
                    set_once(&meta, &mut out.versioned, |m| m.value()?.parse())
                } else if meta.path.is_ident("typed_table") {
                    set_once(&meta, &mut out.typed_table, |m| m.value()?.parse())
                } else if meta.path.is_ident("triggers") {
                    set_once(&meta, &mut out.triggers, |m| Ok(m.path.span()))
                } else {
                    Err(unsupported(&meta, "enum attribute", ENUM_KEYS))
                }
            });
            if let Err(error) = result {
                errors.push(error);
            }
        }
        out
    }
}

/// `major.minor.patch`, checked at expansion time.
#[derive(Clone, Copy)]
pub(crate) struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    /// Parses `"1.21.60"`; the error points at the literal.
    pub(crate) fn parse(lit: &LitStr) -> Result<Self> {
        let text = lit.value();
        // Digits only: `u16::from_str` alone would also take a leading `+`.
        let parts: Vec<Option<u16>> = text
            .split('.')
            .map(|part| {
                let digits = !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
                digits.then(|| part.parse().ok()).flatten()
            })
            .collect();
        match parts.as_slice() {
            [Some(major), Some(minor), Some(patch)] => Ok(Version {
                major: *major,
                minor: *minor,
                patch: *patch,
            }),
            _ => Err(Error::new(
                lit.span(),
                format!(
                    "`{text}` is not a version: expected `major.minor.patch`, e.g. \"1.21.60\""
                ),
            )),
        }
    }
}

/// One `FieldMeta`: `{ name = "a.b", since_engine = "1.21.0" }`.
pub(crate) struct Field {
    pub name: LitStr,
    pub since_engine: Version,
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let brace = braced!(content in input);
        let mut name: Option<LitStr> = None;
        let mut since_engine: Option<Version> = None;
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![=]>()?;
            let value: LitStr = content.parse()?;
            let slot_taken = if key == "name" {
                name.replace(value).is_some()
            } else if key == "since_engine" {
                since_engine.replace(Version::parse(&value)?).is_some()
            } else {
                return Err(Error::new(
                    key.span(),
                    format!("unsupported field key `{key}`; expected `name` or `since_engine`"),
                ));
            };
            if slot_taken {
                return Err(Error::new(key.span(), format!("duplicate `{key}`")));
            }
            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }
        match (name, since_engine) {
            (Some(name), Some(since_engine)) => Ok(Field { name, since_engine }),
            (None, _) => Err(Error::new(brace.span.join(), "missing `name` in the field")),
            (_, None) => Err(Error::new(
                brace.span.join(),
                "missing `since_engine` in the field",
            )),
        }
    }
}

/// One table row: the `ComponentMeta` of a typed variant or of a dynamic key.
pub(crate) struct Row {
    pub name: LitStr,
    pub released: Option<Version>,
    pub deprecated_since: Option<Version>,
    pub deprecated_message: Option<LitStr>,
    pub supersedes: Option<LitStr>,
    /// The tokens inside `requires = [...]`.
    pub requires: TokenStream,
    pub area: Ident,
    pub fields: Vec<Field>,
    pub parse: Path,
    /// `molang`: the component type holds Molang fields, and the enum's `MolangFields` walk
    /// forwards to it.
    pub molang: bool,
    /// `typed = "1.26.30"`: the format version from which the game reads it with its typed
    /// struct.
    pub typed: Option<Version>,
    /// `shorthand = scalar`.
    pub shorthand: bool,
    /// `goal`: the row is an AI goal; the flag's span, for errors.
    pub goal: Option<Span>,
    /// `triggers = ["member", ..]`: trigger members the payload type does not list itself.
    pub triggers: Vec<LitStr>,
}

/// A row while its attributes are read; every key at most once.
#[derive(Default)]
struct RowParts {
    name: Option<LitStr>,
    released: Option<Version>,
    deprecated_since: Option<Version>,
    deprecated_message: Option<LitStr>,
    supersedes: Option<LitStr>,
    requires: Option<TokenStream>,
    area: Option<Ident>,
    fields: Option<Vec<Field>>,
    parse: Option<Path>,
    molang: Option<Span>,
    typed: Option<Version>,
    shorthand: Option<Span>,
    goal: Option<Span>,
    triggers: Option<Vec<LitStr>>,
}

impl RowParts {
    /// Reads one `#[component(...)]` into the parts.
    fn read(&mut self, attr: &Attribute) -> Result<()> {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                set_once(&meta, &mut self.name, |m| m.value()?.parse())
            } else if meta.path.is_ident("released") {
                set_once(&meta, &mut self.released, version_value)
            } else if meta.path.is_ident("deprecated_since") {
                set_once(&meta, &mut self.deprecated_since, version_value)
            } else if meta.path.is_ident("deprecated_message") {
                set_once(&meta, &mut self.deprecated_message, |m| m.value()?.parse())
            } else if meta.path.is_ident("supersedes") {
                set_once(&meta, &mut self.supersedes, |m| m.value()?.parse())
            } else if meta.path.is_ident("requires") {
                set_once(&meta, &mut self.requires, |m| {
                    let content;
                    let input = m.value()?;
                    bracketed!(content in input);
                    content.parse()
                })
            } else if meta.path.is_ident("area") {
                set_once(&meta, &mut self.area, |m| {
                    let area: Ident = m.value()?.parse()?;
                    if ["Public", "Vanilla", "Internal"].iter().any(|a| area == a) {
                        Ok(area)
                    } else {
                        Err(Error::new(
                            area.span(),
                            format!(
                                "unknown area `{area}`; expected `Public`, `Vanilla` or `Internal`"
                            ),
                        ))
                    }
                })
            } else if meta.path.is_ident("fields") {
                set_once(&meta, &mut self.fields, |m| {
                    let content;
                    let input = m.value()?;
                    bracketed!(content in input);
                    let fields = Punctuated::<Field, Token![,]>::parse_terminated(&content)?;
                    let mut seen = HashSet::new();
                    for field in &fields {
                        if !seen.insert(field.name.value()) {
                            return Err(Error::new(
                                field.name.span(),
                                format!("duplicate field `{}`", field.name.value()),
                            ));
                        }
                    }
                    Ok(fields.into_iter().collect())
                })
            } else if meta.path.is_ident("parse") {
                set_once(&meta, &mut self.parse, |m| m.value()?.parse())
            } else if meta.path.is_ident("molang") {
                set_once(&meta, &mut self.molang, |m| Ok(m.path.span()))
            } else if meta.path.is_ident("typed") {
                set_once(&meta, &mut self.typed, version_value)
            } else if meta.path.is_ident("shorthand") {
                set_once(&meta, &mut self.shorthand, |m| {
                    let shape: Ident = m.value()?.parse()?;
                    if shape == "scalar" {
                        Ok(shape.span())
                    } else {
                        Err(Error::new(
                            shape.span(),
                            format!("unknown shorthand `{shape}`; expected `scalar`"),
                        ))
                    }
                })
            } else if meta.path.is_ident("goal") {
                set_once(&meta, &mut self.goal, |m| Ok(m.path.span()))
            } else if meta.path.is_ident("triggers") {
                set_once(&meta, &mut self.triggers, |m| {
                    let content;
                    let input = m.value()?;
                    bracketed!(content in input);
                    let members = Punctuated::<LitStr, Token![,]>::parse_terminated(&content)?;
                    Ok(members.into_iter().collect())
                })
            } else {
                Err(unsupported(&meta, "component attribute", ROW_KEYS))
            }
        })
    }

    /// The finished row; `name` and `parse` are required. Errors point at `attr`.
    fn finish(self, attr: &Attribute) -> Result<Row> {
        let Some(name) = self.name else {
            return Err(Error::new_spanned(
                attr,
                "missing `name`: the component's JSON key, `name = \"minecraft:…\"`",
            ));
        };
        let Some(parse) = self.parse else {
            return Err(Error::new_spanned(
                attr,
                "missing `parse`: the component's parse function, `parse = path::to::fn`",
            ));
        };
        Ok(Row {
            name,
            released: self.released,
            deprecated_since: self.deprecated_since,
            deprecated_message: self.deprecated_message,
            supersedes: self.supersedes,
            requires: self.requires.unwrap_or_default(),
            area: self
                .area
                .unwrap_or_else(|| Ident::new("Public", proc_macro2::Span::call_site())),
            fields: self.fields.unwrap_or_default(),
            parse,
            molang: self.molang.is_some(),
            typed: self.typed,
            shorthand: self.shorthand.is_some(),
            goal: self.goal,
            triggers: self.triggers.unwrap_or_default(),
        })
    }
}

/// The row of a typed variant: every `#[component]` on it, merged. `None` when it has none.
pub(crate) fn typed_row(attrs: &[Attribute]) -> Result<Option<Row>> {
    let attrs: Vec<&Attribute> = component_attrs(attrs).collect();
    let Some(first) = attrs.first() else {
        return Ok(None);
    };
    let mut parts = RowParts::default();
    for attr in &attrs {
        parts.read(attr)?;
    }
    parts.finish(first).map(Some)
}

/// The rows of the dynamic variant: one per `#[component]` on it. A dynamic key holds JSON,
/// so `molang` is an error there.
pub(crate) fn dynamic_rows(attrs: &[Attribute]) -> Result<Vec<Row>> {
    component_attrs(attrs)
        .map(|attr| {
            let mut parts = RowParts::default();
            parts.read(attr)?;
            if let Some(span) = parts.molang {
                return Err(Error::new(
                    span,
                    "a dynamic key holds JSON, not Molang fields: `molang` applies to typed \
                     variants",
                ));
            }
            if let Some(span) = parts.goal {
                return Err(Error::new(
                    span,
                    "a dynamic key holds JSON, not a goal struct: `goal` applies to typed variants",
                ));
            }
            if let Some(span) = parts.shorthand {
                return Err(Error::new(
                    span,
                    "a dynamic key holds JSON as written: `shorthand` applies to typed variants",
                ));
            }
            parts.finish(attr)
        })
        .collect()
}

/// Whether `attr` lists the key `key`: an identifier at the top level of its list, first or
/// after a comma.
pub(crate) fn names_key(attr: &Attribute, key: &str) -> bool {
    let Ok(list) = attr.meta.require_list() else {
        return false;
    };
    let mut at_key = true;
    for token in list.tokens.clone() {
        match token {
            proc_macro2::TokenTree::Ident(ident) if at_key && ident == key => return true,
            proc_macro2::TokenTree::Punct(punct) if punct.as_char() == ',' => at_key = true,
            _ => at_key = false,
        }
    }
    false
}

/// The `#[component(...)]` attributes among `attrs`.
pub(crate) fn component_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("component"))
}

/// Parses `meta`'s value into `slot`, which must still be empty.
fn set_once<T>(
    meta: &ParseNestedMeta,
    slot: &mut Option<T>,
    parse: impl FnOnce(&ParseNestedMeta) -> Result<T>,
) -> Result<()> {
    if slot.is_some() {
        return Err(meta.error(format!("duplicate `{}`", path_text(&meta.path))));
    }
    *slot = Some(parse(meta)?);
    Ok(())
}

/// A `"major.minor.patch"` value.
fn version_value(meta: &ParseNestedMeta) -> Result<Version> {
    let lit: LitStr = meta.value()?.parse()?;
    Version::parse(&lit)
}

/// The error for a key the attribute does not know.
fn unsupported(meta: &ParseNestedMeta, what: &str, expected: &str) -> Error {
    meta.error(format!(
        "unsupported {what} `{}`; expected one of {expected}",
        path_text(&meta.path)
    ))
}

/// A path as written, for messages.
fn path_text(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

//! `ItemDescriptor`: one item, or a set of items by tag.

use facet_value::VObject;
use molang::json::MolangSource;

use super::read::{compact, molang_source, molang_value};
use super::{finding, unrecognized_except};
use crate::diagnostic::{DiagnosticCode, Diagnostics, JsonPath, Severity, VanillaText};
use crate::document::Extra;
use crate::id::ItemId;
use crate::json::Value;

/// An item, or a set of items by tag, as the game's item descriptor.
///
/// JSON: `"ns:name"`, `{"item": "ns:name"}`, `{"name": "ns:name"}` (the form vanilla's cereal
/// writer produces), `{"tags": "<molang>"}` or `{"item_tag": "ns:tag"}`. Any other shape is a
/// content-log error in vanilla, which then builds an empty descriptor that matches nothing; it
/// is kept here as [`ItemDescriptor::Invalid`] so writing reproduces it, and
/// [`ItemDescriptor::check`] reports it. An object with several of the four keys takes the first
/// of `item`, `name`, `tags`, `item_tag` and keeps the others in `extra`.
#[derive(Clone, Debug, PartialEq, facet::Facet)]
#[facet(opaque, proxy = Value)]
#[repr(u8)]
#[non_exhaustive]
#[doc(alias = "item_descriptor")]
pub enum ItemDescriptor {
    /// One item: `"ns:name"`, `{"item": …}` or `{"name": …}`, as `form` says. The identifier
    /// is kept as written, a legacy `ns:name:aux` included.
    Item {
        /// The item.
        id: ItemId,
        /// The JSON form read.
        form: ItemDescriptorForm,
        /// Unknown members of the object form.
        extra: Extra,
    },
    /// `{"tags": "<molang>"}`: an expression over `query.any_tag` / `query.all_tags`, evaluated
    /// against the item. Its Molang version is the using member's.
    Tags {
        /// The expression (JSON `tags`).
        tags: MolangSource,
        /// Unknown members.
        extra: Extra,
    },
    /// `{"item_tag": "ns:tag"}`: every item that carries the tag.
    ItemTag {
        /// The tag (JSON `item_tag`).
        item_tag: String,
        /// Unknown members.
        extra: Extra,
    },
    /// A shape vanilla rejects (it builds an empty descriptor that matches nothing), as written.
    Invalid(Value),
}

/// The JSON form an [`ItemDescriptor::Item`] was read from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ItemDescriptorForm {
    /// `"ns:name"`.
    #[default]
    String,
    /// `{"item": "ns:name"}`.
    Item,
    /// `{"name": "ns:name"}`.
    Name,
}

/// The keys that select a descriptor arm, in the order the reader tries them. The game's own order
/// among several keys is not known, so the crate fixes this one.
const KEYS: [&str; 4] = ["item", "name", "tags", "item_tag"];

impl ItemDescriptor {
    /// One item, written as a string.
    pub fn item(id: ItemId) -> Self {
        Self::Item {
            id,
            form: ItemDescriptorForm::String,
            extra: Extra::default(),
        }
    }

    /// The items a tag expression matches.
    pub fn tags(tags: MolangSource) -> Self {
        Self::Tags {
            tags,
            extra: Extra::default(),
        }
    }

    /// The items that carry `tag`.
    pub fn item_tag(tag: impl Into<String>) -> Self {
        Self::ItemTag {
            item_tag: tag.into(),
            extra: Extra::default(),
        }
    }

    /// The findings vanilla's reader reports for this descriptor, located under `at`:
    /// - a wrong shape: `InvalidType` kept error with vanilla's text;
    /// - an object with several of `item`, `name`, `tags`, `item_tag`: `InvalidValue`
    ///   information (the crate's own: the game's constructor tries `item`, then `tags`,
    ///   then `item_tag`, and logs nothing);
    /// - every other member kept in `extra`: `UnrecognizedField` information at its own path
    ///   (the game's constructor reads a JSON value and ignores the rest without a
    ///   line).
    ///
    /// A wrong shape is a kept error: the game logs `"ItemDescriptor requires a string"` at error
    /// level and builds an empty descriptor, so the document loads on.
    pub fn check(&self, at: &JsonPath) -> Diagnostics {
        let mut out = Diagnostics::default();
        let extra = match self {
            // "ItemDescriptor requires a string, …": vanilla logs it at level 3 and builds an
            // empty descriptor, a kept error.
            Self::Invalid(value) => {
                out.push_kept_error(finding(
                    DiagnosticCode::InvalidType {
                        text: Some(VanillaText::ItemDescriptorShape {
                            value: compact(value),
                        }),
                    },
                    Severity::Error,
                    at,
                ));
                return out;
            }
            Self::Item { extra, .. } | Self::Tags { extra, .. } | Self::ItemTag { extra, .. } => {
                extra
            }
        };
        if KEYS.iter().any(|key| extra.get(key).is_some()) {
            // The first key in `KEYS` order wins, with an information.
            out.push(finding(
                DiagnosticCode::InvalidValue { text: None },
                Severity::Info,
                at,
            ));
        }
        unrecognized_except(extra, at, &KEYS, Severity::Info, &mut out);
        out
    }

    /// Reads the object form; `None` for a shape vanilla rejects.
    fn read_object(object: &VObject) -> Option<Self> {
        let key = KEYS.iter().find(|key| object.get(key).is_some())?;
        let member = object.get(key)?;
        let mut extra = Extra::default();
        for (name, value) in object {
            if name.as_str() != *key {
                extra.insert(name.as_str().to_owned(), value.clone());
            }
        }
        let item = |form| {
            let id = member.as_string()?;
            Some(Self::Item {
                id: ItemId::parse(id.as_str()),
                form,
                extra: extra.clone(),
            })
        };
        match *key {
            "item" => item(ItemDescriptorForm::Item),
            "name" => item(ItemDescriptorForm::Name),
            "tags" => Some(Self::Tags {
                tags: molang_source(member)?,
                extra,
            }),
            _ => Some(Self::ItemTag {
                item_tag: member.as_string()?.as_str().to_owned(),
                extra,
            }),
        }
    }
}

impl From<Value> for ItemDescriptor {
    fn from(value: Value) -> Self {
        let read = if let Some(id) = value.as_string() {
            Some(Self::Item {
                id: ItemId::parse(id.as_str()),
                form: ItemDescriptorForm::String,
                extra: Extra::default(),
            })
        } else {
            value.as_object().and_then(Self::read_object)
        };
        read.unwrap_or(Self::Invalid(value))
    }
}

impl From<&ItemDescriptor> for Value {
    fn from(descriptor: &ItemDescriptor) -> Self {
        let (key, member, extra) = match descriptor {
            ItemDescriptor::Invalid(value) => return value.clone(),
            ItemDescriptor::Item {
                id,
                form: ItemDescriptorForm::String,
                extra,
            } if extra.is_empty() => return Value::from(id.as_str()),
            ItemDescriptor::Item { id, form, extra } => {
                let key = if *form == ItemDescriptorForm::Name {
                    "name"
                } else {
                    "item"
                };
                (key, Value::from(id.as_str()), extra)
            }
            ItemDescriptor::Tags { tags, extra } => ("tags", molang_value(tags), extra),
            ItemDescriptor::ItemTag { item_tag, extra } => {
                ("item_tag", Value::from(item_tag.as_str()), extra)
            }
        };
        let mut object = VObject::new();
        object.insert(key, member);
        for (name, value) in extra.iter() {
            object.insert(name, value.clone());
        }
        Value::from(object)
    }
}

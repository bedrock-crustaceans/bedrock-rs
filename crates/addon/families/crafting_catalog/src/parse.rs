//! Reading a catalogue payload. Every level is read from its own slice of the text, with the
//! JSON type checked on the token: vanilla's cereal reader fails the file on a wrong type, a
//! missing required member, an unknown category or a constraint, and tells every unknown member
//! as an `UnrecognizedField` warning, which the strict 1.26.50 slice turns into the error that
//! skips the file (`Family::error_mask`).

use facet_json::RawJson;
use indexmap::IndexMap;

use super::{CatalogItem, Category, CategoryName, CraftingCatalog, Group, GroupIdentifier};
use bedrock_addon_core::diagnostic::{
    Diagnostic, DiagnosticCode, Diagnostics, JsonPath, Span, VanillaText,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::ItemId;
use bedrock_addon_core::json::{JsonSlice, parse_at};
use bedrock_addon_core::versioning::ParseCx;

/// The JSON type a value's first token gives it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Token {
    Object,
    Array,
    String,
    Other,
}

impl Token {
    /// The type of the value `text` holds, after whitespace and comments.
    fn of(text: &str) -> Token {
        let mut rest = text;
        loop {
            rest = rest.trim_start();
            if let Some(after) = rest.strip_prefix("//") {
                rest = after.split_once('\n').map_or("", |(_, tail)| tail);
            } else if let Some(after) = rest.strip_prefix("/*") {
                rest = after.split_once("*/").map_or("", |(_, tail)| tail);
            } else {
                break;
            }
        }
        match rest.as_bytes().first() {
            Some(b'{') => Token::Object,
            Some(b'[') => Token::Array,
            Some(b'"') => Token::String,
            _ => Token::Other,
        }
    }

    /// The type as a message names it.
    fn name(self) -> &'static str {
        match self {
            Token::Object => "an object",
            Token::Array => "an array",
            Token::String => "a string",
            Token::Other => "a number, a boolean or null",
        }
    }
}

/// Vanilla's text for a group name that fails its namespace constraint.
const GROUP_NAME_TEXT: &str = "string must be prefixed with a namespace (eg. namespace:value)";

/// The pattern an item name must match; a mismatch reads "expected string to match the
/// \"<pattern>\" regex".
const ITEM_NAME_REGEX: &str = "^(?:.)+:(?:.)+$";

/// The code of a group name that fails its namespace constraint.
fn group_name_code() -> DiagnosticCode {
    DiagnosticCode::InvalidValue {
        text: Some(VanillaText::Fixed(GROUP_NAME_TEXT)),
    }
}

/// The code of an item name that does not match its pattern.
fn item_name_code() -> DiagnosticCode {
    bedrock_addon_core::diagnostic::cereal::string_mismatch(ITEM_NAME_REGEX)
}

/// Whether `text` matches `^(?:.)+:(?:.)+$`: a `:` with at least one character on each side,
/// and no line terminator anywhere (`.` matches `:` too, so `::x` and `a:b:c:d` pass). The
/// pattern has no letter, so the group name's `icase` flag and the item name's case-sensitive
/// match accept the same strings.
fn is_namespaced(text: &str) -> bool {
    let bytes = text.as_bytes();
    !bytes.iter().any(|b| matches!(b, b'\n' | b'\r'))
        && bytes
            .iter()
            .enumerate()
            .any(|(index, b)| *b == b':' && index > 0 && index + 1 < bytes.len())
}

/// The whole of `slice` as a span of the file.
fn span(slice: JsonSlice<'_>) -> Span {
    let to_u32 = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    Span {
        offset: to_u32(slice.offset),
        len: to_u32(slice.text.len()),
    }
}

/// The members of an object level, by key, each a slice of the file.
type Members<'a> = IndexMap<String, JsonSlice<'a>>;

/// One payload being read: where findings go.
struct Reader<'d> {
    d: &'d mut Diagnostics,
}

impl Reader<'_> {
    /// An error-level finding at `at`.
    fn error(&mut self, code: DiagnosticCode, at: JsonPath, slice: Option<JsonSlice<'_>>) {
        let finding = Diagnostic::error(code).at(at);
        self.d.push(match slice {
            Some(slice) => finding.span(span(slice)),
            None => finding,
        });
    }

    /// Requires `slice` to hold a value of type `expected` (a wrong JSON
    /// type fails the file).
    fn expect(&mut self, slice: JsonSlice<'_>, expected: Token, at: &JsonPath) -> bool {
        let found = Token::of(slice.text);
        if found == expected {
            return true;
        }
        self.error(
            DiagnosticCode::MalformedInput {
                message: format!("expected {}, found {}", expected.name(), found.name()),
                text: None,
            },
            at.clone(),
            Some(slice),
        );
        false
    }

    /// Reads `slice` as `T` once its token is known to fit.
    fn read<'a, T: facet::Facet<'a>>(&mut self, slice: JsonSlice<'a>, at: &JsonPath) -> Option<T> {
        match parse_at::<T>(slice) {
            Ok(value) => Some(value),
            Err(error) => {
                // An entry is graded as at a member whenever its full path is not the root.
                let at_member = !at.is_empty() || error.has_path();
                self.d
                    .push(error.into_diagnostic(at, |_, error| error.code_with_path(at_member)));
                None
            }
        }
    }

    /// The members of the object in `slice`.
    fn object<'a>(&mut self, slice: JsonSlice<'a>, at: &JsonPath) -> Option<Members<'a>> {
        if !self.expect(slice, Token::Object, at) {
            return None;
        }
        let raw: IndexMap<String, RawJson<'a>> = self.read(slice, at)?;
        Some(
            raw.iter()
                .filter_map(|(key, raw)| Some((key.clone(), slice.nested(raw)?)))
                .collect(),
        )
    }

    /// The elements of the array in `slice`.
    fn array<'a>(&mut self, slice: JsonSlice<'a>, at: &JsonPath) -> Option<Vec<JsonSlice<'a>>> {
        if !self.expect(slice, Token::Array, at) {
            return None;
        }
        let raw: Vec<RawJson<'a>> = self.read(slice, at)?;
        Some(raw.iter().filter_map(|raw| slice.nested(raw)).collect())
    }

    /// The string in `slice`.
    fn string(&mut self, slice: JsonSlice<'_>, at: &JsonPath) -> Option<String> {
        if !self.expect(slice, Token::String, at) {
            return None;
        }
        self.read(slice, at)
    }

    /// The string in `slice`, which must be `namespace:name`; a
    /// violation is `text`, vanilla's message of the constraint.
    fn constrained(
        &mut self,
        slice: JsonSlice<'_>,
        at: &JsonPath,
        code: DiagnosticCode,
    ) -> Option<String> {
        let value = self.string(slice, at)?;
        if is_namespaced(&value) {
            return Some(value);
        }
        self.error(code, at.clone(), Some(slice));
        None
    }

    /// The members of an object level other than `known`: each an `UnrecognizedField`
    /// warning, never kept: the game's reader skips the member it does not
    /// bind and stores nothing of it. The slice decides whether the warning stays one (below
    /// 1.26.50 the game logs it at the content log's warning level and goes on) or becomes an
    /// error (from 1.26.50 on). The returned `Extra` is empty.
    fn unknown(&mut self, members: &Members<'_>, known: &[&str], at: &JsonPath) -> Extra {
        for (key, slice) in members {
            if known.contains(&key.as_str()) {
                continue;
            }
            self.d.push(
                Diagnostic::warn(DiagnosticCode::UnrecognizedField { text: None })
                    .at(at.child(key))
                    .span(span(*slice)),
            );
        }
        Extra::default()
    }

    /// The required member `key` of `members`, or the finding of its
    /// absence.
    fn required<'a>(
        &mut self,
        members: &Members<'a>,
        key: &str,
        at: &JsonPath,
    ) -> Option<(JsonSlice<'a>, JsonPath)> {
        let path = at.child(key);
        if let Some(slice) = members.get(key) {
            return Some((*slice, path));
        }
        self.error(
            DiagnosticCode::MissingRequiredField { text: None },
            path,
            None,
        );
        None
    }

    /// `categories`, `groups` or `items`: a required array member, each element read by `each`
    /// with its path. `None` when the member or any element failed.
    fn list<'a, T>(
        &mut self,
        members: &Members<'a>,
        key: &str,
        at: &JsonPath,
        each: impl Fn(&mut Self, JsonSlice<'a>, &JsonPath) -> Option<T>,
    ) -> Option<Vec<T>> {
        let (slice, path) = self.required(members, key, at)?;
        let elements = self.array(slice, &path)?;
        let read: Vec<Option<T>> = elements
            .into_iter()
            .enumerate()
            .map(|(index, slice)| each(self, slice, &path.index(index)))
            .collect();
        read.into_iter().collect()
    }

    /// The payload: `{ "categories": [...] }`.
    fn catalog(&mut self, slice: JsonSlice<'_>, at: &JsonPath) -> Option<CraftingCatalog> {
        let members = self.object(slice, at)?;
        let extra = self.unknown(&members, &["categories"], at);
        let categories = self.list(&members, "categories", at, Self::category)?;
        Some(CraftingCatalog { categories, extra })
    }

    /// One `categories` entry: `{ "category_name": "...", "groups": [...] }`.
    fn category(&mut self, slice: JsonSlice<'_>, at: &JsonPath) -> Option<Category> {
        let members = self.object(slice, at)?;
        let extra = self.unknown(&members, &["category_name", "groups"], at);
        let category_name = self.category_name(&members, at);
        let groups = self.list(&members, "groups", at, Self::group);
        Some(Category {
            category_name: category_name?,
            groups: groups?,
            extra,
        })
    }

    /// `category_name`: one of the four names, compared with case.
    // An unknown category name stays fatal under either slice.
    fn category_name(&mut self, members: &Members<'_>, at: &JsonPath) -> Option<CategoryName> {
        let (slice, path) = self.required(members, "category_name", at)?;
        let name = self.string(slice, &path)?;
        let category = CategoryName::from_name(&name);
        if category.is_none() {
            self.error(
                DiagnosticCode::InvalidValue { text: None },
                path,
                Some(slice),
            );
        }
        category
    }

    /// One `groups` entry: `{ "group_identifier"?: {...}, "items": [...] }`.
    fn group(&mut self, slice: JsonSlice<'_>, at: &JsonPath) -> Option<Group> {
        let members = self.object(slice, at)?;
        let extra = self.unknown(&members, &["group_identifier", "items"], at);
        let group_identifier = members
            .get("group_identifier")
            .map(|slice| self.group_identifier(*slice, &at.child("group_identifier")));
        let items = self.list(&members, "items", at, Self::item);
        Some(Group {
            group_identifier: match group_identifier {
                Some(identifier) => Some(identifier?),
                None => None,
            },
            items: items?,
            extra,
        })
    }

    /// `group_identifier`: `{ "name": "...", "icon"?: item }`.
    fn group_identifier(&mut self, slice: JsonSlice<'_>, at: &JsonPath) -> Option<GroupIdentifier> {
        let members = self.object(slice, at)?;
        let extra = self.unknown(&members, &["name", "icon"], at);
        let name = match self.required(&members, "name", at) {
            Some((slice, path)) => self.constrained(slice, &path, group_name_code()),
            None => None,
        };
        let icon = members
            .get("icon")
            .map(|slice| self.item(*slice, &at.child("icon")));
        Some(GroupIdentifier {
            name: name?,
            icon: match icon {
                Some(icon) => Some(icon?),
                None => None,
            },
            extra,
        })
    }

    /// An item: the bare string, or `{ "name": "..." }`.
    // The game reads the bare string as if it were the object's `name` member.
    fn item(&mut self, slice: JsonSlice<'_>, at: &JsonPath) -> Option<CatalogItem> {
        let mut extra = Extra::default();
        let name = if Token::of(slice.text) == Token::Object {
            let members = self.object(slice, at)?;
            extra = self.unknown(&members, &["name"], at);
            let (slice, path) = self.required(&members, "name", at)?;
            self.constrained(slice, &path, item_name_code())?
        } else {
            self.constrained(slice, at, item_name_code())?
        };
        Some(CatalogItem {
            name: ItemId::parse(name),
            extra,
        })
    }
}

/// Reads the payload `payload` of a catalogue document; `None` when anything in it fails the
/// file (the reasons are in `d`).
pub(super) fn parse_catalog(
    payload: JsonSlice<'_>,
    cx: &ParseCx<'_>,
    d: &mut Diagnostics,
) -> Option<CraftingCatalog> {
    Reader { d }.catalog(payload, &cx.at)
}

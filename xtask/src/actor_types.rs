//! `cargo xtask actor-types --samples <dir>`: generates
//! `crates/addon/core/src/vanilla/generated/actor_type.rs` from the game's
//! `mojang-entities.json` metadata (in `metadata/vanilladata_modules/` of a checkout of
//! Mojang's `bedrock-samples` repository at a release tag) joined with
//! `crates/addon/core/data/actor_types.toml`.
//!
//! Plain `std`, no dependencies: `bedrock_addon_core`'s test `vanilla.rs` compiles this file
//! with `#[path]` and runs [`generate`] in-process to check the committed output. The output is
//! already in `rustfmt` form, so `cargo fmt` leaves it unchanged.

use std::fmt::Write as _;
use std::path::Path;
use std::{fs, io};

/// The entities metadata file, relative to the checkout of the metadata repository.
pub const ENTITIES: &str = "metadata/vanilladata_modules/mojang-entities.json";
/// The join table, relative to the workspace root.
pub const JOIN_TABLE: &str = "crates/addon/core/data/actor_types.toml";
/// The generated file, relative to the workspace root.
pub const OUTPUT: &str = "crates/addon/core/src/vanilla/generated/actor_type.rs";

/// Runs the step over a checkout of the metadata repository at `samples`.
pub fn run(samples: &Path) -> io::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("xtask has no parent directory"))?;
    let entities = fs::read_to_string(samples.join(ENTITIES))?;
    let join = fs::read_to_string(root.join(JOIN_TABLE))?;
    fs::write(root.join(OUTPUT), generate(&entities, &join)?)
}

/// The generated file for the text of `mojang-entities.json` and of the join table.
///
/// Fails when an identifier of the samples file has no row in the table, a row names no
/// identifier of the samples file, two rows share an id, or either input is malformed.
pub fn generate(entities_json: &str, join_toml: &str) -> io::Result<String> {
    let entities = parse_entities(entities_json)?;
    let table = parse_join_table(join_toml)?;
    let mut rows = Vec::with_capacity(entities.len());
    for identifier in &entities {
        let id = table
            .types
            .iter()
            .find(|(row, _)| row == identifier)
            .map(|(_, id)| *id)
            .ok_or_else(|| error(format!("{identifier}: no row in {JOIN_TABLE}")))?;
        let name = identifier
            .strip_prefix("minecraft:")
            .ok_or_else(|| error(format!("{identifier}: not a minecraft: identifier")))?;
        rows.push((identifier.as_str(), variant(name), id));
    }
    for (row, _) in &table.types {
        if !entities.contains(row) {
            return Err(error(format!(
                "{row}: in {JOIN_TABLE} but not in {ENTITIES}"
            )));
        }
    }
    let (default_name, default_id) = &table.default;
    let mut ids: Vec<u32> = rows.iter().map(|(_, _, id)| *id).collect();
    ids.push(*default_id);
    ids.sort_unstable();
    if let Some(pair) = ids.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(error(format!(
            "two actor types share the id {:#x}",
            pair[0]
        )));
    }
    rows.sort_by(|a, b| a.0.cmp(b.0));
    Ok(render(&rows, default_name, *default_id))
}

/// The join table: `[types]` maps identifiers to ids, `[default]` names the custom-entity type.
#[derive(Debug)]
pub struct JoinTable {
    /// `[types]`, in file order.
    pub types: Vec<(String, u32)>,
    /// The one row of `[default]`.
    pub default: (String, u32),
}

/// Reads the join table: the TOML subset it is written in (comments, `[types]` and `[default]`
/// headers, `"key" = <integer>` rows with decimal or `0x` hexadecimal values).
pub fn parse_join_table(text: &str) -> io::Result<JoinTable> {
    let mut types = Vec::new();
    let mut default = None;
    let mut section = "";
    for (index, line) in text.lines().enumerate() {
        let at = |message: &str| error(format!("{JOIN_TABLE}:{}: {message}", index + 1));
        let line = strip_comment(line).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = match name {
                "types" => "types",
                "default" => "default",
                _ => return Err(at("unknown section")),
            };
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| at("expected `key = value`"))?;
        let key = key
            .trim()
            .strip_prefix('"')
            .and_then(|k| k.strip_suffix('"'))
            .ok_or_else(|| at("the key must be a quoted string"))?;
        let value = value.trim();
        let id = match value.strip_prefix("0x") {
            Some(hex) => u32::from_str_radix(hex, 16),
            None => value.parse(),
        }
        .map_err(|_| at("the value must be an integer"))?;
        match section {
            "types" => {
                if types.iter().any(|(row, _)| row == key) {
                    return Err(at("duplicate row"));
                }
                types.push((key.to_owned(), id));
            }
            "default" if default.is_none() => default = Some((key.to_owned(), id)),
            "default" => return Err(at("[default] holds one row")),
            _ => return Err(at("a row outside [types] / [default]")),
        }
    }
    let default = default.ok_or_else(|| error(format!("{JOIN_TABLE}: no [default] row")))?;
    Ok(JoinTable { types, default })
}

/// The text before a `#` that is outside a string.
fn strip_comment(line: &str) -> &str {
    let mut in_string = false;
    for (index, byte) in line.bytes().enumerate() {
        match byte {
            b'"' => in_string = !in_string,
            b'#' if !in_string => return &line[..index],
            _ => {}
        }
    }
    line
}

/// `zombie_villager_v2` → `ZombieVillagerV2`.
fn variant(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_ascii_uppercase().to_string() + chars.as_str()
            })
        })
        .collect()
}

/// The Rust source of the generated file.
fn render(rows: &[(&str, String, u32)], default_name: &str, default_id: u32) -> String {
    let mut out = String::new();
    out.push_str(
        "// Generated by `cargo xtask actor-types --samples <dir>` (xtask/src/actor_types.rs) from
// the game's `mojang-entities.json` metadata joined with
// crates/addon/core/data/actor_types.toml. Do not edit; regenerate.

/// Vanilla's actor type of an entity: one variant per vanilla entity identifier, plus `Mob`
/// for custom entities.
///
/// The type decides which vanilla-only upgrade steps an entity document receives. A vanilla
/// identifier has its own type; a custom identifier takes the type of its
/// `runtime_identifier` when that names a vanilla entity, and is otherwise `Mob`, so the
/// default branch of every per-type rule applies to it
/// ([`ActorType::for_entity`](crate::vanilla::ActorType::for_entity)). [`ActorType::id`] is
/// vanilla's numeric value: the low byte names the type, the higher bits its categories
/// (`0x100` mob, `0x300` pathfinder mob, `0xb00` monster, `0x1300` animal, `0x80000`
/// minecart, `0x400000` projectile).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[repr(u32)]
pub enum ActorType {
",
    );
    for (identifier, variant, id) in rows {
        let _ = writeln!(out, "    /// `{identifier}`.");
        let _ = writeln!(out, "    {variant} = {},", hex(*id));
    }
    let _ = writeln!(
        out,
        "    /// A custom entity whose `runtime_identifier` names no vanilla entity.
    {default_name} = {},
}}

#[allow(clippy::too_many_lines, reason = \"one arm per vanilla entity\")]
impl ActorType {{
    /// The type of a vanilla entity named without its `minecraft:` namespace.
    pub(in crate::vanilla) fn from_vanilla_name(name: &str) -> Option<Self> {{
        match name {{",
        hex(default_id)
    );
    for (identifier, variant, _) in rows {
        let name = &identifier["minecraft:".len()..];
        let _ = writeln!(out, "            \"{name}\" => Some(Self::{variant}),");
    }
    let _ = writeln!(
        out,
        "            _ => None,
        }}
    }}

    /// The vanilla identifier of the type; `None` for `{default_name}`.
    pub(in crate::vanilla) fn vanilla_identifier(self) -> Option<&'static str> {{
        match self {{"
    );
    for (identifier, variant, _) in rows {
        let _ = writeln!(
            out,
            "            Self::{variant} => Some(\"{identifier}\"),"
        );
    }
    let _ = writeln!(
        out,
        "            Self::{default_name} => None,
        }}
    }}
}}"
    );
    out
}

/// `0x130c` or `0x80_0050`: `_` every four digits from the right, as clippy's
/// `unreadable_literal` asks.
fn hex(id: u32) -> String {
    let digits = format!("{id:x}");
    let mut out = String::from("0x");
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 4 == 0 {
            out.push('_');
        }
        out.push(digit);
    }
    out
}

fn error(message: String) -> io::Error {
    io::Error::other(message)
}

/// The identifiers of `mojang-entities.json`: `data_items[].name` of a `vanilla_data` file whose
/// `vanilla_data_type` is `entity`.
fn parse_entities(text: &str) -> io::Result<Vec<String>> {
    let root = Json::parse(text)?;
    if root.member("vanilla_data_type").and_then(Json::as_str) != Some("entity") {
        return Err(error(format!(
            "{ENTITIES}: vanilla_data_type is not \"entity\""
        )));
    }
    let Some(Json::Array(items)) = root.member("data_items") else {
        return Err(error(format!("{ENTITIES}: no data_items array")));
    };
    items
        .iter()
        .map(|item| {
            item.member("name")
                .and_then(Json::as_str)
                .map(str::to_owned)
                .ok_or_else(|| error(format!("{ENTITIES}: a data item has no string name")))
        })
        .collect()
}

/// A JSON value, enough of one to read the samples file.
#[derive(Debug)]
enum Json {
    Null,
    Bool,
    Number,
    String(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn parse(text: &str) -> io::Result<Self> {
        let mut parser = JsonParser {
            bytes: text.as_bytes(),
            at: 0,
        };
        let value = parser.value()?;
        parser.skip_whitespace();
        if parser.at != parser.bytes.len() {
            return Err(parser.fail("trailing text"));
        }
        Ok(value)
    }

    fn member(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Json::String(text) => Some(text),
            _ => None,
        }
    }
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl JsonParser<'_> {
    fn fail(&self, message: &str) -> io::Error {
        error(format!("{ENTITIES}: byte {}: {message}", self.at))
    }

    fn skip_whitespace(&mut self) {
        while self
            .bytes
            .get(self.at)
            .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
        {
            self.at += 1;
        }
    }

    fn eat(&mut self, byte: u8) -> bool {
        self.skip_whitespace();
        if self.bytes.get(self.at) == Some(&byte) {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, byte: u8) -> io::Result<()> {
        if self.eat(byte) {
            Ok(())
        } else {
            Err(self.fail(&format!("expected `{}`", char::from(byte))))
        }
    }

    fn literal(&mut self, word: &str) -> bool {
        if self.bytes[self.at..].starts_with(word.as_bytes()) {
            self.at += word.len();
            true
        } else {
            false
        }
    }

    fn value(&mut self) -> io::Result<Json> {
        self.skip_whitespace();
        match self.bytes.get(self.at) {
            Some(b'{') => {
                self.at += 1;
                let mut members = Vec::new();
                if !self.eat(b'}') {
                    loop {
                        self.skip_whitespace();
                        let key = self.string()?;
                        self.expect(b':')?;
                        members.push((key, self.value()?));
                        if self.eat(b'}') {
                            break;
                        }
                        self.expect(b',')?;
                    }
                }
                Ok(Json::Object(members))
            }
            Some(b'[') => {
                self.at += 1;
                let mut items = Vec::new();
                if !self.eat(b']') {
                    loop {
                        items.push(self.value()?);
                        if self.eat(b']') {
                            break;
                        }
                        self.expect(b',')?;
                    }
                }
                Ok(Json::Array(items))
            }
            Some(b'"') => Ok(Json::String(self.string()?)),
            Some(b'-' | b'0'..=b'9') => {
                while self
                    .bytes
                    .get(self.at)
                    .is_some_and(|b| matches!(b, b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'))
                {
                    self.at += 1;
                }
                Ok(Json::Number)
            }
            _ if self.literal("true") || self.literal("false") => Ok(Json::Bool),
            _ if self.literal("null") => Ok(Json::Null),
            _ => Err(self.fail("expected a JSON value")),
        }
    }

    fn string(&mut self) -> io::Result<String> {
        if self.bytes.get(self.at) != Some(&b'"') {
            return Err(self.fail("expected a string"));
        }
        self.at += 1;
        let mut out = Vec::new();
        loop {
            match self.bytes.get(self.at) {
                None => return Err(self.fail("unterminated string")),
                Some(b'"') => {
                    self.at += 1;
                    break;
                }
                Some(b'\\') => {
                    let escaped = match self.bytes.get(self.at + 1) {
                        Some(b'"') => b'"',
                        Some(b'\\') => b'\\',
                        Some(b'/') => b'/',
                        Some(b'n') => b'\n',
                        Some(b't') => b'\t',
                        Some(b'r') => b'\r',
                        Some(b'b') => 8,
                        Some(b'f') => 12,
                        _ => return Err(self.fail("unsupported escape")),
                    };
                    out.push(escaped);
                    self.at += 2;
                }
                Some(&byte) => {
                    out.push(byte);
                    self.at += 1;
                }
            }
        }
        String::from_utf8(out).map_err(|_| self.fail("a string is not UTF-8"))
    }
}

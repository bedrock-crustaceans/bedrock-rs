//! `cargo xtask entity-schemas`: renders `crates/addon/families/entity/src/game_schema/table.rs` from the
//! schema dump of the game's entity components, goals and event responses,
//! `xtask/src/script_meta/game/entity_schemas.txt` (one `owner` line per component,
//! goal or event response, followed by an indented line per schema member).
//!
//! Plain `std`, no dependencies; the test `the_committed_schema_table_is_the_render_of_the_dump`
//! runs [`generate`] in-process to check the committed table. The table is one
//! `#[rustfmt::skip]` item, so `cargo fmt` leaves it as rendered.

use std::fmt::Write as _;
use std::path::Path;
use std::{fs, io};

/// The dump, relative to the workspace root.
pub const INPUT: &str = "xtask/src/script_meta/game/entity_schemas.txt";
/// The generated table, relative to the workspace root.
pub const OUTPUT: &str = "crates/addon/families/entity/src/game_schema/table.rs";

/// Writes [`OUTPUT`] from [`INPUT`], and the goal numeric checks' table
/// ([`crate::goal_numeric`]).
pub fn run() -> io::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("xtask has no parent directory"))?;
    let dump = fs::read_to_string(root.join(INPUT))?;
    fs::write(root.join(OUTPUT), generate(&dump)?)?;
    let checks = fs::read_to_string(root.join(crate::goal_numeric::INPUT))?;
    fs::write(
        root.join(crate::goal_numeric::OUTPUT),
        crate::goal_numeric::generate(&checks)?,
    )
}

/// One owner of the dump: a component, goal or event response and its schema.
#[derive(Debug, PartialEq)]
pub struct Owner {
    /// The JSON name (`minecraft:ageable`, `play_sound`).
    pub name: String,
    /// `component` or `event_response`.
    pub kind: String,
    /// The first typed version of a migrated owner (its legacy reader reads documents below it).
    pub typed_since: Option<(u16, u16, u16)>,
    /// The root's children, same-named lines merged into one member.
    pub members: Vec<Member>,
}

/// A child of a node: a name pattern and its type options, in order.
#[derive(Debug, PartialEq)]
pub struct Member {
    /// The pattern as the game matches it (`a|b`, `[a-zA-Z0-9_.:-]+`).
    pub name: String,
    /// `Required child %s not found` when absent.
    pub required: bool,
    /// The type options, in order.
    pub options: Vec<Node>,
}

/// A type option.
#[derive(Debug, PartialEq, Default)]
pub struct Node {
    /// The type name the game prints in `Allowed types:` (may be empty).
    pub type_name: String,
    /// The JSON types `canParse` accepts.
    pub accepts: Vec<String>,
    /// The node kind: `object`, `array`, `enum` or `value` (a typed node, opaque unless it has
    /// children).
    pub kind: String,
    /// Object children.
    pub members: Vec<Member>,
    /// Array element options.
    pub elements: Vec<Node>,
    /// Enum constants.
    pub values: Vec<String>,
}

fn error(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

/// The owners of `dump`, in file order.
pub fn parse(dump: &str) -> io::Result<Vec<Owner>> {
    let mut owners = Vec::new();
    let mut lines: Vec<(usize, &str)> = Vec::new();
    let mut header: Option<&str> = None;
    let flush = |header: Option<&str>,
                 lines: &[(usize, &str)],
                 owners: &mut Vec<Owner>|
     -> io::Result<()> {
        if let Some(header) = header {
            owners.push(owner(header, lines)?);
        }
        Ok(())
    };
    for raw in dump.lines() {
        if raw.starts_with('#') || raw.trim().is_empty() {
            continue;
        }
        if let Some(rest) = raw.strip_prefix("owner ") {
            flush(header, &lines, &mut owners)?;
            header = Some(rest);
            lines.clear();
            continue;
        }
        let indent = raw.len() - raw.trim_start().len();
        if indent % 2 != 0 || indent == 0 {
            return Err(error(format!("bad indentation: {raw:?}")));
        }
        lines.push((indent / 2, raw.trim_start()));
    }
    flush(header, &lines, &mut owners)?;
    Ok(owners)
}

fn owner(header: &str, lines: &[(usize, &str)]) -> io::Result<Owner> {
    let mut parts = header.split(' ');
    let name = parts
        .next()
        .ok_or_else(|| error("owner without a name"))?
        .to_owned();
    let mut kind = String::new();
    let mut typed_since = None;
    for part in parts {
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| error(format!("{name}: bad field {part:?}")))?;
        match key {
            "kind" => value.clone_into(&mut kind),
            "typed_since" if value != "-" => {
                let v: Vec<u16> = value
                    .split('.')
                    .map(str::parse)
                    .collect::<Result<_, _>>()
                    .map_err(|_| error(format!("{name}: bad version {value}")))?;
                let [a, b, c] = v[..] else {
                    return Err(error(format!("{name}: bad version {value}")));
                };
                typed_since = Some((a, b, c));
            }
            _ => {}
        }
    }
    let mut at = 0;
    let mut root = Node::default();
    children(lines, &mut at, 1, &mut root, &name)?;
    if at != lines.len() {
        return Err(error(format!("{name}: line {at} does not nest")));
    }
    Ok(Owner {
        name,
        kind,
        typed_since,
        members: root.members,
    })
}

/// Reads the lines at `level` from `at` into `node`.
fn children(
    lines: &[(usize, &str)],
    at: &mut usize,
    level: usize,
    node: &mut Node,
    owner: &str,
) -> io::Result<()> {
    while let Some(&(depth, text)) = lines.get(*at) {
        if depth < level {
            return Ok(());
        }
        if depth > level {
            return Err(error(format!("{owner}: {text:?} nests too deep")));
        }
        *at += 1;
        if let Some(value) = text.strip_prefix("= ") {
            node.values.push(value.to_owned());
        } else if let Some(option) = text.strip_prefix("[] ") {
            let mut element = option_node(option, owner)?;
            children(lines, at, level + 1, &mut element, owner)?;
            node.elements.push(element);
        } else {
            let (head, option) = text
                .split_once(" : ")
                .ok_or_else(|| error(format!("{owner}: bad line {text:?}")))?;
            let (name, required) = match head.strip_suffix(" !") {
                Some(name) => (name, true),
                None => (head, false),
            };
            let mut child = option_node(option, owner)?;
            children(lines, at, level + 1, &mut child, owner)?;
            // A name listed twice is one child with two type options (`getChildSchemaOptions`).
            if let Some(member) = node.members.iter_mut().find(|m| m.name == name) {
                member.required |= required;
                member.options.push(child);
            } else {
                node.members.push(Member {
                    name: name.to_owned(),
                    required,
                    options: vec![child],
                });
            }
        }
    }
    Ok(())
}

/// `<type name> <kind> [<accepts>]` (the type name may be empty).
fn option_node(text: &str, owner: &str) -> io::Result<Node> {
    let open = text
        .rfind(" [")
        .ok_or_else(|| error(format!("{owner}: no accepts in {text:?}")))?;
    let accepts = text[open + 2..]
        .strip_suffix(']')
        .ok_or_else(|| error(format!("{owner}: bad accepts in {text:?}")))?;
    let head = &text[..open];
    let (type_name, rest) = head
        .split_once(' ')
        .ok_or_else(|| error(format!("{owner}: bad option {text:?}")))?;
    let kind = rest;
    let accepts: Vec<String> = accepts.split(',').map(str::to_owned).collect();
    for a in &accepts {
        if !ACCEPTS.iter().any(|(name, _)| name == a) {
            return Err(error(format!("{owner}: unknown JSON type {a:?}")));
        }
    }
    Ok(Node {
        type_name: type_name.to_owned(),
        accepts,
        kind: kind.to_owned(),
        ..Node::default()
    })
}

/// The JSON type names of the dump and the table's constants for them.
const ACCEPTS: [(&str, &str); 8] = [
    ("null", "NULL"),
    ("int", "INT"),
    ("uint", "UINT"),
    ("real", "REAL"),
    ("string", "STRING"),
    ("bool", "BOOL"),
    ("array", "ARRAY"),
    ("object", "OBJECT"),
];

/// The generated table for the text of the dump.
pub fn generate(dump: &str) -> io::Result<String> {
    let mut owners = parse(dump)?;
    owners.sort_by(|a, b| a.name.as_bytes().cmp(b.name.as_bytes()));
    let mut out = String::from(
        "// Generated by `cargo xtask entity-schemas` from the game's schema description\n\
         // `xtask/src/script_meta/game/entity_schemas.txt`. Do not edit: an xtask test checks it.\n",
    );
    let mut body = String::new();
    for (title, kind) in [
        (
            "The components and goals, sorted bytewise by name.",
            "component",
        ),
        (
            "The entity event responses, sorted bytewise by name.",
            "event_response",
        ),
    ] {
        let list: Vec<&Owner> = owners.iter().filter(|o| o.kind == kind).collect();
        let constant = if kind == "component" {
            "COMPONENTS"
        } else {
            "EVENT_RESPONSES"
        };
        let _ = write!(
            body,
            "\n/// {title}\n#[rustfmt::skip]\npub(super) static {constant}: [Owner; {}] = [\n",
            list.len()
        );
        for owner in list {
            let since = match owner.typed_since {
                Some((a, b, c)) => format!("Some(({a}, {b}, {c}))"),
                None => "None".to_owned(),
            };
            let _ = writeln!(
                body,
                "    Owner {{ name: {:?}, typed_since: {since}, members: {} }},",
                owner.name,
                members(&owner.members)
            );
        }
        body.push_str("];\n");
    }
    // Import what the table uses, in `rustfmt`'s order.
    let order = [
        "ARRAY", "BOOL", "INT", "Member", "NULL", "OBJECT", "Owner", "REAL", "STRING", "UINT", "n",
        "o",
    ];
    let uses: Vec<&str> = order
        .into_iter()
        .filter(|name| {
            ["Member", "Owner"].contains(name)
                || body.contains(&format!("{name}("))
                || body.contains(&format!("{name} "))
                || body.contains(&format!("{name},"))
        })
        .collect();
    let _ = write!(out, "\nuse super::{{{}}};\n", uses.join(", "));
    out.push_str(&body);
    Ok(out)
}

fn members(list: &[Member]) -> String {
    if list.is_empty() {
        return "&[]".to_owned();
    }
    let items: Vec<String> = list
        .iter()
        .map(|m| {
            let options: Vec<String> = m.options.iter().map(node).collect();
            format!(
                "Member {{ name: {:?}, required: {}, options: &[{}] }}",
                m.name,
                m.required,
                options.join(", ")
            )
        })
        .collect();
    format!("&[{}]", items.join(", "))
}

fn node(node: &Node) -> String {
    let accepts: Vec<&str> = node
        .accepts
        .iter()
        .filter_map(|a| ACCEPTS.iter().find(|(name, _)| name == a).map(|(_, c)| *c))
        .collect();
    let elements: Vec<String> = node.elements.iter().map(self::node).collect();
    let values: Vec<String> = iteration_order(&node.values)
        .iter()
        .map(|v| format!("{v:?}"))
        .collect();
    // An object node validates its children; a typed node only when it has some (a struct read
    // through an object of children); otherwise it is opaque (its own parse reads it).
    let constructor = if node.kind == "object" || !node.members.is_empty() {
        "o"
    } else {
        "n"
    };
    format!(
        "{constructor}({:?}, {}, {}, &[{}], &[{}])",
        node.type_name,
        accepts.join(" | "),
        members(&node.members),
        elements.join(", "),
        values.join(", ")
    )
}

/// The game's string hash: 64-bit FNV-1 (multiply, then xor), 0 for "".
fn hashed_string(text: &str) -> u64 {
    if text.is_empty() {
        return 0;
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3) ^ u64::from(byte);
    }
    hash
}

/// libc++'s bucket of `hash` among `count` buckets (`__constrain_hash`).
fn constrain(hash: u64, count: u64) -> u64 {
    if count.is_power_of_two() {
        hash & (count - 1)
    } else if hash < count {
        hash
    } else {
        hash % count
    }
}

/// libc++'s `__next_prime` for the small counts an enum map reaches.
fn next_prime(n: u64) -> u64 {
    (n.max(2)..=u64::MAX)
        .find(|&p| (2..p).take_while(|d| d * d <= p).all(|d| p % d != 0))
        .unwrap_or(n)
}

/// The order a libc++ `std::unordered_map<HashedString, T>` that `keys` were inserted into, in
/// order, iterates them (`__node_insert_unique` with `max_load_factor` 1 and `__rehash_unique`):
/// the order the game lists the allowed keys of an enum in its "not allowed" message (it walks
/// the map's node list).
pub fn iteration_order(keys: &[String]) -> Vec<String> {
    // The node list, front first; `buckets[b]` is the index in `list` of the node before bucket
    // b's first node (`None` = the list head), or unset.
    let mut list: Vec<(u64, &String)> = Vec::new();
    let mut count: u64 = 0;
    for key in keys {
        let hash = hashed_string(key);
        if list.iter().any(|(h, k)| *h == hash && *k == key) {
            continue;
        }
        let size = list.len() as u64;
        if count == 0 || size + 1 > count {
            let power2 = count > 2 && count.is_power_of_two();
            let mut n = (2 * count + u64::from(!power2)).max(size + 1);
            n = if n == 1 {
                2
            } else if n & (n - 1) != 0 {
                next_prime(n)
            } else {
                n
            };
            if n > count {
                count = n;
                list = rehash(&list, count);
            }
        }
        // Insert: at the front of the list when the bucket is empty, else after the node
        // before the bucket's first node (that is, first in the bucket).
        let bucket = constrain(hash, count);
        match list
            .iter()
            .position(|(h, _)| constrain(*h, count) == bucket)
        {
            None => list.insert(0, (hash, key)),
            Some(first) => list.insert(first, (hash, key)),
        }
    }
    list.into_iter().map(|(_, k)| k.clone()).collect()
}

/// `__do_rehash<true>`: the list regrouped bucket by bucket, each node moved to the front of its
/// bucket's group when the bucket was already seen.
fn rehash<'a>(list: &[(u64, &'a String)], count: u64) -> Vec<(u64, &'a String)> {
    let mut out: Vec<(u64, &String)> = Vec::new();
    for &(hash, key) in list {
        let bucket = constrain(hash, count);
        let previous_bucket = out.last().map(|(h, _)| constrain(*h, count));
        match out.iter().position(|(h, _)| constrain(*h, count) == bucket) {
            Some(first) if previous_bucket != Some(bucket) => out.insert(first, (hash, key)),
            _ => out.push((hash, key)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"# a comment
owner minecraft:b kind=component typed_since=1.26.60
  items : array array [array]
    [] object value [null,object]
      heal_amount : int value [int,uint,real]
    [] string value [string]
  items : string value [string]
  min_difficulty : enumerated_value value [string]
    = easy
  item :  value [string,object]
owner minecraft:a kind=component typed_since=-
owner play_sound kind=event_response typed_since=-
  sound ! : string value [string]
";

    #[test]
    fn a_name_listed_twice_is_one_member_with_two_options() {
        let owners = parse(SAMPLE).unwrap();
        let b = &owners[0];
        assert_eq!(b.typed_since, Some((1, 26, 60)));
        assert_eq!(b.members.len(), 3);
        assert_eq!(b.members[0].name, "items");
        assert_eq!(b.members[0].options.len(), 2);
        assert_eq!(b.members[0].options[0].elements.len(), 2);
        assert_eq!(
            b.members[0].options[0].elements[0].members[0].name,
            "heal_amount"
        );
        assert_eq!(b.members[1].options[0].values, ["easy"]);
        assert_eq!(b.members[2].options[0].type_name, "");
        assert!(owners[2].members[0].required);
        assert!(owners[1].members.is_empty());
    }

    #[test]
    fn the_table_sorts_owners_and_splits_the_event_responses() {
        let table = generate(SAMPLE).unwrap();
        let a = table.find("\"minecraft:a\"").unwrap();
        let b = table.find("\"minecraft:b\"").unwrap();
        assert!(a < b, "{table}");
        assert!(table.contains("COMPONENTS: [Owner; 2]"), "{table}");
        assert!(table.contains("EVENT_RESPONSES: [Owner; 1]"), "{table}");
        assert!(
            table.contains("n(\"int\", INT | UINT | REAL, &[], &[], &[])"),
            "{table}"
        );
        assert!(
            table.contains("Member { name: \"sound\", required: true"),
            "{table}"
        );
    }

    #[test]
    fn a_bad_line_is_an_error() {
        assert!(parse("owner x kind=component\n  broken\n").is_err());
        assert!(parse("owner x kind=component\n  a : float value [nothing]\n").is_err());
        assert!(parse("owner x kind=component\n      a : float value [real]\n").is_err());
    }

    /// The crate's table is the render of the dump, byte for byte.
    #[test]
    fn the_committed_schema_table_is_the_render_of_the_dump() {
        let generated = generate(include_str!("script_meta/game/entity_schemas.txt")).unwrap();
        assert!(
            generated
                == include_str!("../../crates/addon/families/entity/src/game_schema/table.rs"),
            "crates/addon/families/entity/src/game_schema/table.rs is stale: run `cargo xtask entity-schemas`"
        );
    }

    #[test]
    fn the_committed_dump_parses() {
        let owners = parse(include_str!("script_meta/game/entity_schemas.txt")).unwrap();
        assert_eq!(owners.len(), 240);
        assert_eq!(
            owners.iter().filter(|o| o.kind == "event_response").count(),
            12
        );
        assert_eq!(
            owners.iter().filter(|o| o.typed_since.is_none()).count(),
            86
        );
    }
}

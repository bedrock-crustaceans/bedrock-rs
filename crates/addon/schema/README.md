# bedrock_addon_schema

The `schema::` attribute grammar of the `bedrock_addon` crate family. A typed document struct states its per-field facts as facet extension attributes, `#[facet(schema::fallback = 1.0, schema::min = 0.0)]`, beside facet's own; facet's derive checks every key by name at compile time, typos included. Part of [bedrock-rs](https://github.com/bedrock-crustaceans/bedrock-rs).

**This crate is an implementation detail.** Its one user-facing path is `bedrock_addon_core::schema`, which re-exports it: a file that uses the attributes writes `use bedrock_addon_core::schema;`, and a module that can see another item named `schema` cannot use them. The crate is a leaf of its own because facet's grammar macro exports its dispatcher with `#[macro_export]`, which places it at a crate root.

## Keys

| Key | Value | Kind | Read by |
| --- | --- | --- | --- |
| `since_format` | `"1.21.60"` | stored in the `Shape` | `#[derive(Schema)]`; the `Shape` keeps it, `Field::get_attr(Some("schema"), "since_format")` |
| `since_engine` | `"1.20.0"` | stored | `#[derive(Schema)]` |
| `unknown` | `"keep"`, `"report"`, `"reject"`, `"ignore"`, `"component"` or `"silent"` | stored | `#[derive(Schema)]` |
| `quiet` | `"a b"`, member names | compile-time | `#[derive(Schema)]` on a struct: unknown members kept and told nothing of |
| `fallback` | an expression | compile-time | `#[derive(Schema)]` |
| `min`, `max`, `above`, `below` | a number, or the name of another field | compile-time | `#[derive(Schema)]` |
| `min_len`, `max_len` | a whole number | compile-time | `#[derive(Schema)]`: the bounds of a list, array or map |
| `multiple_of` | a positive number | compile-time | `#[derive(Schema)]` |
| `low_code`, `high_code` | a `Code` expression | compile-time | `#[derive(Schema)]`: the code a broken bound is told under |
| `kept` | (marker) | stored as `()` | `#[derive(Schema)]`: a broken bound is a kept error |
| `keys` | a `schema::Keys` path | compile-time | `#[derive(Schema)]` on a map keyed by `String` |
| `molang` | a `MolangFieldSpec` path | compile-time | `#[derive(Schema)]` |
| `molang_with` | a function path | compile-time | `#[derive(Schema)]` |
| `read_with` | a function path, `fn(&Node, &mut Read) -> Option<T>` | compile-time | `#[derive(Schema)]`: the field's read in place of its type's `FromJson` |
| `absent_with` | a function path, `fn(&Node, &mut Read) -> Option<T>` given the object | compile-time | `#[derive(Schema)]`: an `Option` field's value when the object lacks the member (asked in its place in a declared-order read, after the members in a file-order one) |
| `check_with` | a function path, `fn(&mut T, &mut SchemaCx)` | compile-time | `#[derive(Schema)]`: the field's check |
| `rule` | a function path, `fn(&mut Self, &mut SchemaCx)` | compile-time | `#[derive(Schema)]` on a struct: a rule across its fields at the check |
| `after_read` | a function path, `fn(&mut Self, &Node, &mut Read)` | compile-time | `#[derive(Schema)]` on a struct: a rule at the read, with the object as written |
| `guard` | a function path, `fn(&Node, &mut Read) -> bool` | compile-time | `#[derive(Schema)]` on a field (before the member is read) or a struct (after the members, before the unknown ones): `false` ends the struct's read there |
| `tag_with` | a function path | compile-time | `#[derive(Schema)]` on an internally tagged enum: names the variant |
| `trigger` | (marker) | stored as `()` | `#[derive(Schema)]` on a struct: it is an event trigger itself |
| `foreign_event` | (marker) | stored as `()` | `#[derive(Schema)]` |
| `required` | (marker) | stored as `()` | `#[derive(Schema)]` on an `Option` field |
| `wrong_type`, `missing`, `invalid`, `negative` | a `CodeDef<0>` or `CodeDef<1>` path | compile-time | `#[derive(Schema)]`: the code a wrong type, a missing member, a refused value or a negative number is told under; a `CodeDef<1>` is fed the read's subject (`Read::subject`: the block, the file), and a read without one tells its own code |
| `finite` | (marker) | stored as `()` | `#[derive(Schema)]` on a float field |
| `iterated` | (marker) | stored as `()` | `#[derive(Schema)]` on a list field |
| `drop` | (marker) | stored as `()` | `#[derive(Schema)]` on a list field: a failed element is left out alone |
| `if_kind` | (marker) | stored as `()` | `#[derive(Schema)]` on an `Option` field: a member of another JSON kind than its type takes is absent, told nothing |
| `reads_null` | (marker) | stored as `()` | `#[derive(Schema)]` on an `Option` field: a `null` is read as a value |
| `or_kept` | `"told"` or `"silent"` | compile-time | `#[derive(Schema)]` on an `Option` field: a value that fails is kept as written |
| `string_only` | (marker) | stored as `()` | `#[derive(Schema)]` on a string enum |
| `order` | `"game"` | compile-time | `#[derive(Schema)]` on a string enum |


A compile-time key is checked by name by facet and stored as nothing; its value is read from the tokens by `#[derive(Schema)]`, which also checks each stored key's string (a version, or one of the words) at compile time.

# bedrock_addon_schema

The `schema::` attribute grammar of the `bedrock_addon` crate family. A typed document struct states its per-field facts as facet extension attributes, `#[facet(schema::fallback = 1.0, schema::min = 0.0)]`, beside facet's own; facet's derive checks every key by name at compile time, typos included. Part of [bedrock-rs](https://github.com/bedrock-crustaceans/bedrock-rs).

**This crate is an implementation detail.** Its one user-facing path is `bedrock_addon_core::schema`, which re-exports it: a file that uses the attributes writes `use bedrock_addon_core::schema;`, and a module that can see another item named `schema` cannot use them. The crate is a leaf of its own because facet's grammar macro exports its dispatcher with `#[macro_export]`, which places it at a crate root.

## Keys

| Key | Value | Kind | Read by |
| --- | --- | --- | --- |
| `since_format` | `"1.21.60"` | stored in the `Shape` | the JSON door, through `Field::get_attr(Some("schema"), "since_format")` |
| `since_engine` | `"1.20.0"` | stored | the door |
| `unknown` | `"keep"`, `"report"`, `"reject"` or `"ignore"` | stored | the door |
| `fallback` | an expression | compile-time | `#[derive(Schema)]` |
| `min`, `max`, `above`, `below` | a number, or the name of another field | compile-time | `#[derive(Schema)]` |
| `molang` | a `MolangFieldSpec` path | compile-time | `#[derive(Schema)]` |
| `molang_with` | a function path | compile-time | `#[derive(Schema)]` |
| `foreign_event` | (marker) | stored as `()` | `#[derive(Schema)]` |
| `lowercase` | (marker) | stored as `()` | `#[derive(Schema)]` |
| `expected` | `"a control flag name"` | compile-time | `#[derive(Schema)]` on a string enum |
| `case_insensitive` | (marker) | stored as `()` | `#[derive(Schema)]` on a string enum |

A compile-time key is checked by name by facet and stored as nothing; its value is read from the tokens by `#[derive(Schema)]`, which also checks each stored key's string (a version, or one of the four words) at compile time.

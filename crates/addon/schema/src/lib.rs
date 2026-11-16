//! The `schema::` attribute grammar of the `bedrock_addon` crate family.
//!
//! A typed document struct states its per-field facts as facet extension attributes in one
//! namespace, `#[facet(schema::fallback = 1.0, schema::min = 0.0)]`, beside facet's own
//! `default`, `rename_all` and `flatten`. facet's derive checks every key by name. Three keys
//! are stored in the `Shape`, for the reader while it reads the text; the others are
//! compile-time only, read from the tokens by `#[derive(Schema)]`.
//!
//! The hand-written hooks form one grid, a field's or a struct's, at the read or at the check:
//! `read_with` (a field's read, `fn(&Node, &mut Read) -> Option<T>`; `schema::read_later` for a
//! member the struct's `after_read` reads), `check_with` (a field's check), `after_read` (a
//! struct's rule at the read, with the object as written) and `rule` (a struct's rule at the
//! check, on the typed value); `molang_with` is the Molang walk's, on a field or a struct.
//!
//! The one path users name is `bedrock_addon_core::schema`, which re-exports this crate with
//! the `Schema` trait beside it. The crate is a leaf of its own because facet's grammar macro
//! exports its dispatcher with `#[macro_export]`, at a crate root, and the namespace `schema::`
//! has to resolve to that root. It depends on facet, for the grammar, and on
//! `bedrock_addon_derive`, whose `#[derive(Schema)]` it re-exports beside the grammar.
//!
//! Modules: none. Features: none.

/// `#[derive(Schema)]`, in the macro namespace; core puts the trait of the same name beside it.
pub use bedrock_addon_derive::Schema;

facet::define_attr_grammar! {
    ns "schema";
    crate_path ::bedrock_addon_core::schema;

    /// The keys of the `schema::` namespace. The variant names are the keys in snake case.
    #[non_exhaustive]
    pub enum Attr {
        /// `since_format = "1.21.60"` (stored): the format version from which the member is
        /// read; in a document below it the member is no member of the struct: an unknown one,
        /// told and kept by the struct's rule (refused under a strict mask).
        SinceFormat(&'static str),
        /// `since_engine = "1.20.0"` (stored): the engine version from which the member is
        /// read, judged by the derived check of a validating context (`SchemaCx::validating`)
        /// on the value as the pipeline holds it, after the upgrade: below `since` on the game's
        /// own engine the member is `core/child_not_valid`, an error; in a pack whose
        /// `min_engine_version` is below it, a `FieldSinceEngine` information. The read reads
        /// the member whatever the engine.
        SinceEngine(&'static str),
        /// `unknown = "keep"` (stored, on a field or a container): what an unknown member
        /// is where this struct differs from the slice's rule: `keep`, `report`, `reject`,
        /// `ignore`, `component` (a name that is no component of its object, as the game
        /// tells it), or `silent` (kept and told nothing).
        Unknown(&'static str),
        /// `fallback = x` (compile-time, an `Option` field): the value absence stands for;
        /// the getter returns it, and it is never written into the data. A text fallback's
        /// getter lends the text (`&str`, the member's own by `AsRef<str>`); a pair `(low, high)`
        /// on a range answers the two ends the range resolves to (`shared::Resolve`).
        Fallback(arbitrary),
        /// `min = x` (compile-time): the value is `>= x`; a number or another field's name. On a
        /// fixed array or a `Vec` it is the bound of each element: the first one outside is the
        /// one finding, at its index. On a value that holds numbers (a range, through
        /// `Bounded`) it is the bound of each of them: the first one outside is the one
        /// finding, at the member. On a map it is the bound of each value: every one outside is
        /// told, at its key. A limit written with the `f64` suffix (`179.9_f64`) is a
        /// double the value is widened to, and the finding's text states it as one.
        Min(arbitrary),
        /// `max = x` (compile-time): the value is `<= x`.
        Max(arbitrary),
        /// `above = x` (compile-time): the value is `> x`.
        Above(arbitrary),
        /// `below = x` (compile-time): the value is `< x`.
        Below(arbitrary),
        /// `min_len = n` (compile-time, a list, a fixed array or a map, below its `Option`): the
        /// field holds at least `n` elements; one finding at the field when it holds fewer.
        MinLen(arbitrary),
        /// `max_len = n` (compile-time, as `min_len`): the field holds at most `n` elements.
        MaxLen(arbitrary),
        /// `low_code = CODE` (compile-time, a field with a low bound, `min` or `above`): an
        /// expression of type `Code`, made when the bound breaks, that a value below it is told
        /// under in place of the context's own wording.
        LowCode(arbitrary),
        /// `high_code = CODE` (compile-time, a field with a high bound, `max` or `below`): as
        /// `low_code`, for a value past it.
        HighCode(arbitrary),
        /// `kept` (marker, a field with bounds): the game logs the line and keeps the value: the
        /// finding is a kept error, and nothing fails or resets.
        Kept,
        /// `keys = KEYS` (compile-time, a map field keyed by `String`): the rule its members'
        /// names are read by, a `schema::Keys` by its path: a name its pattern does not take is
        /// an unknown member of the map (told, not kept), and a map that must have one tells a
        /// missing member when none is read; read in the profile's order and name rule.
        Keys(arbitrary),
        /// `multiple_of = n` (compile-time): the value is a multiple of the positive number
        /// `n`, checked after the field's range as `value % n == 0.0` in `f64`: exact for an
        /// integer step and a binary fraction (`0.5`, `0.25`), not for a step `f64` holds only
        /// near (`0.1`, where `0.5` and `1.0` are told as off the step).
        MultipleOf(arbitrary),
        /// `molang = SPEC` (compile-time): the field's `MolangFieldSpec`, where it is not the
        /// family's default. On a struct it is the default spec of the Molang walk below it (a
        /// field with none of its own, and the fields of the structs it holds), which a family
        /// whose descriptors read their `tags` otherwise states once at its documents' roots.
        Molang(arbitrary),
        /// `molang_with = path` (compile-time): a hand-written Molang walk of the field,
        /// `fn(&mut T, &mut MolangVisitor<'_>)`. On a struct, its whole walk,
        /// `fn(&mut Self, &mut MolangVisitor<'_>)`, in place of the walk over its fields.
        MolangWith(arbitrary),
        /// `check_with = path` (compile-time): the check of the field's value, in place of the
        /// walk into its type: `fn(&mut T, &mut SchemaCx<'_>)`, `T` the field's type (an
        /// `Option` field's inner type), called for a member that is present with the context
        /// at the member, after the field's own bounds.
        CheckWith(arbitrary),
        /// `rule = path` (compile-time, on a struct): a rule across its fields,
        /// `fn(&mut Self, &mut SchemaCx<'_>)`, called with the context at the struct's own path
        /// after every field's check (`after_read` is its twin at the read).
        Rule(arbitrary),
        /// `quiet = "a b"` (compile-time, on a struct): the members named are unknown ones
        /// the struct keeps and tells nothing of, where it tells the others: the members the
        /// game's own files carry and its reader never reads. It is `unknown = "silent"` for
        /// the named members alone.
        Quiet(arbitrary),
        /// `after_read = path` (compile-time, on a struct): a rule over the value just read,
        /// `fn(&mut Self, &Node<'_>, &mut Read<'_>)`, called by the struct's own read after its
        /// members and its unknown ones, with the object as written: what a member cannot say
        /// alone (a default that follows a sibling, a member kept beside the typed one). It
        /// tells its findings to the read and does not fail the owner; a struct that is
        /// flattened into another is not hooked. It is `rule` at the read: `rule` judges the
        /// typed value in the check (a `SchemaCx`, no JSON); this one runs in the read, with the
        /// object as written, in the order the read tells and in the phase it is given.
        AfterRead(arbitrary),
        /// `trigger` (marker, on a struct): the struct is an event trigger itself (an entity's
        /// `on_*` component, `{event, filters, target}`): its trigger member is its own path,
        /// beside those of its fields.
        Trigger,
        /// `read_with = path` (compile-time): the field's read, in place of its type's
        /// `FromJson`: `fn(&Node<'_>, &mut Read<'_>) -> Option<T>`, `T` the field's type (an
        /// `Option` field's inner type), called for a member that is written.
        ReadWith(arbitrary),
        /// `members = ["a", "b"]` (compile-time): the field is fed by these members of its
        /// struct's object, not by one of its own name: a value the game reads from several keys
        /// (a literal or the name of where to find it, one list under three spellings). Each is
        /// a member of the struct, none an unknown one. The field is read once, at its place (in
        /// file order where the file has the first of them, after the members when it has
        /// none), whether or not any is written: `FromMembers<N>` on its type, or its
        /// `read_with`, `fn(&Node<'_>, &[Option<Node<'_>>; N], T, &mut Read<'_>) -> Option<T>`,
        /// `T` the field's type, given the object, each member as written and the field's
        /// `default` (`None` for an `Option`), says what was read and how two written together
        /// merge. A member may feed several fields; each is read where the first of its members
        /// is.
        Members(arbitrary),
        /// `tag_with = path` (compile-time, an internally tagged enum): the name of the variant
        /// is the function's, `fn(&Node<'_>, &mut Read<'_>) -> Option<&'static str>`, which looks
        /// at the object, tells what it refuses (the game's own words for a name that is none)
        /// and returns the variant's JSON name, or `None` once it has told. In place of the
        /// string at the tag member, for a tag the game normalises (a namespace dropped, an
        /// engine that has no such name) or words its own way. The enum may be flattened into a
        /// struct (`flatten`), which reads the variant where it declares the enum.
        TagWith(arbitrary),
        /// `required` (marker, an `Option` field): the document must write the member, though
        /// its value is optional in the type: absence is a missing required member.
        Required,
        /// `wrong_type = CODE` (compile-time, a `CodeDef<0>` by its path): the code a value of
        /// the wrong JSON type is told under at the member, in place of the read's.
        WrongType(arbitrary),
        /// `missing = CODE` (compile-time): the code the member is told missing under.
        Missing(arbitrary),
        /// `invalid = CODE` (compile-time): the code a value the member's type refuses (an
        /// unknown name, a number out of range) is told under at the member. On an enum with
        /// data it is the code of a value that names no variant, told as facet-json told a
        /// variant lookup: at the member the value belongs to (a list's elements at the list),
        /// over the value. Internally tagged: a tag that names none; externally tagged: a name
        /// that is none, and a `null`, bool or number, which none can be; untagged: a negative
        /// whole number no arm takes (an unsigned arm refuses it), where the arm's own scalar rule
        /// would tell a wrong type. Without the key the read's own words stand.
        Invalid(arbitrary),
        /// `negative = CODE` (compile-time, an unsigned integer member): the code a negative whole
        /// number given to the member is told under, at the member, in place of the wrong type the
        /// strict profiles tell and the invalid value the converting one does: the game binds the
        /// number and the constraint refuses it. A number beyond the member's width is not one.
        Negative(arbitrary),
        /// `finite` (marker, a float field): a number beyond the float's width reads as its
        /// largest finite value of that sign, told as an `InvalidValue` information.
        Finite,
        /// `string_only` (marker, a string enum): the enum is read from a string and nothing
        /// else, as the game binds it: another JSON type is a wrong type.
        StringOnly,
        /// `iterated` (marker, a list field): the member is iterated as jsoncpp iterates a
        /// value: an array's elements, an object's values in key order, nothing for any other;
        /// a member that is no array is told a wrong type and read so.
        Iterated,
        /// `drop` (marker, a list field): an element that fails to read is left out alone, told
        /// by its own findings, and the rest is kept, whatever the reading does with a failed
        /// element: the game reads each element by itself and skips the one it cannot take. The
        /// file's later elements are named by the places they have in the file.
        Drop,
        /// `reads_null` (marker, an `Option` field): a `null` member is a value like any other,
        /// read by the field's type (or `read_with`) as `Some`, where the reading's null rule
        /// would make it `None`: the game's reader asks for the member and takes what is there
        /// (jsoncpp's `asString` of a `null` is `""`). With `or_kept`, a `null` the type refuses
        /// is kept as written. Without it a `null` is the reading's null rule's, whatever reads
        /// the member: a `read_with` fn never sees one.
        ReadsNull,
        /// `or_kept = "told"` or `"silent"` (compile-time, an `Option` field): a value that
        /// fails to read leaves the field `None` and is kept as written beside the typed value
        /// (with the struct's unknown members), its failure told (or not) and failing nothing.
        OrKept(arbitrary),
        /// `foreign_event` (marker): an `EventTrigger` field whose event belongs to another
        /// entity, so it is no trigger of this one.
        ForeignEvent,
        /// `order = "game"` (compile-time, a string enum): the enum states its own `ALL`, in
        /// the game's order, and the derive emits none; it checks at compile time that `ALL`
        /// names every variant once. `NAMES` and `EXPECTED` stay in declaration order.
        Order(arbitrary),
    }
}

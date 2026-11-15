//! The `schema::` attribute grammar of the `bedrock_addon` crate family.
//!
//! A typed document struct states its per-field facts as facet extension attributes in one
//! namespace, `#[facet(schema::fallback = 1.0, schema::min = 0.0)]`, beside facet's own
//! `default`, `rename_all` and `flatten`. facet's derive checks every key by name. Three keys
//! are stored in the `Shape`, for the JSON door while it reads the text; the others are
//! compile-time only, read from the tokens by `#[derive(Schema)]`.
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
        /// read: written in a pack whose `min_engine_version` is below it, the member is read
        /// and told as a `FieldSinceEngine` information. The derived check of a validating
        /// context (`SchemaCx::validating`) judges the same key on the value as the pipeline
        /// holds it, after the upgrade, and adds the game's own engine: below `since` the member
        /// is `core/child_not_valid`, an error. A family that checks that way reads with
        /// `Versions::engine` unset, so a member is told once.
        SinceEngine(&'static str),
        /// `unknown = "keep"` (stored, on a field or a container): what an unknown member
        /// is where this struct differs from the slice's rule: `keep`, `report`, `reject`,
        /// `ignore`, or `component` (a name that is no component of its object, as the game
        /// tells it).
        Unknown(&'static str),
        /// `fallback = x` (compile-time, an `Option` field): the value absence stands for;
        /// the getter returns it, and it is never written into the data.
        Fallback(arbitrary),
        /// `min = x` (compile-time): the value is `>= x`; a number or another field's name. On a
        /// fixed array or a `Vec` it is the bound of each element: the first one outside is the
        /// one finding, at its index. On a value that holds numbers (a range, through
        /// `Bounded`) it is the bound of each of them: the first one outside is the one
        /// finding, at the member. A limit written with the `f64` suffix (`179.9_f64`) is a
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
        /// `fn(&mut T, &mut MolangVisitor<'_>)`.
        MolangWith(arbitrary),
        /// `check_with = path` (compile-time): the check of the field's value, in place of the
        /// walk into its type: `fn(&mut T, &mut SchemaCx<'_>)`, `T` the field's type (an
        /// `Option` field's inner type), called for a member that is present with the context
        /// at the member, after the field's own bounds.
        CheckWith(arbitrary),
        /// `rule = path` (compile-time, on a struct): a rule across its fields,
        /// `fn(&mut Self, &mut SchemaCx<'_>)`, called with the context at the struct's own path
        /// after every field's check.
        Rule(arbitrary),
        /// `read_with = path` (compile-time): the field's read, in place of its type's
        /// `FromJson`: `fn(&Node<'_>, &mut Read<'_>) -> Option<T>`, `T` the field's type (an
        /// `Option` field's inner type), called for a member that is written.
        ReadWith(arbitrary),
        /// `required` (marker, an `Option` field): the document must write the member, though
        /// its value is optional in the type: absence is a missing required member.
        Required,
        /// `wrong_type = CODE` (compile-time, a `CodeDef<0>` by its path): the code a value of
        /// the wrong JSON type is told under at the member, in place of the read's.
        WrongType(arbitrary),
        /// `missing = CODE` (compile-time): the code the member is told missing under.
        Missing(arbitrary),
        /// `invalid = CODE` (compile-time): the code a value the member's type refuses (an
        /// unknown name, a number out of range) is told under at the member.
        Invalid(arbitrary),
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
        /// `elements = "drop"` or `"fail"` (compile-time, a list field): what a failed element
        /// does, whatever the reading: left out, the rest kept; or the list fails.
        Elements(arbitrary),
        /// `or_kept = "told"` or `"silent"` (compile-time, an `Option` field): a value that
        /// fails to read leaves the field `None` and is kept as written beside the typed value
        /// (with the struct's unknown members), its failure told (or not) and failing nothing.
        OrKept(arbitrary),
        /// `foreign_event` (marker): an `EventTrigger` field whose event belongs to another
        /// entity, so it is no trigger of this one.
        ForeignEvent,
        /// `lowercase` (marker): a string the game lower-cases (ASCII letters) as it reads it,
        /// telling nothing; the check does the same.
        Lowercase,
        /// `expected = "a control flag name"` (compile-time, a string enum): what the enum
        /// takes, in words, for the error of a name it does not know.
        Expected(arbitrary),
        /// `case_insensitive` (marker, a string enum): a name is read in any case.
        CaseInsensitive,
        /// `order = "game"` (compile-time, a string enum): the enum states its own `ALL`, in
        /// the game's order, and the derive emits none; it checks at compile time that `ALL`
        /// names every variant once. `NAMES` and `EXPECTED` stay in declaration order.
        Order(arbitrary),
        /// `otherwise` (marker, the newtype variant of an untagged enum): the arm a value takes
        /// when the arm the read chooses fails (whose findings stand) or none takes it.
        Otherwise,
    }
}

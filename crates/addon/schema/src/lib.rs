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
        /// read; a document below it that writes the member is refused.
        SinceFormat(&'static str),
        /// `since_engine = "1.20.0"` (stored): the engine version from which the member is
        /// read.
        SinceEngine(&'static str),
        /// `unknown = "keep"` (stored, on a field or a container): what an unknown member
        /// is where this struct differs from the slice's rule: `keep`, `report`, `reject` or
        /// `ignore`.
        Unknown(&'static str),
        /// `fallback = x` (compile-time, an `Option` field): the value absence stands for;
        /// the getter returns it, and it is never written into the data.
        Fallback(arbitrary),
        /// `min = x` (compile-time): the value is `>= x`; a number or another field's name.
        Min(arbitrary),
        /// `max = x` (compile-time): the value is `<= x`.
        Max(arbitrary),
        /// `above = x` (compile-time): the value is `> x`.
        Above(arbitrary),
        /// `below = x` (compile-time): the value is `< x`.
        Below(arbitrary),
        /// `multiple_of = n` (compile-time): the value is a multiple of the positive number
        /// `n`, checked after the field's range as `value % n == 0.0` in `f64`: exact for an
        /// integer step and a binary fraction (`0.5`, `0.25`), not for a step `f64` holds only
        /// near (`0.1`, where `0.5` and `1.0` are told as off the step).
        MultipleOf(arbitrary),
        /// `molang = SPEC` (compile-time): the field's `MolangFieldSpec`, where it is not the
        /// family's default.
        Molang(arbitrary),
        /// `molang_with = path` (compile-time): a hand-written Molang walk of the field,
        /// `fn(&mut T, &mut MolangVisitor<'_>)`.
        MolangWith(arbitrary),
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
    }
}

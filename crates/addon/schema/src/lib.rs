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
        /// `after_members = path` (compile-time, on a struct): `after_read` run earlier, on the
        /// value built from the members, before the unknown members a profile tells after them
        /// (and only when the value is built): the checks a game's reader runs as it binds the
        /// members, which come before the crate's word on the members it never reads.
        AfterMembers(arbitrary),
        /// `guard = path` (compile-time, on a field or a struct): a rule of the game's reader at
        /// its place in the read, `fn(&Node<'_>, &mut Read<'_>) -> bool` over the object as
        /// written. On a field it runs where the read comes to the member (in a read in declared
        /// order always, in file order where the file has it), before the member is read; on a
        /// struct after the members, before `after_members` and the unknown ones. `false` ends
        /// the struct's read there: what is still to come (members, `after_members`, unknown
        /// members, missing ones) is neither read nor told and the struct fails, with what the
        /// function told (or nothing: a value the game drops without a word). `after_read` is
        /// the rule that cannot fail it.
        Guard(arbitrary),
        /// `unknown_with = path` (compile-time, on a struct): the rule each unknown member is
        /// told and kept by, from its name, in place of the struct's `unknown` word:
        /// `fn(&str, &Node<'_>, &mut Read<'_>) -> UnknownRule`, called where the read tells the
        /// member. For a reader that tells only the names it picks (a spelling the game asserts
        /// on) and keeps the others in silence; the function may tell a line of its own at the
        /// member (a kept error for a key the game no longer reads) and return `Silent`.
        UnknownWith(arbitrary),
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
        /// `absent_with = path` (compile-time, an `Option` field): the field's value when the
        /// object lacks the member (or holds a `null` the reading takes for absence),
        /// `fn(&Node<'_>, &mut Read<'_>) -> Option<T>` given the object: what the game's reader
        /// does with a member it does not find by what it read before (a member a pack of one
        /// type must write, a default another member decides). It tells what it must; `None`
        /// leaves the field absent. Asked in the member's place where the read asks for members
        /// in declaration order, after the members where it reads them in file order.
        AbsentWith(arbitrary),
        /// `read_beside = path` (compile-time, a struct's field): the field's read given the
        /// object the member is in, `fn(&Node<'_>, &Node<'_>, &mut Read<'_>) -> Option<T>`
        /// (the member, its object, the read): `read_with` for a member whose read follows what
        /// its siblings are (a list the game skips beside its plain spelling, a sampler's
        /// keyword the game does not look up once its index is out of range).
        ReadBeside(arbitrary),
        /// `tag_with = path` (compile-time, an internally tagged enum): the name of the variant
        /// is the function's, `fn(&Node<'_>, &mut Read<'_>) -> Option<&'static str>`, which looks
        /// at the object, tells what it refuses (the game's own words for a name that is none)
        /// and returns the variant's JSON name, or `None` once it has told. In place of the
        /// string at the tag member, for a tag the game normalises (a namespace dropped, an
        /// engine that has no such name) or words its own way. The enum may be flattened into a
        /// struct (`flatten`), which reads the variant where it declares the enum. On an
        /// externally tagged enum whose every variant holds a value, flattened into a struct, the
        /// variant is a member of the struct's object, keyed by its name: the function names it,
        /// where by default the first of those members the object has and is not `null` is the
        /// one (every other one the object has told `core/ignored_payload` at the unknown severity
        /// and kept; none at all the first one missing).
        TagWith(arbitrary),
        /// `required` (marker, an `Option` field): the document must write the member, though
        /// its value is optional in the type: absence is a missing required member.
        Required,
        /// `wrong_type = CODE` (compile-time, a `CodeDef<0>` by its path): the code a value of
        /// the wrong JSON type is told under at the member, in place of the read's. This key,
        /// `missing`, `invalid` and `negative` also take a `CodeDef<1>`, fed the read's subject
        /// (`Read::subject`: the block an entry is read for, the file a document is in); a read
        /// that names none tells its own code there, and a code of more arguments does not
        /// compile. On a tuple struct, the code of a value that is no array, where the member
        /// holding it states none.
        WrongType(arbitrary),
        /// `arity = CODE` (compile-time, a tuple struct, a `CodeDef<0>` by its path): an array of
        /// another length than the struct's fields is told under CODE after its elements and
        /// read on, as the game asserts and reads its first elements: an element past the
        /// fields is not read, a missing one is absent (each field an `Option` or with a
        /// `default`). Without it another length is a wrong type.
        Arity(arbitrary),
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
        /// `ends` (marker): a member that fails to read, or a required one told missing where a
        /// read in declared order asks for it, ends the read of its struct there: nothing after
        /// it is read or told, its unknown members neither, and the struct fails, as a game reader
        /// that gives up an element at its first bad member. With `or_kept`, a value of the JSON
        /// kind the field's type takes that fails ends the read; one of another kind is kept as
        /// written.
        Ends,
        /// `if_kind` (marker, an `Option` field): the member is read only when it is of the JSON
        /// kind the field's type takes (`FromJson::EXPECTED`, `read_with` or not); of another
        /// kind it is absent, told nothing: jsoncpp's `isArray()` / `isString()` asked before a
        /// read, a member of another type ignored (dialogue's command lists and buttons).
        IfKind,
        /// `reads_null` (marker, an `Option` field): a `null` member is a value like any other,
        /// read by the field's type (or `read_with`) as `Some`, where the reading's null rule
        /// would make it `None`: the game's reader asks for the member and takes what is there
        /// (jsoncpp's `asString` of a `null` is `""`). With `or_kept`, a `null` the type refuses
        /// is kept as written. Without it a `null` is the reading's null rule's, whatever reads
        /// the member: a `read_with` fn never sees one.
        ReadsNull,
        /// `scope` (marker, a field): once its member is read (or, in a read in declaration
        /// order, found absent: the field's default), the field's value is in scope under its JSON
        /// name for the members the struct reads after it and every read below them, until the
        /// struct's read ends (`Read::scope`): the context the game's reader holds while it reads
        /// on (the range of the table a sound of it starts from, the base of a block sound type).
        Scope,
        /// `reads_absent` (marker, a field): a member the object lacks is read all the same, as
        /// the `null` jsoncpp's `operator[]` gives for a missing key, by the field's type (or
        /// `read_with`) at a node that knows it is absent (`Node::is_absent`): the game reads the
        /// member and tells what it tells of its absence. A value read is the field's; none leaves
        /// it absent (its default, `None`, or missing where it is required). In a read in declared
        /// order it is read where it is declared; in file order, after the members.
        ReadsAbsent,
        /// `or_kept = "told"` or `"silent"` (compile-time, an `Option` field): a value that
        /// fails to read leaves the field `None` and is kept as written beside the typed value
        /// (with the struct's unknown members), its failure told (or not) and failing nothing.
        /// A read that keeps the object's order (`Read::keep_in_file_order`) puts it among the
        /// struct's own unknown members where the file has it; otherwise after them.
        OrKept(arbitrary),
        /// `foreign_event` (marker): an `EventTrigger` field whose event belongs to another
        /// entity, so it is no trigger of this one.
        ForeignEvent,
        /// `read_by = "sprite text"` (compile-time, a field): the parts of the game that read the
        /// member, names separated by spaces, in the field's row (`FieldRow::read_by`): for code
        /// that asks at run time which part reads a member, as a lint of the members no part of
        /// a control reads. It names at least one, none twice. The read does not look at it.
        ReadBy(arbitrary),
        /// `getter = "bool"` (compile-time, a field): the getter the game reads the member with,
        /// by the name the family gives it, in the field's row (`FieldRow::getter`); not the
        /// accessor the derive makes for a `fallback`. The read does not look at it.
        Getter(arbitrary),
        /// `era(ERA, key = value, ..)` (compile-time, a field; repeatable): keys that hold in a
        /// document whose format version is in `ERA` (an expression of type `schema::Era`, a
        /// range of formats: `Era::below(v)`, `Era::since(v)`), in place of the field's own:
        /// one of the game's readers reads the format's documents (particle's legacy parser
        /// below 1.20.80, its strict reader from it). The first era the format is in holds; a
        /// read that knows no format (`Read::format`) is in none. The keys: `wrong_type`,
        /// `missing`, `invalid` (the codes, each the reader's line, told at the code's own
        /// severity where a field's own codes are graded by the reading; the member fails or
        /// takes its default as the reading says). A `missing` code on a field that has a
        /// `default` or is an `Option` is told where the member is absent, before the field
        /// takes its default. `molang = SPEC`: the field's Molang spec in the era, for the walk
        /// of a document of that format (`MolangVisitor::format`; a hand walk of a part of
        /// another era's document sets it with `MolangVisitor::at_format`) and for the read of
        /// the member (`Read::member_molang`).
        Era(arbitrary),
        /// `order = "game"` (compile-time, a string enum): the enum states its own `ALL`, in
        /// the game's order, and the derive emits none; it checks at compile time that `ALL`
        /// names every variant once. `NAMES` and `EXPECTED` stay in declaration order.
        Order(arbitrary),
    }
}

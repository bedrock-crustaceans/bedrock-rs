//! The generic loader constructors a family's `LOADER` is built with: [`registry`], [`files`]
//! and [`merged`], each a [`CollectionLoader`] over the family's [`Walk`].
//!
//! A loader that needs more than the constructor gives sets the field over it:
//! `CollectionLoader { needs: &["block"], ..load::registry::<Recipe>(WALK) }`.

use super::{CollectionLoader, LoadContext, StackCx, Walk, WalkFile};
use crate::document::unversioned::origin_of;
use crate::document::{
    Content, Contribution, Document, Files, FilesBuilder, FilesRule, JsonDefinition, Registry,
    RegistryBuilder, Rejected,
};
use crate::id::Id;
use crate::pack_identity::PackPath;
use crate::versioning::parse_document;

/// What one walked file of a [`files`] or [`merged`] collection parses to: the path the
/// collection keys it by (no subpack folder), and the document in its pack or its rejection.
pub type FileCopy<C> = (PackPath, Result<Document<C>, Rejected>);

/// The loader of a registry of `D`: every file `walk` reads is one definition, parsed in its
/// pack through `D`'s family ([`parse_document`]), and the definitions combine by
/// `D::combine_rule` (the stable ones) and `D::beta_rule`.
///
/// A file that fails names the identifier its payload gave (`Family::identity`), when it gave
/// one, so that a rule that holds a failed claim (`TopWinsFallback` with a notice,
/// `ClaimBeforeValidate`) sees it; under every other rule the identifier of a failed file
/// changes nothing.
///
/// The rules are items of `D` rather than arguments: the loader's `combine` is a function
/// pointer, which no argument of this function can reach.
pub const fn registry<D: JsonDefinition>(
    walk: Walk,
) -> CollectionLoader<Registry<D>, Contribution<D>> {
    CollectionLoader {
        walk,
        parse: parse_definition::<D>,
        combine: combine_definitions::<D>,
        needs: &[],
        then: None,
    }
}

/// The loader of a collection of `C` by path: every file `walk` reads is one copy, parsed by
/// [`Content::parse`] and put in its pack ([`FileCopy`]), and the top pack's copy of each path
/// is the effective one (`FilesRule::TopWins`).
pub const fn files<C: Content>(walk: Walk) -> CollectionLoader<Files<C>, FileCopy<C>> {
    CollectionLoader {
        walk,
        parse: parse_copy::<C>,
        combine: combine_files::<C>,
        needs: &[],
        then: None,
    }
}

/// The loader of a value merged from every copy `walk` reads: each parsed as [`files`] parses
/// it, then `combine` folds them, in walk order, with the whole stack in view. `M` is the
/// collection (`Merged<T>`, or the family's own).
pub const fn merged<C: Content, M>(
    walk: Walk,
    combine: fn(Vec<FileCopy<C>>, &StackCx<'_>) -> M,
) -> CollectionLoader<M, FileCopy<C>> {
    CollectionLoader {
        walk,
        parse: parse_copy::<C>,
        combine,
        needs: &[],
        then: None,
    }
}

/// One definition file of a registry walk, in its pack.
fn parse_definition<D: JsonDefinition>(file: &WalkFile, cx: &LoadContext) -> Contribution<D> {
    let (identifier, document) = parse_document::<D::Family>(origin_of(file), &file.bytes, cx);
    match document {
        Ok(document) => Contribution::Stable(document.identifier().clone(), document),
        Err(rejected) => Contribution::Rejected(identifier.map(Id::parse), rejected),
    }
}

fn combine_definitions<D: JsonDefinition>(
    parsed: Vec<Contribution<D>>,
    stack: &StackCx<'_>,
) -> Registry<D> {
    RegistryBuilder::combine(
        parsed,
        D::combine_rule(stack),
        D::beta_rule(stack),
        stack.options,
    )
}

/// One copy of a files walk, keyed by its path without a subpack folder, in its pack.
fn parse_copy<C: Content>(file: &WalkFile, cx: &LoadContext) -> FileCopy<C> {
    let origin = origin_of(file);
    let copy = match C::parse(&file.path, &file.bytes, cx) {
        Ok(document) => Ok(document.relocated(origin)),
        Err(rejected) => Err(rejected.relocated(origin)),
    };
    (file.key.clone(), copy)
}

fn combine_files<C: Content>(parsed: Vec<FileCopy<C>>, stack: &StackCx<'_>) -> Files<C> {
    FilesBuilder::combine(parsed, FilesRule::TopWins, stack.options)
}

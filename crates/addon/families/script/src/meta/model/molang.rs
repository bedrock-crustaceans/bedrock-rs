//! `metadata/molang_modules/mojang-molang-queries.json`: the Molang queries and math functions
//! the engine registers. A cross-check only: the crate's Molang facts
//! come from molangx, not from this file.

use facet::Facet;

use crate::meta::MetaError;

/// The Molang queries file.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct MolangQueriesFile {
    /// `module_type`: `molang`.
    pub module_type: String,
    /// `name`: `engine-molang`.
    pub name: String,
    /// `queries`.
    pub queries: Vec<QueryMeta>,
    /// `math_functions`.
    pub math_functions: Vec<MathFunction>,
}

impl MolangQueriesFile {
    /// Reads the file strictly: a key no field takes is an error.
    ///
    /// # Errors
    ///
    /// Returns a [`MetaError`] when the bytes are not UTF-8, are not JSON of the expected
    /// shape, nest deeper than the JSON depth limit, or hold a member no field takes.
    pub fn parse(bytes: &[u8]) -> Result<Self, MetaError> {
        crate::meta::parse::strict(bytes).map(|(file, _)| file)
    }
}

/// One query.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct QueryMeta {
    /// `name`: `query.…`.
    pub name: String,
    /// `description`.
    pub description: String,
    /// `min_args`.
    pub min_args: u8,
    /// `max_args`: absent when the query takes any number of arguments above `min_args`.
    pub max_args: Option<u8>,
    /// `return_type`: `bool`, `float`, `matrix`, `hash_type_64`, `member_array`,
    /// `actor_array` or `actor_single`.
    pub return_type: String,
    /// `version_ranges`: the Molang versions it exists in, and its query sets.
    pub version_ranges: Vec<QueryVersionRange>,
}

/// One entry of `version_ranges`.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct QueryVersionRange {
    /// `first_version`.
    pub first_version: String,
    /// `last_version`: absent while the query still exists.
    pub last_version: Option<String>,
    /// `query_sets`: `default`, `world_gen`, `tags`.
    pub query_sets: Vec<String>,
}

/// One math function.
#[derive(Clone, Debug, PartialEq, Eq, Facet)]
pub struct MathFunction {
    /// `name`: `math.…`.
    pub name: String,
    /// `description`.
    pub description: String,
    /// `min_args`.
    pub min_args: u8,
    /// `max_args`.
    pub max_args: u8,
}

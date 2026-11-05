use bedrock_addon_core::molang::{Molang, MolangFieldSpec, MolangVisitor};
use bedrock_addon_core::schema::{self, Schema};
use facet::Facet;

const SPEC: MolangFieldSpec = MolangFieldSpec::DEFAULT;

fn walk(_: &mut Option<Molang>, _: &mut MolangVisitor<'_>) {}

#[derive(Facet, Schema)]
struct Specs {
    #[facet(schema::molang = SPEC, schema::molang_with = walk)]
    both: Option<Molang>,
    #[facet(schema::molang = SPEC)]
    not_molang: Option<String>,
}

fn main() {}

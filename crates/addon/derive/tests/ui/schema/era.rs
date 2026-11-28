use bedrock_addon_core::diagnostic::{Class, CodeDef, Kind};
use bedrock_addon_core::document::UnknownMembers;
use bedrock_addon_core::schema::{self, Schema, VersionRange};
use bedrock_addon_core::version::FormatVersion;
use facet::Facet;

const OLD: VersionRange = VersionRange::below(FormatVersion::new(1, 20, 80));
static LINE: CodeDef<0> = CodeDef::new("test/line", Kind::InvalidValue, Class::KeptError, "line");

#[derive(Facet, Schema)]
struct Inner {
    v: Option<u8>,
}

#[derive(Facet, Schema)]
struct Gated {
    #[facet(flatten, schema::era(OLD, wrong_type = LINE))]
    inner: Inner,
    #[facet(schema::era(OLD, colour = LINE))]
    unknown_key: Option<u8>,
    #[facet(schema::era(OLD))]
    no_key: Option<u8>,
    #[facet(schema::era(OLD, missing = LINE, missing = LINE))]
    twice: Option<u8>,
    #[facet(schema::era(OLD, wrong_type = LINE))]
    extra: UnknownMembers,
    #[facet(schema::era(OLD, molang = bedrock_addon_core::molang::MolangFieldSpec::KEPT))]
    plain_molang: Option<u8>,
}

fn main() {}

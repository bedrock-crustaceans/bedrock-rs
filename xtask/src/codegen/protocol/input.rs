use crate::codegen::protocol::entry::DefineVersionsEntry;
use crate::codegen::protocol::kw;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Token, bracketed};

pub struct DefineVersionsInput {
    pub versions: Punctuated<DefineVersionsEntry, Token![,]>,
}

impl Parse for DefineVersionsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<kw::versions>()?;
        input.parse::<Token![!]>()?;

        let bracket;
        bracketed!(bracket in input);

        let versions = Punctuated::parse_terminated(&bracket)?;

        input.parse::<Token![;]>()?;

        Ok(DefineVersionsInput { versions })
    }
}

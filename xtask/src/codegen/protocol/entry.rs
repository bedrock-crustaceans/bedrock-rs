use crate::codegen::protocol::diff::DefineVersionsDiffList;
use crate::codegen::protocol::kw;
use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::{LitInt, LitStr, Token, braced, parenthesized};

pub struct DefineVersionsEntry {
    pub version: u32,
    pub branch: LitStr,
    pub game_version: LitStr,
    pub raknet_version: Option<u8>,
    pub packets: Option<DefineVersionsDiffList>,
    pub types: Option<DefineVersionsDiffList>,
    pub enums: Option<DefineVersionsDiffList>,
    pub ident: Option<Ident>,
}

impl Parse for DefineVersionsEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let paren;
        let brace;
        parenthesized!(paren in input);

        let version = paren.parse::<LitInt>()?.base10_parse()?;
        paren.parse::<Token![,]>()?;
        let branch = paren.parse::<LitStr>()?;
        paren.parse::<Token![,]>()?;
        let game_version = paren.parse::<LitStr>()?;

        input.parse::<Token![:]>()?;
        braced!(brace in input);

        let mut raknet_version = None;
        let mut packets = None;
        let mut types = None;
        let mut enums = None;

        while !brace.is_empty() {
            if brace.peek(kw::raknet_version) {
                brace.parse::<kw::raknet_version>()?;
                brace.parse::<Token![:]>()?;
                if raknet_version.is_some() {
                    return Err(brace.error("duplicate `raknet_version` definition"));
                }
                raknet_version = Some(brace.parse::<LitInt>()?.base10_parse()?);
            } else if brace.peek(kw::packets) {
                brace.parse::<kw::packets>()?;
                if packets.is_some() {
                    return Err(brace.error("duplicate `packets` section"));
                }
                packets = Some(brace.parse()?);
            } else if brace.peek(kw::types) {
                brace.parse::<kw::types>()?;
                if types.is_some() {
                    return Err(brace.error("duplicate `types` section"));
                }
                types = Some(brace.parse()?);
            } else if brace.peek(kw::enums) {
                brace.parse::<kw::enums>()?;
                if enums.is_some() {
                    return Err(brace.error("duplicate `enums` section"));
                }
                enums = Some(brace.parse()?);
            } else {
                return Err(
                    brace.error("expected `raknet_version`, `packets`, `types`, or `enums`")
                );
            }

            if !brace.is_empty() {
                brace.parse::<Token![,]>()?;
            }
        }

        let ident = if input.peek(Token![as]) {
            input.parse::<Token![as]>()?;
            Some(input.parse::<Ident>()?)
        } else {
            None
        };

        Ok(Self {
            version,
            branch,
            raknet_version,
            game_version,
            packets,
            types,
            enums,
            ident,
        })
    }
}

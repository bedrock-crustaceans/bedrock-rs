use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::collections::BTreeMap;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Path, Token, bracketed};

pub struct DefineVersionsDiffList {
    pub entries: Punctuated<DefineVersionsDiffEntry, Token![,]>,
    pub path: Path,
}

pub enum DefineVersionsDiffEntry {
    Added {
        ident: Ident,
        path: Path,
        versioned: bool,
    },
    Removed {
        ident: Ident,
    },
    Modified {
        ident: Ident,
        path: Path,
        versioned: bool,
    },
}

impl DefineVersionsDiffList {
    pub fn collapse(&self, map: &mut BTreeMap<Ident, TokenStream>) -> syn::Result<()> {
        for entry in &self.entries {
            let base_path = &self.path;
            match entry {
                DefineVersionsDiffEntry::Added {
                    ident,
                    path,
                    versioned,
                } => {
                    let tokens = if *versioned {
                        quote!(#base_path::#path<Self>)
                    } else {
                        quote!(#base_path::#path)
                    };
                    map.insert(ident.clone(), tokens);
                }
                DefineVersionsDiffEntry::Removed { ident } => {
                    if !map.contains_key(ident) {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("cannot remove {} because it was never added", ident),
                        ));
                    }

                    map.remove(ident);
                }
                DefineVersionsDiffEntry::Modified {
                    ident,
                    path,
                    versioned,
                } => {
                    if !map.contains_key(ident) {
                        let mut err = syn::Error::new(
                            ident.span(),
                            format!("cannot modify `{}` because it was never added", ident),
                        );

                        err.combine(syn::Error::new(ident.span(), "did you mean to use `+`?"));

                        return Err(err);
                    }

                    let tokens = if *versioned {
                        quote!(#base_path::#path<Self>)
                    } else {
                        quote!(#base_path::#path)
                    };
                    map.insert(ident.clone(), tokens);
                }
            }
        }

        Ok(())
    }
}

impl Parse for DefineVersionsDiffList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<Token![:]>()?;

        let content;
        bracketed!(content in input);
        let entries = Punctuated::parse_terminated(&content)?;

        input.parse::<Token![in]>()?;
        let path: Path = input.parse()?;

        Ok(Self { entries, path })
    }
}

impl Parse for DefineVersionsDiffEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![+]) {
            input.parse::<Token![+]>()?;

            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            Ok(Self::Added {
                ident,
                path: input.parse()?,
                versioned: input.parse::<Option<Token![^]>>()?.is_some(),
            })
        } else if input.peek(Token![%]) {
            input.parse::<Token![%]>()?;

            let ident: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            Ok(Self::Modified {
                ident,
                path: input.parse()?,
                versioned: input.parse::<Option<Token![^]>>()?.is_some(),
            })
        } else if input.peek(Token![-]) {
            input.parse::<Token![-]>()?;

            let ident: Ident = input.parse()?;

            Ok(Self::Removed { ident })
        } else {
            Err(input.error("expected one of +, %, or -"))
        }
    }
}

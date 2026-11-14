use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Path, Token, bracketed};

/// The package name of the version crate whose module ident is `module`.
pub fn crate_name(module: &impl std::fmt::Display) -> String {
    format!("bedrock_protocol_{module}")
}

/// Where the type of one packet, type or enum lives. It is kept as structure, not as a finished
/// path, because the path depends on the crate it is written into.
#[derive(Clone)]
pub struct Item {
    /// The module ident of the version crate that owns the type (`v662`).
    pub owner: Ident,
    /// The module path inside the owner crate (`packets`).
    pub module: Vec<Ident>,
    /// The path of the type inside that module.
    pub path: Path,
    /// Whether the type is generic over the version (`^`).
    pub versioned: bool,
}

impl Item {
    /// The path of the type as the crate of the version `current` writes it: `crate::…` in the
    /// owner, `bedrock_protocol_<owner>::…` anywhere else, with `<Self>` when it is generic.
    pub fn path_in(&self, current: &Ident) -> TokenStream {
        let Self {
            owner,
            module,
            path,
            versioned,
        } = self;
        let root = if owner == current {
            quote!(crate)
        } else {
            let name = format_ident!("{}", crate_name(owner));
            quote!(#name)
        };
        let generic = versioned.then(|| quote!(<Self>));
        quote!(#root #(::#module)* :: #path #generic)
    }
}

pub struct DefineVersionsDiffList {
    pub entries: Punctuated<DefineVersionsDiffEntry, Token![,]>,
    /// The `in` path: the module ident of the owning version crate, then the module path inside
    /// that crate.
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
    /// The module ident of the version crate that owns the items of this list: the first segment
    /// of the `in` path.
    pub fn owner(&self) -> &Ident {
        // A parsed `syn::Path` has at least one segment.
        &self.path.segments[0].ident
    }

    /// The module path inside the owner crate: the `in` path after its first segment.
    pub fn module(&self) -> Vec<Ident> {
        let segments = self.path.segments.iter().skip(1);
        segments.map(|segment| segment.ident.clone()).collect()
    }

    fn item(&self, path: &Path, versioned: bool) -> Item {
        Item {
            owner: self.owner().clone(),
            module: self.module(),
            path: path.clone(),
            versioned,
        }
    }

    /// Applies this version's `+`, `%` and `-` entries to `map`.
    ///
    /// # Errors
    ///
    /// Returns an error if an entry removes or modifies an item that was never added, or adds an
    /// item that already exists.
    pub fn collapse(&self, map: &mut BTreeMap<Ident, Item>) -> syn::Result<()> {
        for entry in &self.entries {
            match entry {
                DefineVersionsDiffEntry::Added {
                    ident,
                    path,
                    versioned,
                } => {
                    map.insert(ident.clone(), self.item(path, *versioned));
                }
                DefineVersionsDiffEntry::Removed { ident } => {
                    if !map.contains_key(ident) {
                        return Err(syn::Error::new(
                            ident.span(),
                            format!("cannot remove {ident} because it was never added"),
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
                            format!("cannot modify `{ident}` because it was never added"),
                        );

                        err.combine(syn::Error::new(ident.span(), "did you mean to use `+`?"));

                        return Err(err);
                    }

                    map.insert(ident.clone(), self.item(path, *versioned));
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

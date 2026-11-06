//! `version_chain!`: a family's history enum (`history::<Name>History`) and its `VersionChain`
//! impl.

/// Emits a family's chain: the history enum (`<Name>History`, one variant per struct, oldest first, then the beta
/// struct), its [`VersionChain`](crate::versioning::VersionChain) impl (`VERSIONS`, `HAS_BETA`,
/// `parse` by the range model, `parse_struct` dispatch with Molang pass 1, the walk Molang
/// pass 2 takes, `upgrade_to_end` through the adjacent [`Upgrade`](crate::versioning::Upgrade)
/// impls) and two inherent methods: `parsed_version(&self) -> FormatVersion` (the struct's
/// version; named so that `Document<history::GlowHistory>`'s own `version` does not hide it) and
/// `to_json(&self) -> Result<String, EncodeError>` (the document in the struct's own version,
/// envelope included).
///
/// The component table is `#[derive(Components)]`'s (`<Enum>::TABLE`, returned by
/// `Family::components`); the chain only names structs. Every struct implements
/// [`VersionStruct`](crate::versioning::VersionStruct); `molang = none;` also emits an empty
/// `MolangFields` impl for each of them (for chains without Molang fields).
///
/// The stable list may be empty (a beta-only family, `Latest = NoStable`); `beta` is optional
/// (`Family::Beta = Infallible`). The last stable struct must be `Family::Latest`, and each
/// struct's `Upgrade::Next` the following one; the compiler checks both.
///
/// ```
/// # use bedrock_addon_core::diagnostic::Diagnostics;
/// # use bedrock_addon_core::document::Extra;
/// # use bedrock_addon_core::json::{EncodeError, JsonSlice, Value};
/// # use bedrock_addon_core::load::LoadContext;
/// # use bedrock_addon_core::version::FormatVersion;
/// use bedrock_addon_core::versioning::{
///     ComponentTable, Family, Loaded, NoComponents, ParseCx, Upgrade, UpgradeCx, VersionStruct,
///     Versioned, load, parse_payload, write_payload,
/// };
/// use facet::Facet;
///
/// /// The family.
/// pub struct Glow;
///
/// pub mod history {
///     use super::*;
///
///     /// 1.0.0: `{ "level": n }`.
///     #[derive(Facet)]
///     pub struct V1 {
///         pub level: u8,
///         #[facet(skip, rename = "\u{0}extra")]
///         pub extra: Extra,
///     }
///
///     bedrock_addon_core::version_chain! {
///         family = Glow;
///         /// The struct the file's `format_version` selected.
///         pub enum GlowHistory {
///             V1_0_0(V1),
///             V1_1_0(super::GlowDoc),
///         }
///         molang = none;
///     }
/// }
///
/// /// 1.1.0, the newest: `{ "brightness": n }`.
/// #[derive(Facet)]
/// pub struct GlowDoc {
///     pub brightness: u8,
///     #[facet(skip, rename = "\u{0}extra")]
///     pub extra: Extra,
/// }
///
/// impl Versioned for history::V1 {
///     const VERSION: FormatVersion = FormatVersion::new(1, 0, 0);
/// }
/// impl Versioned for GlowDoc {
///     const VERSION: FormatVersion = FormatVersion::new(1, 1, 0);
/// }
/// impl VersionStruct for history::V1 {
///     fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
///         parse_payload(payload, cx, d)
///     }
///     fn write(&self) -> Result<Value, EncodeError> {
///         write_payload(self)
///     }
/// }
/// impl VersionStruct for GlowDoc {
///     fn parse(payload: JsonSlice<'_>, cx: &ParseCx<'_>, d: &mut Diagnostics) -> Option<Self> {
///         parse_payload(payload, cx, d)
///     }
///     fn write(&self) -> Result<Value, EncodeError> {
///         write_payload(self)
///     }
/// }
///
/// /// The one step: `level` is renamed `brightness`.
/// impl Upgrade for history::V1 {
///     type Next = GlowDoc;
///     fn upgrade(self, _: &mut UpgradeCx<'_>) -> GlowDoc {
///         GlowDoc { brightness: self.level, extra: self.extra }
///     }
/// }
///
/// impl Family for Glow {
///     const NAME: &'static str = "glow";
///     const PAYLOAD_KEY: &'static str = "demo:glow";
///     const FLOOR: FormatVersion = FormatVersion::new(1, 0, 0);
///     type Latest = GlowDoc;
///     type Beta = std::convert::Infallible;
///     type Component = NoComponents;
///     type History = history::GlowHistory;
///     fn components() -> &'static ComponentTable<Self> {
///         &ComponentTable::EMPTY
///     }
/// }
///
/// # fn main() {
/// let text = r#"{"format_version": "1.0.0", "demo:glow": {"level": 7}}"#;
/// let outcome = load::<Glow>(JsonSlice::new(text), &LoadContext::detached());
/// let Some(Loaded::Stable(glow)) = outcome.value else { panic!() };
/// assert_eq!(glow.brightness, 7);
/// assert!(outcome.version.upgraded);
/// # }
/// ```
// hidden: macro support: named by the expansion of the exported macro
#[doc(hidden)]
#[macro_export]
macro_rules! version_chain {
    (
        family = $family:ty;
        $(#[$meta:meta])*
        $vis:vis enum $any:ident {
            $( $(#[$vmeta:meta])* $variant:ident ( $ty:ty ) ),* $(,)?
        }
        $( beta = $(#[$bmeta:meta])* $beta:ident ( $beta_ty:ty ); )?
        $( molang = $molang:ident; )?
    ) => {
        $(#[$meta])*
        $vis enum $any {
            $( $(#[$vmeta])* $variant($ty), )*
            $( $(#[$bmeta])* $beta($beta_ty), )?
        }

        impl $crate::versioning::VersionChain<$family> for $any {
            const VERSIONS: &'static [$crate::version::FormatVersion] =
                &[ $( <$ty as $crate::versioning::Versioned>::VERSION ),* ];
            const HAS_BETA: bool = $crate::version_chain!(@has_beta $($beta)?);

            fn parse_struct(
                payload: $crate::json::JsonSlice<'_>,
                index: $crate::versioning::StructIndex,
                cx: &$crate::versioning::ParseCx<'_>,
                d: &mut $crate::diagnostic::Diagnostics,
            ) -> ::core::option::Option<Self> {
                match index {
                    $crate::versioning::StructIndex::Stable(i) => {
                        let v = *<Self as $crate::versioning::VersionChain<$family>>::VERSIONS
                            .get(i)?;
                        $(
                            if v == <$ty as $crate::versioning::Versioned>::VERSION {
                                return $crate::versioning::__macro::parse_with::<$ty>(payload, cx, d)
                                    .map(Self::$variant);
                            }
                        )*
                        let _ = (v, payload, cx, d);
                        ::core::option::Option::None
                    }
                    $crate::versioning::StructIndex::Beta => {
                        $crate::version_chain!(@parse_beta payload cx d $($beta $beta_ty)?)
                    }
                }
            }

            fn parse(
                payload: $crate::json::JsonSlice<'_>,
                v: $crate::version::FormatVersion,
                cx: &$crate::load::LoadContext,
                d: &mut $crate::diagnostic::Diagnostics,
            ) -> ::core::option::Option<Self> {
                $crate::versioning::__macro::parse_selected::<$family>(payload, v, cx, d)
            }

            fn with_molang(
                loaded: &mut $crate::versioning::Loaded<$family>,
                visit: &mut dyn ::core::ops::FnMut(&mut dyn $crate::molang::MolangFields),
            ) {
                match loaded {
                    $crate::versioning::Loaded::Stable(value) => visit(value),
                    $crate::versioning::Loaded::Beta(value) => visit(value),
                }
            }

            fn upgrade_to_end(
                self,
                cx: &mut $crate::versioning::UpgradeCx<'_>,
            ) -> $crate::versioning::Loaded<$family> {
                // One step at a time, each in a frame of its own, the value on the heap between
                // them; the walk's frame is the same whatever the length of the chain.
                #[allow(unused_mut, reason = "a chain of one struct has no step")]
                let mut current = ::std::boxed::Box::new(self);
                $crate::version_chain!(@walk_loop current cx [] [ $( ($variant $ty) )* ] [ $($beta)? ])
            }
        }

        impl $any {
            /// The version of the struct that parsed the file ([`BETA_VERSION`] for the beta
            /// struct).
            ///
            /// [`BETA_VERSION`]: $crate::versioning::BETA_VERSION
            #[allow(dead_code)]
            $vis fn parsed_version(&self) -> $crate::version::FormatVersion {
                match self {
                    $( Self::$variant(_) => <$ty as $crate::versioning::Versioned>::VERSION, )*
                    $( Self::$beta(_) => $crate::versioning::BETA_VERSION, )?
                }
            }

            /// The document in the struct's own version (round trip without upgrade).
            #[allow(dead_code)]
            $vis fn to_json(&self) -> ::core::result::Result<::std::string::String, $crate::json::EncodeError> {
                let payload = match self {
                    $( Self::$variant(value) => $crate::versioning::VersionStruct::write(value)?, )*
                    $( Self::$beta(value) => $crate::versioning::VersionStruct::write(value)?, )?
                };
                $crate::versioning::__macro::document_json::<$family>(self.parsed_version(), &payload)
            }
        }

        impl $crate::document::WriteWith for $any {
            fn write_with(
                &self,
                unknown: &$crate::document::Unknown,
                out: &mut ::std::vec::Vec<u8>,
                style: $crate::json::WriteStyle,
            ) -> ::core::result::Result<::std::vec::Vec<$crate::diagnostic::Diagnostic>, $crate::json::EncodeError> {
                let payload = match self {
                    $( Self::$variant(value) => $crate::versioning::VersionStruct::write_with_unknown(value, unknown)?, )*
                    $( Self::$beta(value) => $crate::versioning::VersionStruct::write_with_unknown(value, unknown)?, )?
                };
                $crate::versioning::__macro::write_document_with::<$family>(self.parsed_version(), payload, unknown, out, style)
            }
        }

        $crate::version_chain!(@molang [ $($molang)? ] [ $($ty,)* $($beta_ty,)? ]);
    };

    (@has_beta) => { false };
    (@has_beta $beta:ident) => { true };

    (@parse_beta $payload:ident $cx:ident $d:ident) => {{
        let _ = ($payload, $cx, $d);
        ::core::option::Option::None
    }};
    (@parse_beta $payload:ident $cx:ident $d:ident $beta:ident $beta_ty:ty) => {
        $crate::versioning::__macro::parse_with::<$beta_ty>($payload, $cx, $d).map(Self::$beta)
    };

    // The walk: a loop over one match on the boxed value, with one arm per stable struct. The
    // arm of every struct but the last hands the box to `__macro::step`, which upgrades the
    // value one version in its own frame; the arm of the last struct and the beta arm return.
    // A match arm per starting struct with one `let` per later step kept n(n-1)/2 values in
    // one frame, and an arm that passes the value itself keeps two per struct.
    (@walk_loop $current:ident $cx:ident [ $($arms:tt)* ] [] [ $($beta:ident)? ]) => {
        loop {
            match *$current {
                $($arms)*
                $( Self::$beta(value) => return $crate::versioning::Loaded::Beta(value), )?
            }
        }
    };
    (@walk_loop $current:ident $cx:ident [ $($arms:tt)* ] [ ($v:ident $t:ty) ] [ $($beta:ident)? ]) => {
        $crate::version_chain!(@walk_loop $current $cx [
            $($arms)*
            Self::$v(value) => return $crate::versioning::Loaded::Stable(value),
        ] [] [ $($beta)? ])
    };
    (@walk_loop $current:ident $cx:ident [ $($arms:tt)* ]
        [ ($v:ident $t:ty) ($vn:ident $tn:ty) $( ($vr:ident $tr:ty) )* ] [ $($beta:ident)? ]
    ) => {
        $crate::version_chain!(@walk_loop $current $cx [
            $($arms)*
            Self::$v(_) => {
                $current = $crate::versioning::__macro::step::<Self, $t>(
                    $current,
                    |any| match any {
                        Self::$v(value) => ::core::result::Result::Ok(value),
                        other => ::core::result::Result::Err(::std::boxed::Box::new(other)),
                    },
                    Self::$vn,
                    &mut *$cx,
                );
            }
        ] [ ($vn $tn) $( ($vr $tr) )* ] [ $($beta)? ])
    };

    (@molang [] [ $($ty:ty,)* ]) => {};
    (@molang [ none ] [ $($ty:ty,)* ]) => {
        $( $crate::impl_no_molang!($ty); )*
    };
}

//! A module's `type` and the pack type it declares.

use std::fmt;

use crate::pack_identity::PackType;

/// A module's `type` (`data`, `resources`, `script`, `client_script`, `skin_pack`, …): one of
/// the sixteen names vanilla's module-type table knows, kept as written.
///
/// Every name maps to a [`PackType`] ([`ModuleType::pack_type`]); the last module of a manifest
/// decides the pack's type. Names are matched exactly: `javascript`, `Data` or any other name
/// outside the table is an invalid module type.
#[doc(alias = "type")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ModuleType {
    /// `invalid`.
    Invalid,
    /// `addon`.
    Addon,
    /// `data`: a behaviour pack.
    Data,
    /// `plugin`: a behaviour pack.
    Plugin,
    /// `client_data`: a behaviour pack.
    ClientData,
    /// `interface`: a behaviour pack.
    Interface,
    /// `behaviorpack`: a behaviour pack.
    BehaviorPack,
    /// `script`: a behaviour pack's script module.
    Script,
    /// `resourcepack`: a resource pack.
    ResourcePack,
    /// `resources`: a resource pack.
    Resources,
    /// `client_script`: a resource pack's script module.
    ClientScript,
    /// `world_template`.
    WorldTemplate,
    /// `worldtemplate`.
    Worldtemplate,
    /// `skin_pack`.
    SkinPack,
    /// `skinpack`.
    Skinpack,
    /// `persona_piece`.
    PersonaPiece,
}

impl ModuleType {
    /// Every module type, in the order of vanilla's table.
    pub const ALL: [ModuleType; 16] = [
        Self::Invalid,
        Self::Addon,
        Self::Data,
        Self::Plugin,
        Self::ClientData,
        Self::Interface,
        Self::BehaviorPack,
        Self::Script,
        Self::ResourcePack,
        Self::Resources,
        Self::ClientScript,
        Self::WorldTemplate,
        Self::Worldtemplate,
        Self::SkinPack,
        Self::Skinpack,
        Self::PersonaPiece,
    ];

    /// The module type spelled `name`, matched exactly; `None` for a name outside the table.
    #[must_use]
    pub fn from_name(name: &str) -> Option<ModuleType> {
        Self::ALL.into_iter().find(|ty| ty.name() == name)
    }

    /// The JSON spelling.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Invalid => "invalid",
            Self::Addon => "addon",
            Self::Data => "data",
            Self::Plugin => "plugin",
            Self::ClientData => "client_data",
            Self::Interface => "interface",
            Self::BehaviorPack => "behaviorpack",
            Self::Script => "script",
            Self::ResourcePack => "resourcepack",
            Self::Resources => "resources",
            Self::ClientScript => "client_script",
            Self::WorldTemplate => "world_template",
            Self::Worldtemplate => "worldtemplate",
            Self::SkinPack => "skin_pack",
            Self::Skinpack => "skinpack",
            Self::PersonaPiece => "persona_piece",
        }
    }

    /// The pack type the module declares.
    #[must_use]
    pub fn pack_type(self) -> PackType {
        // Vanilla's table maps each module type to a pack type.
        match self {
            Self::Invalid => PackType::Invalid,
            Self::Addon => PackType::Addon,
            Self::Data
            | Self::Plugin
            | Self::ClientData
            | Self::Interface
            | Self::BehaviorPack
            | Self::Script => PackType::Behavior,
            Self::ResourcePack | Self::Resources | Self::ClientScript => PackType::Resource,
            Self::WorldTemplate | Self::Worldtemplate => PackType::WorldTemplate,
            Self::SkinPack | Self::Skinpack => PackType::Skins,
            Self::PersonaPiece => PackType::PersonaPiece,
        }
    }

    /// The module type a new manifest of `pack_type` gets: `data`, `resources`, `skin_pack`,
    /// `world_template`, `persona_piece`, `addon`, else `invalid`.
    #[must_use]
    pub fn for_pack_type(pack_type: PackType) -> ModuleType {
        match pack_type {
            PackType::Behavior => Self::Data,
            PackType::Resource => Self::Resources,
            PackType::Skins => Self::SkinPack,
            PackType::WorldTemplate => Self::WorldTemplate,
            PackType::PersonaPiece => Self::PersonaPiece,
            PackType::Addon => Self::Addon,
            _ => Self::Invalid,
        }
    }
}

/// The JSON spelling.
impl fmt::Display for ModuleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

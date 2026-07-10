use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::version::AddonSemanticVersion;

/// The manifest file contains all the basic information about the pack that Minecraft needs to identify it.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddonManifest {
    /// The syntax version used in the manifest file.
    /// This may be one for skin packs or 2 for resource, behavior, and world templates.
    pub format_version: u32,
    /// Section containing information regarding the name of the pack, description, and other features that are public facing.
    pub header: AddonManifestHeader,
    /// Section containing information regarding the type of content that is being brought in.
    pub modules: Vec<AddonManifestModule>,
    /// Section containing definitions for any other packs that are required in order for this manifest.json file to work.
    #[serde(default)]
    pub dependencies: Vec<AddonManifestDependency>,
    /// Section containing optional features that can be enabled in Minecraft.
    pub capabilities: Option<Vec<String>>,
    /// Section containing the metadata about the file such as authors and licensing information.
    pub metadata: Option<AddonManifestMetadata>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Section containing information regarding the name of the pack, description, and other features that are public facing.
pub struct AddonManifestHeader {
    /// This is the name of the pack as it appears within Minecraft. This is a required field.
    pub name: String,
    /// This is a short description of the pack. It will appear in the game below the name of the pack. We recommend keeping it to 1-2 lines.
    pub description: Option<String>,
    /// This is the version of your pack in the format [majorVersion, minorVersion, revision]. The version number is used when importing a pack that has been imported before. The new pack will replace the old one if the version is higher, and ignored if it's the same or lower.
    pub version: AddonSemanticVersion,
    /// This is a special type of identifier that uniquely identifies this pack from any other pack. UUIDs are written in the format xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx where each x is a hexadecimal value (0-9 or a-f). We recommend using an online service to generate this and guarantee their uniqueness.
    pub uuid: Uuid,
    /// This option will generate a random seed every time a template is loaded and allow the player to change the seed before creating a new world. (world template manifest JSON only)
    pub allow_random_seed: Option<bool>,
    /// This option is required for any world templates. This will lock the player from modifying the options of the world. (world template manifest JSON only)
    pub lock_template_options: Option<bool>,
    /// For resource packs, an optional string that specifies whether this resource pack can be used across the game or at an individual world level. Valid values are "world", which specifies that a pack is only addable in the context of a world "global", which means that a pack is only addable across the game, and "any" which indicates that a pack can apply either across the game or to a specific world. If not specified, this is interpreted as "any".
    pub pack_scope: Option<String>,
    /// This is the version of the base game your world template requires, specified as [majorVersion, minorVersion, revision]. We use this to determine what version of the base game resource and behavior packs to apply when your content is used. (world template manifest JSON only)
    pub base_game_version: Option<[u32; 3]>,
    /// This is the minimum version of the game that this pack was written for. This is a required field for resource and behavior packs. This helps the game identify whether any backwards compatibility is needed for your pack. You should always use the highest version currently available when creating packs.
    pub min_engine_version: Option<[u32; 3]>,
}

/// Section containing information regarding the type of content that is being brought in.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddonManifestModule {
    /// This is a short description of the module. This is not user-facing at the moment but is a good place to remind yourself why the module is defined.
    pub description: Option<String>,
    /// This is the type of the module. Can be any of the following: `resources`, `data`, `world_template` or `script`.
    #[serde(rename = "type")]
    pub module_type: String,
    /// This is a unique identifier for the module in the same format as the pack's UUID in the header. This should be different from the pack's UUID, and different for every module.
    pub uuid: Uuid,
    /// This is the version of the module in the same format as the pack's version in the header. This can be used to further identify changes in your pack.
    pub version: AddonSemanticVersion,
    /// Only present if `type`(`module_type`) is `script`. This indicates the language in which scripts are written in the pack. The only supported value is `javascript`.
    pub language: Option<String>,
}

/// Section containing definitions for any other packs that are required in order for this manifest.json file to work.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddonManifestDependency {
    /// For dependencies on built-in scripting modules, contains the name of the module. (for example, @minecraft/server)
    #[serde(rename = "module_name")]
    pub name: Option<String>,
    /// This is the unique identifier of the pack that this pack depends on. It needs to be the exact same UUID that the pack has defined in the header section of its manifest file.
    pub uuid: Option<Uuid>,
    /// This is the specific version of the pack that your pack depends on. Should match the version the other pack has in its manifest file.
    pub version: AddonSemanticVersion,
}

/// Section containing the metadata about the file such as authors and licensing information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddonManifestMetadata {
    /// Name of the pack authors.
    pub authors: Option<Vec<String>>,
    /// The license of the pack.
    pub license: Option<String>,
    /// This is used to identify tools used to generate a manifest.json file. The tool names are strings that must be [a-zA-Z0-9_-] and 32 characters maximum. The tool version number are semver strings for each version that modified the manifest.json file.
    pub generated_with: Option<serde_json::Value>,
    /// This optional string is used to identify a targeted context for this pack. The only supported value is "addon", indicating this pack is intended to be added to players' worlds. Setting product_type to "addon" should not change how the pack functions in-game.
    pub product_type: Option<String>,
    /// The home website of your pack.
    pub url: Option<String>,
}

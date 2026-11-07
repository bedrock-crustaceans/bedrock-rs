//! The `aim_assist` family's codes: the game's content-log lines, byte for byte, each a definition
//! beside the rules that raise it. Each id is `aim_assist/<name>`, the name its constant's in lower
//! case.
//!
//! The game's spelling is kept. The cereal texts are in `camera::shape`.

use bedrock_addon_core::diagnostic::{Class, CodeDef, CodeMeta, Kind};

/// A1: a preset `identifier` without a namespace. The argument is the id.
pub static PRESET_NAMESPACE: CodeDef<1> = CodeDef::new(
    "aim_assist/preset_namespace",
    Kind::InvalidValue,
    Class::Error,
    "preset id {} has no namespace",
)
.game("Namespace required for preset Id: {}");

/// A2: a preset `identifier` that is not `namespace:name`. The argument is the id.
pub static PRESET_FORMAT: CodeDef<1> = CodeDef::new(
    "aim_assist/preset_format",
    Kind::InvalidValue,
    Class::Error,
    "preset id {} is no namespace:name",
)
.game("Failed to add aim assist preset with invalid namespace Id: {}");

/// A3: a category reference without a namespace. The argument is the id.
pub static CATEGORY_ID_NAMESPACE: CodeDef<1> = CodeDef::new(
    "aim_assist/category_id_namespace",
    Kind::InvalidValue,
    Class::Error,
    "category id {} has no namespace",
)
.game("Namespace required for category Id: {}");

/// A4: a category reference that is not `namespace:name`. The argument is the id.
pub static CATEGORY_ID_FORMAT: CodeDef<1> = CodeDef::new(
    "aim_assist/category_id_format",
    Kind::InvalidValue,
    Class::Error,
    "category id {} is no namespace:name",
)
.game("Invalid namespace format for category Id: {}");

/// A5: a category `name` without a namespace. The argument is the id.
pub static CATEGORY_NAMESPACE: CodeDef<1> = CodeDef::new(
    "aim_assist/category_namespace",
    Kind::InvalidValue,
    Class::Error,
    "category {} has no namespace",
)
.game("Namespace required for category: {}");

/// A6: a category `name` that is not `namespace:name`. The argument is the id.
pub static CATEGORY_FORMAT: CodeDef<1> = CodeDef::new(
    "aim_assist/category_format",
    Kind::InvalidValue,
    Class::Error,
    "category {} is no namespace:name",
)
.game("Invalid namespace format for category: {}");

/// Every definition of this module.
pub static ALL: &[&CodeMeta] = &[
    PRESET_NAMESPACE.meta(),
    PRESET_FORMAT.meta(),
    CATEGORY_ID_NAMESPACE.meta(),
    CATEGORY_ID_FORMAT.meta(),
    CATEGORY_NAMESPACE.meta(),
    CATEGORY_FORMAT.meta(),
];

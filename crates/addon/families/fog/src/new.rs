//! Constructors and the inherent helpers of the fog types: what the game makes of a value
//
// A present distance entry starts from `FogSetting::ZERO_SETTING` and takes each written member.

use super::{
    ColorNorm, DistanceType, FogColor, FogDescription, FogDistanceSetting, FogSettings,
    FogWaterDistanceSetting, Keyframed,
};
use bedrock_addon_core::document::Extra;
use bedrock_addon_core::id::FogId;

/// The three bytes of `#RRGGBB`; 0 for a digit pair that is no hex.
fn hex_bytes(text: &str) -> [u8; 3] {
    let digits = text.strip_prefix('#').unwrap_or(text);
    [0, 2, 4].map(|start| {
        digits
            .get(start..start + 2)
            .and_then(|pair| u8::from_str_radix(pair, 16).ok())
            .unwrap_or(0)
    })
}

impl FogSettings {
    /// Settings named `identifier` that set nothing.
    #[must_use]
    pub fn new(identifier: FogId) -> Self {
        Self {
            description: FogDescription::new(identifier),
            distance: None,
            volumetric: None,
            extra: Extra::default(),
        }
    }

    /// The identifier's namespace as the game takes it: the lower-cased text before the first
    /// `:`, or the whole lower-cased identifier when it has none.
    #[must_use]
    pub fn namespace(&self) -> String {
        let identifier = self.description.identifier.as_str();
        identifier
            .split(':')
            .next()
            .unwrap_or(identifier)
            .to_ascii_lowercase()
    }
}

impl FogDescription {
    /// A description with `identifier`.
    #[must_use]
    pub fn new(identifier: FogId) -> Self {
        Self {
            identifier,
            extra: Extra::default(),
        }
    }
}

impl DistanceType {
    /// The JSON spelling: `"fixed"` or `"render"`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixed => "fixed",
            Self::Render => "render",
        }
    }
}

impl FogColor {
    /// The colour as the game uses it: red, green and blue from 0 to 1 (each byte over 255).
    #[must_use]
    pub fn rgb(&self) -> [f32; 3] {
        let bytes = match self {
            Self::Hex(text) => hex_bytes(text),
            Self::Rgb255(rgb) => *rgb,
        };
        bytes.map(|c| f32::from(c) / 255.0)
    }
}

impl ColorNorm {
    /// The colour as the game uses it: each component clamped to 0 to 1.
    #[must_use]
    pub fn rgb(&self) -> [f32; 3] {
        match self {
            Self::Hex(text) => hex_bytes(text).map(|c| f32::from(c) / 255.0),
            Self::Array(rgb) => rgb.map(|c| c.clamp(0.0, 1.0)),
        }
    }
}

impl FogDistanceSetting {
    /// `fog_start`; 0 when the setting leaves it out.
    #[must_use]
    pub fn fog_start(&self) -> f32 {
        self.fog_start.unwrap_or(0.0)
    }

    /// `fog_end`; 0 when the setting leaves it out.
    #[must_use]
    pub fn fog_end(&self) -> f32 {
        self.fog_end.unwrap_or(0.0)
    }

    /// `render_distance_type`; `fixed` when the setting leaves it out.
    #[must_use]
    pub fn render_distance_type(&self) -> DistanceType {
        self.render_distance_type.unwrap_or_default()
    }
}

impl FogWaterDistanceSetting {
    /// `fog_start`; 0 when the setting leaves it out.
    #[must_use]
    pub fn fog_start(&self) -> f32 {
        self.fog_start.unwrap_or(0.0)
    }

    /// `fog_end`; 0 when the setting leaves it out.
    #[must_use]
    pub fn fog_end(&self) -> f32 {
        self.fog_end.unwrap_or(0.0)
    }

    /// `render_distance_type`; `fixed` when the setting leaves it out.
    #[must_use]
    pub fn render_distance_type(&self) -> DistanceType {
        self.render_distance_type.unwrap_or_default()
    }
}

impl<T> Keyframed<T> {
    /// The one value, when the member is not keyed by time.
    pub fn single(&self) -> Option<&T> {
        match self {
            Self::Single(value) => Some(value),
            Self::Keys(_) => None,
        }
    }

    /// Every value the member holds, with its time key (`None` for a single value).
    pub fn values(&self) -> Vec<(Option<f32>, &T)> {
        match self {
            Self::Single(value) => vec![(None, value)],
            Self::Keys(keys) => keys
                .iter()
                .map(|(time, value)| (Some(*time), value))
                .collect(),
        }
    }
}

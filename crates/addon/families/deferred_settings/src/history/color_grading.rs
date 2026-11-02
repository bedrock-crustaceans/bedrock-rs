//! Colour-grading settings as written at format 1.21.40: without
//! `color_grading.temperature`, which the step to 1.21.90 inserts switched off.

use crate::ColorGradingSettings;

/// Format 1.21.40.
pub mod v1_21_40 {
    use crate::{ColorGradingDescription, Highlights, Midtones, Shadows, ToneMapping};
    use bedrock_addon_core::document::Extra;

    /// Colour-grading settings at format 1.21.40.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct ColorGradingSettings {
        /// The settings' name (JSON `description`).
        pub description: Option<ColorGradingDescription>,
        /// The grading controls (JSON `color_grading`).
        pub color_grading: Option<ColorGrading>,
        /// The tone mapper (JSON `tone_mapping`).
        pub tone_mapping: Option<ToneMapping>,
        /// Members the game does not know, kept for write-back.
        pub extra: Extra,
    }

    /// The grading controls at format 1.21.40: without `temperature`.
    #[derive(Clone, Debug, Default, PartialEq)]
    pub struct ColorGrading {
        /// The controls of the mid tones (JSON `midtones`).
        pub midtones: Option<Midtones>,
        /// The controls of the bright tones (JSON `highlights`).
        pub highlights: Option<Highlights>,
        /// The controls of the dark tones (JSON `shadows`).
        pub shadows: Option<Shadows>,
        /// Members the game does not know, kept for write-back.
        pub extra: Extra,
    }
}

chain! {
    latest = ColorGradingSettings;
    /// A colour-grading file parsed by the struct its `format_version` selects, not upgraded.
    enum Any {
        /// Format 1.21.40.
        V1_21_40(v1_21_40::ColorGradingSettings),
        /// Format 1.21.90: the newest colour-grading settings.
        V1_21_90(ColorGradingSettings),
    }
}

use facet::Facet;

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct SculkSensor {
    #[facet(rename = "VibrationListener")]
    pub vibration_listener: VibrationListener,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct PendingVibration {
    pub distance: f32,
    pub source: i64,
    pub vibration: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct VibrationListener {
    pub event: Option<i32>,
    pub pending: Option<PendingVibration>,
    pub selector: nbtx::Value,
    pub ticks: Option<i32>,
}

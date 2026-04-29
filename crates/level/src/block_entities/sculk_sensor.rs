#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SculkSensor {
    #[serde(rename = "VibrationListener")]
    pub vibration_listener: VibrationListener,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PendingVibration {
    pub distance: f32,
    pub source: i64,
    pub vibration: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VibrationListener {
    pub event: Option<i32>,
    pub pending: Option<PendingVibration>,
    pub selector: nbtx::Value,
    pub ticks: Option<i32>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PistonState {
    Retracted = 0,
    Pushing = 1,
    Extended = 2,
    Pulling = 3,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PistonProgress {
    None,
    Half,
    Full,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Piston {
    /// Not sure how this is actually stored. Will need to create a custom type for this for clarity.
    pub attached_blocks: Vec<i32>,
    /// Same for this field.
    pub break_blocks: Vec<i32>,
    pub last_progress: PistonProgress,
    pub new_state: PistonState,
    pub progress: PistonProgress,
    pub state: PistonState,
    pub sticky: bool,
}

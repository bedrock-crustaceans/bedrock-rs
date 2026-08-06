use facet::Facet;


/// Stored as a `Byte` tag holding the numeric state.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Facet)]
#[facet(nbtx::variant_as(i8))]
#[repr(i8)]
pub enum PistonState {
    Retracted = 0,
    Pushing = 1,
    Extended = 2,
    Pulling = 3,
}

/// How far through its stroke a piston arm is.
///
/// The three positions a `Float` tag ever holds here. nbtx can only carry an
/// enum as a string or an integer, never a float, so this is no longer the type
/// of the fields themselves — [`PistonArm::progress`] and
/// [`PistonArm::last_progress`] hold the raw `f32` and convert through here.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum PistonProgress {
    None,
    Half,
    Full,
}

impl From<PistonProgress> for f32 {
    fn from(progress: PistonProgress) -> f32 {
        match progress {
            PistonProgress::None => 0.0,
            PistonProgress::Half => 0.5,
            PistonProgress::Full => 1.0,
        }
    }
}

impl TryFrom<f32> for PistonProgress {
    type Error = f32;

    /// Errors with the value itself if it is none of `0.0`, `0.5` or `1.0`.
    fn try_from(value: f32) -> Result<Self, f32> {
        Ok(match value {
            0.0 => Self::None,
            0.5 => Self::Half,
            1.0 => Self::Full,
            other => return Err(other),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Facet)]
#[facet(rename_all = "PascalCase")]
#[cfg_attr(not(feature = "deny-unknown-fields"), facet(nbtx::allow_unknown_fields))]
pub struct PistonArm {
    /// Not sure how this is actually stored. Will need to create a custom type for this for clarity.
    pub attached_blocks: Vec<i32>,
    /// Same for this field.
    pub break_blocks: Vec<i32>,
    /// See [`PistonProgress`], which this converts to.
    pub last_progress: f32,
    pub new_state: PistonState,
    /// See [`PistonProgress`], which this converts to.
    pub progress: f32,
    pub state: PistonState,
    pub sticky: bool,
}

impl PistonArm {
    /// How far through its stroke the arm is, or the raw value if it is not one
    /// of the three positions the game writes.
    pub fn progress(&self) -> Result<PistonProgress, f32> {
        PistonProgress::try_from(self.progress)
    }

    /// The stroke position of the previous tick. See [`Self::progress`].
    pub fn last_progress(&self) -> Result<PistonProgress, f32> {
        PistonProgress::try_from(self.last_progress)
    }
}

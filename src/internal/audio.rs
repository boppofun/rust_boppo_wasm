//! Internal audio parameter types used for controller communication with the host.
/// Parameter type for a controller command sent to the host.
#[repr(i32)]
pub enum AudioParameter {
    /// Stop the sound.
    Stop = 0,
    /// Pause or unpause the sound.
    Pause = 1,
    /// Adjust the volume multiplier.
    Volume = 2,
    /// Adjust the playback speed multiplier.
    Speed = 3,
}

impl TryFrom<i32> for AudioParameter {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Stop),
            1 => Ok(Self::Pause),
            2 => Ok(Self::Volume),
            3 => Ok(Self::Speed),
            _ => Err("Unknown audio parameter code."),
        }
    }
}

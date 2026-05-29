#[repr(i32)]
pub enum AudioParameter {
    Stop = 0,
    Pause = 1,
    Volume = 2,
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

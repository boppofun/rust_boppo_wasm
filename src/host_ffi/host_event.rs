use boppo_core::ButtonEvent;

#[non_exhaustive]
pub enum HostEvent {
    Exit,
    Button(ButtonEvent),
    Audio,
    Timeout,
}

impl HostEvent {
    fn event_type_u8(&self) -> u8 {
        match self {
            Self::Exit => 0,
            Self::Button(_) => 1,
            Self::Audio => 2,
            Self::Timeout => 3,
        }
    }

    /// Payload should never exceed 56 bits.
    fn payload<'a>(&self) -> Vec<u8> {
        match self {
            Self::Button(b) => b.as_u16().to_le_bytes().to_vec(),
            Self::Exit => Vec::new(),
            Self::Audio => todo!(),
            Self::Timeout => Vec::new(),
        }
    }
}

impl TryFrom<i64> for HostEvent {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        let buffer = value.to_le_bytes();
        let event_type = buffer[0];
        match event_type {
            0 => Ok(Self::Exit),
            1 => {
                let mut u16_buffer = [0u8; 2];
                u16_buffer.copy_from_slice(&buffer[6..8]);
                Ok(Self::Button(ButtonEvent::from_u16(u16::from_le_bytes(
                    u16_buffer,
                ))))
            }
            2 => todo!(),
            3 => Ok(Self::Timeout),
            n => Err(format!("Unknown event type {n}")),
        }
    }
}

impl From<&HostEvent> for i64 {
    fn from(value: &HostEvent) -> Self {
        let event_type_u8 = value.event_type_u8();
        let payload = value.payload();
        assert!(
            payload.len() < 8,
            "Event payload was bigger than expected : {} over the limit of 56 bits.",
            payload.len() * 8
        );
        let mut buffer = [0u8; 8];
        buffer[0] = event_type_u8;
        buffer[(8 - payload.len())..].copy_from_slice(&payload);
        i64::from_le_bytes(buffer)
    }
}

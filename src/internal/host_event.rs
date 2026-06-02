use boppo_core::ButtonEvent;

/// An event received from the Boppo WASM host via `boppo_poll`.
#[non_exhaustive]
pub enum HostEvent {
    /// The host has requested that the activity exit.
    Exit,
    /// A button was pressed or released.
    Button(ButtonEvent),
    /// A sound with the given controller ID has finished playing.
    FinishedAudio(u64),
    /// The poll timeout elapsed with no other event.
    Timeout,
}

// i64 event encoding layout :
//   byte 0:   HostEvent type (0=Exit, 1=Button, 2=FinishedAudio, 3=Timeout)
//   bytes 1-7: payload
//
// FinishedAudio payload:
//   bytes 1-3: unused
//   bytes 4-7: handle as i32
//
// ButtonEvent payload:
//   bytes 1-4: unused as of now
//   bytes 5-7: event u16 representation
//
// No payload for other event types

impl HostEvent {
    fn event_type_u8(&self) -> u8 {
        match self {
            Self::Exit => 0,
            Self::Button(_) => 1,
            Self::FinishedAudio(_) => 2,
            Self::Timeout => 3,
        }
    }

    /// Payload should never exceed 56 bits.
    fn payload(&self) -> [u8; 7] {
        let mut result = [0u8; 7];
        match self {
            Self::Button(b) => result[5..7].copy_from_slice(&b.as_u16().to_le_bytes()),
            Self::Exit => {}
            Self::FinishedAudio(controller_id) => {
                result.copy_from_slice(&controller_id.to_le_bytes()[0..7]);
            }
            Self::Timeout => {}
        }
        result
    }
}

impl TryFrom<i64> for HostEvent {
    type Error = u8;

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
            2 => {
                let mut u64_buffer = [0u8; 8];
                u64_buffer[0..7].copy_from_slice(&buffer[1..8]);
                let controller_id = u64::from_le_bytes(u64_buffer);
                Ok(Self::FinishedAudio(controller_id))
            }
            3 => Ok(Self::Timeout),
            n => Err(n),
        }
    }
}

impl From<&HostEvent> for i64 {
    fn from(value: &HostEvent) -> Self {
        let event_type_u8 = value.event_type_u8();
        let payload = value.payload();
        let mut buffer = [0u8; 8];
        buffer[0] = event_type_u8;
        buffer[1..].copy_from_slice(&payload);
        i64::from_le_bytes(buffer)
    }
}

impl From<HostEvent> for i64 {
    fn from(value: HostEvent) -> Self {
        i64::from(&value)
    }
}

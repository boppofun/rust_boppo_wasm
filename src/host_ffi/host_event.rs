use boppo_core::ButtonEvent;

use crate::host_ffi::audio::AudioEvent;

#[cfg(feature = "wasm_client")]
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    /// Polling function for Button events with optionnal timeout.
    /// If timeout_ms < 0, poll will happen indefinitely.
    /// This can be used to poll for button events or wait a certain time if not event was received
    /// in between.
    /// Returns a HostEvent i64 representation.
    pub fn boppo_wasm_poll(timeout_ms: i32) -> i64;
}

#[non_exhaustive]
pub enum HostEvent {
    Exit,
    Button(ButtonEvent),
    Audio(AudioEvent),
    Timeout,
}

// i64 wire layout (little-endian):
//   byte 0:   HostEvent type (0=Exit, 1=Button, 2=Audio, 3=Timeout)
//   bytes 1-7: payload
//
// Audio payload:
//   bytes 1-2: req_id as i16 (Opened only; 0 for Finished)
//   byte  3:   AudioEvent sub-type (0=Opened, 1=Finished)
//   bytes 4-7: handle as i32

impl HostEvent {
    fn event_type_u8(&self) -> u8 {
        match self {
            Self::Exit => 0,
            Self::Button(_) => 1,
            Self::Audio(_) => 2,
            Self::Timeout => 3,
        }
    }

    /// Payload should never exceed 56 bits.
    fn payload(&self) -> [u8; 7] {
        let mut result = [0u8; 7];
        match self {
            Self::Button(b) => result[5..7].copy_from_slice(&b.as_u16().to_le_bytes()),
            Self::Exit => {}
            Self::Audio(audio_event) => match audio_event {
                AudioEvent::Opened { req_id, handle } => {
                    result[0..2].copy_from_slice(&(*req_id as i16).to_le_bytes());
                    result[2] = 0;
                    result[3..7].copy_from_slice(&handle.to_le_bytes());
                }
                AudioEvent::Finished(handle) => {
                    result[2] = 1;
                    result[3..7].copy_from_slice(&handle.to_le_bytes());
                }
            },
            Self::Timeout => {}
        }
        result
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
            2 => {
                let audio_sub_type = buffer[3];
                let handle = i32::from_le_bytes(buffer[4..8].try_into().unwrap());
                match audio_sub_type {
                    0 => {
                        let req_id = i16::from_le_bytes(buffer[1..3].try_into().unwrap()) as i32;
                        Ok(Self::Audio(AudioEvent::Opened { req_id, handle }))
                    }
                    1 => Ok(Self::Audio(AudioEvent::Finished(handle))),
                    n => Err(format!("Unknown audio event sub-type {n}")),
                }
            }
            3 => Ok(Self::Timeout),
            n => Err(format!("Unknown event type {n}")),
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

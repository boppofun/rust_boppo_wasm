//! Commands for controlling the Boppo device.
use crate::Error;

/// Execute a command.
///
/// See the full list of supported commands at [developer.boppo.com/docs/commands](https://developer.boppo.com/docs/commands)
///
/// ## Examples
///
/// ```no_run
/// use boppo_wasm::commands::execute_command;
///
/// // stop the activity and put the device to sleep
/// execute_command("sleep").unwrap();
///
/// // start a new activity
/// execute_command("start wasm ...").unwrap();
///
/// // Send a MIDI note "on" message over USB
/// execute_command("send_midi_usb_note_on 60").unwrap(); // play middle C
/// ```
pub fn execute_command(command: &str) -> Result<(), Error> {
    Error::result_from_i32(unsafe {
        crate::internal::host_ffi::boppo_execute_command(command.as_ptr(), command.len())
    })?;
    Ok(())
}

/// Execute a command, outputting to `buf` instead of stdout.
///
/// See the full list of supported commands at [developer.boppo.com/docs/commands](https://developer.boppo.com/docs/commands)
///
/// ## Examples
///
/// ```no_run
/// use boppo_wasm::commands::execute_command_with_buffer;
///
/// let mut buf = [0u8; 16];
///
/// // retreive the high score for an activity, and store its string in `buf`
/// execute_command_with_buffer("score_retreive ...", &mut buf).unwrap();
/// ```
pub fn execute_command_with_buffer(command: &str, buffer: &mut [u8]) -> Result<(), Error> {
    Error::result_from_i32(unsafe {
        crate::internal::host_ffi::boppo_execute_command_with_buffer(
            command.as_ptr(),
            command.len(),
            buffer.as_mut_ptr(),
            buffer.len(),
        )
    })?;
    Ok(())
}

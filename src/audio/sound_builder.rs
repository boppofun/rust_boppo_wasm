//! [`SoundBuilder`] and related types for constructing sounds before playback.
use super::Controller;
use boppo_core::audio::{ControllerParams, SoundInstruction};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

static CONTROLLER_ID_COUNTER: AtomicU64 = AtomicU64::new(5_000_000_000);

/// Build a sound for playback.
///
/// This wraps a [`SoundInstruction`] and provides a more convenient API for building
/// sounds and controlling them via [`Controller`] with auto-assigned IDs.
///
/// ## Clone
///
/// While SoundBuilder implements Clone, cloning a SoundBuilder with a Controller
/// will clone the ID. If played, any existing Controllers with the ID would now
/// control the new sound even if the old sound is still playing.
#[must_use = "SoundBuilder does nothing unless passed to audio::play or a similar function"]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct SoundBuilder(SoundInstruction);

impl SoundBuilder {
    /// Play a file at the given path.
    ///
    /// If a relative path is provided, it is relative to the directory the wasm
    /// file is located in.
    ///
    /// If an absolute path is provided, it is relative to the activities
    /// directory (i.e. `/sd/activities/`).
    ///
    /// SoundBuilder::file can be constructed from a String/&str using `From`/`Into`:
    ///
    /// ```
    /// # use boppo_wasm::audio::SoundBuilder;
    /// let sound: SoundBuilder = "music.mp3".into();
    /// ```
    pub fn file(path: impl Into<String>) -> Self {
        Self(SoundInstruction::PlayFile(path.into()))
    }

    /// Play a sequence of sounds one after another.
    pub fn list(sounds: Vec<SoundBuilder>) -> Self {
        Self(SoundInstruction::List(
            sounds.into_iter().map(SoundInstruction::from).collect(),
        ))
    }

    /// Play multiple sounds at the same time.
    pub fn simultaneous(sounds: Vec<SoundBuilder>) -> Self {
        Self(SoundInstruction::Simultaneous(
            sounds.into_iter().map(SoundInstruction::from).collect(),
        ))
    }

    /// Insert silence for the given duration.
    ///
    /// Useful for pausing or delaying playback (e.g. in the middle of a
    /// [`SoundBuilder::list`]).
    pub fn silence(duration: Duration) -> Self {
        Self(SoundInstruction::Silence(duration))
    }

    /// Speak a number aloud.
    ///
    /// The number is spoken in the system language.
    pub fn speak_number(n: i64) -> Self {
        Self(SoundInstruction::SpeakNumber(n))
    }

    /// Play the error sound.
    pub fn error_sound() -> Self {
        Self(SoundInstruction::ErrorSound)
    }

    /// A sound that immediately returns without playing anything.
    pub fn empty_sound() -> Self {
        Self(SoundInstruction::EmptySound)
    }

    /// Repeat this sound `times` times.
    ///
    /// The repeated sound must not contain a [`SoundBuilder::controller`] either
    /// directly or indirectly.
    pub fn repeat(self, times: u64) -> Self {
        Self(SoundInstruction::Repeat(Box::new(self.0), Some(times)))
    }

    /// Repeat this sound indefinitely.
    ///
    /// The repeated sound must not contain a [`SoundBuilder::controller`] either
    /// directly or indirectly.
    pub fn repeat_forever(self) -> Self {
        Self(SoundInstruction::Repeat(Box::new(self.0), None))
    }

    /// Apply a volume multiplier to this sound.
    ///
    /// `multiplier` is a linear scale factor: `1.0` = original volume, `0.5` = half,
    /// `2.0` = double.
    ///
    /// If you also want to change the volume after playback has started (and
    /// optionally before too), use [`SoundBuilder::controller`] or
    /// [`SoundBuilder::controller_with_opts`] instead.
    pub fn volume(self, multiplier: f32) -> Self {
        Self(SoundInstruction::Volume(multiplier, Box::new(self.0)))
    }

    /// Apply a speed multiplier to this sound.
    ///
    /// `multiplier` is a linear scale factor: `1.0` = original speed, `2.0` = double speed.
    /// Pitch is adjusted proportionally to speed (faster → higher pitch, slower → lower pitch).
    ///
    /// If you also want to change the speed after playback has started (and
    /// optionally before too), use [`SoundBuilder::controller`] or
    /// [`SoundBuilder::controller_with_opts`] instead.
    pub fn speed(self, multiplier: f32) -> Self {
        Self(SoundInstruction::Speed(multiplier, Box::new(self.0)))
    }

    /// Wrap the instruction in a controller and return a [`Controller`] that can
    /// be used to control the sound after it starts playing.
    ///
    /// You can control the speed, volume, and paused state of the sound, stop it
    /// completely, and receive notifications when it finishes.
    ///
    /// A controller cannot be nested within [`SoundBuilder::repeat`] or
    /// [`SoundBuilder::repeat_forever`].
    ///
    /// The controller ID is assigned automatically from a monotonically increasing
    /// counter starting at 5,000,000,000.
    pub fn controller(self) -> (Self, Controller) {
        let id = next_controller_id();
        let instruction = SoundInstruction::Controller(
            Box::new(self.0),
            ControllerParams {
                id,
                speed: None,
                volume: None,
                paused: None,
            },
        );
        (Self(instruction), Controller::new(id))
    }

    /// Wrap the instruction in a controller with initial options and return a
    /// [`Controller`] that can be used to control the sound after it starts playing.
    ///
    /// See [`SoundBuilder::controller`] for full details and constraints.
    pub fn controller_with_opts(self, opts: ControllerOpts) -> (Self, Controller) {
        let id = next_controller_id();
        let instruction = SoundInstruction::Controller(
            Box::new(self.0),
            ControllerParams {
                id,
                speed: opts.speed,
                volume: opts.volume,
                paused: opts.paused,
            },
        );
        (Self(instruction), Controller::new(id))
    }

    /// Create a Builder from a SoundInstruction.
    ///
    /// Care should be taken that any contained controller IDs do not
    /// conflict with auto assigned builder IDs.
    pub fn from_instruction(instruction: SoundInstruction) -> Self {
        Self(instruction)
    }

    /// Convert this Builder into a SoundInstruction
    pub fn into_instruction(self) -> SoundInstruction {
        self.0
    }

    /// Return a reference to the underlying [`SoundInstruction`].
    pub fn as_instruction(&self) -> &SoundInstruction {
        &self.0
    }
}

impl From<String> for SoundBuilder {
    fn from(path: String) -> Self {
        Self::file(path)
    }
}

impl From<&str> for SoundBuilder {
    fn from(path: &str) -> Self {
        Self::file(path)
    }
}

impl<T: Into<SoundBuilder>> From<Vec<T>> for SoundBuilder {
    fn from(sounds: Vec<T>) -> Self {
        Self::list(sounds.into_iter().map(Into::into).collect())
    }
}

/// Initial options for a [`SoundBuilder::controller_with_opts`] call.
///
/// All fields are optional and default to the device defaults (speed `1.0`,
/// volume `1.0`, paused `false`). Use the chainable setter methods to override
/// only the values you care about.
///
/// ```
/// # use boppo_wasm::audio::{SoundBuilder, ControllerOpts};
/// let (sound, ctrl) = SoundBuilder::file("music.mp3")
///     .controller_with_opts(ControllerOpts::new().volume(0.5).pause());
/// ```
#[derive(Debug, Default, Clone)]
pub struct ControllerOpts {
    /// Initial playback speed multiplier. Defaults to `1.0`.
    speed: Option<f32>,
    /// Initial volume multiplier. Defaults to `1.0`.
    volume: Option<f32>,
    /// Whether the sound starts paused (set via [`ControllerOpts::pause`]).
    paused: Option<bool>,
}

impl ControllerOpts {
    /// Create a new `ControllerOpts` with all fields set to their defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the initial playback speed multiplier (default `1.0`).
    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Set the initial volume multiplier (default `1.0`).
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = Some(volume);
        self
    }

    /// Start the sound paused (default is to start playing immediately).
    pub fn pause(mut self) -> Self {
        self.paused = Some(true);
        self
    }
}

impl From<SoundBuilder> for SoundInstruction {
    fn from(builder: SoundBuilder) -> Self {
        builder.0
    }
}

/// Assign the next controller ID from the monotonically increasing counter.
///
/// [`SoundBuilder::controller`] uses this internally. Call it directly if you
/// need to construct a [`SoundInstruction::Controller`] by hand and want to
/// avoid ID collisions with builder-assigned IDs.
///
/// IDs below 5,000,000,000 are never assigned by this counter and are safe to
/// use manually.
pub fn next_controller_id() -> u64 {
    CONTROLLER_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

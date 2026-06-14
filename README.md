 # [Boppo](https://developer.boppo.com) WebAssembly Activity API for Rust [![Docs Passing]][docs.rs] [![Latest Version]][crates.io]

 ## Getting Started

 To get started developing on Boppo read our
 [developer documentation](https://developer.boppo.com/) with instructions
 to clone our template repository.

 Activities are compiled to WASM and run on the tablet. The main entry point
 for most activities is [`init_and_run_async`].

 ## Lights

 For setting the color of lights see the set_color and similar functions on
 [`Button`], [`Buttons`], and [`Lights`].

 By default all changes are immediately flushed to the hardware. If you want
 to make multiple changes in a row and performance matters you can use
 [`Framebuffer`] or modify the auto flush behavior using [`MainFramebuffer`].

 Each Boppo button has 4 LED lights which are represented by [`LightDir`].

 For drawing and animating simple shapes treating the entire Boppo surface as
 a display see [`lights_plane`].

 ## Button Events

 You can receive button change events as an async stream using
 [`ButtonEvents`].

 You can also query the current state of the buttons using
 [`Button::is_pressed`] and [`Buttons::currently_pressed`].

 You can use [`Button::wait_for_press`] and [`Button::wait_for_release`] to
 wait for a button to be in a specific state.

 ## Audio

 Play audio using [`audio::play`] or the `audio::play_*` helper functions.

 ## Guidelines

 See Boppo's [Activity
 Guidelines](https://developer.boppo.com/docs/activity-guidelines) for
 guidelines on creating great activities.uch of the API (e.g. handling button
 events, setting button colors) is located in the boppo_core crate and exposed
 here.

## License

This project is licensed under either of Apache License, Version 2.0 or MIT license at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[Latest Version]: https://img.shields.io/crates/v/boppo_wasm.svg
[crates.io]: https://crates.io/crates/boppo_wasm
[Docs Passing]: https://img.shields.io/docsrs/boppo_wasm.svg
[docs.rs]: https://docs.rs/boppo_wasm

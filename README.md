# Boppo Webassembly Activity Framework for Rust

This repository contains the Rust framework for creating local Webassembly activities using Rust.

In development

## Initial architecture diagram

![Architectural diagram](./static/WASM_architecture.svg)

### High-level main activity execution sequence

![Architectural diagram](./static/threadmodel-main.svg)

### Light setting sequence

![Architectural diagram](./static/threadmodel-lights.svg)

### Button events sequence

![Architectural diagram](./static/threadmodel-buttons.svg)

## Key design questions remaining

1. AOT compilation of wasm modules would probably be beneficial for activities; but this requires a compatible Wamr version to compile the wasm file -- an extra step. Wouldn't it be interesting, for "third party activity developers" developer experience, to provide the build step as a web service on the boppo activity developer portal (or whatever it ends up being named ;) ) ? This way they can compile to wasm and we do the final AOT step. We can keep interpreter mode for development or fallback, but AOT seems interesting on embedded devices -- limited resources and all.


## Findings for later

### Integration of Wamr

WAMR has an [officially supported component](https://components.espressif.com/components/espressif/wasm-micro-runtime/versions/2.4.0~1/readme) for esp.

The wamr-rust-sdk can't work easily for ESP32 because it relies on a sys crate based on the C wamr source, and those assume desktop use, so really hard to compile for ESP.

To add this component in the embassy project, esp-idf-sys can pick up an environment variable : ESP_IDF_EXTRA_COMPONENT_DIRS from which it will load extra esp-idf components from the given dir.

It probably requires a bridge crate though, with some kind of `build.rs` with only the relevant wamr functions reexported. from the crate, we can define a yml file that defines esp-idf component dependencies in this format:

```yml
dependencies:
  wasm-micro-runtime/wamr:
    version: ">=2.1.0"
```

We can probably integrate WAMR i

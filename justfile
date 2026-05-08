check:
    cargo check

release-checks:
    cargo build
    cargo clippy
    cargo test
    cargo doc

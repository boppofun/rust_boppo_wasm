check:
    cargo check

release-checks:
    cargo build
    cargo clippy
    cargo test --all-features -q
    cargo doc --all-features

_update-git-head-main:
    #!/usr/bin/env fish
    if not test (git rev-parse HEAD) = (git rev-parse refs/heads/main)
        echo "Not on main branch"
        exit 1
    end
    git checkout main

release-prepare: release-checks _update-git-head-main
    #!/usr/bin/env fish
    release-plz update

release-commit:
    #!/usr/bin/env fish
    set crate_version (cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version')
    echo $crate_version
    jj commit -m "chore(release): v$crate_version"

release-execute-dry-run: release-checks _update-git-head-main
    release-plz release --dry-run

release-execute: release-checks _update-git-head-main
    release-plz release

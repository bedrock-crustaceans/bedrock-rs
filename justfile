# Arguments reach a recipe as `$1`, `$2`, ... unsplit and unglobbed: `-p 'bedrock_network*'`
# arrives as the two words cargo expects.
set positional-arguments

# List the recipes.
default:
    @just --list

# Run every check CI runs, over the whole workspace.
ready: fmt-check clippy test check-versions doctest

# Format every crate.
fmt:
    cargo fmt --all

# Fail on any file `fmt` would change.
fmt-check:
    cargo fmt --all --check

# Lint every target under every feature; any warning fails.
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run the tests under nextest's CI profile: the workspace, or the packages given (`-p bedrock_level`).
test *packages='--workspace':
    cargo nextest run --profile ci "$@"

# Build each version crate the way the facade does, without its `version` feature.
check-versions:
    cargo check -p bedrock_network_protocol --no-default-features

# Run the doctests, which nextest skips.
doctest *packages='--workspace':
    cargo test --doc "$@"

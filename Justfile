set shell := ["zsh", "-cu"]

# List the available commands and what each one does
default:
    @just --list

# Download Rust/Bun dependencies and install the repository's Git hooks
setup:
    just _in-dev-shell _setup

[private]
_setup:
    cargo fetch
    bun install --frozen-lockfile
    just hooks-install

# Regenerate the ZMK keymap and preview JSON from the Rust layout
generate:
    just _in-dev-shell _generate

[private]
_generate:
    cargo run --quiet -- generate

# Serve the live preview on an available localhost port
run:
    just _in-dev-shell _run

[private]
_run: _generate
    bun preview/server.ts

# Build the optimized Rust generator after regenerating its outputs
build:
    just _in-dev-shell _build

[private]
_build: _generate
    cargo build --release

# Run the Rust layout and generator tests
test:
    just _in-dev-shell _test

[private]
_test:
    cargo test

# Check Rust, preview code, and committed generated artifacts
lint:
    just _in-dev-shell _lint

[private]
_lint:
    cargo clippy --all-targets -- -D warnings
    bunx biome check preview package.json biome.json
    cargo run --quiet -- check

# Format Rust and preview code
fmt:
    just _in-dev-shell _fmt

[private]
_fmt:
    cargo fmt
    bunx biome format --write preview package.json biome.json

# Apply automatic Rust and preview-code fixes
fix:
    just _in-dev-shell _fix

[private]
_fix:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets
    bunx biome check --write preview package.json biome.json
    cargo fmt

# Download the pinned ZMK/Zephyr sources and Python tools (run once)
firmware-setup:
    just _in-dev-shell _firmware-setup

[private]
_firmware-setup:
    test -d tmp/zmk-venv || uv venv tmp/zmk-venv
    uv pip install --python tmp/zmk-venv/bin/python pip west
    test -d .west || tmp/zmk-venv/bin/west init -l config
    tmp/zmk-venv/bin/west update
    tmp/zmk-venv/bin/west zephyr-export
    uv pip install --python tmp/zmk-venv/bin/python -r zephyr/scripts/requirements.txt -r modules/lib/nanopb/extra/requirements.txt

# Regenerate the keymap and compile UF2 firmware for both Dao halves
firmware:
    just _in-dev-shell _firmware

[private]
_firmware: _generate
    test -d .west || { echo 'Run `just firmware-setup` once first.' >&2; exit 1; }
    tmp/zmk-venv/bin/west build -s zmk/app -d build/left -b dao_left -- -DZMK_CONFIG={{justfile_directory()}}/config
    tmp/zmk-venv/bin/west build -s zmk/app -d build/right -b dao_right -- -DZMK_CONFIG={{justfile_directory()}}/config

# Configure Git to use the repository's versioned hooks
hooks-install:
    git config core.hooksPath .githooks

# Run the complete commit gate (normally invoked automatically by Git)
hook-pre-commit:
    just _in-dev-shell _hook-pre-commit

[private]
_hook-pre-commit:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test
    cargo run --quiet -- check
    bunx biome check preview package.json biome.json

[private]
_in-dev-shell recipe:
    if [[ -n "${IN_NIX_SHELL:-}" ]]; then just {{recipe}}; else nix develop --command just {{recipe}}; fi

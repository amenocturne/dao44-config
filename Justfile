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
    just _hooks-install

[private]
_generate:
    cargo run --quiet -- generate

# Serve the live preview on an available localhost port
run:
    just _in-dev-shell _run

[private]
_run: _generate
    bun preview/server.ts

[private]
_build: _generate
    cargo build --release

[private]
_test:
    cargo test

[private]
_lint:
    cargo clippy --all-targets -- -D warnings
    bunx biome check preview package.json biome.json
    cargo run --quiet -- check

[private]
_fmt:
    cargo fmt
    bunx biome format --write preview package.json biome.json

[private]
_fix:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets
    bunx biome check --write preview package.json biome.json
    cargo fmt

# Set up or build the keyboard firmware; omit COMMAND for help
firmware command="" target="":
    @if [[ -n "${IN_NIX_SHELL:-}" ]]; then just _firmware-command {{quote(command)}} {{quote(target)}}; else nix develop --command just _firmware-command {{quote(command)}} {{quote(target)}}; fi

[private]
_firmware-command command target:
    @command={{quote(command)}}; target={{quote(target)}}; if [[ -z "$command" || "$command" == "help" || "$command" == "--help" ]]; then just _firmware-help; elif [[ "$command" == "setup" && -z "$target" ]]; then just _firmware-setup; elif [[ "$command" == "build" && -n "$target" ]]; then if [[ "$target" == "all" ]]; then just _firmware; elif [[ "$target" == "left" || "$target" == "right" ]]; then just _firmware-half "$target"; else echo "unknown firmware target: $target" >&2; just _firmware-help >&2; exit 2; fi; elif [[ "$command" == "build" ]]; then echo 'firmware build needs a target' >&2; just _firmware-help >&2; exit 2; else echo "unknown firmware command: $command" >&2; just _firmware-help >&2; exit 2; fi

[private]
_firmware-help:
    @echo 'Usage: just firmware <command> [target]'
    @echo
    @echo 'Commands:'
    @echo '  setup             Download the pinned ZMK/Zephyr build environment'
    @echo '  build left        Build firmware for the left half'
    @echo '  build right       Build firmware for the right half'
    @echo '  build all         Build firmware for both halves'

[private]
_firmware-setup:
    test -d tmp/zmk-venv || uv venv tmp/zmk-venv
    uv pip install --python tmp/zmk-venv/bin/python pip west
    test -d .west || tmp/zmk-venv/bin/west init -l config
    tmp/zmk-venv/bin/west update
    tmp/zmk-venv/bin/west zephyr-export
    uv pip install --python tmp/zmk-venv/bin/python -r zephyr/scripts/requirements.txt -r modules/lib/nanopb/extra/requirements.txt

[private]
_firmware: _generate
    test -d .west || { echo 'Run `just firmware setup` once first.' >&2; exit 1; }
    tmp/zmk-venv/bin/west build -s zmk/app -d build/left -b dao_left -- -DZMK_CONFIG={{justfile_directory()}}/config
    tmp/zmk-venv/bin/west build -s zmk/app -d build/right -b dao_right -- -DZMK_CONFIG={{justfile_directory()}}/config

[private]
_firmware-half half: _generate
    @half={{quote(half)}}; if [[ "$half" != "left" && "$half" != "right" ]]; then echo "unknown keyboard half: $half" >&2; exit 1; fi; test -d .west || { echo 'Run `just firmware setup` once first.' >&2; exit 1; }; tmp/zmk-venv/bin/west build -s zmk/app -d "build/$half" -b "dao_$half" -- -DZMK_CONFIG={{justfile_directory()}}/config

# Flash one half, all connected halves, or inspect the mapping; omit TARGET for help
flash target="":
    @if [[ -n "${IN_NIX_SHELL:-}" ]]; then just _flash {{quote(target)}}; else nix develop --command just _flash {{quote(target)}}; fi

[private]
_flash target:
    @target={{quote(target)}}; if [[ -z "$target" || "$target" == "help" || "$target" == "--help" ]]; then cargo run --quiet -- flash; elif [[ "$target" == "check" ]]; then cargo run --quiet -- flash check; elif [[ "$target" == "left" || "$target" == "right" ]]; then just _firmware-half "$target" && cargo run --quiet -- flash "$target"; elif [[ "$target" == "all" ]]; then just _firmware && cargo run --quiet -- flash all; else cargo run --quiet -- flash "$target"; fi

# Run contributor checks and maintenance; omit COMMAND for help
dev command="":
    @if [[ -n "${IN_NIX_SHELL:-}" ]]; then just _dev {{quote(command)}}; else nix develop --command just _dev {{quote(command)}}; fi

[private]
_dev command:
    @command={{quote(command)}}; if [[ -z "$command" || "$command" == "help" || "$command" == "--help" ]]; then just _dev-help; elif [[ "$command" == "generate" ]]; then just _generate; elif [[ "$command" == "build" ]]; then just _build; elif [[ "$command" == "test" ]]; then just _test; elif [[ "$command" == "lint" ]]; then just _lint; elif [[ "$command" == "format" ]]; then just _fmt; elif [[ "$command" == "fix" ]]; then just _fix; elif [[ "$command" == "check" ]]; then just _hook-pre-commit; else echo "unknown dev command: $command" >&2; just _dev-help >&2; exit 2; fi

[private]
_dev-help:
    @echo 'Usage: just dev <command>'
    @echo
    @echo 'Commands:'
    @echo '  generate          Regenerate the keymap and preview data'
    @echo '  build             Build the optimized Rust generator'
    @echo '  test              Run the Rust tests'
    @echo '  lint              Check Rust, preview code, and generated files'
    @echo '  format            Format Rust and preview code'
    @echo '  fix               Apply automatic lint and formatting fixes'
    @echo '  check             Run the complete pre-commit gate'

[private]
_hooks-install:
    git config core.hooksPath .githooks

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

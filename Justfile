set shell := ["zsh", "-cu"]

default:
    @just --list

setup:
    just _in-dev-shell _setup

[private]
_setup:
    cargo fetch
    bun install --frozen-lockfile
    just hooks-install

generate:
    just _in-dev-shell _generate

[private]
_generate:
    cargo run --quiet -- generate

run:
    just _in-dev-shell _run

[private]
_run: _generate
    bun preview/server.ts

build:
    just _in-dev-shell _build

[private]
_build: _generate
    cargo build --release

test:
    just _in-dev-shell _test

[private]
_test:
    cargo test

lint:
    just _in-dev-shell _lint

[private]
_lint:
    cargo clippy --all-targets -- -D warnings
    bunx biome check preview package.json biome.json
    cargo run --quiet -- check

fmt:
    just _in-dev-shell _fmt

[private]
_fmt:
    cargo fmt
    bunx biome format --write preview package.json biome.json

fix:
    just _in-dev-shell _fix

[private]
_fix:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets
    bunx biome check --write preview package.json biome.json
    cargo fmt

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

firmware:
    just _in-dev-shell _firmware

[private]
_firmware: _generate
    test -d .west || { echo 'Run `just firmware-setup` once first.' >&2; exit 1; }
    tmp/zmk-venv/bin/west build -s zmk/app -d build/left -b dao_left -- -DZMK_CONFIG={{justfile_directory()}}/config
    tmp/zmk-venv/bin/west build -s zmk/app -d build/right -b dao_right -- -DZMK_CONFIG={{justfile_directory()}}/config

hooks-install:
    git config core.hooksPath .githooks

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

set shell := ["zsh", "-cu"]

default:
    @just --list

setup:
    cargo fetch
    bun install --frozen-lockfile
    just hooks-install

generate:
    cargo run --quiet -- generate

run port="4173": generate
    bun preview/server.ts {{port}}

build: generate
    cargo build --release

test:
    cargo test

lint:
    cargo clippy --all-targets -- -D warnings
    bunx biome check preview package.json biome.json
    cargo run --quiet -- check

fmt:
    cargo fmt
    bunx biome format --write preview package.json biome.json

fix:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets
    bunx biome check --write preview package.json biome.json
    cargo fmt

firmware-setup:
    test -d tmp/zmk-venv || uv venv tmp/zmk-venv
    uv pip install --python tmp/zmk-venv/bin/python pip west
    test -d .west || tmp/zmk-venv/bin/west init -l config
    tmp/zmk-venv/bin/west update
    tmp/zmk-venv/bin/west zephyr-export
    uv pip install --python tmp/zmk-venv/bin/python -r zephyr/scripts/requirements.txt -r modules/lib/nanopb/extra/requirements.txt

firmware: generate
    test -d .west || { echo 'Run `just firmware-setup` once first.' >&2; exit 1; }
    tmp/zmk-venv/bin/west build -s zmk/app -d build/left -b dao_left -- -DZMK_CONFIG={{justfile_directory()}}/config
    tmp/zmk-venv/bin/west build -s zmk/app -d build/right -b dao_right -- -DZMK_CONFIG={{justfile_directory()}}/config

hooks-install:
    git config core.hooksPath .githooks

hook-pre-commit:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test
    cargo run --quiet -- check
    bunx biome check preview package.json biome.json

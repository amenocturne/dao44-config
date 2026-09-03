# Dao44 Config

A typed Rust source of truth for a personal Dao44 layout, with generated ZMK configuration and a live visual preview.

Change the layout, see the real keyboard immediately, then build firmware locally without waiting for GitHub Actions.

## Why

The old configuration loop hid intent inside devicetree syntax and made every experiment wait on a remote pipeline. This project keeps the ergonomic layout in ordinary Rust, validates the physical 44-key invariants, and generates the machine-facing artifacts.

It deliberately does not bake Colemak-DH into firmware. The keyboard emits ANSI positions while macOS owns Colemak-DH and Russian, keeping the split and built-in keyboards consistent.

## Features

**One typed layout** — Base, Nav/WM, and Num/Symbol are described in `src/layout.rs`; invalid layer sizes and settled recovery/modifier invariants fail generation.

**Two generated artifacts** — the same Rust model produces `config/dao.keymap` for ZMK and `generated/layout.json` for the preview.

**Live visual feedback** — editing Rust regenerates the artifacts and reloads the browser. The preview shows the real Dao44 stagger, layer tabs, raw/Colemak/Russian legends, and full tap/hold details.

**Fast local path** — Rust generation is incremental and small. A Nix shell pins the app and native ZMK build dependencies; left and right firmware use separate build caches.

## Quick start

```sh
just setup
just run
```

The recipes enter the pinned Nix environment themselves; only Nix and `just` need to be available globally. Open the URL printed by `just run`, then edit `src/layout.rs`. The server asks the operating system for an available localhost port, so multiple previews can run without colliding. The page reloads after a successful generation; compile failures are printed in the terminal and leave the last valid preview visible.

To prepare the larger ZMK checkout once and build both halves locally:

```sh
just firmware-setup
just firmware
```

The first setup downloads ZMK, Zephyr modules, and Python build dependencies into ignored project-local directories. Later builds reuse `build/left` and `build/right`.

On macOS, register each physical half once while it is the only `NRF52BOOT` volume mounted:

```sh
just register-left
just register-right
```

The commands store the bootloaders' unique USB serials in an ignored local `.env`; the identical volume labels do not need to change. Afterwards, double-tap reset on either or both halves and run `just flash`. It builds both images, verifies every mounted volume against the saved side before writing anything, then flashes all recognized halves. `just flash-check` performs the same discovery without writing.

The manifest pins the final ZMK revision before its Zephyr 4.1 migration. The upstream Dao module still uses Zephyr's 3.x board model, so upgrading ZMK independently currently breaks board discovery; treat the two revisions as one compatibility unit.

## Current boundary

The generated layout is an executable draft. Literal F1–F12, typed workspace chords, number mode, and Hyper-gated recovery actions are represented in ZMK. Two behaviors remain intentionally visible as design work before flashing: forcing Base specifically on Nav release from latched Num, and the exact firmware-native equivalents of every MacBook system key.

See [the layout contract](docs/layout.md) for settled behavior and open questions, and [the architecture](docs/architecture.md) for ownership and generation flow.

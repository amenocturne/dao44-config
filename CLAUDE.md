# Dao44 Config

Personal Dao44 keyboard configuration. Rust is the canonical layout owner; `generated/layout.json` and `config/dao.keymap` are generated outputs and must never be edited directly.

Preserve the transition-first constraint: macOS owns Colemak-DH and Russian, while firmware emits ANSI positions. Do not fill unassigned keys speculatively. The settled layout and explicitly open firmware questions live in `docs/layout.md`.

After changing `src/`, run generation before verification. `cargo run -- check` must prove committed generated artifacts match the Rust source. The complete command surface is in the Justfile.

Read `docs/architecture.md` before changing model/generator ownership or adding another frontend state source.

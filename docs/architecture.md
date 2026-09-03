# Architecture

## Ownership

`src/layout.rs` is the only editable layout definition. It assembles typed actions from `src/model.rs`; those action variants own both their visual meaning and their ZMK rendering. `src/generate.rs` is boundary glue: it validates the model and writes two projections.

```text
src/layout.rs ──> validated LayoutSpec ──> generated/layout.json ──> browser preview
                                    └──> config/dao.keymap ──────> ZMK build
```

The browser never reconstructs firmware semantics. It renders the generated preview payload and owns only ephemeral UI state: active layer, host legend, and selected key.

## Trust boundaries

The Rust source is trusted project code. Generation validates the 44-key count and the stable positions of layer entry, Base recovery, Nav access, and mirrored Shift holds before writing anything. `cargo run -- check` regenerates in memory and compares both committed artifacts byte-for-byte, preventing a preview/keymap split.

The ZMK and Ergonaut repositories are external inputs pinned in `config/west.yml`. ZMK is intentionally pinned to the final pre-Zephyr-4.1 revision because the upstream Dao module still uses Zephyr's 3.x board model. Updating either revision is a compatibility change and should be followed by a pristine firmware build.

## Live loop

The Bun server watches Rust and preview sources. Rust changes run the generator first; only a successful run emits the browser reload event. A compiler error therefore cannot replace the last valid generated layout. Preview-only changes reload without invoking Cargo.

## Firmware boundary

The generator emits ordinary ZMK devicetree rather than linking Rust into firmware. Rust provides the authoring language, validation, and macros; ZMK remains responsible for timing and hardware behavior. This keeps firmware upstream-compatible and avoids maintaining an FFI layer inside Zephyr.

On macOS, the flashing boundary identifies physical halves by USB serial rather than the shared `NRF52BOOT` label. Registration writes those serials only to ignored `.env`. Flashing discovers candidate volumes, validates their UF2 identity, joins each disk's device-tree location to its USB serial, and resolves every serial to a side before copying anything. Unknown, duplicate, or ambiguous devices abort the complete operation rather than allowing a positional guess.

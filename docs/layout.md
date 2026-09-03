# Layout contract

This document preserves the decisions behind the executable draft. Physical IDs `P00`–`P43` follow `dao_full_layout`: 14 top, 12 home, 12 bottom, and 6 thumb keys.

## Global rules

- Firmware emits raw ANSI/QWERTY-position HID codes. macOS alone interprets Colemak-DH or Russian.
- Russian keeps the ordinary 44-key punctuation/letter positions; no firmware Colemak layer may distort it.
- The physical index positions `P18` and `P21` own mirrored Shift holds across layers.
- Escape is rare but always present: tap `P00` for Escape, hold it for real Ctrl+Option+Command Hyper.
- `P41` is always the Nav escape hatch. The intended invariant is that releasing Nav returns to Base even if Nav began from latched Num.
- Empty positions stay empty until actual use supplies evidence.

## Base

The alpha block is untouched ANSI. Host legend switching in the preview changes labels, never firmware actions.

- `P18`: tap raw F (Colemak-DH T), hold left Shift.
- `P21`: tap raw J (Colemak-DH N), hold right Shift.
- `P26`: tap toggles Num/Symbol; hold activates it momentarily.
- `/` and `\` remain on the bottom-right positions; Shift produces `?` and `|`.
- Thumbs, left to right: Ctrl, Option, Cmd | Enter/Nav, Space, Backspace. Space owns the natural middle thumb position because it is substantially more frequent than Enter.

This keeps Ctrl-W, Ctrl-B, Ctrl-C, Ctrl-Space, Option-Space, Space leader, Cmd/Option-Backspace, and combined Cmd/Option+Shift editing available without a second translation scheme.

## Nav / WM

Hold Enter/Nav.

- A tap emits Enter; a hold activates Nav. Pressing another key while it is held resolves to Nav immediately, with no tap-then-hold Enter-repeat exception.
- Left arrows reuse WASD geometry: W=up and A/S/D=left/down/right.
- The left index home position is Shift.
- Right top row is AeroSpace WS6–WS10.
- Right home row is WS1–WS5. WS2–WS5 hold Shift, Cmd, Option, and Ctrl respectively.
- Workspace taps emit the existing AeroSpace Hyper+digit shortcuts; adding Shift moves the focused window.
- Bottom row is literal F1–F12, six per half.

The earlier arrows-right/workspaces-left variant was discarded because it broke the arrow cluster. WASD already supplied the desired shape and freed the right hand for workspace taps and text-selection modifiers.

## Num / Symbol

Tap Num for a run or hold it for one insertion.

- Home row is `1 2 3 4 5 | 6 7 8 9 0`.
- Tap 4 / hold left Shift and tap 7 / hold right Shift, preserving the same physical Shift positions as Base.
- Real Shift+number produces symbols, including genuine Cmd+Shift+3 rather than a `#` macro.
- Minus and equal share the top-right edge; Shift produces underscore and plus.
- `P26` returns a latched Num layer to Base.
- Left bottom recovery actions require the complete Hyper chord: clear current bond, clear all bonds, reset central/left, bootloader central/left.
- Right bottom selects Bluetooth profiles 1–5 through the same Hyper gate. USB needs no mode key.

## Open hardware questions

1. Tune tapping term, quick-tap, flavor, and hold triggers against real Colemak rolls and tap-then-hold repeat.
2. Test whether changing Bluetooth profiles while real HID Hyper is held can leak a stuck modifier. If so, keep the physical gesture but replace the gate with a silent firmware-only Admin layer.
3. Implement and verify a Nav behavior whose release forces Base from every starting layer; ordinary `&lt` returns to its previous layer.
4. Decide whether the MacBook behavior means literal F-keys with explicit system morphs or system actions by default with an F-key modifier. Some Apple actions do not have portable HID usages.
5. Decide Caps Lock / both-Shifts behavior only after Russian usage is tested.
6. Confirm which physical half is central before treating reset and bootloader labels as flash instructions.

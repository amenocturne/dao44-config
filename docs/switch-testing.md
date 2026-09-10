# Switch testing

`just switch-test left` builds and flashes temporary firmware in which every physical switch emits one unique printable ASCII character. It replaces the normal behavior of all 44 keys until `just flash left` restores the daily layout.

Keep macOS on the Colemak-DH ANSI input source while testing. Physical position `n`, counted from `0` in the same order as `P00` through `P43` in the preview, emits the character with ASCII value `n + 44`:

```text
,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVW
```

To test chatter, press one switch 20 times, press Enter on the MacBook keyboard, and continue with the next switch. A healthy line contains exactly 20 copies of its expected character. Extra copies identify chatter; missing copies identify dropped presses. Because the character's ASCII value minus 44 is its physical index, the output remains traceable without preserving a separate legend beside the test.

The diagnostic firmware deliberately has no Enter, modifiers, layers, Bluetooth controls, or firmware escape key. Double-tap the left half's physical reset and run `just flash left` to restore the normal layout.

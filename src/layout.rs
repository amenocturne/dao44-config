use crate::model::{
    Action, Category, GuardedAction, HostLegend, Layer, LayerId, LayoutSpec, Modifier,
};

fn key(code: &'static str, label: &'static str, category: Category) -> Action {
    Action::Key {
        code,
        label,
        category,
        secondary: "",
    }
}

fn annotated_key(
    code: &'static str,
    label: &'static str,
    category: Category,
    secondary: &'static str,
) -> Action {
    Action::Key {
        code,
        label,
        category,
        secondary,
    }
}

fn shift_tap(
    code: &'static str,
    label: &'static str,
    modifier: Modifier,
    secondary: &'static str,
) -> Action {
    Action::ModTap {
        modifier,
        tap_code: code,
        label,
        tap: label,
        secondary,
    }
}

fn guarded(
    label: &'static str,
    detail: &'static str,
    action: GuardedAction,
    category: Category,
) -> Action {
    Action::Guarded {
        label,
        detail,
        action,
        category,
    }
}

pub fn dao44() -> LayoutSpec {
    LayoutSpec {
        title: "Dao44 · transition-first layout",
        revision: "v1 · executable draft",
        layers: vec![base(), nav(), num()],
        host_legends: vec![colemak(), russian()],
        open_questions: vec![
            "Tune the 180 ms home-row Shift threshold on physical hardware without changing its pure timing rule.",
            "Decide whether the Hyper-gated admin strip should become a silent firmware-only Admin layer before switching Bluetooth hosts.",
            "Confirm the desired MacBook-style system actions for F1–F12; generated firmware currently emits literal F-key codes.",
            "Leave unused Num bottom-row positions empty until real use reveals a need.",
        ],
    }
}

fn base() -> Layer {
    use Category::{Alpha, Modifier as Mod, Navigation, Symbol};
    use Modifier::{LeftShift, RightShift};

    Layer {
        id: LayerId::Base,
        name: "Base",
        description: "Raw ANSI positions interpreted by host-owned Colemak-DH or Russian, with familiar thumb modifiers and mirrored index-finger Shift holds.",
        keys: vec![
            Action::HyperEscape,
            annotated_key("GRAVE", "`", Symbol, "Russian host layout: ё"),
            key("Q", "q", Alpha),
            key("W", "w", Alpha),
            key("E", "e", Alpha),
            key("R", "r", Alpha),
            key("T", "t", Alpha),
            key("Y", "y", Alpha),
            key("U", "u", Alpha),
            key("I", "i", Alpha),
            key("O", "o", Alpha),
            key("P", "p", Alpha),
            key("LBKT", "[", Symbol),
            key("RBKT", "]", Symbol),
            key("TAB", "Tab", Navigation),
            key("A", "a", Alpha),
            key("S", "s", Alpha),
            key("D", "d", Alpha),
            shift_tap("F", "f", LeftShift, "Colemak-DH host interpretation: t"),
            key("G", "g", Alpha),
            key("H", "h", Alpha),
            shift_tap("J", "j", RightShift, "Colemak-DH host interpretation: n"),
            key("K", "k", Alpha),
            key("L", "l", Alpha),
            key("SEMI", ";", Symbol),
            key("SQT", "'", Symbol),
            Action::None,
            key("Z", "z", Alpha),
            key("X", "x", Alpha),
            key("C", "c", Alpha),
            key("V", "v", Alpha),
            key("B", "b", Alpha),
            key("N", "n", Alpha),
            key("M", "m", Alpha),
            key("COMMA", ",", Symbol),
            key("DOT", ".", Symbol),
            annotated_key("FSLH", "/", Symbol, "Shift + / produces ?"),
            annotated_key("BSLH", "\\", Symbol, "Shift + \\ produces |"),
            key("LCTRL", "Ctrl", Mod),
            key("LALT", "Option", Mod),
            key("LGUI", "Cmd", Mod),
            Action::NavEnter,
            key("SPACE", "Space", Alpha),
            Action::BackspaceNum,
        ],
    }
}

fn nav() -> Layer {
    use Category::{Modifier as Mod, Navigation};
    use Modifier::{RightAlt, RightControl, RightGui, RightShift};

    let mut keys = vec![Action::None; 44];
    keys[0] = Action::HyperEscape;
    keys[3] = annotated_key("UP", "↑", Navigation, "Physical W position");
    for (index, workspace) in (7..=11).zip(6..=10) {
        keys[index] = Action::Workspace {
            number: workspace,
            hold: None,
        };
    }
    keys[14] = key("TAB", "Tab", Navigation);
    keys[15] = annotated_key("LEFT", "←", Navigation, "Physical A position");
    keys[16] = annotated_key("DOWN", "↓", Navigation, "Physical S position");
    keys[17] = annotated_key("RIGHT", "→", Navigation, "Physical D position");
    keys[18] = key("LSHFT", "Shift", Mod);
    keys[20] = Action::Workspace {
        number: 1,
        hold: None,
    };
    keys[21] = Action::Workspace {
        number: 2,
        hold: Some(RightShift),
    };
    keys[22] = Action::Workspace {
        number: 3,
        hold: Some(RightGui),
    };
    keys[23] = Action::Workspace {
        number: 4,
        hold: Some(RightAlt),
    };
    keys[24] = Action::Workspace {
        number: 5,
        hold: Some(RightControl),
    };

    let system_labels = [
        "display brightness down",
        "display brightness up",
        "Mission Control",
        "Spotlight",
        "Dictation",
        "Do Not Disturb",
        "previous track",
        "play / pause",
        "next track",
        "mute",
        "volume down",
        "volume up",
    ];
    for (offset, system_label) in system_labels.into_iter().enumerate() {
        keys[26 + offset] = Action::Function {
            number: (offset + 1) as u8,
            system_label,
        };
    }
    keys[38] = key("LCTRL", "Ctrl", Mod);
    keys[39] = key("LALT", "Option", Mod);
    keys[40] = key("LGUI", "Cmd", Mod);
    keys[41] = Action::Transparent;
    keys[42] = key("SPACE", "Space", Category::Alpha);
    keys[43] = Action::Transparent;

    Layer {
        id: LayerId::Nav,
        name: "Nav / WM",
        description: "Hold Enter/Nav: WASD-shaped arrows on the left, AeroSpace workspaces and hold modifiers on the right, and literal F1–F12 below.",
        keys,
    }
}

fn num() -> Layer {
    use Category::{Alpha, Connection, Danger, Modifier as Mod, Navigation, Symbol};
    use GuardedAction::{
        Bootloader, ClearAllBluetooth, ClearCurrentBluetooth, Reset, SelectBluetooth,
    };
    use Modifier::{LeftShift, RightShift};

    let mut keys = vec![Action::None; 44];
    keys[0] = Action::HyperEscape;
    keys[13] = annotated_key("EQUAL", "=", Symbol, "With Shift: plus");
    keys[14] = key("TAB", "Tab", Navigation);
    keys[15] = annotated_key("N1", "1", Symbol, "With Shift: !");
    keys[16] = annotated_key("N2", "2", Symbol, "With Shift: @");
    keys[17] = annotated_key("N3", "3", Symbol, "With Shift: #");
    keys[18] = shift_tap("N4", "4", LeftShift, "With opposite Shift: $");
    keys[19] = annotated_key("N5", "5", Symbol, "With Shift: %");
    keys[20] = annotated_key("N6", "6", Symbol, "With Shift: ^");
    keys[21] = shift_tap("N7", "7", RightShift, "With opposite Shift: &");
    keys[22] = annotated_key("N8", "8", Symbol, "With Shift: *");
    keys[23] = annotated_key("N9", "9", Symbol, "With Shift: (");
    keys[24] = annotated_key("N0", "0", Symbol, "With Shift: )");
    keys[25] = annotated_key("MINUS", "−", Symbol, "With Shift: underscore");
    keys[27] = guarded(
        "Clear BT",
        "Hyper + tap clears the selected Bluetooth profile",
        ClearCurrentBluetooth,
        Danger,
    );
    keys[28] = guarded(
        "Clear all",
        "Hyper + tap clears every Bluetooth bond",
        ClearAllBluetooth,
        Danger,
    );
    keys[29] = guarded(
        "Reboot L",
        "Hyper + tap restarts the central half",
        Reset,
        Danger,
    );
    keys[30] = guarded(
        "Flash L",
        "Hyper + tap enters the central-half bootloader",
        Bootloader,
        Danger,
    );
    for profile in 0..5 {
        keys[32 + profile as usize] = guarded(
            match profile {
                0 => "BT 1",
                1 => "BT 2",
                2 => "BT 3",
                3 => "BT 4",
                _ => "BT 5",
            },
            "Hyper + tap selects this Bluetooth profile",
            SelectBluetooth(profile),
            Connection,
        );
    }
    keys[38] = key("LCTRL", "Ctrl", Mod);
    keys[39] = key("LALT", "Option", Mod);
    keys[40] = key("LGUI", "Cmd", Mod);
    keys[41] = Action::NavEnter;
    keys[42] = key("SPACE", "Space", Alpha);
    keys[43] = Action::BackspaceNum;

    Layer {
        id: LayerId::Num,
        name: "Num / Symbol",
        description: "Hold Backspace for a stateless Num / Symbol layer: 1–0 occupy the home row, and 4/7 preserve the same mirrored index-finger hold-Shifts as Base.",
        keys,
    }
}

fn colemak() -> HostLegend {
    HostLegend {
        id: "colemak-dh-ansi",
        name: "Colemak-DH ANSI",
        keys: &[
            ("P01", "`"),
            ("P02", "q"),
            ("P03", "w"),
            ("P04", "f"),
            ("P05", "p"),
            ("P06", "b"),
            ("P07", "j"),
            ("P08", "l"),
            ("P09", "u"),
            ("P10", "y"),
            ("P11", ";"),
            ("P12", "["),
            ("P13", "]"),
            ("P15", "a"),
            ("P16", "r"),
            ("P17", "s"),
            ("P18", "t"),
            ("P19", "g"),
            ("P20", "m"),
            ("P21", "n"),
            ("P22", "e"),
            ("P23", "i"),
            ("P24", "o"),
            ("P25", "'"),
            ("P27", "z"),
            ("P28", "x"),
            ("P29", "c"),
            ("P30", "d"),
            ("P31", "v"),
            ("P32", "k"),
            ("P33", "h"),
            ("P34", ","),
            ("P35", "."),
            ("P36", "/"),
            ("P37", "\\"),
        ],
    }
}

fn russian() -> HostLegend {
    HostLegend {
        id: "russian",
        name: "Russian",
        keys: &[
            ("P01", "ё"),
            ("P02", "й"),
            ("P03", "ц"),
            ("P04", "у"),
            ("P05", "к"),
            ("P06", "е"),
            ("P07", "н"),
            ("P08", "г"),
            ("P09", "ш"),
            ("P10", "щ"),
            ("P11", "з"),
            ("P12", "х"),
            ("P13", "ъ"),
            ("P15", "ф"),
            ("P16", "ы"),
            ("P17", "в"),
            ("P18", "а"),
            ("P19", "п"),
            ("P20", "р"),
            ("P21", "о"),
            ("P22", "л"),
            ("P23", "д"),
            ("P24", "ж"),
            ("P25", "э"),
            ("P27", "я"),
            ("P28", "ч"),
            ("P29", "с"),
            ("P30", "м"),
            ("P31", "и"),
            ("P32", "т"),
            ("P33", "ь"),
            ("P34", "б"),
            ("P35", "ю"),
            ("P36", "."),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settled_layer_invariants_are_encoded() {
        dao44().validate().unwrap();
    }

    #[test]
    fn every_layer_has_the_real_dao44_key_count() {
        assert!(dao44().layers.iter().all(|layer| layer.keys.len() == 44));
    }
}

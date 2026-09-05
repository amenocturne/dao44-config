use std::collections::BTreeMap;

use anyhow::{Result, bail, ensure};
use serde::Serialize;

pub const KEY_COUNT: usize = 44;
pub const SHIFT_TAPPING_TERM_MS: u16 = 180;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LayerId {
    Base,
    Nav,
    Num,
}

impl LayerId {
    pub const fn ident(self) -> &'static str {
        match self {
            Self::Base => "BASE",
            Self::Nav => "NAV",
            Self::Num => "NUM",
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Nav => "nav-wm",
            Self::Num => "num-symbol",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Modifier {
    LeftShift,
    RightControl,
    RightAlt,
    RightGui,
    RightShift,
}

impl Modifier {
    pub const fn zmk(self) -> &'static str {
        match self {
            Self::LeftShift => "LSHFT",
            Self::RightControl => "RCTRL",
            Self::RightAlt => "RALT",
            Self::RightGui => "RGUI",
            Self::RightShift => "RSHFT",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::RightControl => "Ctrl",
            Self::RightAlt => "Option",
            Self::RightGui => "Cmd",
            Self::LeftShift | Self::RightShift => "Shift",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Category {
    Alpha,
    Modifier,
    Layer,
    Navigation,
    Symbol,
    System,
    Connection,
    Danger,
    Unassigned,
}

impl Category {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Modifier => "modifier",
            Self::Layer => "layer",
            Self::Navigation => "navigation",
            Self::Symbol => "symbol",
            Self::System => "system",
            Self::Connection => "connection",
            Self::Danger => "danger",
            Self::Unassigned => "unassigned",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardedAction {
    ClearCurrentBluetooth,
    ClearAllBluetooth,
    Reset,
    Bootloader,
    SelectBluetooth(u8),
}

impl GuardedAction {
    pub fn id(self) -> String {
        match self {
            Self::ClearCurrentBluetooth => "clear_bt".into(),
            Self::ClearAllBluetooth => "clear_all".into(),
            Self::Reset => "reset".into(),
            Self::Bootloader => "bootloader".into(),
            Self::SelectBluetooth(profile) => format!("bt_{}", profile + 1),
        }
    }

    pub fn zmk(self) -> String {
        match self {
            Self::ClearCurrentBluetooth => "&bt BT_CLR".into(),
            Self::ClearAllBluetooth => "&bt BT_CLR_ALL".into(),
            Self::Reset => "&sys_reset".into(),
            Self::Bootloader => "&bootloader".into(),
            Self::SelectBluetooth(profile) => format!("&bt BT_SEL {profile}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    None,
    Transparent,
    Key {
        code: &'static str,
        label: &'static str,
        category: Category,
        secondary: &'static str,
    },
    ModTap {
        modifier: Modifier,
        tap_code: &'static str,
        label: &'static str,
        tap: &'static str,
        secondary: &'static str,
    },
    HyperEscape,
    BackspaceNum,
    NavEnter,
    Workspace {
        number: u8,
        hold: Option<Modifier>,
    },
    Function {
        number: u8,
        system_label: &'static str,
    },
    Guarded {
        label: &'static str,
        detail: &'static str,
        action: GuardedAction,
        category: Category,
    },
}

impl Action {
    pub fn zmk(self) -> String {
        match self {
            Self::None => "&none".into(),
            Self::Transparent => "&trans".into(),
            Self::Key { code, .. } => format!("&kp {code}"),
            Self::ModTap {
                modifier, tap_code, ..
            } => {
                format!("&shift_tap {} {tap_code}", modifier.zmk())
            }
            Self::HyperEscape => "&mt LC(LA(LGUI)) ESC".into(),
            Self::BackspaceNum => "&lt NUM BSPC".into(),
            Self::NavEnter => "&lt NAV RET".into(),
            Self::Workspace { number, hold } => {
                let digit = if number == 10 {
                    "N0".to_owned()
                } else {
                    format!("N{number}")
                };
                let chord = format!("LC(LA(LG({digit})))");
                match hold {
                    Some(modifier) => format!("&mt {} {chord}", modifier.zmk()),
                    None => format!("&kp {chord}"),
                }
            }
            Self::Function { number, .. } => format!("&kp F{number}"),
            Self::Guarded { action, .. } => format!("&guard_{}_ctl", action.id()),
        }
    }

    pub fn preview(self) -> PreviewAction {
        match self {
            Self::None => PreviewAction::new("—", "Intentionally unassigned", Category::Unassigned),
            Self::Transparent => PreviewAction::new("▽", "Transparent to Base", Category::Unassigned),
            Self::Key { label, category, secondary, .. } => {
                PreviewAction::new(label, label, category).secondary(secondary)
            }
            Self::ModTap { modifier, label, tap, secondary, .. } => {
                PreviewAction::new(label, tap, Category::Modifier)
                    .hold(modifier.label())
                    .hold_face("hold: Shift")
                    .secondary(secondary)
                    .note(format!(
                        "Tap wins until the {SHIFT_TAPPING_TERM_MS} ms threshold; other key presses do not force Shift."
                    ))
            }
            Self::HyperEscape => PreviewAction::new("Esc", "Escape", Category::Modifier)
                .hold("Hyper · Ctrl+Option+Command")
                .hold_face("hold: Hyper")
                .note("Remote but available on every layer."),
            Self::BackspaceNum => PreviewAction::new("Backspace", "Backspace", Category::Layer)
                .hold("Momentary Num / Symbol")
                .hold_face("hold: Num")
                .secondary("Release always returns to Base"),
            Self::NavEnter => PreviewAction::new("Enter", "Return / Enter", Category::Layer)
                .hold("Momentary Nav / WM")
                .hold_face("hold: Nav")
                .secondary("Nav release is intended to return to Base"),
            Self::Workspace { number, hold } => {
                let mut action = PreviewAction::new(
                    format!("WS {number}"),
                    format!("Focus AeroSpace workspace {number}"),
                    Category::System,
                )
                .secondary(format!("With Shift: move window to workspace {number}"));
                if let Some(modifier) = hold {
                    action.hold = Some(modifier.label().into());
                    action.hold_face = Some(format!("hold: {}", modifier.label()));
                }
                action
            }
            Self::Function { number, system_label } => {
                PreviewAction::new(format!("F{number}"), format!("F{number}"), Category::System)
                    .secondary(format!("Planned Hyper action: {system_label}"))
                    .note("Literal F-key generation is implemented; firmware-native Mac system morph remains an explicit design question.")
            }
            Self::Guarded { label, detail, category, .. } => {
                PreviewAction::new(label, "Unassigned without the complete Hyper chord", category)
                    .hold_face("Hyper + tap")
                    .secondary(detail)
                    .note("The generated keymap requires Ctrl+Option+Command through nested mod-morph gates.")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Layer {
    pub id: LayerId,
    pub name: &'static str,
    pub description: &'static str,
    pub keys: Vec<Action>,
}

#[derive(Clone, Debug)]
pub struct LayoutSpec {
    pub title: &'static str,
    pub revision: &'static str,
    pub layers: Vec<Layer>,
    pub host_legends: Vec<HostLegend>,
    pub open_questions: Vec<&'static str>,
}

impl LayoutSpec {
    pub fn validate(&self) -> Result<()> {
        ensure!(!self.layers.is_empty(), "layout needs at least one layer");
        for layer in &self.layers {
            ensure!(
                layer.keys.len() == KEY_COUNT,
                "{} has {} keys; expected {KEY_COUNT}",
                layer.name,
                layer.keys.len()
            );
        }

        let base = self.layer(LayerId::Base)?;
        let nav = self.layer(LayerId::Nav)?;
        let num = self.layer(LayerId::Num)?;
        ensure!(matches!(base.keys[26], Action::None));
        ensure!(matches!(num.keys[26], Action::None));
        ensure!(matches!(base.keys[43], Action::BackspaceNum));
        ensure!(matches!(num.keys[43], Action::BackspaceNum));
        ensure!(
            matches!(base.keys[41], Action::NavEnter),
            "P41 must own Nav entry"
        );
        ensure!(
            matches!(num.keys[41], Action::NavEnter),
            "Nav must remain reachable from Num"
        );
        ensure!(matches!(
            base.keys[18],
            Action::ModTap {
                modifier: Modifier::LeftShift,
                ..
            }
        ));
        ensure!(matches!(
            base.keys[21],
            Action::ModTap {
                modifier: Modifier::RightShift,
                ..
            }
        ));
        ensure!(matches!(
            num.keys[18],
            Action::ModTap {
                modifier: Modifier::LeftShift,
                ..
            }
        ));
        ensure!(matches!(
            num.keys[21],
            Action::ModTap {
                modifier: Modifier::RightShift,
                ..
            }
        ));
        ensure!(matches!(num.keys[25], Action::Key { code: "MINUS", .. }));
        ensure!(matches!(num.keys[13], Action::Key { code: "EQUAL", .. }));
        ensure!(matches!(nav.keys[18], Action::Key { code: "LSHFT", .. }));
        ensure_unique_guard_ids(self)
    }

    pub fn layer(&self, id: LayerId) -> Result<&Layer> {
        self.layers
            .iter()
            .find(|layer| layer.id == id)
            .ok_or_else(|| anyhow::anyhow!("missing {} layer", id.ident()))
    }

    pub fn guarded_actions(&self) -> impl Iterator<Item = GuardedAction> + '_ {
        self.layers
            .iter()
            .flat_map(|layer| layer.keys.iter())
            .filter_map(|key| {
                if let Action::Guarded { action, .. } = key {
                    Some(*action)
                } else {
                    None
                }
            })
    }
}

#[derive(Clone, Debug)]
pub struct HostLegend {
    pub id: &'static str,
    pub name: &'static str,
    pub keys: &'static [(&'static str, &'static str)],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPayload {
    pub title: &'static str,
    pub revision: &'static str,
    pub geometry: Vec<Geometry>,
    pub layers: Vec<PreviewLayer>,
    pub host_legends: Vec<PreviewHostLegend>,
    pub sequences: Vec<Sequence>,
    pub open_questions: Vec<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewLayer {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub keys: BTreeMap<String, PreviewAction>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewHostLegend {
    pub id: &'static str,
    pub name: &'static str,
    pub keys: BTreeMap<&'static str, &'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewAction {
    pub primary: String,
    pub tap: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold_face: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary: Option<String>,
    pub category: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl PreviewAction {
    fn new(primary: impl Into<String>, tap: impl Into<String>, category: Category) -> Self {
        Self {
            primary: primary.into(),
            tap: tap.into(),
            hold: None,
            hold_face: None,
            secondary: None,
            category: category.slug(),
            note: None,
        }
    }
    fn hold(mut self, value: impl Into<String>) -> Self {
        self.hold = Some(value.into());
        self
    }
    fn hold_face(mut self, value: impl Into<String>) -> Self {
        self.hold_face = Some(value.into());
        self
    }
    fn secondary(mut self, value: impl Into<String>) -> Self {
        let value = value.into();
        if !value.is_empty() {
            self.secondary = Some(value);
        }
        self
    }
    fn note(mut self, value: impl Into<String>) -> Self {
        self.note = Some(value.into());
        self
    }
}

#[derive(Serialize)]
pub struct Geometry {
    pub id: String,
    pub index: usize,
    pub w: i32,
    pub h: i32,
    pub x: i32,
    pub y: i32,
    pub rot: f32,
    pub rx: i32,
    pub ry: i32,
    pub hand: &'static str,
    pub region: &'static str,
}

#[derive(Serialize)]
pub struct Sequence {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub steps: Vec<SequenceStep>,
}

#[derive(Serialize)]
pub struct SequenceStep {
    pub key: &'static str,
    pub label: &'static str,
}

fn ensure_unique_guard_ids(spec: &LayoutSpec) -> Result<()> {
    let mut seen = std::collections::BTreeSet::new();
    for action in spec.guarded_actions() {
        if !seen.insert(action.id()) {
            bail!("duplicate guarded action id: {}", action.id());
        }
    }
    Ok(())
}

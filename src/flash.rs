use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, ensure};

const LEFT_SERIAL_KEY: &str = "DAO_LEFT_BOOTLOADER_SERIAL";
const RIGHT_SERIAL_KEY: &str = "DAO_RIGHT_BOOTLOADER_SERIAL";
const BOOT_VOLUME_PREFIX: &str = "NRF52BOOT";

pub const HELP: &str = "Flash Dao44 firmware

Usage:
  just flash left    Build and flash the left half
  just flash right   Build and flash the right half
  just flash all     Build and flash every connected, known half
  just flash check   Show connected halves without writing

The first explicit left/right flash remembers that half's USB serial in local .env.";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Half {
    Left,
    Right,
}

impl Half {
    fn label(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    fn env_key(self) -> &'static str {
        match self {
            Self::Left => LEFT_SERIAL_KEY,
            Self::Right => RIGHT_SERIAL_KEY,
        }
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct FlashConfig {
    left_serial: Option<String>,
    right_serial: Option<String>,
}

impl FlashConfig {
    fn from_env(contents: &str) -> Result<Self> {
        let values = parse_env(contents)?;
        let config = Self {
            left_serial: non_empty(values.get(LEFT_SERIAL_KEY)),
            right_serial: non_empty(values.get(RIGHT_SERIAL_KEY)),
        };
        if let (Some(left), Some(right)) = (&config.left_serial, &config.right_serial) {
            ensure!(
                left != right,
                "left and right bootloader serials must be different"
            );
        }
        Ok(config)
    }

    fn half_for_serial(&self, serial: &str) -> Option<Half> {
        if self.left_serial.as_deref() == Some(serial) {
            Some(Half::Left)
        } else if self.right_serial.as_deref() == Some(serial) {
            Some(Half::Right)
        } else {
            None
        }
    }

    fn serial_for_half(&self, half: Half) -> Option<&str> {
        match half {
            Half::Left => self.left_serial.as_deref(),
            Half::Right => self.right_serial.as_deref(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UsbDevice {
    location: u32,
    product: String,
    serial: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BootloaderVolume {
    mount: PathBuf,
    serial: String,
}

#[derive(Debug, Eq, PartialEq)]
struct FlashPlan {
    half: Half,
    serial: String,
    mount: PathBuf,
    firmware: PathBuf,
}

pub fn print_help() {
    println!("{HELP}");
}

pub fn flash_half(root: &Path, half: Half) -> Result<()> {
    let volumes = discover_bootloaders()?;
    ensure!(
        !volumes.is_empty(),
        "no {BOOT_VOLUME_PREFIX} bootloader is mounted; double-tap reset on the {} half",
        half.label()
    );
    let env_path = root.join(".env");
    let existing = fs::read_to_string(&env_path).unwrap_or_default();
    let config = FlashConfig::from_env(&existing)?;
    let volume = select_half_volume(&config, half, volumes)?;

    let config = if config.serial_for_half(half).is_none() {
        let updated = upsert_env(&existing, half.env_key(), &volume.serial)?;
        fs::write(&env_path, &updated)
            .with_context(|| format!("failed to update {}", env_path.display()))?;
        println!("remembered {} half ({})", half.label(), volume.serial);
        FlashConfig::from_env(&updated)?
    } else {
        config
    };

    execute_flashes(plan_flashes(root, &config, vec![volume])?, false)
}

pub fn flash_registered(root: &Path, dry_run: bool) -> Result<()> {
    let env_path = root.join(".env");
    let contents = fs::read_to_string(&env_path).with_context(|| {
        format!(
            "{} is missing; run `just flash left` and `just flash right` once first",
            env_path.display()
        )
    })?;
    let config = FlashConfig::from_env(&contents)?;
    let volumes = discover_bootloaders()?;
    ensure!(
        !volumes.is_empty(),
        "no {BOOT_VOLUME_PREFIX} bootloader is mounted; double-tap reset on one or both halves"
    );
    execute_flashes(plan_flashes(root, &config, volumes)?, dry_run)
}

fn select_half_volume(
    config: &FlashConfig,
    half: Half,
    volumes: Vec<BootloaderVolume>,
) -> Result<BootloaderVolume> {
    if let Some(serial) = config.serial_for_half(half) {
        let mut matching = volumes.into_iter().filter(|volume| volume.serial == serial);
        let volume = matching.next().with_context(|| {
            format!(
                "the connected bootloaders do not include the registered {} half ({serial})",
                half.label()
            )
        })?;
        ensure!(
            matching.next().is_none(),
            "more than one mounted volume has the {} half's USB serial",
            half.label()
        );
        return Ok(volume);
    }

    let other_serial = config.serial_for_half(other_half(half));
    let mut candidates = volumes
        .into_iter()
        .filter(|volume| Some(volume.serial.as_str()) != other_serial);
    let volume = candidates.next().with_context(|| {
        format!(
            "no unregistered bootloader is available to remember as the {} half",
            half.label()
        )
    })?;
    ensure!(
        candidates.next().is_none(),
        "more than one unregistered bootloader is mounted; connect only the {} half for its first flash",
        half.label()
    );
    Ok(volume)
}

fn execute_flashes(plans: Vec<FlashPlan>, dry_run: bool) -> Result<()> {
    for plan in &plans {
        println!(
            "{}: {} -> {} ({})",
            if dry_run { "would flash" } else { "flashing" },
            plan.firmware.display(),
            plan.mount.display(),
            plan.half.label()
        );
    }
    if dry_run {
        return Ok(());
    }

    for plan in plans {
        let destination = plan.mount.join(format!("dao-{}.uf2", plan.half.label()));
        fs::copy(&plan.firmware, &destination).with_context(|| {
            format!(
                "failed to flash {} half at {}; the bootloader may have disconnected during the copy",
                plan.half.label(),
                plan.mount.display()
            )
        })?;
        println!("flashed {} half; it should now reboot", plan.half.label());
    }
    Ok(())
}

fn plan_flashes(
    root: &Path,
    config: &FlashConfig,
    volumes: Vec<BootloaderVolume>,
) -> Result<Vec<FlashPlan>> {
    let mut seen = HashSet::new();
    let mut plans = Vec::with_capacity(volumes.len());
    for volume in volumes {
        let half = config.half_for_serial(&volume.serial).with_context(|| {
            format!(
                "bootloader {} at {} is not known; run `just flash left` or `just flash right` with that half connected",
                volume.serial,
                volume.mount.display()
            )
        })?;
        ensure!(
            seen.insert(half),
            "more than one mounted device maps to the {} half",
            half.label()
        );
        let firmware = root.join("build").join(half.label()).join("zephyr/zmk.uf2");
        ensure!(
            firmware.is_file(),
            "{} firmware is missing at {}; run `just firmware` first",
            half.label(),
            firmware.display()
        );
        plans.push(FlashPlan {
            half,
            serial: volume.serial,
            mount: volume.mount,
            firmware,
        });
    }
    plans.sort_by_key(|plan| match plan.half {
        Half::Left => 0,
        Half::Right => 1,
    });
    Ok(plans)
}

fn discover_bootloaders() -> Result<Vec<BootloaderVolume>> {
    ensure!(
        cfg!(target_os = "macos"),
        "automatic bootloader discovery currently supports macOS only"
    );
    let usb_devices = usb_devices()?;
    let mut volumes = Vec::new();
    for entry in fs::read_dir("/Volumes").context("failed to inspect /Volumes")? {
        let mount = entry?.path();
        let Some(name) = mount.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.starts_with(BOOT_VOLUME_PREFIX) || !mount.is_dir() {
            continue;
        }
        let info = fs::read_to_string(mount.join("INFO_UF2.TXT")).unwrap_or_default();
        ensure!(
            info.contains("Model: Nordic nRF52840 DK")
                && info.contains("Board-ID: nRF52840-pca10056-v1"),
            "{} is named like a Dao bootloader but has unexpected UF2 identity",
            mount.display()
        );
        let disk_info = command_output("diskutil", &["info", "-plist", path_str(&mount)?])?;
        let device_tree_path = plist_string(&disk_info, "DeviceTreePath")
            .with_context(|| format!("{} has no USB device-tree path", mount.display()))?;
        let location = location_from_device_tree_path(&device_tree_path)?;
        let device = usb_devices.get(&location).with_context(|| {
            format!(
                "could not associate {} with its USB device at {location:#010x}",
                mount.display()
            )
        })?;
        ensure!(
            device.product == "PCA10056",
            "{} is attached to unexpected USB product {:?}",
            mount.display(),
            device.product
        );
        volumes.push(BootloaderVolume {
            mount,
            serial: device.serial.clone(),
        });
    }
    volumes.sort_by(|left, right| left.mount.cmp(&right.mount));
    Ok(volumes)
}

fn usb_devices() -> Result<HashMap<u32, UsbDevice>> {
    let output = command_output(
        "ioreg",
        &["-p", "IOUSB", "-r", "-c", "IOUSBHostDevice", "-l", "-w0"],
    )?;
    Ok(parse_ioreg(&output)
        .into_iter()
        .map(|device| (device.location, device))
        .collect())
}

fn command_output(command: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .with_context(|| format!("failed to run {command}"))?;
    ensure!(
        output.status.success(),
        "{command} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    String::from_utf8(output.stdout).with_context(|| format!("{command} returned non-UTF-8 output"))
}

fn parse_ioreg(contents: &str) -> Vec<UsbDevice> {
    let mut devices = Vec::new();
    let mut current: Option<(u32, String, Option<String>)> = None;
    for line in contents.lines() {
        if let Some((product, location)) = parse_ioreg_header(line) {
            if let Some(device) = complete_usb_device(current.take()) {
                devices.push(device);
            }
            current = Some((location, product, None));
        } else if let Some(serial) = quoted_property(line, "USB Serial Number") {
            if let Some((_, _, current_serial)) = current.as_mut() {
                *current_serial = Some(serial);
            }
        }
    }
    if let Some(device) = complete_usb_device(current) {
        devices.push(device);
    }
    devices
}

fn parse_ioreg_header(line: &str) -> Option<(String, u32)> {
    let marker = "+-o ";
    let start = line.find(marker)? + marker.len();
    let header = &line[start..];
    if !header.contains("<class IOUSBHostDevice") {
        return None;
    }
    let (product, rest) = header.split_once('@')?;
    let location_hex = rest.split_whitespace().next()?;
    let location = u32::from_str_radix(location_hex, 16).ok()?;
    Some((product.to_owned(), location))
}

fn quoted_property(line: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\" = \"");
    let rest = line.split_once(&marker)?.1;
    Some(rest.split_once('"')?.0.to_owned())
}

fn complete_usb_device(current: Option<(u32, String, Option<String>)>) -> Option<UsbDevice> {
    let (location, product, serial) = current?;
    Some(UsbDevice {
        location,
        product,
        serial: serial?,
    })
}

fn plist_string(contents: &str, key: &str) -> Option<String> {
    let marker = format!("<key>{key}</key>");
    let after_key = contents.split_once(&marker)?.1;
    let after_open = after_key.split_once("<string>")?.1;
    Some(after_open.split_once("</string>")?.0.to_owned())
}

fn location_from_device_tree_path(path: &str) -> Result<u32> {
    let (_, suffix) = path
        .rsplit_once('@')
        .with_context(|| format!("unexpected USB device-tree path {path:?}"))?;
    u32::from_str_radix(suffix, 16)
        .with_context(|| format!("unexpected USB location in device-tree path {path:?}"))
}

fn parse_env(contents: &str) -> Result<HashMap<String, String>> {
    let mut values = HashMap::new();
    for (index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .with_context(|| format!("invalid .env line {}: expected KEY=VALUE", index + 1))?;
        values.insert(key.trim().to_owned(), unquote(value.trim()).to_owned());
    }
    Ok(values)
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn non_empty(value: Option<&String>) -> Option<String> {
    value.filter(|value| !value.is_empty()).cloned()
}

fn upsert_env(contents: &str, key: &str, value: &str) -> Result<String> {
    ensure!(
        !value.is_empty() && !value.contains(['\n', '\r', '=']),
        "invalid value for {key}"
    );
    let mut found = false;
    let mut lines = Vec::new();
    for line in contents.lines() {
        let matches_key = line
            .split_once('=')
            .is_some_and(|(candidate, _)| candidate.trim() == key);
        if matches_key {
            ensure!(!found, "{key} appears more than once in .env");
            lines.push(format!("{key}={value}"));
            found = true;
        } else {
            lines.push(line.to_owned());
        }
    }
    if !found {
        lines.push(format!("{key}={value}"));
    }
    Ok(format!("{}\n", lines.join("\n").trim_start_matches('\n')))
}

fn other_half(half: Half) -> Half {
    match half {
        Half::Left => Half::Right,
        Half::Right => Half::Left,
    }
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .with_context(|| format!("path is not valid UTF-8: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_ioreg_devices_by_location_and_serial() {
        let input = r#"
+-o PCA10056@01100000  <class IOUSBHostDevice, id 1>
      "USB Product Name" = "PCA10056"
      "USB Serial Number" = "LEFT123"
+-o PCA10056@02100000  <class IOUSBHostDevice, id 2>
      "USB Product Name" = "PCA10056"
      "USB Serial Number" = "RIGHT456"
"#;
        assert_eq!(
            parse_ioreg(input),
            vec![
                UsbDevice {
                    location: 0x01100000,
                    product: "PCA10056".into(),
                    serial: "LEFT123".into(),
                },
                UsbDevice {
                    location: 0x02100000,
                    product: "PCA10056".into(),
                    serial: "RIGHT456".into(),
                },
            ]
        );
    }

    #[test]
    fn extracts_disk_location_from_plist() {
        let plist = "<key>DeviceTreePath</key><string>IODeviceTree:/usb@02100000</string>";
        let path = plist_string(plist, "DeviceTreePath").unwrap();
        assert_eq!(location_from_device_tree_path(&path).unwrap(), 0x02100000);
    }

    #[test]
    fn parses_and_updates_local_registration() {
        let contents =
            "# local hardware\nDAO_LEFT_BOOTLOADER_SERIAL=LEFT123\nDAO_RIGHT_BOOTLOADER_SERIAL=\n";
        let updated = upsert_env(contents, RIGHT_SERIAL_KEY, "RIGHT456").unwrap();
        assert_eq!(
            FlashConfig::from_env(&updated).unwrap(),
            FlashConfig {
                left_serial: Some("LEFT123".into()),
                right_serial: Some("RIGHT456".into()),
            }
        );
    }

    #[test]
    fn rejects_one_serial_registered_for_both_halves() {
        let result = FlashConfig::from_env(
            "DAO_LEFT_BOOTLOADER_SERIAL=SAME\nDAO_RIGHT_BOOTLOADER_SERIAL=SAME\n",
        );
        assert!(result.is_err());
    }

    #[test]
    fn explicit_half_selects_its_registered_volume() {
        let config = FlashConfig {
            left_serial: Some("LEFT123".into()),
            right_serial: Some("RIGHT456".into()),
        };
        let selected = select_half_volume(
            &config,
            Half::Right,
            vec![
                volume("NRF52BOOT", "LEFT123"),
                volume("NRF52BOOT 1", "RIGHT456"),
            ],
        )
        .unwrap();

        assert_eq!(selected.serial, "RIGHT456");
    }

    #[test]
    fn first_explicit_half_ignores_the_known_other_half() {
        let config = FlashConfig {
            left_serial: Some("LEFT123".into()),
            right_serial: None,
        };
        let selected = select_half_volume(
            &config,
            Half::Right,
            vec![
                volume("NRF52BOOT", "LEFT123"),
                volume("NRF52BOOT 1", "RIGHT456"),
            ],
        )
        .unwrap();

        assert_eq!(selected.serial, "RIGHT456");
    }

    #[test]
    fn first_explicit_half_rejects_ambiguous_bootloaders() {
        let result = select_half_volume(
            &FlashConfig::default(),
            Half::Left,
            vec![
                volume("NRF52BOOT", "FIRST"),
                volume("NRF52BOOT 1", "SECOND"),
            ],
        );

        assert!(result.is_err());
    }

    fn volume(name: &str, serial: &str) -> BootloaderVolume {
        BootloaderVolume {
            mount: PathBuf::from("/Volumes").join(name),
            serial: serial.into(),
        }
    }
}

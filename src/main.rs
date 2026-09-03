mod flash;
mod generate;
mod layout;
mod model;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "generate".into());
    let spec = layout::dao44();

    match command.as_str() {
        "generate" => generate::write_all(&root, &spec),
        "check" => {
            spec.validate()?;
            check_generated(&root, &spec)
        }
        "register-left" => flash::register(&root, flash::Half::Left),
        "register-right" => flash::register(&root, flash::Half::Right),
        "flash-check" => flash::flash(&root, true),
        "flash" => flash::flash(&root, false),
        other => bail!(
            "unknown command {other:?}; expected generate, check, register-left, register-right, flash-check, or flash"
        ),
    }
}

fn check_generated(root: &Path, spec: &model::LayoutSpec) -> Result<()> {
    let expected_preview = generate::preview_json(spec)?;
    let expected_keymap = generate::zmk_keymap(spec)?;
    check_file(&root.join("generated/layout.json"), &expected_preview)?;
    check_file(&root.join("config/dao.keymap"), &expected_keymap)
}

fn check_file(path: &Path, expected: &str) -> Result<()> {
    let actual = std::fs::read_to_string(path)
        .with_context(|| format!("generated file is missing: {}", path.display()))?;
    if actual != expected {
        bail!("{} is stale; run `just generate`", path.display());
    }
    Ok(())
}

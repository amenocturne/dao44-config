mod generate;
mod layout;
mod model;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

fn main() -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let command = std::env::args().nth(1).unwrap_or_else(|| "generate".into());
    let spec = layout::dao44();

    match command.as_str() {
        "generate" => generate::write_all(&root, &spec),
        "check" => {
            spec.validate()?;
            check_generated(&root, &spec)
        }
        other => bail!("unknown command {other:?}; expected generate or check"),
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

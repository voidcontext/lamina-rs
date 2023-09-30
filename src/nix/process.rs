use std::{path::Path, process::Command};

pub fn override_input(input_name: &str, input_url: &str, dir: &Path) -> anyhow::Result<()> {
    let mut cmd = Command::new("nix");
    cmd.args([
        "flake",
        "lock",
        dir.to_str().expect("Couldn't convert path to string"),
        "--override-input",
        input_name,
        input_url,
    ]);

    log::debug!("running command: {:?}", cmd);

    cmd.status()?;

    Ok(())
}

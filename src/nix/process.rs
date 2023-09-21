use std::process::Command;

pub fn override_input(input_name: &str, input_url: &str) -> anyhow::Result<()> {
    let mut cmd = Command::new("nix");
    cmd.args(["flake", "lock", "--override-input", input_name, input_url]);

    log::debug!("running command: {:?}", cmd);

    cmd.status()?;

    Ok(())
}

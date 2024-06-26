use std::{
    env::current_dir,
    fs,
    io::{self, Write},
    path::Path,
};

use assert_cmd::Command;

use cmd_lib_macros::run_cmd;
use lamina::nix::flake_lock::FlakeLock;
use sealed_test::prelude::*;

use pretty_assertions::assert_eq;

fn load_flake_lock(p: &Path) -> FlakeLock {
    let mut path = p.to_path_buf();
    path.push("flake.lock");
    let flake_lock_json = fs::read_to_string(path).expect("Could read flake.lock");

    serde_json::from_str::<FlakeLock>(&flake_lock_json).expect("Couldn't parse flake.lock")
}

fn run_test(
    src_dir: &str,
    dst_dir: &str,
    src_input_name: &str,
    dst_input_name: &str,
    expected_flake_nix: &str,
) {
    run_cmd!(
        git init .;
        git add .;
    )
    .unwrap();

    let mut working_dir = current_dir().unwrap();
    working_dir.push(dst_dir);

    let output = Command::cargo_bin("lamina")
        .unwrap()
        .args([
            "sync",
            format!("../{src_dir}").as_str(),
            src_input_name,
            ".",
            dst_input_name,
        ])
        .current_dir(working_dir)
        .unwrap();

    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());

    let synced = fs::read_to_string(format!("{dst_dir}/flake.nix")).unwrap();
    assert_eq!(synced, expected_flake_nix);

    let source_flake_lock = load_flake_lock(Path::new(src_dir));
    let destination_flake_lock = load_flake_lock(Path::new(dst_dir));

    assert_eq!(
        source_flake_lock
            .nodes
            .get(src_input_name)
            .map(|n| n.original.clone()),
        destination_flake_lock
            .nodes
            .get(dst_input_name)
            .map(|n| n.original.clone())
    );
    assert_eq!(
        source_flake_lock
            .nodes
            .get(src_input_name)
            .map(|n| n.locked.clone()),
        destination_flake_lock
            .nodes
            .get(dst_input_name)
            .map(|n| n.locked.clone())
    );
}

#[sealed_test(files=["tests/fixtures/nested", "tests/fixtures/oneline"])]
fn test_sync_1() {
    let expected = r#"{
  inputs.nixpkgs-indirect-ref.url = "nixpkgs/8b3ad2fe8e06c2189908b7164f2f0bf2126b83b1";
  inputs.nixpkgs-indirect-rev.url = "nixpkgs/dc1517e4b9e481e15bf7c80740a6a8a1423fe3ad";
  inputs.nixpkgs-github.url = "github:Nixos/nixpkgs/release-23.05";
  inputs.nix-rust-utils-git.url = "git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.3.0";

  outputs = {...}: {};
}
"#;

    run_test(
        "nested",
        "oneline",
        "nixpkgs-indirect-rev",
        "nixpkgs-indirect-ref",
        expected,
    );
}

#[sealed_test(files=["tests/fixtures/nested", "tests/fixtures/oneline"])]
fn test_sync_2() {
    let expected = r#"{
  inputs = {
    nixpkgs-indirect-ref.url = "nixpkgs/release-23.05";
    nixpkgs-indirect-rev = {
      url = "nixpkgs/release-23.05";
    };
    nixpkgs-github = {
      url = "github:Nixos/nixpkgs/release-23.05";
    };
    nix-rust-utils-git = {
      url = "git+https://github.com/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.4.0";
    };
  };

  outputs = {...}: {};
}
"#;

    run_test(
        "oneline",
        "nested",
        "nixpkgs-indirect-ref",
        "nixpkgs-indirect-rev",
        expected,
    );
}

#[sealed_test(files=["tests/fixtures/nested", "tests/fixtures/oneline"])]
fn test_batch_sync() {
    run_cmd!(
        git init .;
        git add .;
    )
    .unwrap();

    let working_dir = current_dir().unwrap().to_str().unwrap().to_string();

    let output = Command::cargo_bin("lamina")
        .unwrap()
        .args([
            "-d",
            "batch-sync",
            format!("{working_dir}/nested").as_str(),
            format!("{working_dir}/oneline").as_str(),
            "nixpkgs-indirect-ref",
            "nixpkgs-indirect-rev",
            "nixpkgs-github",
            // "nix-rust-utils-git", // TODO: can't we switch git repositories?
        ])
        .env("HOME", working_dir.as_str())
        .unwrap();

    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());

    let synced = fs::read_to_string(format!("{working_dir}/oneline/flake.nix"))
        .expect("Cannot read fixture: oneline/flake.nix");
    assert_eq!(
        synced,
        r#"{
  inputs.nixpkgs-indirect-ref.url = "nixpkgs/release-23.05";
  inputs.nixpkgs-indirect-rev.url = "nixpkgs/8b3ad2fe8e06c2189908b7164f2f0bf2126b83b1";
  inputs.nixpkgs-github.url = "github:Nixos/nixpkgs/release-23.05";
  inputs.nix-rust-utils-git.url = "git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.3.0";

  outputs = {...}: {};
}
"#
    );

    let source_flake_lock = load_flake_lock(Path::new(&format!("{working_dir}/nested")));
    let destination_flake_lock = load_flake_lock(Path::new(&format!("{working_dir}/oneline")));

    for input_name in &[
        "nixpkgs-indirect-ref",
        "nixpkgs-indirect-rev",
        "nixpkgs-github",
    ] {
        assert_eq!(
            source_flake_lock
                .nodes
                .get(*input_name)
                .map(|n| n.original.clone()),
            destination_flake_lock
                .nodes
                .get(*input_name)
                .map(|n| n.original.clone())
        );
        assert_eq!(
            source_flake_lock
                .nodes
                .get(*input_name)
                .map(|n| n.locked.clone()),
            destination_flake_lock
                .nodes
                .get(*input_name)
                .map(|n| n.locked.clone())
        );
    }
}

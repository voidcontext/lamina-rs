use std::{
    env::current_dir,
    fs,
    io::{self, Write},
    path::Path,
};

use assert_cmd::Command;

use cmd_lib_macros::run_cmd;
use lamina::nix::file::flake_lock::FlakeLock;
use sealed_test::prelude::*;

use pretty_assertions::assert_eq;

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

    let source_flake_lock =
        FlakeLock::try_from(Path::new(src_dir)).expect("Couldn't parse source flake");
    let destination_flake_lock =
        FlakeLock::try_from(Path::new(dst_dir)).expect("Couldn't parse destination flake");

    assert_eq!(
        source_flake_lock.original_of(src_input_name),
        destination_flake_lock.original_of(dst_input_name)
    );
    assert_eq!(
        source_flake_lock.locked_of(src_input_name),
        destination_flake_lock.locked_of(dst_input_name)
    );
}

#[sealed_test(files=["tests/fixtures/nested", "tests/fixtures/oneline"])]
#[ignore]
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
#[ignore]
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
#[ignore]
fn test_batch_sync() {
    run_cmd!(
        git init .;
        git add .;
    )
    .unwrap();

    let mut working_dir = current_dir().unwrap();
    working_dir.push("oneline");

    let output = Command::cargo_bin("lamina")
        .unwrap()
        .args([
            "batch-sync",
            "../nested",
            "nixpkgs-indirect-ref",
            "nixpkgs-indirect-rev",
            "nixpkgs-github",
            // "nix-rust-utils-git", // TODO: can't we switch git repositories?
        ])
        .current_dir(working_dir)
        .unwrap();

    println!("status: {}", output.status);
    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    assert!(output.status.success());

    let synced = fs::read_to_string("oneline/flake.nix").unwrap();
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

    let source_flake_lock =
        FlakeLock::try_from(Path::new("nested")).expect("Couldn't parse source flake");
    let destination_flake_lock =
        FlakeLock::try_from(Path::new("oneline")).expect("Couldn't parse destination flake");

    for input_name in &[
        "nixpkgs-indirect-ref",
        "nixpkgs-indirect-rev",
        "nixpkgs-github",
    ] {
        assert_eq!(
            source_flake_lock.original_of(input_name),
            destination_flake_lock.original_of(input_name)
        );
        assert_eq!(
            source_flake_lock.locked_of(input_name),
            destination_flake_lock.locked_of(input_name)
        );
    }
}

use std::{env::current_dir, path::PathBuf, process::Command};

use anyhow::Result;

use crate::nix::FlakeLock;

pub fn sync(dst_input: &str, src_path: PathBuf, src_input: &str) -> anyhow::Result<()> {
    let source_flake_lock = FlakeLock::try_from(src_path)?;
    let destination_flake_lock = FlakeLock::try_from(current_dir()?)?;

    let source_rev = source_flake_lock.locked_rev_of(src_input).ok_or_else(|| {
        anyhow::Error::msg(format!("{src_input} doesn't have a revision in source"))
    })?;

    let destination_rev = destination_flake_lock
        .locked_rev_of(dst_input)
        .ok_or_else(|| {
            anyhow::Error::msg(format!("{dst_input} doesn't have a revision in source"))
        })?;

    println!("destination rev of {dst_input} is: {destination_rev}");
    println!("source rev of {src_input} is: {source_rev}");

    let input_override = input_override_arg(
        dst_input,
        src_input,
        &destination_flake_lock,
        &source_flake_lock,
    )?;

    Command::new("nix")
        .args([
            "flake",
            "lock",
            "--override-input",
            dst_input,
            &input_override,
        ])
        .status()?;

    Ok(())
}

fn input_override_arg(
    dst_input: &str,
    src_input: &str,
    dst: &FlakeLock,
    src: &FlakeLock,
) -> Result<String> {
    let same_original_definition = dst
        .original_of(dst_input)
        .as_ref()
        .zip(src.original_of(src_input).as_ref())
        .map(|(dst, src)| dst == src);

    match same_original_definition {
        Some(true) => {
            let locked = src
                .nodes
                .get(src_input)
                .ok_or_else(|| anyhow::Error::msg("Input is missing"))?
                .locked
                .as_ref()
                .ok_or_else(|| anyhow::Error::msg("Locked block is missing from source"))?;

            match locked {
                crate::nix::Locked::Git {
                    rev,
                    r#ref,
                    url,
                    last_modified: _,
                } => {
                    let mut query_str = format!("rev={rev}");
                    if let Some(r#ref) = r#ref {
                        query_str = format!("ref={ref}&{query_str}");
                    }
                    Ok(format!("git+{url}?{query_str}"))
                }
                crate::nix::Locked::Github {
                    rev: _,
                    r#ref: _,
                    owner: _,
                    repo: _,
                    last_modified: _,
                } => todo!(),
                crate::nix::Locked::GitLab {
                    rev: _,
                    r#ref: _,
                    owner: _,
                    repo: _,
                    last_modified: _,
                } => todo!(),
            }
        }
        Some(false) => Err(anyhow::Error::msg(
            "Cannot override inputs when the original declaration is different",
        )),
        None => Err(anyhow::Error::msg("Origin is missing from flake.lock")),
    }
}

#[cfg(test)]
mod tests {

    use rstest::rstest;

    use crate::commands::sync::input_override_arg;
    use crate::nix::fixtures::{flake_lock, flake_lock_with_node, inputs};
    use crate::nix::{Locked, Node, Original};
    use time::OffsetDateTime;

    #[rstest]
    #[case(
        None,
        Some(Original::github("owner", "repo", Some("ref"))),
        "Origin is missing from flake.lock"
    )]
    #[case(
        Some(Original::git_url_only("some-git-url")),
        None,
        "Origin is missing from flake.lock"
    )]
    #[case(
        Some(Original::git_url_only("some-git-url")),
        Some(Original::github("owner", "repo", Some("ref"))),
        "Cannot override inputs when the original declaration is different"
    )]
    #[case(
        Some(Original::git_url_only("some-git-url")),
        Some(Original::git_with_ref("some-git-url", "ref")),
        "Cannot override inputs when the original declaration is different"
    )]
    #[case(
        Some(Original::git_with_ref("some-git-url", "ref1")),
        Some(Original::git_with_ref("some-git-url", "ref2")),
        "Cannot override inputs when the original declaration is different"
    )]
    #[case(
        Some(Original::git_with_ref("some-git-url", "ref")),
        Some(Original::git_with_ref_and_rev("some-git-url", "ref", "some-rev")),
        "Cannot override inputs when the original declaration is different"
    )]
    fn input_override_arg_returns_error_when_dst_and_src_original_different_or_missing(
        #[case] original1: Option<Original>,
        #[case] original2: Option<Original>,
        #[case] expected_error: &str,
    ) {
        let result = input_override_arg(
            "nix-rust-utils",
            "nix-rust-utils",
            &flake_lock(original1),
            &flake_lock(original2),
        );

        assert_eq!(
            result.map_err(|e| e.to_string()),
            Err(String::from(expected_error))
        );
    }

    #[rstest]
    #[case(
        Original::git_url_only("https://example.com/user/repo.git"),
        Node {
            flake: None,
            inputs: Some(inputs("nixpkgs")),
            locked: Some(Locked::Git {
                rev: String::from("f542386b0646cf39b9475a200979adabd07d98b2"),
                r#ref: None,
                url: String::from("https://example.com/user/repo.git"),
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            }),
            original: Some(Original::git_url_only("https://example.com/user/repo.git")),
        },
        "git+https://example.com/user/repo.git?rev=f542386b0646cf39b9475a200979adabd07d98b2"
    )]
    #[case(
        Original::git_with_ref("https://example.com/user/repo.git", "refs/tags/v0.1.0"),
        Node {
            flake: None,
            inputs: Some(inputs("nixpkgs")),
            locked: Some(Locked::Git {
                rev: String::from("f542386b0646cf39b9475a200979adabd07d98b2"),
                r#ref: Some(String::from("refs/tags/v0.1.0")),
                url: String::from("https://example.com/user/repo.git"),
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            }),
            original: Some(Original::git_with_ref("https://example.com/user/repo.git", "refs/tags/v0.1.0")),
        },
        "git+https://example.com/user/repo.git?ref=refs/tags/v0.1.0&rev=f542386b0646cf39b9475a200979adabd07d98b2"
    )]
    #[case(
        Original::git_with_rev("https://example.com/user/repo.git", "f542386b0646cf39b9475a200979adabd07d98b2"),
        Node {
            flake: None,
            inputs: Some(inputs("nixpkgs")),
            locked: Some(Locked::Git {
                rev: String::from("f542386b0646cf39b9475a200979adabd07d98b2"),
                r#ref: None,
                url: String::from("https://example.com/user/repo.git"),
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            }),
            original: Some(Original::git_with_rev("https://example.com/user/repo.git", "f542386b0646cf39b9475a200979adabd07d98b2")),
        },
        "git+https://example.com/user/repo.git?rev=f542386b0646cf39b9475a200979adabd07d98b2"
    )]
    fn input_override_arg_returns_correct_argument(
        #[case] original1: Original,
        #[case] node2: Node,
        #[case] expected: &str,
    ) {
        let result = input_override_arg(
            "nix-rust-utils",
            "nix-rust-utils",
            &flake_lock(Some(original1)),
            &flake_lock_with_node(node2),
        );

        assert_eq!(
            result.map_err(|e| e.to_string()),
            Ok(String::from(expected))
        );
    }
}

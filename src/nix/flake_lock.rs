use crate::nix::file::flake_lock::FlakeLock;
use crate::nix::SyncStrategy;
use anyhow::anyhow;

use super::{
    file::flake_lock::{Locked, Original},
    SyncInputNames,
};

/// Calculates the sync strategy based on the given source and destination
pub fn sync_strategy<'a>(
    src_lock: &FlakeLock,
    dst_lock: &FlakeLock,
    input_names: &'a SyncInputNames,
) -> anyhow::Result<SyncStrategy<'a>> {
    let src_original = src_lock.original_of(input_names.source()).ok_or_else(|| {
        anyhow!(
            "Couldn't find {}'s Original at source.",
            input_names.source()
        )
    })?;
    let src_locked = src_lock
        .locked_of(input_names.source())
        .ok_or_else(|| anyhow!("Couldn't find {}'s Locked at source.", input_names.source()))?;
    let dst_original = dst_lock
        .original_of(input_names.destination())
        .ok_or_else(|| {
            anyhow!(
                "Couldn't find {}'s Original at destination.",
                input_names.destination()
            )
        })?;

    let override_url = override_url(&src_original, &src_locked)?;

    println!(
        "src: {} == dst: {}",
        src_original.base(),
        dst_original.base()
    );

    if src_original.base() == dst_original.base() {
        if src_original.r#ref() == dst_original.r#ref() && src_original.rev() == dst_original.rev()
        {
            Ok(SyncStrategy::lock_only(override_url, input_names))
        } else {
            Ok(SyncStrategy::flake_nix_and_lock(override_url, input_names))
        }
    } else {
        Err(anyhow!(
            "Cannot sync inputst with different type or from different git repository"
        ))
    }
}

fn override_url(original: &Original, locked: &Locked) -> anyhow::Result<String> {
    match original {
        Original::Indirect {
            id,
            rev: _,
            r#ref: _,
        } => Ok(format!("{id}/{}", &**locked.rev())),
        _ => match locked {
            Locked::Git {
                rev,
                r#ref,
                url,
                last_modified: _,
            } => {
                let mut query_str = format!("rev={}", &**rev);
                if let Some(r#ref) = r#ref {
                    query_str = format!("ref={}&{query_str}", &**r#ref);
                }
                Ok(format!("git+{url}?{query_str}"))
            }
            Locked::Github {
                rev,
                r#ref: _,
                owner,
                repo,
                last_modified: _,
            } => Ok(format!("github:{owner}/{repo}/{}", &**rev)),
            Locked::GitLab {
                rev: _,
                r#ref: _,
                owner: _,
                repo: _,
                last_modified: _,
            } => Err(anyhow!("Gitlab is not supported yet")),
        },
    }
}

#[cfg(test)]
mod tests {

    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::sync_strategy;
    use crate::nix::file::flake_lock::fixtures::{
        flake_lock_with_node, git_node_with_ref, git_node_with_rev, git_node_with_url_only,
        github_node_with_owner_and_repo_only, github_node_with_ref, nixpkgs_node,
        nixpkgs_node_with_ref,
    };
    use crate::nix::file::flake_lock::LockedRev;
    use crate::nix::file::flake_lock::Node;
    use crate::nix::file::flake_lock::OriginalRef;
    use crate::nix::{SyncInputNames, SyncStrategy};

    lazy_static::lazy_static! {
        static ref HASH_1: LockedRev = LockedRev::from("f542386b0646cf39b9475a200979adabd07d98b2");
        static ref HASH_2: LockedRev = LockedRev::from("4468e5deabf5e6d0740cd1a77df56f67093ec943");
        static ref INPUT_NAMES: SyncInputNames = SyncInputNames::same("nix-rust-utils".to_string());
    }

    #[rstest]
    // src -> dest -> strategy
    #[case( // git url -> git url -> lock only
        git_node_with_url_only("https://example.com/user/repo.git", &HASH_1),
        git_node_with_url_only("https://example.com/user/repo.git", &HASH_2),
        Ok(SyncStrategy::lock_only(format!(
            "git+https://example.com/user/repo.git?rev={}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// git url + ref -> git url -> flake.nix + lock
        git_node_with_url_only("https://example.com/user/repo.git", &HASH_1),
        git_node_with_ref("https://example.com/user/repo.git", &HASH_2,&OriginalRef::from("refs/tags/v0.2.0")),
        Ok(SyncStrategy::flake_nix_and_lock(format!(
            "git+https://example.com/user/repo.git?ref=refs/tags/v0.2.0&rev={}", &**HASH_2         
        ), &INPUT_NAMES))
    )]
    #[case(// git url + ref -> git url + ref (different ref) -> flake.nix + lock
        git_node_with_ref("https://example.com/user/repo.git", &HASH_1,&OriginalRef::from("refs/tags/v0.1.0")),
        git_node_with_ref("https://example.com/user/repo.git", &HASH_2,&OriginalRef::from("refs/tags/v0.2.0")),
        Ok(SyncStrategy::flake_nix_and_lock(format!(
            "git+https://example.com/user/repo.git?ref=refs/tags/v0.2.0&rev={}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// git url + ref -> git url + ref (same ref) -> lock only
        git_node_with_ref("https://example.com/user/repo.git", &HASH_1,&OriginalRef::from("refs/tags/v0.2.0")),
        git_node_with_ref("https://example.com/user/repo.git", &HASH_2,&OriginalRef::from("refs/tags/v0.2.0")),
        Ok(SyncStrategy::lock_only(format!(
            "git+https://example.com/user/repo.git?ref=refs/tags/v0.2.0&rev={}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// git url + rev -> git url + rev (same rev) -> noop
        git_node_with_rev("https://example.com/user/repo.git", &HASH_1),
        git_node_with_rev("https://example.com/user/repo.git", &HASH_1),
        Ok(SyncStrategy::lock_only(format!(
            "git+https://example.com/user/repo.git?rev={}", &**HASH_1
        ), &INPUT_NAMES)) // TODO: this should be NOOP really
    )]
    #[case(// git url + rev -> git url + rev (different rev) -> flake.nix + lock
        git_node_with_rev("https://example.com/user/repo.git", &HASH_1),
        git_node_with_rev("https://example.com/user/repo.git", &HASH_2),
        Ok(SyncStrategy::flake_nix_and_lock(format!(
            "git+https://example.com/user/repo.git?rev={}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// github -> github -> lock only
        github_node_with_owner_and_repo_only("owner", "repo", &HASH_1),
        github_node_with_owner_and_repo_only("owner", "repo", &HASH_2),
        Ok(SyncStrategy::lock_only(format!(
            "github:owner/repo/{}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// github + ref -> github + ref (same ref) -> lock only
        github_node_with_ref("owner", "repo", &HASH_1, &OriginalRef::from("main")),
        github_node_with_ref("owner", "repo", &HASH_2, &OriginalRef::from("main")),
        Ok(SyncStrategy::lock_only(format!(
            "github:owner/repo/{}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// github + ref -> github + ref (different ref) -> flake.nix + lock
        github_node_with_ref("owner", "repo", &HASH_1, &OriginalRef::from("main")),
        github_node_with_ref("owner", "repo", &HASH_2, &OriginalRef::from("feature-branch")),
        Ok(SyncStrategy::flake_nix_and_lock(format!(
            "github:owner/repo/{}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// indirect + ref -> indirect + ref (same ref) -> lock only
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &HASH_1),
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &HASH_2),
        Ok(SyncStrategy::lock_only(format!(
            "nixpkgs/{}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// github -> git -> error
        git_node_with_url_only("https://example.com/user/repo.git", &HASH_1),
        github_node_with_ref("owner", "repo", &HASH_1, &OriginalRef::from("main")),
        Err("Cannot sync inputst with different type or from different git repository")
    )]
    #[case(
        nixpkgs_node(&HASH_1, None),
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &HASH_2),
        Err("Couldn't find nix-rust-utils's Original at destination.")
    )]
    #[case(
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &HASH_2),
        nixpkgs_node(&HASH_1, None),
        Err("Couldn't find nix-rust-utils's Original at source.")
    )]
    fn input_override_arg_returns_correct_argument(
        #[case] node1: Node,
        #[case] node2: Node,
        #[case] expected: Result<SyncStrategy, &str>,
    ) {
        let result = sync_strategy(
            &flake_lock_with_node("nix-rust-utils", node2),
            &flake_lock_with_node("nix-rust-utils", node1),
            &INPUT_NAMES,
        );

        assert_eq!(
            result.map_err(|e| e.to_string()),
            expected.map_err(String::from)
        );
    }
}

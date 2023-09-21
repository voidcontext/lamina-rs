use crate::nix::file::flake_lock::FlakeLock;
use crate::nix::SyncStrategy;
use anyhow::anyhow;

use super::file::flake_lock::{Locked, Original};

/// Calculates the sync strategy based on the given source and destination
pub fn sync_strategy(
    src_lock: &FlakeLock,
    src_input_name: &str,
    dst_lock: &FlakeLock,
    dst_input_name: &str,
) -> anyhow::Result<SyncStrategy> {
    let src_original = src_lock
        .original_of(src_input_name)
        .ok_or_else(|| anyhow!("Couldn't find {src_input_name}'s Original at source."))?;
    let src_locked = src_lock
        .locked_of(src_input_name)
        .ok_or_else(|| anyhow!("Couldn't find {src_input_name}'s Locked at source."))?;
    let dst_original = dst_lock
        .original_of(dst_input_name)
        .ok_or_else(|| anyhow!("Couldn't find {dst_input_name}'s Original at destination."))?;

    let override_url = override_url(&src_original, &src_locked)?;

    println!(
        "src: {} == dst: {}",
        src_original.base(),
        dst_original.base()
    );

    if src_original.base() == dst_original.base() {
        if src_original.r#ref() == dst_original.r#ref() && src_original.rev() == dst_original.rev()
        {
            Ok(SyncStrategy::LockOnly(override_url))
        } else {
            Ok(SyncStrategy::FlakeNixAndLock(override_url))
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
    use crate::nix::SyncStrategy;

    fn hash1() -> LockedRev {
        LockedRev::from("f542386b0646cf39b9475a200979adabd07d98b2")
    }
    fn hash2() -> LockedRev {
        LockedRev::from("4468e5deabf5e6d0740cd1a77df56f67093ec943")
    }

    #[rstest]
    // src -> dest -> strategy
    #[case( // git url -> git url -> lock only
        git_node_with_url_only("https://example.com/user/repo.git", &hash1()),
        git_node_with_url_only("https://example.com/user/repo.git", &hash2()),
        Ok(SyncStrategy::LockOnly(format!(
            "git+https://example.com/user/repo.git?rev={}", &*hash2()
        )))
    )]
    #[case(// git url + ref -> git url -> flake.nix + lock
        git_node_with_url_only("https://example.com/user/repo.git", &hash1()),
        git_node_with_ref("https://example.com/user/repo.git", &hash2(),&OriginalRef::from("refs/tags/v0.2.0")),
        Ok(SyncStrategy::FlakeNixAndLock(format!(
            "git+https://example.com/user/repo.git?ref=refs/tags/v0.2.0&rev={}", &*hash2()         
        )))
    )]
    #[case(// git url + ref -> git url + ref (different ref) -> flake.nix + lock
        git_node_with_ref("https://example.com/user/repo.git", &hash1(),&OriginalRef::from("refs/tags/v0.1.0")),
        git_node_with_ref("https://example.com/user/repo.git", &hash2(),&OriginalRef::from("refs/tags/v0.2.0")),
        Ok(SyncStrategy::FlakeNixAndLock(format!(
            "git+https://example.com/user/repo.git?ref=refs/tags/v0.2.0&rev={}", &*hash2()
        )))
    )]
    #[case(// git url + ref -> git url + ref (same ref) -> lock only
        git_node_with_ref("https://example.com/user/repo.git", &hash1(),&OriginalRef::from("refs/tags/v0.2.0")),
        git_node_with_ref("https://example.com/user/repo.git", &hash2(),&OriginalRef::from("refs/tags/v0.2.0")),
        Ok(SyncStrategy::LockOnly(format!(
            "git+https://example.com/user/repo.git?ref=refs/tags/v0.2.0&rev={}", &*hash2()
        )))
    )]
    #[case(// git url + rev -> git url + rev (same rev) -> noop
        git_node_with_rev("https://example.com/user/repo.git", &hash1()),
        git_node_with_rev("https://example.com/user/repo.git", &hash1()),
        Ok(SyncStrategy::LockOnly(format!(
            "git+https://example.com/user/repo.git?rev={}", &*hash1()
        ))) // TODO: this should be NOOP really
    )]
    #[case(// git url + rev -> git url + rev (different rev) -> flake.nix + lock
        git_node_with_rev("https://example.com/user/repo.git", &hash1()),
        git_node_with_rev("https://example.com/user/repo.git", &hash2()),
        Ok(SyncStrategy::FlakeNixAndLock(format!(
            "git+https://example.com/user/repo.git?rev={}", &*hash2()
        )))
    )]
    #[case(// github -> github -> lock only
        github_node_with_owner_and_repo_only("owner", "repo", &hash1()),
        github_node_with_owner_and_repo_only("owner", "repo", &hash2()),
        Ok(SyncStrategy::LockOnly(format!(
            "github:owner/repo/{}", &*hash2()
        )))
    )]
    #[case(// github + ref -> github + ref (same ref) -> lock only
        github_node_with_ref("owner", "repo", &hash1(), &OriginalRef::from("main")),
        github_node_with_ref("owner", "repo", &hash2(), &OriginalRef::from("main")),
        Ok(SyncStrategy::LockOnly(format!(
            "github:owner/repo/{}", &*hash2()
        )))
    )]
    #[case(// github + ref -> github + ref (different ref) -> flake.nix + lock
        github_node_with_ref("owner", "repo", &hash1(), &OriginalRef::from("main")),
        github_node_with_ref("owner", "repo", &hash2(), &OriginalRef::from("feature-branch")),
        Ok(SyncStrategy::FlakeNixAndLock(format!(
            "github:owner/repo/{}", &*hash2()
        )))
    )]
    #[case(// indirect + ref -> indirect + ref (same ref) -> lock only
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &hash1()),
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &hash2()),
        Ok(SyncStrategy::LockOnly(format!(
            "nixpkgs/{}", &*hash2()
        )))
    )]
    #[case(// github -> git -> error
        git_node_with_url_only("https://example.com/user/repo.git", &hash1()),
        github_node_with_ref("owner", "repo", &hash1(), &OriginalRef::from("main")),
        Err("Cannot sync inputst with different type or from different git repository")
    )]
    #[case(
        nixpkgs_node(&hash1(), None),
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &hash2()),
        Err("Couldn't find nix-rust-utils's Original at destination.")
    )]
    #[case(
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &hash2()),
        nixpkgs_node(&hash1(), None),
        Err("Couldn't find nix-rust-utils's Original at source.")
    )]
    fn input_override_arg_returns_correct_argument(
        #[case] node1: Node,
        #[case] node2: Node,
        #[case] expected: Result<SyncStrategy, &str>,
    ) {
        let result = sync_strategy(
            &flake_lock_with_node("nix-rust-utils", node2),
            "nix-rust-utils",
            &flake_lock_with_node("nix-rust-utils", node1),
            "nix-rust-utils",
        );

        assert_eq!(
            result.map_err(|e| e.to_string()),
            expected.map_err(String::from)
        );
    }
}

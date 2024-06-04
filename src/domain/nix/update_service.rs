use crate::domain::{
    git::{self},
    Error, Result,
};

use super::Node;

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateStatus {
    AlreadyLatest,
    Outdated(Update),
    NotAvailable(String),
    Error(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Update {
    Lock(git::Commit),
    Input(git::Ref, git::Commit),
}

impl UpdateStatus {
    pub(in crate::domain::nix::update_service) fn not_available(reason: &str) -> Self {
        Self::NotAvailable(String::from(reason))
    }
}

pub trait UpdateService {
    fn available_update(&self, input: &Node) -> UpdateStatus;
}

#[allow(clippy::module_name_repetitions)]
pub struct UpdateServiceImpl<RS: git::RepositoryService> {
    git_repository_service: RS,
}

impl<RS: git::RepositoryService> UpdateServiceImpl<RS> {
    pub fn new(git_repository_service: RS) -> Self {
        Self {
            git_repository_service,
        }
    }
}

impl<RS: git::RepositoryService> UpdateService for UpdateServiceImpl<RS> {
    fn available_update(&self, node: &Node) -> UpdateStatus {
        available_update_for_node(&self.git_repository_service, node)
            .unwrap_or_else(|err| UpdateStatus::Error(err.to_string()))
    }
}

fn available_update_for_node<RS: git::RepositoryService>(
    git_repository_service: &RS,
    node: &Node,
) -> Result<UpdateStatus> {
    if node.original.rev.is_some() {
        return Ok(UpdateStatus::not_available(
            "Exact revision is provided in input definition",
        ));
    }

    let remote_ref = match node.original.source.clone() {
        super::OriginalSource::GitHub { owner, repo } => {
            Ok(git::RemoteReference::GitHub { repo, owner })
        }
        super::OriginalSource::GitLab { owner, repo } => {
            Ok(git::RemoteReference::GitLab { repo, owner })
        }
        super::OriginalSource::Git { url } => Ok(git::RemoteReference::Url(
            url.strip_prefix("git+").unwrap_or(&url).to_string(),
        )),
        super::OriginalSource::Indirect { id } if id == "nixpkgs" => {
            Ok(git::RemoteReference::GitHub {
                repo: String::from("nixpkgs"),
                owner: String::from("NixOS"),
            })
        }
        super::OriginalSource::Indirect { id: _ }
        | super::OriginalSource::SourceHut { owner: _, repo: _ }
        | super::OriginalSource::Mercurial {}
        | super::OriginalSource::File {}
        | super::OriginalSource::Path {}
        | super::OriginalSource::Tarball {} => {
            Err(Error::Error(String::from("Source is not supported yet")))
        }
    }?;

    let original_ref = node.original.clone().r#ref.unwrap_or_default();
    let git_ref = git_repository_service.find_branch_or_tag(&remote_ref, &original_ref)?;

    log::debug!("Found branch or tag: {git_ref:?}");

    if let Some(git_ref) = git_ref {
        match git_ref {
            git::Ref::Head(branch) => {
                let commit = git_repository_service
                    .resolve_ref(
                        &remote_ref,
                        &git::Ref::Remote(format!("origin/{}", &*branch)),
                    )?
                    .ok_or(Error::Error(String::from("Couldn't resolve reference")))?;

                if commit.sha == node.locked.rev {
                    Ok(UpdateStatus::AlreadyLatest)
                } else {
                    Ok(UpdateStatus::Outdated(Update::Lock(commit)))
                }
            }
            git::Ref::Tag(tag) => {
                let latest_tag =
                    git_repository_service
                        .find_latest_tag(&remote_ref)?
                        .ok_or(Error::Error(String::from(
                            "Couldn't find any tags in remote",
                        )))?;

                log::debug!("Found latest tag: {latest_tag:?}");

                if latest_tag == tag {
                    Ok(UpdateStatus::AlreadyLatest)
                } else {
                    let tag_ref = git::Ref::Tag(latest_tag);
                    let commit = git_repository_service
                        .resolve_ref(&remote_ref, &tag_ref)?
                        .ok_or(Error::Error(String::from(
                            "Couldn't resolve the reference of the latest tag",
                        )))?;

                    Ok(UpdateStatus::Outdated(Update::Input(tag_ref, commit)))
                }
            }
            git::Ref::Remote(_) | git::Ref::HEAD =>
            // HEAD is a symbolic reference and should be resolved to a branch, tag or commit
            // Remotes should be resolved to a branch, tag or commit
            {
                panic!("This shouldn't happen")
            }
        }
    } else {
        Ok(UpdateStatus::NotAvailable(format!(
            "Coudln't find reference: {:?}",
            &original_ref
        )))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mockall::predicate::*;
    use pretty_assertions::assert_eq;
    use time::OffsetDateTime;

    use crate::domain::{
        git::{Commit, CommitSha, MockRepositoryService, Ref, RemoteReference, Tag},
        nix::{
            flake_lock::fixtures::{git_node_with_ref, git_node_with_rev},
            update_service::{Update, UpdateStatus},
            Locked, LockedSource, Node, Original, OriginalRef,
        },
    };

    use super::{UpdateService, UpdateServiceImpl};

    #[test]
    fn test_available_update_should_return_not_available_when_original_rev_is_provided() {
        let git_repository_service = MockRepositoryService::new();

        let service = UpdateServiceImpl::new(git_repository_service);

        let node = git_node_with_rev(
            "git+ssh://git@git.example.com/user/repo",
            &CommitSha::from("some-sha"),
        );

        let result = service.available_update(&node);

        assert_eq!(
            result,
            UpdateStatus::NotAvailable(String::from(
                "Exact revision is provided in input definition",
            ))
        );
    }

    #[test]
    fn test_available_update_should_return_already_latest_when_branch_is_up_to_date() {
        let mut git_repository_service = MockRepositoryService::new();
        let remote_ref = RemoteReference::url("ssh://git@git.example.com/user/repo");
        git_repository_service
            .expect_find_branch_or_tag()
            .with(eq(remote_ref.clone()), eq("main"))
            .once()
            .return_once(|_, _| Ok(Some(Ref::head("main"))));
        git_repository_service
            .expect_resolve_ref()
            .with(eq(remote_ref), eq(Ref::remote("origin/main")))
            .once()
            .return_once(|_, _| Ok(Some(Commit::from_raw("some-sha", 1_717_539_161))));

        let service = UpdateServiceImpl::new(git_repository_service);

        let node = git_node_with_ref(
            "git+ssh://git@git.example.com/user/repo",
            &CommitSha::from("some-sha"),
            &OriginalRef::from("main"),
        );

        let result = service.available_update(&node);

        assert_eq!(result, UpdateStatus::AlreadyLatest);
    }

    #[test]
    fn test_available_update_should_return_outdated_when_branch_head_points_to_different_commit() {
        let mut git_repository_service = MockRepositoryService::new();
        let commit = Commit::from_raw("other-sha", 1_717_539_161);
        let remote_ref = RemoteReference::url("ssh://git@git.example.com/user/repo");
        git_repository_service
            .expect_find_branch_or_tag()
            .with(eq(remote_ref.clone()), eq("main"))
            .once()
            .return_once(|_, _| Ok(Some(Ref::head("main"))));
        {
            let commit = commit.clone();
            git_repository_service
                .expect_resolve_ref()
                .with(eq(remote_ref), eq(Ref::remote("origin/main")))
                .once()
                .return_once(|_, _| Ok(Some(commit)));
        }

        let service = UpdateServiceImpl::new(git_repository_service);

        let node = git_node_with_ref(
            "git+ssh://git@git.example.com/user/repo",
            &CommitSha::from("some-sha"),
            &OriginalRef::from("main"),
        );

        let result = service.available_update(&node);

        assert_eq!(result, UpdateStatus::Outdated(Update::Lock(commit)));
    }

    #[test]
    fn test_available_update_should_return_already_latest_when_tag_is_latest() {
        let mut git_repository_service = MockRepositoryService::new();
        let remote_ref = RemoteReference::url("ssh://git@git.example.com/user/repo");

        git_repository_service
            .expect_find_branch_or_tag()
            .with(eq(remote_ref.clone()), eq("v0.1.0"))
            .once()
            .return_once(|_, _| Ok(Some(Ref::tag("v0.1.0"))));

        git_repository_service
            .expect_find_latest_tag()
            .with(eq(remote_ref))
            .once()
            .return_once(|_| Ok(Some(Tag::from("v0.1.0"))));

        let service = UpdateServiceImpl::new(git_repository_service);

        let node = git_node_with_ref(
            "git+ssh://git@git.example.com/user/repo",
            &CommitSha::from("some-sha"),
            &OriginalRef::from("v0.1.0"),
        );

        let result = service.available_update(&node);

        assert_eq!(result, UpdateStatus::AlreadyLatest);
    }

    #[test]
    fn test_available_update_should_return_outdated_when_theres_a_newer_tag() {
        let mut git_repository_service = MockRepositoryService::new();
        let remote_ref = RemoteReference::url("ssh://git@git.example.com/user/repo");
        let tag = Tag::from("v0.1.0");
        let tag_ref = Ref::Tag(tag.clone());
        let commit = Commit::from_raw("other-sha", 1_717_539_161);

        git_repository_service
            .expect_find_branch_or_tag()
            .with(eq(remote_ref.clone()), eq("v0.1.0"))
            .once()
            .return_once(|_, _| Ok(Some(Ref::tag("v0.2.0"))));

        {
            let tag = tag.clone();
            git_repository_service
                .expect_find_latest_tag()
                .with(eq(remote_ref.clone()))
                .once()
                .return_once(|_| Ok(Some(tag)));
        }

        {
            let commit = commit.clone();
            git_repository_service
                .expect_resolve_ref()
                .with(eq(remote_ref), eq(tag_ref.clone()))
                .once()
                .return_once(|_, _| Ok(Some(commit)));
        }

        let service = UpdateServiceImpl::new(git_repository_service);

        let node = git_node_with_ref(
            "git+ssh://git@git.example.com/user/repo",
            &CommitSha::from("some-sha"),
            &OriginalRef::from("v0.1.0"),
        );

        let result = service.available_update(&node);

        assert_eq!(
            result,
            UpdateStatus::Outdated(Update::Input(tag_ref, commit))
        );
    }

    #[test]
    fn test_available_update_should_return_not_available_when_original_source_is_indirect_and_not_nixpkgs(
    ) {
        let git_repository_service = MockRepositoryService::new();

        let service = UpdateServiceImpl::new(git_repository_service);

        let node = Node {
            inputs: HashMap::new(),
            locked: Locked {
                rev: CommitSha::from("some-sha"),
                r#ref: None,
                source: LockedSource::GitHub {
                    owner: String::from("some-user"),
                    repo: String::from("other-flake"),
                },
                last_modified: OffsetDateTime::from_unix_timestamp(1_685_572_332).unwrap(),
            },
            original: Original {
                rev: None,
                r#ref: Some(OriginalRef::from("main")),
                source: crate::domain::nix::OriginalSource::Indirect {
                    id: String::from("some-other-flake"),
                },
            },
        };

        let result = service.available_update(&node);

        assert_eq!(
            result,
            UpdateStatus::Error(String::from(
                "an error happened: \"Source is not supported yet\"",
            ))
        );
    }
}

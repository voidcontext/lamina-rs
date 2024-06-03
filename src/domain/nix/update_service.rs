use crate::domain::{
    git::{self},
    Error, Result,
};

use super::Node;

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateDetails {
    AlreadyLatest,
    To(git::Commit, Option<git::Ref>),
    NotAvailable(String),
    Error(String),
}

impl UpdateDetails {
    pub(in crate::domain::nix::update_service) fn not_available(reason: &str) -> Self {
        Self::NotAvailable(String::from(reason))
    }
}

pub trait UpdateService {
    fn available_update(&self, input: &Node) -> UpdateDetails;
}

#[allow(clippy::module_name_repetitions)]
pub struct UpdateServiceImpl<RS: git::RepositoryService> {
    git_repository_service: RS,
}

impl<RS: git::RepositoryService> UpdateServiceImpl<RS> {
    #[allow(dead_code)] // TODO: remove this
    pub fn new(git_repository_service: RS) -> Self {
        Self {
            git_repository_service,
        }
    }
}

impl<RS: git::RepositoryService> UpdateService for UpdateServiceImpl<RS> {
    fn available_update(&self, node: &Node) -> UpdateDetails {
        available_update_for_node(&self.git_repository_service, node)
            .unwrap_or_else(|err| UpdateDetails::Error(err.to_string()))
    }
}

fn available_update_for_node<RS: git::RepositoryService>(
    git_repository_service: &RS,
    node: &Node,
) -> Result<UpdateDetails> {
    if node.original.rev.is_some() {
        return Ok(UpdateDetails::not_available(
            "Exact revision is provided in input definition",
        ));
    }

    let remote_ref = match node.original.source.clone() {
        super::OriginalSource::GitHub { owner, repo } => {
            Some(git::RemoteReference::GitHub { repo, owner })
        }
        super::OriginalSource::GitLab { owner, repo } => {
            Some(git::RemoteReference::GitLab { repo, owner })
        }
        super::OriginalSource::Git { url } => Some(git::RemoteReference::Url(
            url.strip_prefix("git+").unwrap_or(&url).to_string(),
        )),
        super::OriginalSource::Indirect { id: _ } =>
        // Indirect sources needs to be resolved from the flake registry, this is not supported at the moment
        {
            None
        }
    };

    if let Some(remote_ref) = remote_ref {
        let git_ref = git_repository_service
            .find_branch_or_tag(&node.original.clone().r#ref.unwrap_or_default())?;

        if let Some(git_ref) = git_ref {
            match git_ref {
                git::Ref::Head(_) => {
                    let commit = git_repository_service
                        .resolve_ref(&remote_ref, &git_ref)?
                        .ok_or(Error::Error(String::from("Couldn't resolve reference")))?;

                    if commit.sha == node.locked.rev {
                        Ok(UpdateDetails::AlreadyLatest)
                    } else {
                        Ok(UpdateDetails::To(commit, None))
                    }
                }
                git::Ref::Tag(_) => todo!(),
                git::Ref::Remote(_) => todo!(),
                git::Ref::HEAD => todo!(),
            }
        } else {
            Ok(UpdateDetails::NotAvailable(String::from(
                "Coudln't find reference",
            )))
        }
    } else {
        Ok(UpdateDetails::NotAvailable(String::from(
            "Indirect inputs from flake registry are not supported",
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
        git::{Commit, CommitSha, MockRepositoryService, Ref, RemoteReference},
        nix::{
            flake_lock::fixtures::{git_node_with_ref, git_node_with_rev},
            update_service::UpdateDetails,
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
            UpdateDetails::NotAvailable(String::from(
                "Exact revision is provided in input definition",
            ))
        );
    }

    #[test]
    fn test_available_update_should_return_already_latest_when_up_to_date() {
        let mut git_repository_service = MockRepositoryService::new();
        git_repository_service
            .expect_find_branch_or_tag()
            .with(eq("main"))
            .once()
            .return_once(|_| Ok(Some(Ref::head("main"))));
        git_repository_service
            .expect_resolve_ref()
            .with(
                eq(RemoteReference::url("ssh://git@git.example.com/user/repo")),
                eq(Ref::head("main")),
            )
            .once()
            .return_once(|_, _| Ok(Some(Commit::from_raw("some-sha", 1_717_539_161))));

        let service = UpdateServiceImpl::new(git_repository_service);

        let node = git_node_with_ref(
            "git+ssh://git@git.example.com/user/repo",
            &CommitSha::from("some-sha"),
            &OriginalRef::from("main"),
        );

        let result = service.available_update(&node);

        assert_eq!(result, UpdateDetails::AlreadyLatest);
    }

    #[test]
    fn test_available_update_should_return_to_commit_when_ref_points_to_different_commit() {
        let mut git_repository_service = MockRepositoryService::new();
        let commit = Commit::from_raw("other-sha", 1_717_539_161);
        git_repository_service
            .expect_find_branch_or_tag()
            .with(eq("main"))
            .once()
            .return_once(|_| Ok(Some(Ref::head("main"))));
        {
            let commit = commit.clone();
            git_repository_service
                .expect_resolve_ref()
                .with(
                    eq(RemoteReference::url("ssh://git@git.example.com/user/repo")),
                    eq(Ref::head("main")),
                )
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

        assert_eq!(result, UpdateDetails::To(commit, None));
    }

    #[test]
    fn test_available_update_should_return_not_available_when_original_source_is_indirect() {
        let git_repository_service = MockRepositoryService::new();

        let service = UpdateServiceImpl::new(git_repository_service);

        let node = Node {
            inputs: HashMap::new(),
            locked: Locked {
                rev: CommitSha::from("some-sha"),
                r#ref: None,
                source: LockedSource::GitHub {
                    owner: String::from("NixOS"),
                    repo: String::from("nixpkgs"),
                },
                last_modified: OffsetDateTime::from_unix_timestamp(1_685_572_332).unwrap(),
            },
            original: Original {
                rev: None,
                r#ref: Some(OriginalRef::from("main")),
                source: crate::domain::nix::OriginalSource::Indirect {
                    id: String::from("nixpkgs"),
                },
            },
        };

        let result = service.available_update(&node);

        assert_eq!(
            result,
            UpdateDetails::NotAvailable(String::from(
                "Indirect inputs from flake registry are not supported",
            ))
        );
    }
}

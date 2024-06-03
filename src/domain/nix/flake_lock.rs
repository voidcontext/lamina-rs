use std::collections::HashMap;

use nova::newtype;
use time::OffsetDateTime;

use crate::domain::git;

pub struct FlakeLock {
    pub root: RootNode,
    pub nodes: HashMap<String, Node>,
}

impl FlakeLock {
    #[must_use]
    pub fn input_nodes(&self) -> HashMap<String, Node> {
        todo!()
    }
}

pub struct RootNode {
    pub inputs: HashMap<String, InputReference>,
}

pub struct Node {
    pub inputs: HashMap<String, InputReference>,
    pub locked: Locked,
    pub original: Original,
}

pub enum InputReference {
    Alias(String),
    Path(Vec<String>),
}

#[newtype(new, serde, borrow = "str")]
pub type LockedRef = String;

impl From<&str> for LockedRef {
    fn from(value: &str) -> Self {
        Self::new(String::from(value))
    }
}

#[derive(Clone)]
pub struct Locked {
    pub rev: git::CommitSha,
    pub r#ref: Option<LockedRef>,
    pub source: LockedSource,
    pub last_modified: OffsetDateTime,
}

#[derive(Clone)]
pub enum LockedSource {
    GitHub { owner: String, repo: String },
    GitLab { owner: String, repo: String },
    Git { url: String },
}

#[newtype(new, serde, borrow = "str")]
pub type OriginalRef = String;

impl From<&str> for OriginalRef {
    fn from(value: &str) -> Self {
        Self::new(String::from(value))
    }
}

impl Default for OriginalRef {
    fn default() -> Self {
        Self::from("refs/remotes/origin/HEAD")
    }
}

#[derive(Clone, Debug)]
pub struct Original {
    pub rev: Option<git::CommitSha>,
    pub r#ref: Option<OriginalRef>,
    pub source: OriginalSource,
}

// impl Original {
//     pub(crate) fn to_git_ref(&self) -> Result<Option<git::Ref>> {
//         match (&self.rev, &self.r#ref) {
//             (Some(_), _) => Ok(None),
//             (None, Some(reference)) => git::Ref::try_from(&**reference).map(Some),
//             (None, None) => Err(domain::Error::Error(String::from(
//                 "Couldn't determine a git reference since bot rev and ref is missing",
//             ))),
//         }
//     }
// }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OriginalSource {
    GitHub { owner: String, repo: String },
    GitLab { owner: String, repo: String },
    Git { url: String },
    Indirect { id: String },
}

#[cfg(test)]
mod tests {

    #[test]
    #[ignore = "Not implemented"]
    fn test_flake_lock_input_nodes() {}

    // #[test]
    // fn test_original_to_ref_with_ref_only() {
    //     let original = Original {
    //         rev: None,
    //         r#ref: Some(OriginalRef::from("refs/tags/v0.10.1")),
    //         source: OriginalSource::Git {
    //             url: String::from("ssh://git@example.com/repo.git"),
    //         },
    //     };

    //     let result = original.to_git_ref();

    //     assert_eq!(result.unwrap(), Some(Ref::Tag(Tag::from("v0.10.1"))));
    // }

    // #[test]
    // fn test_original_to_ref_with_ref_and_commit() {
    //     let original = Original {
    //         rev: Some(CommitSha::from("some-sha")),
    //         r#ref: Some(OriginalRef::from("refs/tags/v0.10.1")),
    //         source: OriginalSource::Git {
    //             url: String::from("ssh://git@example.com/repo.git"),
    //         },
    //     };

    //     let result = original.to_git_ref();

    //     assert_eq!(result.unwrap(), None);
    // }

    // #[test]
    // fn test_original_to_ref_with_commit_only() {
    //     let original = Original {
    //         rev: Some(CommitSha::from("some-sha")),
    //         r#ref: None,
    //         source: OriginalSource::Git {
    //             url: String::from("ssh://git@example.com/repo.git"),
    //         },
    //     };

    //     let result = original.to_git_ref();

    //     assert_eq!(result.unwrap(), None);
    // }

    // #[test]
    // fn test_original_to_ref_without_ref_and_commit() {
    //     let original = Original {
    //         rev: None,
    //         r#ref: None,
    //         source: OriginalSource::Git {
    //             url: String::from("ssh://git@example.com/repo.git"),
    //         },
    //     };

    //     let result = original.to_git_ref();

    //     let err = result.unwrap_err();

    //     match err {
    //         crate::domain::Error::Error(message) => {
    //             assert_eq!(
    //                 message,
    //                 "Couldn't determine a git reference since bot rev and ref is missing"
    //             );
    //         }
    //         _ => panic!("Error is different than expected"),
    //     }
    // }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use std::collections::HashMap;

    use time::OffsetDateTime;

    use crate::domain::git::CommitSha;

    use super::{
        FlakeLock, InputReference, Locked, LockedRef, LockedSource, Node, Original, OriginalRef,
        OriginalSource, RootNode,
    };

    #[must_use]
    pub(crate) fn inputs(name: &str) -> HashMap<String, InputReference> {
        let mut inputs = HashMap::new();
        inputs.insert(
            String::from(name),
            InputReference::Alias(String::from(name)),
        );
        inputs
    }

    #[must_use]
    pub(crate) fn root_node(input_name: &str) -> RootNode {
        RootNode {
            inputs: inputs(input_name),
        }
    }

    #[must_use]
    pub(crate) fn git_node_with_url_only(url: &str, locked_rev: &CommitSha) -> Node {
        git_node(url, locked_rev, false, None)
    }
    #[must_use]
    pub(crate) fn git_node_with_rev(url: &str, locked_rev: &CommitSha) -> Node {
        git_node(url, locked_rev, true, None)
    }
    #[must_use]
    pub(crate) fn git_node_with_ref(
        url: &str,
        locked_rev: &CommitSha,
        git_ref: &OriginalRef,
    ) -> Node {
        git_node(url, locked_rev, false, Some(git_ref))
    }

    pub(crate) fn git_node(
        url: &str,
        locked_rev: &CommitSha,
        original_rev: bool,
        git_ref: Option<&OriginalRef>,
    ) -> Node {
        Node {
            inputs: inputs("nixpgks"),
            locked: Locked {
                rev: locked_rev.clone(),
                r#ref: git_ref.map(|r| String::from(&**r)).map(LockedRef::new),
                source: LockedSource::Git {
                    url: String::from(url),
                },
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            },
            original: Original {
                source: OriginalSource::Git {
                    url: String::from(url),
                },
                rev: Some(&**locked_rev)
                    .filter(|_| original_rev)
                    .map(CommitSha::from),
                r#ref: git_ref.cloned(),
            },
        }
    }

    #[must_use]
    pub(crate) fn github_node_with_owner_and_repo_only(
        owner: &str,
        repo: &str,
        locked_rev: &CommitSha,
    ) -> Node {
        github_node(owner, repo, locked_rev, false, None)
    }

    #[must_use]
    pub(crate) fn github_node_with_ref(
        owner: &str,
        repo: &str,
        locked_rev: &CommitSha,
        git_ref: &OriginalRef,
    ) -> Node {
        github_node(owner, repo, locked_rev, false, Some(git_ref))
    }

    #[allow(clippy::similar_names)]
    pub(crate) fn github_node(
        owner: &str,
        repo: &str,
        locked_rev: &CommitSha,
        original_rev: bool,
        git_ref: Option<&OriginalRef>,
    ) -> Node {
        Node {
            inputs: inputs("nixpkgs"),
            locked: Locked {
                rev: locked_rev.clone(),
                r#ref: git_ref.map(|r| String::from(&**r)).map(LockedRef::new),
                source: LockedSource::GitHub {
                    owner: String::from(owner),
                    repo: String::from(repo),
                },
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            },
            original: Original {
                source: OriginalSource::GitHub {
                    owner: String::from(owner),
                    repo: String::from(repo),
                },
                rev: Some(&**locked_rev)
                    .filter(|_| original_rev)
                    .map(CommitSha::from),
                r#ref: git_ref.cloned(),
            },
        }
    }

    #[must_use]
    pub(crate) fn nixpkgs_node_with_ref(
        indirect_ref: &OriginalRef,
        locked_rev: &CommitSha,
    ) -> Node {
        indirect_node(
            "nixpkgs",
            Some(indirect_ref),
            "NixOS",
            "nixpgks",
            locked_rev,
            false,
        )
    }

    pub(crate) fn indirect_node(
        id: &str,
        indirect_ref: Option<&OriginalRef>,
        owner: &str,
        repo: &str,
        locked_rev: &CommitSha,
        original_rev: bool,
    ) -> Node {
        Node {
            inputs: inputs("nixpkgs"),
            locked: Locked {
                rev: locked_rev.clone(),
                r#ref: indirect_ref.map(|r| String::from(&**r)).map(LockedRef::new),
                source: LockedSource::GitHub {
                    owner: String::from(owner),
                    repo: String::from(repo),
                },
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            },
            original: Original {
                source: OriginalSource::Indirect {
                    id: String::from(id),
                },
                r#ref: indirect_ref.cloned(),
                rev: Some(&**locked_rev)
                    .filter(|_| original_rev)
                    .map(CommitSha::from),
            },
        }
    }

    #[must_use]
    pub(crate) fn nixpkgs_node(rev: &CommitSha, original: &Original) -> Node {
        Node {
            inputs: HashMap::new(),
            locked: Locked {
                rev: rev.clone(),
                r#ref: None,
                source: LockedSource::GitHub {
                    owner: String::from("NixOS"),
                    repo: String::from("nixpkgs"),
                },
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1683627095).unwrap(),
            },
            original: (*original).clone(),
        }
    }

    #[must_use]
    pub(crate) fn flake_lock_with_node(name: &str, node: Node) -> FlakeLock {
        let mut nodes = HashMap::new();

        nodes.insert(String::from(name), node);

        nodes.insert(
            String::from("nixpkgs"),
            nixpkgs_node(
                &CommitSha::from("a08e061a4ee8329747d54ddf1566d34c55c895eb"),
                &Original {
                    rev: None,
                    r#ref: None,
                    source: OriginalSource::Indirect {
                        id: String::from("nixpkgs"),
                    },
                },
            ),
        );

        FlakeLock {
            nodes,
            root: root_node("nix-rust-utils"),
        }
    }
}

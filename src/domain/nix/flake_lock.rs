use std::collections::HashMap;

use nova::newtype;
use time::OffsetDateTime;

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
pub type LockedRev = String;

impl From<&str> for LockedRev {
    fn from(value: &str) -> Self {
        Self::new(String::from(value))
    }
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
    pub rev: LockedRev,
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
pub type OriginalRev = String;

impl From<&str> for OriginalRev {
    fn from(value: &str) -> Self {
        Self::new(String::from(value))
    }
}

#[newtype(new, serde, borrow = "str")]
pub type OriginalRef = String;

impl From<&str> for OriginalRef {
    fn from(value: &str) -> Self {
        Self::new(String::from(value))
    }
}

#[derive(Clone, Debug)]
pub struct Original {
    pub rev: Option<OriginalRev>,
    pub r#ref: Option<OriginalRef>,
    pub source: OriginalSource,
}

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
}

#[cfg(test)]
pub(crate) mod fixtures {
    use std::collections::HashMap;

    use time::OffsetDateTime;

    use super::{
        FlakeLock, InputReference, Locked, LockedRef, LockedRev, LockedSource, Node, Original,
        OriginalRef, OriginalRev, OriginalSource, RootNode,
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
    pub(crate) fn git_node_with_url_only(url: &str, locked_rev: &LockedRev) -> Node {
        git_node(url, locked_rev, false, None)
    }
    #[must_use]
    pub(crate) fn git_node_with_rev(url: &str, locked_rev: &LockedRev) -> Node {
        git_node(url, locked_rev, true, None)
    }
    #[must_use]
    pub(crate) fn git_node_with_ref(
        url: &str,
        locked_rev: &LockedRev,
        git_ref: &OriginalRef,
    ) -> Node {
        git_node(url, locked_rev, false, Some(git_ref))
    }

    pub(crate) fn git_node(
        url: &str,
        locked_rev: &LockedRev,
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
                    .map(String::from)
                    .map(OriginalRev::new),
                r#ref: git_ref.cloned(),
            },
        }
    }

    #[must_use]
    pub(crate) fn github_node_with_owner_and_repo_only(
        owner: &str,
        repo: &str,
        locked_rev: &LockedRev,
    ) -> Node {
        github_node(owner, repo, locked_rev, false, None)
    }

    #[must_use]
    pub(crate) fn github_node_with_ref(
        owner: &str,
        repo: &str,
        locked_rev: &LockedRev,
        git_ref: &OriginalRef,
    ) -> Node {
        github_node(owner, repo, locked_rev, false, Some(git_ref))
    }

    #[allow(clippy::similar_names)]
    pub(crate) fn github_node(
        owner: &str,
        repo: &str,
        locked_rev: &LockedRev,
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
                    .map(String::from)
                    .map(OriginalRev::new),
                r#ref: git_ref.cloned(),
            },
        }
    }

    #[must_use]
    pub(crate) fn nixpkgs_node_with_ref(
        indirect_ref: &OriginalRef,
        locked_rev: &LockedRev,
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
        locked_rev: &LockedRev,
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
                    .map(String::from)
                    .map(OriginalRev::new),
            },
        }
    }

    #[must_use]
    pub(crate) fn nixpkgs_node(rev: &LockedRev, original: &Original) -> Node {
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
                &LockedRev::from(String::from("a08e061a4ee8329747d54ddf1566d34c55c895eb")),
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

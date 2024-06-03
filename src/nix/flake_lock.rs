use std::collections::HashMap;

use time::OffsetDateTime;

use serde::Deserialize;

use nova::newtype;

#[derive(Debug, Deserialize)]
pub struct FlakeLock {
    pub nodes: HashMap<String, Node>, // used in integration tests
    pub(crate) root: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Node {
    pub(crate) flake: Option<bool>,
    pub(crate) inputs: Option<HashMap<String, InputReference>>,
    pub locked: Option<Locked>,     // used in integration tests
    pub original: Option<Original>, // used in integration tests
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum InputReference {
    Alias(String),
    Path(Vec<String>),
}

#[newtype(new, serde, borrow = "str")]
pub type OriginalRev = String;

#[newtype(new, serde, borrow = "str")]
pub type OriginalRef = String;

#[cfg(test)]
impl From<&str> for OriginalRef {
    fn from(value: &str) -> Self {
        OriginalRef::from(String::from(value))
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Original {
    #[serde(rename_all = "camelCase")]
    Git {
        url: String,
        r#ref: Option<OriginalRef>,
        rev: Option<OriginalRev>,
    },
    #[serde(rename_all = "camelCase")]
    Github {
        owner: String,
        repo: String,
        r#ref: Option<OriginalRef>,
        rev: Option<OriginalRev>,
    },
    #[serde(rename_all = "camelCase")]
    GitLab {
        owner: String,
        repo: String,
        r#ref: Option<OriginalRef>,
        rev: Option<OriginalRev>,
    },
    #[serde(rename_all = "camelCase")]
    Indirect {
        id: String,
        r#ref: Option<OriginalRef>,
        rev: Option<OriginalRev>,
    },
}

#[newtype(new, serde, borrow = "str")]
pub type LockedRev = String;

#[cfg(test)]
impl From<&str> for LockedRev {
    fn from(value: &str) -> Self {
        LockedRev::from(String::from(value))
    }
}

#[newtype(new, serde, borrow = "str")]
pub type LockedRef = String;

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Locked {
    #[serde(rename_all = "camelCase")]
    Git {
        rev: LockedRev,
        r#ref: Option<LockedRef>,
        url: String,
        #[serde(with = "time::serde::timestamp")]
        last_modified: OffsetDateTime,
    },
    #[serde(rename_all = "camelCase")]
    Github {
        rev: LockedRev,
        r#ref: Option<LockedRef>,
        owner: String,
        repo: String,
        #[serde(with = "time::serde::timestamp")]
        last_modified: OffsetDateTime,
    },
    #[serde(rename_all = "camelCase")]
    GitLab {
        rev: LockedRev,
        r#ref: Option<LockedRef>,
        owner: String,
        repo: String,
        #[serde(with = "time::serde::timestamp")]
        last_modified: OffsetDateTime,
    },
}

// Tests

#[cfg(test)]
pub mod fixtures {
    use std::collections::HashMap;

    use time::OffsetDateTime;

    use super::{
        FlakeLock, InputReference, Locked, LockedRef, LockedRev, Node, Original, OriginalRef,
        OriginalRev,
    };

    pub mod original {
        use crate::nix::flake_lock::{Original, OriginalRef, OriginalRev};

        #[must_use]
        pub fn github_with_ref(owner: &str, repo: &str, git_ref: &OriginalRef) -> Original {
            github(owner, repo, None, Some(git_ref))
        }

        #[allow(clippy::similar_names)]
        fn github(
            owner: &str,
            repo: &str,
            git_rev: Option<&OriginalRev>,
            git_ref: Option<&OriginalRef>,
        ) -> Original {
            Original::Github {
                owner: String::from(owner),
                repo: String::from(repo),
                rev: git_rev.cloned(),
                r#ref: git_ref.cloned(),
            }
        }
    }

    #[must_use]
    pub fn inputs(name: &str) -> HashMap<String, InputReference> {
        let mut inputs = HashMap::new();
        inputs.insert(
            String::from(name),
            InputReference::Alias(String::from(name)),
        );
        inputs
    }

    #[must_use]
    pub fn root_node(input_name: &str) -> Node {
        Node {
            flake: None,
            locked: None,
            inputs: Some(inputs(input_name)),
            original: None,
        }
    }

    #[must_use]
    pub fn git_node_with_url_only(url: &str, locked_rev: &LockedRev) -> Node {
        git_node(url, locked_rev, false, None)
    }
    #[must_use]
    pub fn git_node_with_rev(url: &str, locked_rev: &LockedRev) -> Node {
        git_node(url, locked_rev, true, None)
    }
    #[must_use]
    pub fn git_node_with_ref(url: &str, locked_rev: &LockedRev, git_ref: &OriginalRef) -> Node {
        git_node(url, locked_rev, false, Some(git_ref))
    }

    pub fn git_node(
        url: &str,
        locked_rev: &LockedRev,
        original_rev: bool,
        git_ref: Option<&OriginalRef>,
    ) -> Node {
        Node {
            flake: None,
            inputs: Some(inputs("nixpgks")),
            locked: Some(Locked::Git {
                rev: locked_rev.clone(),
                r#ref: git_ref.map(|r| String::from(&**r)).map(LockedRef::new),
                url: String::from(url),
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            }),
            original: Some(Original::Git {
                url: String::from(url),
                rev: Some(&**locked_rev)
                    .filter(|_| original_rev)
                    .map(String::from)
                    .map(OriginalRev::new),
                r#ref: git_ref.cloned(),
            }),
        }
    }

    #[must_use]
    pub fn github_node_with_owner_and_repo_only(
        owner: &str,
        repo: &str,
        locked_rev: &LockedRev,
    ) -> Node {
        github_node(owner, repo, locked_rev, false, None)
    }

    #[must_use]
    pub fn github_node_with_ref(
        owner: &str,
        repo: &str,
        locked_rev: &LockedRev,
        git_ref: &OriginalRef,
    ) -> Node {
        github_node(owner, repo, locked_rev, false, Some(git_ref))
    }

    #[allow(clippy::similar_names)]
    pub fn github_node(
        owner: &str,
        repo: &str,
        locked_rev: &LockedRev,
        original_rev: bool,
        git_ref: Option<&OriginalRef>,
    ) -> Node {
        Node {
            flake: None,
            inputs: Some(inputs("nixpkgs")),
            locked: Some(Locked::Github {
                rev: locked_rev.clone(),
                r#ref: git_ref.map(|r| String::from(&**r)).map(LockedRef::new),
                owner: String::from(owner),
                repo: String::from(repo),
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            }),
            original: Some(Original::Github {
                owner: String::from(owner),
                repo: String::from(repo),
                rev: Some(&**locked_rev)
                    .filter(|_| original_rev)
                    .map(String::from)
                    .map(OriginalRev::new),
                r#ref: git_ref.cloned(),
            }),
        }
    }

    #[must_use]
    pub fn nixpkgs_node_with_ref(indirect_ref: &OriginalRef, locked_rev: &LockedRev) -> Node {
        indirect_node(
            "nixpkgs",
            Some(indirect_ref),
            "NixOS",
            "nixpgks",
            locked_rev,
            false,
        )
    }

    pub fn indirect_node(
        id: &str,
        indirect_ref: Option<&OriginalRef>,
        owner: &str,
        repo: &str,
        locked_rev: &LockedRev,
        original_rev: bool,
    ) -> Node {
        Node {
            flake: None,
            inputs: Some(inputs("nixpkgs")),
            locked: Some(Locked::Github {
                rev: locked_rev.clone(),
                r#ref: indirect_ref.map(|r| String::from(&**r)).map(LockedRef::new),
                owner: String::from(owner),
                repo: String::from(repo),
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            }),
            original: Some(Original::Indirect {
                id: String::from(id),
                r#ref: indirect_ref.cloned(),
                rev: Some(&**locked_rev)
                    .filter(|_| original_rev)
                    .map(String::from)
                    .map(OriginalRev::new),
            }),
        }
    }

    #[must_use]
    pub fn nixpkgs_node(rev: &LockedRev, original: Option<Original>) -> Node {
        Node {
            flake: None,
            inputs: None,
            locked: Some(Locked::Github {
                rev: rev.clone(),
                r#ref: None,
                owner: String::from("NixOS"),
                repo: String::from("nixpkgs"),
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1683627095).unwrap(),
            }),
            original,
        }
    }

    #[must_use]
    pub fn flake_lock_with_node(name: &str, node: Node) -> FlakeLock {
        let mut nodes = HashMap::new();

        nodes.insert(String::from("root"), root_node("nix-rust-utils"));
        nodes.insert(String::from(name), node);

        nodes.insert(
            String::from("nixpkgs"),
            nixpkgs_node(
                &LockedRev::from("a08e061a4ee8329747d54ddf1566d34c55c895eb"),
                None,
            ),
        );

        FlakeLock {
            nodes,
            root: String::from("root"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FlakeLock;

    #[test]
    fn can_deserialize_flake_lock() {
        let result = serde_json::from_str::<FlakeLock>(FLAKE_LOCK_JSON);
        println!("{result:?}");
        assert!(result.is_ok());
    }

    // #[test]
    // fn top_level_nodes_should_return_root_nodes() {
    //     let node = git_node_with_url_only(
    //         "https://git.vdx.hu/voidcontext/nix-rust-utils.git",
    //         &LockedRev::from("3892194d7b3293de8b30f1d19e2af45ba41ba8fd"),
    //     );
    //     let name = "nix-rust-utils";
    //     let top_level_nodes = flake_lock_with_node(name, node.clone()).input_nodes();

    //     assert_eq!(
    //         top_level_nodes,
    //         [(String::from(name), node)].into_iter().collect()
    //     );
    // }

    // #[test]
    // fn locked_rev_of_should_return_locked_revision() {
    //     let node = git_node_with_url_only(
    //         "https://git.vdx.hu/voidcontext/nix-rust-utils.git",
    //         &LockedRev::from("3892194d7b3293de8b30f1d19e2af45ba41ba8fd"),
    //     );
    //     let name = "nix-rust-utils";
    //     assert_eq!(
    //         flake_lock_with_node(name, node).locked_rev_of("nix-rust-utils"),
    //         Some(LockedRev::from("3892194d7b3293de8b30f1d19e2af45ba41ba8fd"))
    //     );
    // }

    // #[test]
    // fn original_of_should_return_original_definition_of_input() {
    //     let node = github_node_with_ref(
    //         "voidcontext",
    //         "nix-rust-utils",
    //         &LockedRev::from("3892194d7b3293de8b30f1d19e2af45ba41ba8fd"),
    //         &OriginalRef::from("refs/heads/main"),
    //     );
    //     let name = "nix-rust-utils";
    //     let original = original::github_with_ref(
    //         "voidcontext",
    //         "nix-rust-utils",
    //         &OriginalRef::from("refs/heads/main"),
    //     );

    //     assert_eq!(
    //         flake_lock_with_node(name, node).original_of(name),
    //         Some(original)
    //     );
    // }

    static FLAKE_LOCK_JSON: &str = r#"{
  "nodes": {
    "crane": {
      "inputs": {
        "flake-compat": "flake-compat",
        "flake-utils": [
          "nix-rust-utils",
          "flake-utils"
        ],
        "nixpkgs": [
          "nix-rust-utils",
          "nixpkgs"
        ],
        "rust-overlay": [
          "nix-rust-utils",
          "rust-overlay"
        ]
      },
      "locked": {
        "lastModified": 1682985522,
        "narHash": "sha256-QpaH83EEJ5t2eucsgcuhdgBnvhnm90D1jrCihAql508=",
        "owner": "ipetkov",
        "repo": "crane",
        "rev": "841b3f7017556aa6ed040744f83472835a5bf98e",
        "type": "github"
      },
      "original": {
        "owner": "ipetkov",
        "repo": "crane",
        "type": "github"
      }
    },
    "flake-compat": {
      "flake": false,
      "locked": {
        "lastModified": 1673956053,
        "narHash": "sha256-4gtG9iQuiKITOjNQQeQIpoIB6b16fm+504Ch3sNKLd8=",
        "owner": "edolstra",
        "repo": "flake-compat",
        "rev": "35bb57c0c8d8b62bbfd284272c928ceb64ddbde9",
        "type": "github"
      },
      "original": {
        "owner": "edolstra",
        "repo": "flake-compat",
        "type": "github"
      }
    },
    "flake-utils": {
      "inputs": {
        "systems": "systems"
      },
      "locked": {
        "lastModified": 1681202837,
        "narHash": "sha256-H+Rh19JDwRtpVPAWp64F+rlEtxUWBAQW28eAi3SRSzg=",
        "owner": "numtide",
        "repo": "flake-utils",
        "rev": "cfacdce06f30d2b68473a46042957675eebb3401",
        "type": "github"
      },
      "original": {
        "owner": "numtide",
        "repo": "flake-utils",
        "type": "github"
      }
    },
    "nix-rust-utils": {
      "inputs": {
        "crane": "crane",
        "flake-utils": "flake-utils",
        "nixpkgs": "nixpkgs",
        "nixpkgs-unstable": "nixpkgs-unstable",
        "rust-overlay": "rust-overlay"
      },
      "locked": {
        "lastModified": 1685572332,
        "narHash": "sha256-B7bZ4Yw9Zm2LwsbYuAMFAU1YGUt5gNX06uDlZFF2jYc=",
        "ref": "refs/heads/main",
        "rev": "3892194d7b3293de8b30f1d19e2af45ba41ba8fd",
        "revCount": 67,
        "type": "git",
        "url": "https://git.vdx.hu/voidcontext/nix-rust-utils.git"
      },
      "original": {
        "type": "git",
        "url": "https://git.vdx.hu/voidcontext/nix-rust-utils.git"
      }
    },
    "nixpkgs": {
      "locked": {
        "lastModified": 1683627095,
        "narHash": "sha256-8u9SejRpL2TrMuHBdhYh4FKc1OGPDLyWTpIbNTtoHsA=",
        "owner": "NixOS",
        "repo": "nixpkgs",
        "rev": "a08e061a4ee8329747d54ddf1566d34c55c895eb",
        "type": "github"
      },
      "original": {
        "id": "nixpkgs",
        "ref": "release-22.11",
        "type": "indirect"
      }
    },
    "nixpkgs-unstable": {
      "locked": {
        "lastModified": 1683535585,
        "narHash": "sha256-ND6gCDEfuMNsaJlZQzEbl6hTGrAzURtLoZoGR5dJaCw=",
        "owner": "NixOS",
        "repo": "nixpkgs",
        "rev": "d9b8ae36f31046dbcf05b6cdc45e860bf19b0b7e",
        "type": "github"
      },
      "original": {
        "id": "nixpkgs",
        "ref": "nixpkgs-unstable",
        "type": "indirect"
      }
    },
    "root": {
      "inputs": {
        "nix-rust-utils": "nix-rust-utils"
      }
    },
    "rust-overlay": {
      "inputs": {
        "flake-utils": [
          "nix-rust-utils",
          "flake-utils"
        ],
        "nixpkgs": [
          "nix-rust-utils",
          "nixpkgs"
        ]
      },
      "locked": {
        "lastModified": 1682993975,
        "narHash": "sha256-LlI5vwUw97NLAwcOYHRLRfhICVdp7MK2KFcUSj0Zwdg=",
        "owner": "oxalica",
        "repo": "rust-overlay",
        "rev": "07f421299826591e2b28e03bbbe19a5292395afe",
        "type": "github"
      },
      "original": {
        "owner": "oxalica",
        "repo": "rust-overlay",
        "type": "github"
      }
    },
    "systems": {
      "locked": {
        "lastModified": 1681028828,
        "narHash": "sha256-Vy1rq5AaRuLzOxct8nz4T6wlgyUR7zLU309k9mBC768=",
        "owner": "nix-systems",
        "repo": "default",
        "rev": "da67096a3b9bf56a91d16901293e51ba5b49a27e",
        "type": "github"
      },
      "original": {
        "owner": "nix-systems",
        "repo": "default",
        "type": "github"
      }
    }
  },
  "root": "root",
  "version": 7
}
    "#;
}

#![allow(dead_code)] // TODO: remove this

use std::collections::HashMap;

use time::OffsetDateTime;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FlakeLock {
    nodes: HashMap<String, Node>,
    root: String,
}

impl FlakeLock {
    pub fn top_level_nodes(&self) -> Vec<NodeWithName> {
        let root_node = self.nodes.get(&self.root).expect("Cannot find root node");

        root_node
            .inputs
            .as_ref()
            .expect("There aren't any inputs")
            .iter()
            .map(|(input_name, node_name)| match node_name {
                InputValue::Scalar(node_name) => NodeWithName {
                    name: input_name.clone(),
                    node: self.nodes.get(node_name).unwrap().clone(),
                },
                InputValue::List(_) => panic!("I don't know what to do"),
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NodeWithName {
    pub name: String,
    pub node: Node,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct Node {
    pub flake: Option<bool>,
    pub inputs: Option<HashMap<String, InputValue>>,
    pub locked: Option<Locked>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum InputValue {
    Scalar(String),
    List(Vec<String>),
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Locked {
    #[serde(rename_all = "camelCase")]
    Git {
        rev: String,
        url: String,
        #[serde(with = "time::serde::timestamp")]
        last_modified: OffsetDateTime,
    },
    #[serde(rename_all = "camelCase")]
    Github {
        rev: String,
        owner: String,
        repo: String,
        #[serde(with = "time::serde::timestamp")]
        last_modified: OffsetDateTime,
    },
    #[serde(rename_all = "camelCase")]
    GitLab {
        rev: String,
        owner: String,
        repo: String,
        #[serde(with = "time::serde::timestamp")]
        last_modified: OffsetDateTime,
    },
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use lazy_static::lazy_static;

    use crate::nix::{FlakeLock, InputValue, Locked, Node, NodeWithName, OffsetDateTime};

    lazy_static! {
        static ref DEFAULT_FLAKE_LOCK: FlakeLock = {
            let mut nodes = HashMap::new();

            nodes.insert(
                String::from("root"),
                Node {
                    flake: None,
                    locked: None,
                    inputs: {
                        let mut inputs = HashMap::new();
                        inputs.insert(
                            String::from("nix-rust-utils"),
                            InputValue::Scalar(String::from("nix-rust-utils")),
                        );
                        Some(inputs)
                    },
                },
            );

            nodes.insert(
                String::from("nix-rust-utils"),
                Node {
                    flake: None,
                    inputs: Some({
                        let mut inputs = HashMap::new();
                        inputs.insert(
                            String::from("nixpkgs"),
                            InputValue::Scalar(String::from("nixpkgs")),
                        );
                        inputs
                    }),
                    locked: Some(Locked::Git {
                        rev: String::from("3892194d7b3293de8b30f1d19e2af45ba41ba8fd"),
                        url: String::from("https://git.vdx.hu/voidcontext/nix-rust-utils.git"),
                        #[allow(clippy::unreadable_literal)]
                        last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
                    }),
                },
            );

            nodes.insert(
                String::from("nixpkgs"),
                Node {
                    flake: None,
                    inputs: None,
                    locked: Some(Locked::Github {
                        rev: String::from("a08e061a4ee8329747d54ddf1566d34c55c895eb"),
                        owner: String::from("NixOS"),
                        repo: String::from("nixpkgs"),
                        #[allow(clippy::unreadable_literal)]
                        last_modified: OffsetDateTime::from_unix_timestamp(1683627095).unwrap(),
                    }),
                },
            );

            FlakeLock {
                nodes,
                root: String::from("root"),
            }
        };
    }

    #[test]
    fn test_can_deserialize_flake_lock() {
        let result = serde_json::from_str::<FlakeLock>(FLAKE_LOCK_JSON);
        assert!(result.is_ok());
    }

    #[test]
    fn test_top_level_nodes_should_return_root_nodes() {
        let top_level_nodes = DEFAULT_FLAKE_LOCK.top_level_nodes();
        assert_eq!(
            top_level_nodes,
            vec![NodeWithName {
                name: String::from("nix-rust-utils"),
                node: DEFAULT_FLAKE_LOCK
                    .nodes
                    .get(&String::from("nix-rust-utils"))
                    .cloned()
                    .unwrap()
            }]
        );
    }

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

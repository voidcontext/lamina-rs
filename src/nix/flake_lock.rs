use std::{collections::HashMap, fs, path::PathBuf};

use time::OffsetDateTime;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FlakeLock {
    nodes: HashMap<String, Node>,
    root: String,
}

impl FlakeLock {
    pub fn input_nodes(&self) -> HashMap<String, Node> {
        let root_node = self.nodes.get(&self.root).expect("Cannot find root node");

        root_node
            .inputs
            .as_ref()
            .expect("There aren't any inputs")
            .iter()
            .map(|(input_name, node_name)| match node_name {
                InputValue::Scalar(node_name) => (
                    input_name.clone(),
                    self.nodes.get(node_name).unwrap().clone(),
                ),
                InputValue::List(_) => panic!("I don't know what to do"),
            })
            .collect()
    }

    pub fn locked_rev_of(&self, input: &str) -> Option<String> {
        let input_nodes = self.input_nodes();
        let locked = input_nodes.get(input)?.locked.as_ref()?;
        Some(locked.rev().clone())
    }

    pub fn original_of(&self, input: &str) -> Option<Original> {
        self.input_nodes().get(input)?.original.clone()
    }
}

impl TryFrom<PathBuf> for FlakeLock {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let path_str = value
            .to_str()
            .ok_or_else(|| anyhow::Error::msg("Couldn't construct path"))?;
        let flake_lock_file = if value.is_dir() {
            format!("{path_str}/flake.lock")
        } else {
            path_str.to_string()
        };

        let flake_lock_json =
            fs::read_to_string(flake_lock_file).expect("Couldn't load flake.lock");

        serde_json::from_str::<FlakeLock>(&flake_lock_json).map_err(anyhow::Error::new)
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
    pub original: Option<Original>,
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
pub enum Original {
    #[serde(rename_all = "camelCase")]
    Git {
        url: String,
        rev: Option<String>,
        r#ref: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Github {
        owner: String,
        repo: String,
        rev: Option<String>,
        r#ref: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    GitLab {
        owner: String,
        repo: String,
        rev: Option<String>,
        r#ref: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Indirect { id: String, r#ref: Option<String> },
}

#[cfg(test)]
impl Original {
    pub fn git_url_only(url: &str) -> Self {
        Original::Git {
            url: String::from(url),
            rev: None,
            r#ref: None,
        }
    }
    pub fn git_with_ref(url: &str, r#ref: &str) -> Self {
        Original::Git {
            url: String::from(url),
            rev: None,
            r#ref: Some(String::from(r#ref)),
        }
    }
    pub fn git_with_rev(url: &str, rev: &str) -> Self {
        Original::Git {
            url: String::from(url),
            rev: Some(String::from(rev)),
            r#ref: None,
        }
    }
    #[allow(clippy::similar_names)]
    pub fn git_with_ref_and_rev(url: &str, rev: &str, r#ref: &str) -> Self {
        Original::Git {
            url: String::from(url),
            rev: Some(String::from(rev)),
            r#ref: Some(String::from(r#ref)),
        }
    }
    pub fn github(owner: &str, repo: &str, r#ref: Option<&str>) -> Self {
        Original::Github {
            owner: String::from(owner),
            repo: String::from(repo),
            rev: None,
            r#ref: r#ref.map(String::from),
        }
    }

    pub fn indirect(id: &str, r#ref: Option<&str>) -> Self {
        Original::Indirect {
            id: String::from(id),
            r#ref: r#ref.map(String::from),
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Locked {
    #[serde(rename_all = "camelCase")]
    Git {
        rev: String,
        r#ref: Option<String>,
        url: String,
        #[serde(with = "time::serde::timestamp")]
        last_modified: OffsetDateTime,
    },
    #[serde(rename_all = "camelCase")]
    Github {
        rev: String,
        r#ref: Option<String>,
        owner: String,
        repo: String,
        #[serde(with = "time::serde::timestamp")]
        last_modified: OffsetDateTime,
    },
    #[serde(rename_all = "camelCase")]
    GitLab {
        rev: String,
        r#ref: Option<String>,
        owner: String,
        repo: String,
        #[serde(with = "time::serde::timestamp")]
        last_modified: OffsetDateTime,
    },
}

impl Locked {
    pub fn rev(&self) -> &String {
        match self {
            Locked::Git {
                rev,
                r#ref: _,
                url: _,
                last_modified: _,
            }
            | Locked::Github {
                rev,
                r#ref: _,
                owner: _,
                repo: _,
                last_modified: _,
            }
            | Locked::GitLab {
                rev,
                r#ref: _,
                owner: _,
                repo: _,
                last_modified: _,
            } => rev,
        }
    }
}

#[cfg(test)]
pub mod fixtures {
    use std::collections::HashMap;

    use time::OffsetDateTime;

    use super::{FlakeLock, InputValue, Locked, Node, Original};

    pub fn inputs(name: &str) -> HashMap<String, InputValue> {
        let mut inputs = HashMap::new();
        inputs.insert(String::from(name), InputValue::Scalar(String::from(name)));
        inputs
    }

    pub fn root_node(input_name: &str) -> Node {
        Node {
            flake: None,
            locked: None,
            inputs: Some(inputs(input_name)),
            original: None,
        }
    }

    pub fn git_node(input_name: &str, rev: &str, url: &str, original: Option<Original>) -> Node {
        Node {
            flake: None,
            inputs: Some(inputs(input_name)),
            locked: Some(Locked::Git {
                rev: String::from(rev),
                r#ref: None,
                url: String::from(url),
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1685572332).unwrap(),
            }),
            original,
        }
    }

    pub fn nixpkgs_node(rev: &str, original: Option<Original>) -> Node {
        Node {
            flake: None,
            inputs: None,
            locked: Some(Locked::Github {
                rev: String::from(rev),
                r#ref: None,
                owner: String::from("NixOS"),
                repo: String::from("nixpkgs"),
                #[allow(clippy::unreadable_literal)]
                last_modified: OffsetDateTime::from_unix_timestamp(1683627095).unwrap(),
            }),
            original,
        }
    }

    pub fn flake_lock(direct_input_original: Option<Original>) -> FlakeLock {
        let mut nodes = HashMap::new();

        nodes.insert(String::from("root"), root_node("nix-rust-utils"));
        nodes.insert(
            String::from("nix-rust-utils"),
            git_node(
                "nixpkgs",
                "3892194d7b3293de8b30f1d19e2af45ba41ba8fd",
                "https://git.vdx.hu/voidcontext/nix-rust-utils.git",
                direct_input_original,
            ),
        );

        nodes.insert(
            String::from("nixpkgs"),
            nixpkgs_node("a08e061a4ee8329747d54ddf1566d34c55c895eb", None),
        );

        FlakeLock {
            nodes,
            root: String::from("root"),
        }
    }

    pub fn flake_lock_with_node(node: Node) -> FlakeLock {
        let mut nodes = HashMap::new();

        nodes.insert(String::from("root"), root_node("nix-rust-utils"));
        nodes.insert(String::from("nix-rust-utils"), node);

        nodes.insert(
            String::from("nixpkgs"),
            nixpkgs_node("a08e061a4ee8329747d54ddf1566d34c55c895eb", None),
        );

        FlakeLock {
            nodes,
            root: String::from("root"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::nix::{
        fixtures::{flake_lock, git_node},
        Original,
    };

    use super::FlakeLock;

    #[test]
    fn can_deserialize_flake_lock() {
        let result = serde_json::from_str::<FlakeLock>(FLAKE_LOCK_JSON);
        println!("{result:?}");
        assert!(result.is_ok());
    }

    #[test]
    fn top_level_nodes_should_return_root_nodes() {
        let top_level_nodes = flake_lock(None).input_nodes();
        assert_eq!(
            top_level_nodes,
            [(
                String::from("nix-rust-utils"),
                git_node(
                    "nixpkgs",
                    "3892194d7b3293de8b30f1d19e2af45ba41ba8fd",
                    "https://git.vdx.hu/voidcontext/nix-rust-utils.git",
                    None,
                )
            )]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn locked_rev_of_should_return_locked_revision() {
        assert_eq!(
            flake_lock(None).locked_rev_of("nix-rust-utils"),
            Some(String::from("3892194d7b3293de8b30f1d19e2af45ba41ba8fd"))
        );
    }

    #[test]
    fn original_of_should_return_locked_revision() {
        let original = Original::github("voidcontext", "nix-rust-utils", Some("refs/heads/main"));
        assert_eq!(
            flake_lock(Some(original.clone())).original_of("nix-rust-utils"),
            Some(original)
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

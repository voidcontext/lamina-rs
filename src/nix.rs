use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FlakeLock {
    nodes: HashMap<String, Node>,
}

#[derive(Debug, Deserialize)]
pub struct Node {
    locked: Option<Locked>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum Locked {
    Git {
        rev: String,
        url: String,
    },
    Github {
        rev: String,
        owner: String,
        repo: String,
    },
}

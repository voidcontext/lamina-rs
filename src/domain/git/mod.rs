mod repository_service;

use nova::newtype;
#[cfg(test)]
pub(crate) use repository_service::MockRepositoryService;
pub use repository_service::RepositoryService;
use time::OffsetDateTime;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Commit {
    pub(crate) sha: CommitSha,
    pub(crate) commit_time: OffsetDateTime,
}

impl Commit {
    #[cfg(test)]
    #[must_use]
    pub fn from_raw(sha: &str, commit_date: i64) -> Self {
        Self {
            sha: CommitSha::from(sha),
            commit_time: OffsetDateTime::from_unix_timestamp(commit_date).unwrap(),
        }
    }
}

#[newtype(new, borrow = "str")]
pub type CommitSha = String;

impl From<&str> for CommitSha {
    fn from(value: &str) -> Self {
        Self::new(String::from(value))
    }
}

#[newtype(new, borrow = "str")]
pub type BranchName = String;

impl From<&str> for BranchName {
    fn from(value: &str) -> Self {
        Self::new(String::from(value))
    }
}

#[newtype(new, borrow = "str")]
pub type Tag = String;

impl From<&str> for Tag {
    fn from(value: &str) -> Self {
        Self::new(String::from(value))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Ref {
    HEAD,
    Head(BranchName),
    Tag(Tag),
    Remote(String),
}

impl Ref {
    #[cfg(test)]
    pub(crate) fn head(name: &str) -> Self {
        Self::Head(BranchName::from(name))
    }
}

// impl TryFrom<&str> for Ref {
//     type Error = domain::Error;

//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         let re = regex::Regex::new(r"^refs/(heads|tags|remotes)\/(.+)$").unwrap();

//         let captures = re
//             .captures(value)
//             .ok_or_else(|| domain::Error::Error(format!("'{value}' is not a valid git ref")))?;

//         match captures.extract() {
//             (_, ["heads", name]) => Ok(Ref::Head(BranchName::from(name))),
//             (_, ["tags", tag]) => Ok(Ref::Tag(Tag::from(tag))),
//             (_, ["remotes", remote]) => Ok(Ref::Remote(String::from(remote))),
//             _ => panic!("This shouldn't happen"),
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq)]
pub enum RemoteReference {
    Url(String),
    GitHub { repo: String, owner: String },
    GitLab { repo: String, owner: String },
}

impl RemoteReference {
    #[cfg(test)]
    pub(crate) fn url<S: AsRef<str>>(url: S) -> Self {
        Self::Url(String::from(url.as_ref()))
    }
}

// #[cfg(test)]
// mod tests {
//     use pretty_assertions::assert_eq;

//     use crate::domain::git::{BranchName, Ref, Tag};

//     #[test]
//     fn test_ref_try_from_branch_ref() {
//         let result = Ref::try_from("refs/heads/main");

//         assert_eq!(result.unwrap(), Ref::Head(BranchName::from("main")));
//     }

//     #[test]
//     fn test_ref_try_from_tag_ref() {
//         let result = Ref::try_from("refs/tags/v0.5.0-alpha.1");

//         assert_eq!(result.unwrap(), Ref::Tag(Tag::from("v0.5.0-alpha.1")));
//     }

//     #[test]
//     fn test_ref_try_from_remote_ref() {
//         let result = Ref::try_from("refs/remotes/origin/main");

//         assert_eq!(result.unwrap(), Ref::Remote(String::from("origin/main")));
//     }

//     #[test]
//     fn test_ref_try_from_invalid() {
//         let result = Ref::try_from("refs/unknown/origin/main");

//         let err = result.unwrap_err();

//         match err {
//             crate::domain::Error::ParserError(message) => {
//                 assert_eq!(message, "'refs/unknown/origin/main' is not a valid git ref");
//             }
//             _ => panic!("Error is different than expected"),
//         }
//     }
// }

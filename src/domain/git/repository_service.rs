use mockall::automock;

use crate::domain::Result;

use super::{Commit, Ref, RemoteReference};

#[automock]
pub trait RepositoryService {
    fn find_branch_or_tag(&self, name: &str) -> Result<Option<Ref>>;
    fn resolve_ref(&self, remote_ref: &RemoteReference, git_ref: &Ref) -> Result<Option<Commit>>;
}

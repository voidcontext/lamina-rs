use mockall::automock;

use crate::domain::Result;

use super::{Commit, Ref, RemoteReference, Tag};

#[automock]
pub trait RepositoryService {
    fn find_branch_or_tag(&self, remote_ref: &RemoteReference, name: &str) -> Result<Option<Ref>>;
    fn find_latest_tag(&self, remote_ref: &RemoteReference) -> Result<Option<Tag>>;
    fn resolve_ref(&self, remote_ref: &RemoteReference, git_ref: &Ref) -> Result<Option<Commit>>;
}

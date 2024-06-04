use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use filenamify::filenamify;
use git2::{build::RepoBuilder, FetchOptions, Repository};
use semver::Version;
use tempdir::TempDir;

use crate::domain::{
    self,
    git::{self, BranchName, Commit, Ref, RemoteReference, Tag},
};

#[allow(clippy::module_name_repetitions)]
pub struct GitRepositoryConfig {
    cache: bool,
    cache_dir: PathBuf,
}

impl GitRepositoryConfig {
    #[must_use]
    pub fn new(cache: bool, cache_dir: PathBuf) -> Self {
        Self { cache, cache_dir }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct GitRepositoryLibGit {
    repos: Mutex<HashMap<String, PathBuf>>,
    config: GitRepositoryConfig,
}

impl Drop for GitRepositoryLibGit {
    fn drop(&mut self) {
        if !self.config.cache {
            if let Ok(repos) = self.repos.lock() {
                repos.iter().for_each(|(_, path)| {
                    log::debug!("deleting temporary repo clone at {path:?}");
                    fs::remove_dir_all(path).unwrap_or_default();
                });
            }
        }
    }
}

impl GitRepositoryLibGit {
    #[must_use]
    pub fn new(config: GitRepositoryConfig) -> Self {
        Self {
            repos: Mutex::new(HashMap::new()),
            config,
        }
    }

    fn load_repo(&self, url: &str) -> domain::Result<Repository> {
        match self.repos.try_lock() {
            Ok(mut mutex) => {
                let repos = &mut *mutex;

                if let Some(path) = repos.get(url) {
                    Repository::open(path).map_err(|err| domain::Error::Error(err.to_string()))
                } else {
                    let mut dir: PathBuf;
                    let repo: Repository;

                    if self.config.cache {
                        dir = self.config.cache_dir.clone();
                        dir.push(filenamify(url));

                        if dir.as_path().exists() {
                            repo = Repository::open(dir.clone())
                                .map_err(|err| domain::Error::Error(err.to_string()))?;

                            let mut remote = repo
                                .find_remote("origin")
                                .map_err(|err| domain::Error::Error(err.to_string()))?;

                            let mut options = FetchOptions::new();
                            options.depth(1);
                            options.download_tags(git2::AutotagOption::All);
                            remote
                                .fetch::<&str>(&[], Some(&mut options), None)
                                .map_err(|err| domain::Error::Error(err.to_string()))?;
                            log::debug!(
                                "loaded and fetched existing repository at {}",
                                dir.as_path().to_string_lossy()
                            );
                        } else {
                            fs::create_dir_all(&dir).unwrap();
                            repo = clone_repo(url, dir.as_path())?;
                            log::debug!(
                                "{url} has been cloned into {}",
                                dir.as_path().to_string_lossy()
                            );
                        }
                    } else {
                        let temp_dir = TempDir::new(&format!("lamina-{}", filenamify(url)))?;
                        dir = temp_dir.into_path();

                        repo = clone_repo(url, dir.as_path())?;
                        log::debug!(
                            "{url} has been cloned into {}",
                            dir.as_path().to_string_lossy()
                        );
                    }

                    repos.insert(String::from(url), dir);

                    Ok(repo)
                }
            }
            Err(_) => todo!(),
        }
    }
}

fn clone_repo(url: &str, dir: &Path) -> domain::Result<Repository> {
    let mut options = FetchOptions::new();
    options.depth(1);
    options.download_tags(git2::AutotagOption::All);

    RepoBuilder::new()
        .fetch_options(options)
        .clone(url, dir)
        .map_err(|err| domain::Error::Error(err.to_string()))
}

impl git::RepositoryService for GitRepositoryLibGit {
    fn find_branch_or_tag(
        &self,
        remote_ref: &RemoteReference,
        name: &str,
    ) -> domain::Result<Option<Ref>> {
        let url = remote_ref.to_url();
        let repo = self.load_repo(&url)?;

        let branch_result = repo.find_reference(&format!("refs/remotes/origin/{name}"));
        log::debug!(
            "find_branch_or_tag | branch_result: {:?}",
            branch_result.as_ref().map(|r| r.name().map(String::from))
        );

        if branch_result.is_ok() {
            Ok(Some(Ref::Head(BranchName::from(name))))
        } else {
            let ref_result = repo.find_reference(&format!("refs/tags/{name}"));
            log::debug!(
                "find_branch_or_tag | ref_result: {:?}",
                ref_result.as_ref().map(|r| r.name().map(String::from))
            );
            if ref_result.is_ok() {
                Ok(Some(Ref::Tag(Tag::from(name))))
            } else {
                Ok(None)
            }
        }
    }

    fn find_latest_tag(&self, remote_ref: &RemoteReference) -> domain::Result<Option<Tag>> {
        let url = remote_ref.to_url();
        let repo = self.load_repo(&url)?;

        let tags = repo
            .tag_names(None)
            .map_err(|err| domain::Error::Error(err.to_string()))?;

        let latest_tag = tags.iter().fold(None, |latest, tag| {
            if let Some(tag) = tag {
                let sem_ver = String::from(tag)
                    .strip_prefix('v')
                    .map(String::from)
                    .and_then(|t| Version::parse(&t).ok());

                if let Some(sv) = sem_ver {
                    if let Some((l, _)) = latest.clone() {
                        if sv.pre.is_empty() && sv.build.is_empty() && sv > l {
                            Some((sv, Tag::from(tag)))
                        } else {
                            latest
                        }
                    } else {
                        Some((sv, Tag::from(tag)))
                    }
                } else {
                    latest
                }
            } else {
                latest
            }
        });

        Ok(latest_tag.map(|(_, tag)| tag))
    }

    fn resolve_ref(
        &self,
        remote_ref: &RemoteReference,
        git_ref: &Ref,
    ) -> domain::Result<Option<Commit>> {
        let url = remote_ref.to_url();
        let repo = self.load_repo(&url)?;

        let gref = repo
            .find_reference(&git_ref.to_string())
            .map_err(|err| domain::Error::Error(err.to_string()))?;

        log::debug!(
            "resolve_ref | find_reference result: {:?}",
            gref.name().map(String::from)
        );

        let resolved = gref
            .resolve()
            .map_err(|err| domain::Error::Error(err.to_string()))?;
        log::debug!(
            "resolve_ref | resolve result: {:?}",
            resolved.name().map(String::from)
        );

        let commit = resolved
            .peel_to_commit()
            .map_err(|err| domain::Error::Error(err.to_string()))?;

        Ok(Some(domain::git::Commit::from_raw(
            &commit.id().to_string(),
            commit.time().seconds(),
        )))
    }
}

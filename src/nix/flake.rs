use std::{collections::HashMap, process::Command};

use crate::{
    domain::{self, fs::FileSystem, nix::FlakeNix},
    fs::ensure_file,
};

use super::flake_lock::{FlakeLock, InputReference, Locked, Original};

pub struct Flake<FS: FileSystem, M: FlakeLockMapper> {
    fs: FS,
    mapper: M,
}

impl<FS: FileSystem, M: FlakeLockMapper> Flake<FS, M> {
    pub fn new(fs: FS, mapper: M) -> Self {
        Self { fs, mapper }
    }
}

impl<FS: FileSystem, M: FlakeLockMapper> domain::nix::Flake for Flake<FS, M> {
    fn load_lock(&self) -> domain::Result<domain::nix::FlakeLock> {
        let current_dir = self.fs.current_dir()?;

        self.load_lock_from(current_dir)
    }

    fn load_lock_from<P: AsRef<std::path::Path>>(
        &self,
        p: P,
    ) -> domain::Result<domain::nix::FlakeLock> {
        let lock_file = ensure_file(p.as_ref(), "flake.lock")?;

        let flake_lock_json = self.fs.read_to_string(lock_file.clone())?;

        let lock = serde_json::from_str::<FlakeLock>(&flake_lock_json).map_err(|err| {
            domain::Error::InvalidFlakeLock {
                reason: err.to_string(),
            }
        })?;

        self.mapper.to_domain(&lock)
    }

    fn load_from<P: AsRef<std::path::Path>>(&self, p: P) -> domain::Result<domain::nix::FlakeNix> {
        let flake_file = ensure_file(p.as_ref(), "flake.nix")?;
        let content = self.fs.read_to_string(flake_file)?;

        Ok(FlakeNix::new(content))
    }

    fn write<P: AsRef<std::path::Path>>(&self, p: P, flake: &FlakeNix) -> domain::Result<()> {
        let flake_file = ensure_file(p.as_ref(), "flake.nix")?;

        self.fs.write(flake_file, &flake.as_string())
    }

    fn override_input<P: AsRef<std::path::Path>>(
        &self,
        p: P,
        input: &str,
        url: &str,
    ) -> domain::Result<()> {
        let path = p.as_ref().to_str().ok_or(domain::Error::Error(format!(
            "Couldn't convert path '{:?}' to str",
            p.as_ref()
        )))?;

        // TODO: some process abstraction
        let mut cmd = Command::new("nix");
        cmd.args(["flake", "lock", path, "--override-input", input, url]);

        log::debug!("running command: {:?}", cmd);

        cmd.status()?;

        Ok(())
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait FlakeLockMapper {
    fn to_domain(&self, f: &FlakeLock) -> domain::Result<domain::nix::FlakeLock>;
}

#[allow(clippy::module_name_repetitions)]
pub struct FlakeLockMapperImpl {}

impl FlakeLockMapper for FlakeLockMapperImpl {
    #[allow(clippy::too_many_lines)] // TODO: fix this
    fn to_domain(&self, f: &FlakeLock) -> domain::Result<domain::nix::FlakeLock> {
        let root = find_root(f).ok_or(domain::Error::Error(String::from(
            "Couldn't find flake root",
        )))?;

        let nodes = f
            .nodes
            .iter()
            .filter(|(name, _)| name != &&String::from("root"))
            .map(|(name, v)| {
                let inputs = v
                    .inputs
                    .as_ref()
                    .unwrap_or(&HashMap::new())
                    .iter()
                    .map(|(name, node)| {
                        (
                            name.clone(),
                            match node {
                                InputReference::Alias(input) => {
                                    domain::nix::InputReference::Alias(input.clone())
                                }
                                InputReference::Path(path) => {
                                    domain::nix::InputReference::Path(path.clone())
                                }
                            },
                        )
                    })
                    .collect::<HashMap<String, domain::nix::InputReference>>();

                let locked = v
                    .locked
                    .as_ref()
                    .ok_or(domain::Error::Error(format!(
                        "'locked' attribute is missing in {name} input"
                    )))
                    .map(|locked| match locked {
                        Locked::Git {
                            rev,
                            r#ref,
                            url,
                            last_modified,
                        } => domain::nix::Locked {
                            rev: domain::git::CommitSha::from(&**rev),
                            r#ref: r#ref.as_ref().map(|r| domain::nix::LockedRef::from(&**r)),
                            source: domain::nix::LockedSource::Git { url: url.clone() },
                            last_modified: *last_modified,
                        },
                        Locked::Github {
                            rev,
                            r#ref,
                            owner,
                            repo,
                            last_modified,
                        } => domain::nix::Locked {
                            rev: domain::git::CommitSha::from(&**rev),
                            r#ref: r#ref.as_ref().map(|r| domain::nix::LockedRef::from(&**r)),
                            source: domain::nix::LockedSource::GitHub {
                                owner: owner.clone(),
                                repo: repo.clone(),
                            },
                            last_modified: *last_modified,
                        },
                        Locked::GitLab {
                            rev,
                            r#ref,
                            owner,
                            repo,
                            last_modified,
                        } => domain::nix::Locked {
                            rev: domain::git::CommitSha::from(&**rev),
                            r#ref: r#ref.as_ref().map(|r| domain::nix::LockedRef::from(&**r)),
                            source: domain::nix::LockedSource::GitLab {
                                owner: owner.clone(),
                                repo: repo.clone(),
                            },
                            last_modified: *last_modified,
                        },
                    })?;

                let original = v
                    .original
                    .as_ref()
                    .ok_or(domain::Error::Error(format!(
                        "'original' attribute is missing in {name} input"
                    )))
                    .map(|original| match original {
                        Original::Git { url, r#ref, rev } => domain::nix::Original {
                            rev: rev.as_ref().map(|r| domain::git::CommitSha::from(&**r)),
                            r#ref: r#ref.as_ref().map(|r| domain::nix::OriginalRef::from(&**r)),
                            source: domain::nix::OriginalSource::Git { url: url.clone() },
                        },
                        Original::Github {
                            owner,
                            repo,
                            r#ref,
                            rev,
                        } => domain::nix::Original {
                            rev: rev.as_ref().map(|r| domain::git::CommitSha::from(&**r)),
                            r#ref: r#ref.as_ref().map(|r| domain::nix::OriginalRef::from(&**r)),
                            source: domain::nix::OriginalSource::GitHub {
                                owner: owner.clone(),
                                repo: repo.clone(),
                            },
                        },
                        Original::GitLab {
                            owner,
                            repo,
                            r#ref,
                            rev,
                        } => domain::nix::Original {
                            rev: rev.as_ref().map(|r| domain::git::CommitSha::from(&**r)),
                            r#ref: r#ref.as_ref().map(|r| domain::nix::OriginalRef::from(&**r)),
                            source: domain::nix::OriginalSource::GitLab {
                                owner: owner.clone(),
                                repo: repo.clone(),
                            },
                        },
                        Original::Indirect { id, r#ref, rev } => domain::nix::Original {
                            rev: rev.as_ref().map(|r| domain::git::CommitSha::from(&**r)),
                            r#ref: r#ref.as_ref().map(|r| domain::nix::OriginalRef::from(&**r)),
                            source: domain::nix::OriginalSource::Indirect { id: id.clone() },
                        },
                    })?;

                Ok((
                    name.clone(),
                    domain::nix::Node {
                        inputs,
                        locked,
                        original,
                    },
                ))
            })
            .collect::<domain::Result<HashMap<String, domain::nix::Node>>>()?;

        Ok(domain::nix::FlakeLock { root, nodes })
    }
}

fn find_root(f: &FlakeLock) -> Option<domain::nix::RootNode> {
    f.nodes.get(&f.root).and_then(|n| {
        n.inputs.as_ref().map(|root_inputs| {
            let inputs = root_inputs
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        match v {
                            InputReference::Alias(input) => {
                                domain::nix::InputReference::Alias(input.clone())
                            }
                            InputReference::Path(_) => panic!("This shouldn't happen"),
                        },
                    )
                })
                .collect();

            domain::nix::RootNode { inputs }
        })
    })
}

#[cfg(test)]
mod tests {
    // TODO: add remaining tests
    #[test]
    #[ignore = "Not implemented"]
    fn flake_load_lock_should_load_from_current_dir() {}

    // TODO: add remaining tests
    #[test]
    #[ignore = "Not implemented"]
    fn flake_load_mapper_should_map_to_domain() {}
}

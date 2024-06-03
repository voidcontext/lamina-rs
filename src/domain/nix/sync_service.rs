use rnix::{
    ast::{self, Attr, AttrpathValue, Entry, HasEntry, InterpolPart},
    Root,
};
use rowan::{
    ast::{AstChildren, AstNode},
    GreenNode, GreenNodeBuilder,
};

use crate::domain::{self, commands::SyncInputNames};
use crate::domain::{Error, Result};

use super::{
    flake_lock::{Locked, LockedSource, Original, OriginalSource},
    FlakeLock, FlakeNix, SyncStrategy,
};

pub trait SyncService {
    fn sync_strategy<'a>(
        &'a self,
        source: &FlakeLock,
        destination: &FlakeLock,
        input: &'a SyncInputNames,
    ) -> Result<SyncStrategy<'a>>;

    fn sync(
        &self,
        source: &FlakeNix,
        destionation: &FlakeNix,
        input: &SyncInputNames,
    ) -> Result<FlakeNix>;
}

#[allow(clippy::module_name_repetitions)]
pub struct SyncServiceImpl {}

impl SyncService for SyncServiceImpl {
    fn sync_strategy<'a>(
        &'a self,
        source: &FlakeLock,
        destination: &FlakeLock,
        input: &'a SyncInputNames,
    ) -> Result<SyncStrategy<'a>> {
        let src_original = source
            .nodes
            .get(input.source())
            .map(|n| n.original.clone())
            .ok_or_else(|| {
                Error::Error(format!(
                    "Couldn't find {}'s Original at source.",
                    input.source()
                ))
            })?;
        let src_locked = source
            .nodes
            .get(input.source())
            .map(|n| n.locked.clone())
            .ok_or_else(|| {
                Error::Error(format!(
                    "Couldn't find {}'s Locked at source.",
                    input.source()
                ))
            })?;
        let dst_original = destination
            .nodes
            .get(input.destination())
            .map(|n| n.original.clone())
            .ok_or_else(|| {
                Error::Error(format!(
                    "Couldn't find {}'s Original at destination.",
                    input.destination()
                ))
            })?;

        let override_url = override_url(&src_original, &src_locked)?;

        log::info!("src: {src_original:?} == dst: {dst_original:?}");

        if src_original.source == dst_original.source {
            if src_original.r#ref == dst_original.r#ref && src_original.rev == dst_original.rev {
                Ok(SyncStrategy::lock_only(override_url, input))
            } else {
                Ok(SyncStrategy::flake_nix_and_lock(override_url, input))
            }
        } else {
            Err(Error::Error(String::from(
                "Cannot sync inputst with different type or from different git repository",
            )))
        }
    }

    fn sync(
        &self,
        source: &FlakeNix,
        destionation: &FlakeNix,
        input: &SyncInputNames,
    ) -> Result<FlakeNix> {
        let source_flake = rnix::Root::parse(&source.as_string())
            .ok()
            .map_err(|err| domain::Error::NixParserError(err.to_string()))?;
        let destination_flake = rnix::Root::parse(&destionation.as_string())
            .ok()
            .map_err(|err| domain::Error::NixParserError(err.to_string()))?;

        let source_input_url = find_input_url(input.source(), &source_flake)?;
        let source_input_url_str = string_content(&source_input_url).ok_or_else(|| {
            domain::Error::NixParserError(String::from("Couldn't find input url value at source"))
        })?;

        let destination_input_url = find_input_url(input.destination(), &destination_flake)?;

        let updated_flake = override_input_url(&destination_input_url, &source_input_url_str);

        Ok(FlakeNix::new(updated_flake.to_string()))
    }
}
fn find_input_url(input: &str, flake: &Root) -> Result<AttrpathValue> {
    let expr = flake.expr().unwrap();
    let set = match expr {
        ast::Expr::AttrSet(set) => Ok(set),
        _ => Err(domain::Error::NixParserError(String::from(
            "root isn't a set",
        ))),
    }?;

    let path = ["inputs", input, "url"];
    find_path(&mut set.entries(), &path)
}

fn find_path(attrset: &mut AstChildren<Entry>, path: &[&str]) -> Result<AttrpathValue> {
    attrset
        .find_map(|entry| match entry {
            ast::Entry::AttrpathValue(attrpath_value) => {
                let attrs = attrpath_value.attrpath().unwrap().attrs();
                let idents = attrs
                    .into_iter()
                    .filter_map(|a| match a {
                        Attr::Ident(ident) => Some(ident.ident_token().unwrap().text().to_string()),
                        _ => None,
                    })
                    .collect::<Vec<_>>();

                let to_compare = path[0..idents.len()].to_vec();
                let remaining = path[idents.len()..path.len()].to_vec();

                if idents == to_compare {
                    if remaining.is_empty() {
                        Some(Ok(attrpath_value))
                    } else {
                        match attrpath_value.value() {
                            Some(ast::Expr::AttrSet(set)) => {
                                Some(find_path(&mut set.entries(), &remaining))
                            }
                            _ => None,
                        }
                    }
                } else {
                    None
                }
            }
            ast::Entry::Inherit(_) => None,
        })
        .unwrap_or_else(|| {
            Err(domain::Error::NixParserError(String::from(
                "Couldn't find path",
            )))
        })
}

fn string_content(attr: &AttrpathValue) -> Option<String> {
    match attr.value().unwrap() {
        rnix::ast::Expr::Str(str) => str.normalized_parts().iter().find_map(|p| match p {
            InterpolPart::Literal(l) => Some(l.clone()),
            InterpolPart::Interpolation(_) => None,
        }),
        _ => None,
    }
}

fn override_input_url(input: &AttrpathValue, input_url: &str) -> GreenNode {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(rowan::SyntaxKind(rnix::SyntaxKind::NODE_STRING as u16));
    builder.token(
        rowan::SyntaxKind(rnix::SyntaxKind::TOKEN_STRING_START as u16),
        "\"",
    );
    builder.token(
        rowan::SyntaxKind(rnix::SyntaxKind::TOKEN_STRING_CONTENT as u16),
        input_url,
    );
    builder.token(
        rowan::SyntaxKind(rnix::SyntaxKind::TOKEN_STRING_END as u16),
        "\"",
    );
    builder.finish_node();
    let node = builder.finish();

    match input.value() {
        Some(rnix::ast::Expr::Str(str)) => str.syntax().replace_with(node),
        _ => todo!(),
    }
}

fn override_url(original: &Original, locked: &Locked) -> Result<String> {
    match &original.source {
        OriginalSource::Indirect { id } => Ok(format!("{id}/{}", &*locked.rev)),
        _ => match &locked.source {
            LockedSource::Git { url } => {
                let mut query_str = format!("rev={}", &*locked.rev);
                if let Some(r#ref) = &locked.r#ref {
                    query_str = format!("ref={}&{query_str}", &**r#ref);
                }
                Ok(format!("git+{url}?{query_str}"))
            }
            LockedSource::GitHub { owner, repo } => {
                Ok(format!("github:{owner}/{repo}/{}", &*locked.rev))
            }
            LockedSource::GitLab { owner: _, repo: _ } => {
                Err(Error::Error(String::from("Gitlab is not supported yet")))
            }
        },
    }
}

#[cfg(test)]
mod fixtures {}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use crate::domain::{
        commands::SyncInputNames,
        git::CommitSha,
        nix::{
            flake_lock::{Node, OriginalRef},
            sync_service::{SyncService, SyncServiceImpl},
            FlakeNix, SyncStrategy,
        },
    };

    fn oneline(url: &str) -> String {
        format!(
            "{{
  inputs.nix-rust-utils.url = \"{url}\";
  outputs = {{nix-rust-utils, ...}}:
    nix-rust-utils.lib.mkOutputs ({{...}}: {{crate.src = ./. ;}});
}}"
        )
    }

    fn input_attrset(url: &str) -> String {
        format!(
            "{{
  inputs = {{
    nix-rust-utils.url = \"{url}\";
  }};
  outputs = {{nix-rust-utils, ...}}:
    nix-rust-utils.lib.mkOutputs ({{...}}: {{crate.src = ./. ;}});
}}"
        )
    }

    fn all_attrset(url: &str) -> String {
        format!(
            "{{
  inputs = {{
    nix-rust-utils = {{
      url = \"{url}\";
    }};
  }};
  outputs = {{nix-rust-utils, ...}}:
    nix-rust-utils.lib.mkOutputs ({{...}}: {{crate.src = ./. ;}});
}}"
        )
    }

    #[rstest]
    #[case(
        "nix-rust-utils",
        oneline("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0"),
        oneline("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git"),
        oneline("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0")
    )]
    #[case(
        "nix-rust-utils",
        input_attrset(
            "git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0"
        ),
        oneline("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git"),
        oneline("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0")
    )]
    #[case(
        "nix-rust-utils",
        all_attrset("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0"),
        oneline("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git"),
        oneline("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0")
    )]
    #[case(
        "nix-rust-utils",
        all_attrset("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0"),
        input_attrset("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git"),
        input_attrset(
            "git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0"
        )
    )]
    #[case(
        "nix-rust-utils",
        oneline("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0"),
        all_attrset("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git"),
        all_attrset("git+https://git.vdx.hu/voidcontext/nix-rust-utils.git?ref=refs/tags/v0.5.0")
    )]
    fn test_sync_str_syncs_flake_input(
        #[case] input: &str,
        #[case] source: String,
        #[case] destination: String,
        #[case] expected: String,
    ) {
        let sync_service = SyncServiceImpl {};
        assert_eq!(
            sync_service
                .sync(
                    &FlakeNix::new(source),
                    &FlakeNix::new(destination),
                    &crate::domain::commands::SyncInputNames::same(input.to_string())
                )
                .unwrap()
                .as_string(),
            expected
        );
        // TODO: test scenario missing: different src and dst input name
    }

    // Sync strategy tests
    use crate::domain::nix::flake_lock::fixtures::{
        flake_lock_with_node, git_node_with_ref, git_node_with_rev, git_node_with_url_only,
        github_node_with_owner_and_repo_only, github_node_with_ref, nixpkgs_node_with_ref,
    };

    lazy_static::lazy_static! {
        static ref HASH_1: CommitSha = CommitSha::from("f542386b0646cf39b9475a200979adabd07d98b2");
        static ref HASH_2: CommitSha = CommitSha::from("4468e5deabf5e6d0740cd1a77df56f67093ec943");
        static ref INPUT_NAMES: SyncInputNames = SyncInputNames::same("nix-rust-utils".to_string());
    }

    #[rstest]
    // src -> dest -> strategy
    #[case( // git url -> git url -> lock only
        git_node_with_url_only("https://example.com/user/repo.git", &HASH_1),
        git_node_with_url_only("https://example.com/user/repo.git", &HASH_2),
        Ok(crate::domain::nix::SyncStrategy::lock_only(format!(
            "git+https://example.com/user/repo.git?rev={}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// git url + ref -> git url -> flake.nix + lock
        git_node_with_url_only("https://example.com/user/repo.git", &HASH_1),
        git_node_with_ref("https://example.com/user/repo.git", &HASH_2,&crate::domain::nix::flake_lock::OriginalRef::from("refs/tags/v0.2.0")),
        Ok(SyncStrategy::flake_nix_and_lock(format!(
            "git+https://example.com/user/repo.git?ref=refs/tags/v0.2.0&rev={}", &**HASH_2         
        ), &INPUT_NAMES))
    )]
    #[case(// git url + ref -> git url + ref (different ref) -> flake.nix + lock
        git_node_with_ref("https://example.com/user/repo.git", &HASH_1,&OriginalRef::from("refs/tags/v0.1.0")),
        git_node_with_ref("https://example.com/user/repo.git", &HASH_2,&OriginalRef::from("refs/tags/v0.2.0")),
        Ok(SyncStrategy::flake_nix_and_lock(format!(
            "git+https://example.com/user/repo.git?ref=refs/tags/v0.2.0&rev={}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// git url + ref -> git url + ref (same ref) -> lock only
        git_node_with_ref("https://example.com/user/repo.git", &HASH_1,&OriginalRef::from("refs/tags/v0.2.0")),
        git_node_with_ref("https://example.com/user/repo.git", &HASH_2,&OriginalRef::from("refs/tags/v0.2.0")),
        Ok(SyncStrategy::lock_only(format!(
            "git+https://example.com/user/repo.git?ref=refs/tags/v0.2.0&rev={}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// git url + rev -> git url + rev (same rev) -> noop
        git_node_with_rev("https://example.com/user/repo.git", &HASH_1),
        git_node_with_rev("https://example.com/user/repo.git", &HASH_1),
        Ok(SyncStrategy::lock_only(format!(
            "git+https://example.com/user/repo.git?rev={}", &**HASH_1
        ), &INPUT_NAMES)) // TODO: this should be NOOP really
    )]
    #[case(// git url + rev -> git url + rev (different rev) -> flake.nix + lock
        git_node_with_rev("https://example.com/user/repo.git", &HASH_1),
        git_node_with_rev("https://example.com/user/repo.git", &HASH_2),
        Ok(SyncStrategy::flake_nix_and_lock(format!(
            "git+https://example.com/user/repo.git?rev={}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// github -> github -> lock only
        github_node_with_owner_and_repo_only("owner", "repo", &HASH_1),
        github_node_with_owner_and_repo_only("owner", "repo", &HASH_2),
        Ok(SyncStrategy::lock_only(format!(
            "github:owner/repo/{}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// github + ref -> github + ref (same ref) -> lock only
        github_node_with_ref("owner", "repo", &HASH_1, &OriginalRef::from("main")),
        github_node_with_ref("owner", "repo", &HASH_2, &OriginalRef::from("main")),
        Ok(SyncStrategy::lock_only(format!(
            "github:owner/repo/{}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// github + ref -> github + ref (different ref) -> flake.nix + lock
        github_node_with_ref("owner", "repo", &HASH_1, &OriginalRef::from("main")),
        github_node_with_ref("owner", "repo", &HASH_2, &OriginalRef::from("feature-branch")),
        Ok(SyncStrategy::flake_nix_and_lock(format!(
            "github:owner/repo/{}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// indirect + ref -> indirect + ref (same ref) -> lock only
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &HASH_1),
        nixpkgs_node_with_ref(&OriginalRef::from("release-23.05"), &HASH_2),
        Ok(SyncStrategy::lock_only(format!(
            "nixpkgs/{}", &**HASH_2
        ), &INPUT_NAMES))
    )]
    #[case(// github -> git -> error
        git_node_with_url_only("https://example.com/user/repo.git", &HASH_1),
        github_node_with_ref("owner", "repo", &HASH_1, &OriginalRef::from("main")),
        Err("an error happened: \"Cannot sync inputst with different type or from different git repository\"")
    )]
    fn input_override_arg_returns_correct_argument(
        #[case] node1: Node,
        #[case] node2: Node,
        #[case] expected: Result<SyncStrategy, &str>,
    ) {
        let sync_service = SyncServiceImpl {};
        let result = sync_service.sync_strategy(
            &flake_lock_with_node("nix-rust-utils", node2),
            &flake_lock_with_node("nix-rust-utils", node1),
            &INPUT_NAMES,
        );

        assert_eq!(
            result.map_err(|e| e.to_string()),
            expected.map_err(String::from)
        );
    }
}

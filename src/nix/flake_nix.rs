use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use rnix::{
    ast::{self, Attr, AttrpathValue, Entry, HasEntry, InterpolPart},
    Root,
};
use rowan::{
    ast::{AstChildren, AstNode},
    GreenNode, GreenNodeBuilder,
};

pub fn sync(
    src_input: &str,
    dst_input: &str,
    source: &Path,
    destination: &Path,
) -> anyhow::Result<()> {
    log::debug!("syncing flake.nix file");
    let source_flake_nix =
        fs::read_to_string(ensure_flake_nix_path(source)).with_context(|| {
            format!(
                "Failed to read {:?}",
                ensure_flake_nix_path(source).to_str()
            )
        })?;
    let destination_flake_nix = fs::read_to_string(ensure_flake_nix_path(destination))
        .with_context(|| {
            format!(
                "Failed to read {:?}",
                ensure_flake_nix_path(destination).to_str()
            )
        })?;

    let updated_flake_nix = sync_str(
        src_input,
        dst_input,
        &source_flake_nix,
        &destination_flake_nix,
    )?;

    fs::write(ensure_flake_nix_path(destination), updated_flake_nix).with_context(|| {
        format!(
            "Failed to write {:?}",
            ensure_flake_nix_path(destination).to_str()
        )
    })?;

    Ok(())
}

fn ensure_flake_nix_path(path: &Path) -> PathBuf {
    let mut path = path.to_path_buf();
    if path.is_dir() {
        path.push("flake.nix");
    }
    path
}

fn sync_str(
    src_input: &str,
    dst_input: &str,
    source_flake_nix: &str,
    destination_flake_nix: &str,
) -> anyhow::Result<String> {
    let source_flake = rnix::Root::parse(source_flake_nix).ok()?;
    let destination_flake = rnix::Root::parse(destination_flake_nix).ok()?;

    let source_input_url = find_input_url(src_input, &source_flake)?;
    let source_input_url_str = string_content(&source_input_url)
        .ok_or_else(|| anyhow::Error::msg("Couldn't find input url value at source"))?;

    let destination_input_url = find_input_url(dst_input, &destination_flake)?;

    let updated_flake = override_input_url(&destination_input_url, &source_input_url_str);

    Ok(updated_flake.to_string())
}

fn find_input_url(input: &str, flake: &Root) -> anyhow::Result<AttrpathValue> {
    let expr = flake.expr().unwrap();
    let set = match expr {
        ast::Expr::AttrSet(set) => Ok(set),
        _ => Err(anyhow::Error::msg("root isn't a set")),
    }?;

    let path = ["inputs", input, "url"];
    find_path(&mut set.entries(), &path)
}

fn find_path(attrset: &mut AstChildren<Entry>, path: &[&str]) -> anyhow::Result<AttrpathValue> {
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
        .unwrap_or_else(|| Err(anyhow!("Couldn't find path")))
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::sync_str;

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
        assert_eq!(
            sync_str(input, input, &source, &destination).unwrap(),
            expected
        ); // TODO: test scenario missing: different src and dst input name
    }
}

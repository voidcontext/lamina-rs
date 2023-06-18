use crate::{cli::Args, nix::FlakeLock};
use clap::Parser;
use cli::Command;
use std::fs;
use time::format_description;

use comfy_table::{presets::UTF8_BORDERS_ONLY, Table};

mod cli;
mod nix;

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Info => {
            let flake_lock_json =
                fs::read_to_string("flake.lock").expect("Couldn't load flake.lock");
            let flake_lock: FlakeLock =
                serde_json::from_str(&flake_lock_json).expect("Couldn't deserialize flake.lock");

            let mut nodes = flake_lock.top_level_nodes();
            nodes.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

            let mut table = Table::new();
            table.load_preset(UTF8_BORDERS_ONLY);
            table.set_header(vec!["input", "last_modified"]);

            let date_format =
                format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

            for n in &nodes {
                let last_modified = match n.node.locked.as_ref().unwrap() {
                    nix::Locked::Git {
                        rev: _,
                        url: _,
                        last_modified,
                    }
                    | nix::Locked::Github {
                        rev: _,
                        owner: _,
                        repo: _,
                        last_modified,
                    }
                    | nix::Locked::GitLab {
                        rev: _,
                        owner: _,
                        repo: _,
                        last_modified,
                    } => last_modified,
                };

                table.add_row(vec![
                    n.name.clone(),
                    last_modified.format(&date_format).unwrap(),
                ]);
            }

            println!("{table}");
        }
    }
}

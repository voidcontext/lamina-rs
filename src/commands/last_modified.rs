use crate::nix::{self, FlakeLock};
use std::fs;
use time::format_description;

use comfy_table::{presets::UTF8_BORDERS_ONLY, Table};

pub fn last_modified() {
    let flake_lock_json = fs::read_to_string("flake.lock").expect("Couldn't load flake.lock");
    let flake_lock: FlakeLock =
        serde_json::from_str(&flake_lock_json).expect("Couldn't deserialize flake.lock");

    let nodes = flake_lock.input_nodes();
    let mut names: Vec<String> = nodes.keys().cloned().collect();
    names.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(vec!["input", "last_modified"]);

    let date_format =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

    for name in names {
        let last_modified = match nodes.get(&name).unwrap().locked.as_ref().unwrap() {
            nix::Locked::Git {
                rev: _,
                r#ref: _,
                url: _,
                last_modified,
            }
            | nix::Locked::Github {
                rev: _,
                r#ref: _,
                owner: _,
                repo: _,
                last_modified,
            }
            | nix::Locked::GitLab {
                rev: _,
                r#ref: _,
                owner: _,
                repo: _,
                last_modified,
            } => last_modified,
        };

        table.add_row(vec![
            name.clone(),
            last_modified.format(&date_format).unwrap(),
        ]);
    }

    println!("{table}");
}

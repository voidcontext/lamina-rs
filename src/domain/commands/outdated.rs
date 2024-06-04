use comfy_table::{presets::UTF8_BORDERS_ONLY, Cell, Color, Table};
use time::format_description;

use crate::domain::{
    console::Console,
    nix::{Flake, UpdateService, UpdateStatus},
    Result,
};

pub fn outdated<F: Flake, C: Console, US: UpdateService>(
    flake: &F,
    console: &C,
    update_service: &US,
) -> Result<()> {
    let flake_lock = flake.load_lock()?;

    let nodes = flake_lock.input_nodes();
    let mut names: Vec<String> = nodes.keys().cloned().collect();
    names.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // let mut names: Vec<String> = nodes.keys().cloned().collect();
    // names.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(vec!["input", "update", "time", "info"]);

    let date_format =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

    for name in names {
        let node = nodes.get(&name).unwrap();
        match update_service.available_update(node) {
            UpdateStatus::AlreadyLatest => {
                table.add_row(vec![
                    Cell::new(name),
                    Cell::new("up to date!").fg(Color::Green),
                    Cell::new(""),
                    Cell::new(""),
                ]);
            }
            UpdateStatus::Outdated(update) => match update {
                crate::domain::nix::Update::Lock(commit) => {
                    table.add_row(vec![
                        Cell::new(name),
                        Cell::new(format!("{} -> {}", &*node.locked.rev, &*commit.sha))
                            .fg(Color::DarkYellow),
                        Cell::new(format!(
                            "{} -> {}",
                            node.locked.last_modified.format(&date_format).unwrap(),
                            commit.commit_time.format(&date_format).unwrap()
                        )),
                        Cell::new(""),
                    ]);
                }
                crate::domain::nix::Update::Input(git_ref, commit) => {
                    table.add_row(vec![
                        Cell::new(name),
                        Cell::new(format!(
                            "{} -> {}",
                            node.locked
                                .r#ref
                                .clone()
                                .map(|r| String::from(&*r))
                                .or(node.original.r#ref.clone().map(|r| String::from(&*r)))
                                .unwrap(),
                            (&git_ref).to_string()
                        ))
                        .fg(Color::DarkYellow),
                        Cell::new(format!(
                            "{} -> {}",
                            node.locked.last_modified.format(&date_format).unwrap(),
                            commit.commit_time.format(&date_format).unwrap()
                        )),
                        Cell::new(""),
                    ]);
                }
            },
            UpdateStatus::NotAvailable(reason) => {
                table.add_row(vec![
                    Cell::new(name),
                    Cell::new("not available").fg(Color::Red),
                    Cell::new(""),
                    Cell::new(format!("reason: {reason}")),
                ]);
            }
            UpdateStatus::Error(message) => {
                table.add_row(vec![
                    Cell::new(name),
                    Cell::new("error").fg(Color::Red),
                    Cell::new(""),
                    Cell::new(format!("message: {message}")),
                ]);
            }
        };
    }

    console.println(format!("{table}"))?;

    Ok(())
}

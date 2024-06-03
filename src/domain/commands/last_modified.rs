use time::format_description;

use comfy_table::{presets::UTF8_BORDERS_ONLY, Table};

use crate::domain::{console::Console, nix::Flake, Result};

pub fn last_modified<F: Flake, C: Console>(flake: &F, console: &C) -> Result<()> {
    let flake_lock = flake.load_lock()?;

    let nodes = flake_lock.input_nodes();
    let mut names: Vec<String> = nodes.keys().cloned().collect();
    names.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(vec!["input", "last_modified"]);

    let date_format =
        format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]").unwrap();

    for name in names {
        let last_modified = nodes.get(&name).unwrap().locked.last_modified;

        table.add_row(vec![
            name.clone(),
            last_modified.format(&date_format).unwrap(),
        ]);
    }

    console.println(format!("{table}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "Not implemented"]
    fn test_last_modified_prints_table() {
        todo!()
    }
}

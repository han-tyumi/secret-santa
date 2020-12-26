use std::collections::HashMap;

use dialoguer::{Input, MultiSelect};

fn main() {
    let names = Input::new()
        .with_prompt("Enter names separated by commas")
        .validate_with(|input: &String| {
            if input.contains(",") {
                Ok(())
            } else {
                Err("at least two names are required")
            }
        })
        .interact_text()
        .unwrap();

    let names: Vec<_> = names
        .split(",")
        .filter_map(|name| {
            let name = name.trim();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect();

    let mut exclusions = HashMap::new();
    let mut pre_check = HashMap::new();

    for name in &names {
        let pre_checked = pre_check.entry(*name).or_insert(vec![]);

        let items: Vec<_> = names
            .iter()
            .copied()
            .filter_map(|n| {
                if n == *name {
                    None
                } else if pre_checked.contains(&n) {
                    Some((n, true))
                } else {
                    Some((n, false))
                }
            })
            .collect();

        let selected = MultiSelect::new()
            .with_prompt(format!("Exclusions for {}", name))
            .items_checked(&items)
            .interact()
            .unwrap();
        let selected: Vec<_> = selected.iter().map(|i| items[*i].0).collect();

        for selected_name in &selected {
            let pre_select_names = pre_check.entry(*selected_name).or_insert(vec![]);
            pre_select_names.push(*name);
        }

        exclusions.insert(*name, selected.clone());
    }
}

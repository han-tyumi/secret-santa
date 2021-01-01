use std::collections::HashMap;

use clap::{App, Arg};
use dialoguer::{Confirm, Input, MultiSelect};
use secret_santa::Santa;

fn main() {
    let matches = App::new("Secret Santa")
        .version("0.1.0")
        .author("Matt Champagne <mmchamp95@gmail.com>")
        .about("Sets up Secret Santa.")
        .arg(
            Arg::with_name("")
                .multiple(true)
                .takes_value(true)
                .min_values(2),
        )
        .get_matches();

    let names: Vec<_> = match matches.values_of("") {
        Some(names) => names.map(str::to_owned).collect(),
        None => {
            let names = Input::new()
                .with_prompt("Enter names separated by commas")
                .validate_with(|input: &String| {
                    if input.contains(',') {
                        Ok(())
                    } else {
                        Err("at least two names are required")
                    }
                })
                .interact_text()
                .unwrap();

            names
                .split(',')
                .filter_map(|name| {
                    let name = name.trim();
                    if name.is_empty() {
                        None
                    } else {
                        Some(name)
                    }
                })
                .map(str::to_owned)
                .collect()
        }
    };

    let mut exceptions = HashMap::new();
    let mut pre_check = HashMap::new();

    println!();

    for name in &names {
        let pre_checked = pre_check.entry(name.clone()).or_insert(vec![]);

        let items: Vec<_> = names
            .iter()
            .cloned()
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
        let selected: Vec<_> = selected.iter().map(|i| items[*i].0.clone()).collect();

        for selected_name in &selected {
            let pre_select_names = pre_check.entry(selected_name.clone()).or_insert(vec![]);
            pre_select_names.push(name.clone());
        }

        exceptions.insert(name.clone(), selected);
    }

    println!();

    let mut santa = Santa::new();

    for name in names {
        let exceptions = exceptions.entry(name.clone()).or_insert(vec![]);
        let exceptions = exceptions
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>();
        let exceptions = exceptions.as_slice();
        santa.add_name_with_exceptions(name.as_str(), exceptions);
    }

    let santa_matcher = santa.matcher();

    loop {
        let result = santa_matcher.generate();
        println!("Result:\n{:#?}", result);

        if !Confirm::new()
            .with_prompt("Regenerate?")
            .interact()
            .unwrap()
        {
            break;
        }
    }
}

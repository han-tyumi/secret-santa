use std::{collections::HashMap, fs};

use clap::{App, Arg};
use dialoguer::{Confirm, Input, MultiSelect};
use secret_santa::Santa;

fn main() {
    let matches = App::new("Secret Santa")
        .arg(
            Arg::with_name("names")
                .multiple(true)
                .takes_value(true)
                .min_values(2)
                .help("Names of the people participating in this secret santa"),
        )
        .arg(
            Arg::with_name("exceptions")
                .long("exceptions")
                .short("E")
                .help("Always show the name exceptions prompt"),
        )
        .arg(
            Arg::with_name("print")
                .long("print")
                .short("P")
                .help("Print results to stdout"),
        )
        .arg(
            Arg::with_name("directory")
                .long("directory")
                .short("D")
                .takes_value(true)
                .help("Save results to named files in the specified directory"),
        )
        .get_matches();

    let (names, mut exceptions): (Vec<_>, HashMap<_, _>) = match matches.values_of("names") {
        Some(names) => {
            let values: Vec<_> = names.map(str::to_owned).collect();
            let mut names = vec![];
            let mut exceptions = HashMap::new();

            for value in &values {
                // TODO: allow ':' or '='
                let values: Vec<_> = value.split('=').collect();

                if values.len() != 2 {
                    names.push(value.to_owned());
                    continue;
                }

                let name = values[0].to_owned();

                names.push(name.clone());

                // TODO: allow quotes w/ spaces or commas between names or just commas
                let exceptions_for_name: Vec<_> = values[1].split(',').map(str::to_owned).collect();

                if exceptions_for_name.len() > 0 {
                    exceptions.insert(name, exceptions_for_name);
                }
            }

            (names, exceptions)
        }
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

            let names = names
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
                .collect();

            (names, HashMap::new())
        }
    };

    let exceptions_len = exceptions.len();
    if matches.is_present("exceptions") || exceptions_len <= 0 {
        let mut pre_check = if exceptions_len > 0 {
            exceptions.clone()
        } else {
            HashMap::new()
        };

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
    }

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

    let santa_matcher = santa.matcher().unwrap();
    let mut result = santa_matcher.generate().unwrap();

    if matches.is_present("print") {
        loop {
            println!();
            println!("{:#?}", result);

            if !Confirm::new()
                .with_prompt("Regenerate?")
                .interact()
                .unwrap()
            {
                break;
            }

            result = santa_matcher.generate().unwrap();
        }
    }

    if let Some(directory) = matches.value_of("directory") {
        fs::create_dir_all(directory).unwrap();

        for (santa, child) in result {
            fs::write(format!("{}/{}.txt", directory, santa), child.as_bytes()).unwrap();
        }
    }
}

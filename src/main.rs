use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    iter::FromIterator,
};

use dialoguer::{Confirm, Input, MultiSelect};

use rand::{self, Rng};

pub trait RemoveRandom {
    type Item;

    fn remove_random<R: Rng>(&mut self, rng: &mut R) -> Option<Self::Item>;
}

impl<T> RemoveRandom for Vec<T> {
    type Item = T;

    fn remove_random<R: Rng>(&mut self, rng: &mut R) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            let index = rng.gen_range(0..self.len());
            Some(self.swap_remove(index))
        }
    }
}

fn main() {
    let mut rng = rand::thread_rng();

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

    let names: HashSet<_> = names
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

    let mut pre_check = HashMap::new();
    let mut selections = HashMap::new();
    let mut order = HashMap::new();
    let mut next_pick: (usize, HashSet<&str>) = (usize::MAX, HashSet::new());

    println!();

    for name in &names {
        let pre_checked = pre_check.entry(*name).or_insert(HashSet::new());

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
        let selected: HashSet<_> = selected.iter().map(|i| items[*i].0).collect();

        for selected_name in &selected {
            let pre_select_names = pre_check.entry(*selected_name).or_insert(HashSet::new());
            pre_select_names.insert(*name);
        }

        let mut order_key = BinaryHeap::new();
        let selections_for_name: HashSet<_> = items
            .iter()
            .copied()
            .filter_map(|item| {
                let (name, ..) = item;

                if selected.contains(name) {
                    None
                } else {
                    order_key.push(name);
                    Some(name)
                }
            })
            .collect();

        let order_key = order_key.into_sorted_vec().join("");
        let num_names = selections_for_name.len();

        let orders = order.entry(order_key).or_insert(HashSet::new());
        orders.insert(*name);

        if (num_names < next_pick.0)
            || (num_names == next_pick.0 && orders.len() >= next_pick.1.len())
        {
            next_pick = (num_names, orders.clone());
        }

        selections.insert(*name, selections_for_name);
    }

    println!();

    let mut result = HashMap::new();

    loop {
        let mut curr_selections = selections.clone();

        loop {
            let (.., names) = next_pick.clone();
            next_pick = (usize::MAX, HashSet::new());
            order.clear();

            for name in names {
                let picks = curr_selections.remove(name).unwrap();
                let picks = Vec::from_iter(picks);

                let index = rng.gen_range(0..picks.len());
                let pick = picks[index];

                result.insert(name, pick);

                if curr_selections.is_empty() {
                    break;
                }

                for (name, picks) in &mut curr_selections {
                    picks.remove(pick);

                    let order_key: BinaryHeap<_> = picks.iter().copied().collect();
                    let num_names = order_key.len();
                    let order_key = order_key.into_sorted_vec().join("");

                    let orders = order.entry(order_key).or_insert(HashSet::new());
                    orders.insert(*name);

                    if (num_names < next_pick.0)
                        || (num_names == next_pick.0 && orders.len() >= next_pick.1.len())
                    {
                        next_pick = (num_names, orders.clone());
                    }
                }
            }

            if curr_selections.is_empty() {
                break;
            }
        }

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

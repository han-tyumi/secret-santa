use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    iter::FromIterator,
};

use rand::{self, Rng};

#[derive(Debug)]
pub struct Santa {
    names: HashSet<String>,
    exceptions: HashMap<String, HashSet<String>>,
    selections: HashMap<String, HashSet<String>>,
    order: HashMap<String, HashSet<String>>,
    next: (usize, HashSet<String>),
}

#[derive(Debug)]
pub struct SantaMatcher {
    santa: Santa,
}

impl Santa {
    /// Creates a new [`Santa`] instance.
    pub fn new() -> Santa {
        Santa {
            names: HashSet::new(),
            exceptions: HashMap::new(),
            selections: HashMap::new(),
            order: HashMap::new(),
            next: (usize::MAX, HashSet::new()),
        }
    }

    /// Sets the names to be used when generating a secret santa mapping.
    pub fn set_names(&mut self, names: &[&str]) -> &mut Self {
        self.names = names.iter().copied().map(str::to_owned).collect();

        self
    }

    /// Sets the exceptions for a name.
    pub fn set_exceptions_for_name(&mut self, name: &str, exceptions: &[&str]) -> &mut Self {
        let exceptions: HashSet<_> = exceptions.iter().copied().map(str::to_owned).collect();
        self.exceptions.insert(name.into(), exceptions);

        self
    }

    /// Adds a name to be used when generating a secret santa mapping.
    pub fn add_name(&mut self, name: &str) -> &mut Self {
        self.names.insert(name.to_owned());
        self
    }

    /// Adds a name and its exceptions to be used when generating a secret santa mapping.
    pub fn add_name_with_exceptions(&mut self, name: &str, exceptions: &[&str]) -> &mut Self {
        self.add_name(name);
        self.set_exceptions_for_name(name, exceptions)
    }

    /// Creates a matcher that can be used to generate secret santa mappings using the currently set names and exceptions.
    pub fn matcher(mut self) -> Result<SantaMatcher, String> {
        self.update_selections();
        self.validate()?;
        Ok(SantaMatcher { santa: self })
    }

    fn update_selections(&mut self) {
        for name in &self.names {
            let exceptions = self.exceptions.entry(name.into()).or_insert(HashSet::new());
            let selections = self.selections.entry(name.into()).or_insert(HashSet::new());
            let mut order_key = BinaryHeap::new();

            // determine available selections and order key from name exceptions
            *selections = self
                .names
                .iter()
                .filter_map(|n| {
                    if n == name || exceptions.contains(n) {
                        None
                    } else {
                        order_key.push(n.clone());
                        Some(n.clone())
                    }
                })
                .collect();

            // insert name for order key
            let order_key = order_key.into_sorted_vec().join("");
            let names = self.order.entry(order_key).or_insert(HashSet::new());
            names.insert(name.into());

            // update next pick
            let selections_len = selections.len();
            if (selections_len < self.next.0)
                || (selections_len == self.next.0 && names.len() >= self.next.1.len())
            {
                self.next = (selections_len, names.clone());
            }
        }
    }

    fn validate(&self) -> Result<(), String> {
        for (name, selections) in &self.selections {
            if selections.len() <= 0 {
                return Err(format!("no selections for {}", name));
            }
        }

        Ok(())
    }
}

impl SantaMatcher {
    /// Generates a secret santa mapping.
    pub fn generate(&self) -> HashMap<String, String> {
        let SantaMatcher { santa } = self;
        let mut result = HashMap::new();
        let mut selections = santa.selections.clone();
        let mut order = HashMap::new();
        let mut next = santa.next.clone();
        let mut rng = rand::thread_rng();

        loop {
            let (.., names) = next.clone();
            next = (usize::MAX, HashSet::new());
            order.clear();

            for name in names {
                let picks = selections.remove(&name).unwrap();
                let picks = Vec::from_iter(picks);

                let index = rng.gen_range(0..picks.len());
                let pick = picks[index].clone();

                result.insert(name, pick.clone());

                if selections.is_empty() {
                    break;
                }

                // TODO: try to reuse update_selections fn
                for (name, picks) in &mut selections {
                    picks.remove(&pick);

                    let order_key: BinaryHeap<_> = picks.clone().into_iter().collect();
                    let num_names = order_key.len();
                    let order_key = order_key.into_sorted_vec().join("");

                    let orders = order.entry(order_key).or_insert(HashSet::new());
                    orders.insert(name.clone());

                    if (num_names < next.0) || (num_names == next.0 && orders.len() >= next.1.len())
                    {
                        next = (num_names, orders.clone());
                    }
                }
            }

            if selections.is_empty() {
                break;
            }
        }

        result
    }
}

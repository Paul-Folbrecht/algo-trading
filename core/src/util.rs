use log::*;
use num_format::{Locale, ToFormattedString};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Instant;

pub fn time<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let now = Instant::now();
    let result = f();
    let elapsed = now.elapsed().as_millis();
    info!(
        "Time for {}: {}ms",
        label,
        elapsed.to_formatted_string(&Locale::en)
    );
    result
}

pub fn print_map<K, V>(label: &str, map: &HashMap<K, V>)
where
    K: Hash + Debug + Ord,
    V: Debug,
{
    let sorted_keys: Vec<&K> = {
        let mut keys: Vec<_> = map.keys().collect();
        keys.sort();
        keys
    };

    info!("\n{}:\n", label);
    sorted_keys.into_iter().for_each(|k| {
        info!("{:?}: {:?}\n", k, map.get(k).unwrap());
    });
}

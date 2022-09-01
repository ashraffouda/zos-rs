use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::fs;

pub struct Params(HashMap<String, Option<Vec<String>>>);

impl Params {
    pub fn exists<S: AsRef<str>>(&self, s: S) -> bool {
        self.0.get(s.as_ref()).is_some()
    }

    // values will return value assigned to flag for example "key=1 key=2" will return Some([1, 2])
    // if key exists but has no values, will return None. you can check if key exist with exists method
    pub fn values<S: AsRef<str>>(&self, k: S) -> Option<&Vec<String>> {
        match self.0.get(k.as_ref()) {
            None => None,
            Some(v) => match v {
                None => None,
                Some(v) => Some(v),
            },
        }
    }

    pub fn value<S: AsRef<str>>(&self, k: S) -> Option<&str> {
        match self.0.get(k.as_ref()) {
            None => None,
            Some(v) => match v {
                Some(v) if v.len() > 0 => Some(v[v.len() - 1].as_str()),
                _ => None,
            },
        }
    }
}

fn parse_params(content: String) -> Params {
    let mut params_map = HashMap::new();
    for option in content.trim().split(" ") {
        let mut parts = option.splitn(2, "=").into_iter();
        // use this to make sure element exists
        let key = match parts.next() {
            Some(key) => key,
            None => continue,
        };

        match params_map.entry(key.to_string()) {
            Entry::Vacant(e) => {
                match parts.next() {
                    Some(value) => e.insert(Some(vec![value.to_string()])),
                    None => e.insert(None),
                };
            }
            Entry::Occupied(mut e) => match parts.next() {
                Some(value) => match e.get_mut() {
                    Some(old_value) => old_value.push(value.to_string()),
                    None => continue,
                },
                None => continue,
            },
        }
    }
    Params(params_map)
}

//params Get kernel cmdline arguments
pub fn get() -> Params {
    let content = match fs::read_to_string("/proc/cmdline") {
        Ok(content) => content,
        Err(err) => {
            log::error!("failed to get cmdline: {}", err);
            return Params(HashMap::default());
        }
    };

    parse_params(content)
}

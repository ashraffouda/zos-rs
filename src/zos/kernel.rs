use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::fs;

fn parse_params(content: String) -> HashMap<String, Vec<String>> {
    let mut params: HashMap<String, Vec<String>> = HashMap::new();
    let cmdline: Vec<&str> = content.trim().split(" ").collect();
    for option in cmdline {
        let optionlist: Vec<&str> = option.split("=").collect();
        let key = optionlist[0].to_string();

        match params.entry(key) {
            Entry::Vacant(e) => {
                if optionlist.len() == 2 {
                    e.insert(vec![optionlist[1].to_string()]);
                } else {
                    e.insert(vec![]);
                }
            }
            Entry::Occupied(mut e) => {
                if optionlist.len() == 2 {
                    e.get_mut().push(optionlist[1].to_string());
                }
            }
        }
    }

    params
}

//GetParams Get kernel cmdline arguments
pub fn get_params() -> HashMap<String, Vec<String>> {
    let content = match fs::read_to_string("/proc/cmdline") {
        Ok(content) => content,
        Err(err) => {
            println!("error: {}", err);
            return HashMap::new();
        }
    };
    parse_params(content)
}

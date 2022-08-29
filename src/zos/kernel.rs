use std::collections::hash_map::Entry;
use std::collections::HashMap;

use std::fs;

// struct Kernel(HashMap<String, Option<Vec<String>>>);

// impl Kernel {
//     pub fn exists<S: AsRef<str>>(&self, s: S) -> bool {
//         self.0.get(s.as_ref()).is_some()
//     }

//     // values will return value assigned to flag for example "key=1 key=2" will return Some([1, 2])
//     // if key exists but has no values, will return None. you can check if key exist with exists method
//     pub fn values<S: AsRef<str>>(&self, k:S) -> Option<&Vec<String>> {
//         match self.0.get(k.as_ref()) {
//             None => None,
//             Some(v) => match v {
//                 None => None,
//                 Some(v) => Some(v)
//             }
//         }
//     }
// }

fn parse_params(content: String) -> HashMap<String, Vec<String>> {
    let mut params: HashMap<String, Option<Vec<String>>> = HashMap::new();
    for option in content.trim().split(" ") {
        let optionlist: Vec<&str> = option.split("=").collect();
        let parts = option.split("=").into_iter();
        // use this to make sure element exists
        //let key = match parts.next();
        // key=1=2
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
pub fn params() -> HashMap<String, Vec<String>> {
    let content = match fs::read_to_string("/proc/cmdline") {
        Ok(content) => content,
        Err(err) => {
            log::error!("failed to get cmdline: {}", err);
            return HashMap::default();
        }
    };

    parse_params(content)
}

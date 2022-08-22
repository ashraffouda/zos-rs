use std::path::Path;

pub const FLAGS_DIR: &str = "/tmp/flags";
// LimitedCache represent the flag cache couldn't mount on ssd or hdd
pub const LIMITED_CACHE: &str = "limited-cache";

// CheckFlag checks the status of a flag based on a key
pub fn check_flag(key: String) -> bool {
    return Path::new(FLAGS_DIR).join(key).exists();
}

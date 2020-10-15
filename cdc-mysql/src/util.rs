//!
//! # Utility file
//!
use std::path::{Path, PathBuf};

pub fn expand_tilde<P: AsRef<Path>>(path_user_input: P) -> Option<PathBuf> {
    let p = path_user_input.as_ref();

    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }

    if p == Path::new("~") {
        return dirs::home_dir();
    }

    dirs::home_dir().map(|mut h| {
        if h == Path::new("/") {
            p.strip_prefix("~").unwrap().to_path_buf()
        } else {
            h.push(p.strip_prefix("~/").unwrap());
            h
        }
    })
}

#[cfg(test)]
pub mod test {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_expand_tilde() {
        let test_file = PathBuf::from("~/data/cdc-consumer/mysql80.offset");

        let expanded = expand_tilde(&test_file);
        assert!(expanded.is_some());

        let expected = PathBuf::from(format!(
            "{}{}",
            std::env::var("HOME").unwrap(),
            "/data/cdc-consumer/mysql80.offset"
        ));
        assert_eq!(expanded.unwrap(), expected);
    }
}

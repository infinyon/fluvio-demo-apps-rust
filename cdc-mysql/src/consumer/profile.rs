//!
//! # Profile file
//!
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use crate::util::expand_tilde;

const DEFAULT_TOPIC: &str = "rust-mysql-cdc";

pub struct Config {
    profile: Profile,
}

impl Config {
    /// try to load from default locations
    pub fn load(path: &Path) -> Result<Self, Error> {
        Self::from_file(path)
    }

    /// read from file
    fn from_file<T: AsRef<Path>>(path: T) -> Result<Self, Error> {
        let path_ref = path.as_ref();

        let file_str: String = read_to_string(path_ref)
            .map_err(|err| Error::new(ErrorKind::NotFound, format!("{}", err)))?;
        let mut profile: Profile = toml::from_str(&file_str)
            .map_err(|err| Error::new(ErrorKind::InvalidData, format!("{}", err)))?;

        if let Some(filter) = profile.filters.as_mut() {
            filter.normalize();
        }

        if let Some(base_path) = expand_tilde(&profile.data.base_path) {
            profile.data.base_path = base_path;
            profile.data.last_offset_file =
                profile.data.base_path.join(profile.data.last_offset_file);
        }

        Ok(Self { profile })
    }

    /// retrieve profile
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    data: Data,
    database: Database,
    filters: Option<Filters>,
    fluvio: Option<Fluvio>,
}
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct Data {
    base_path: PathBuf,
    last_offset_file: PathBuf,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct Database {
    ip_or_host: String,
    port: Option<u16>,
    user: String,
    password: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Filters {
    Include { include_dbs: Vec<String> },
    Exclude { exclude_dbs: Vec<String> },
}

impl Filters {
    fn normalize(&mut self) {
        match self {
            Self::Include { include_dbs } => {
                for name in include_dbs {
                    name.make_ascii_lowercase();
                }
            }
            Self::Exclude { exclude_dbs } => {
                for name in exclude_dbs {
                    name.make_ascii_lowercase()
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Fluvio {
    topic: String,
}

impl Profile {
    pub fn last_offset_file(&self) -> &PathBuf {
        &self.data.last_offset_file
    }

    pub fn ip_or_host(&self) -> Option<String> {
        Some(self.database.ip_or_host.clone())
    }

    pub fn port(&self) -> u16 {
        self.database.port.unwrap_or(3306)
    }

    pub fn user(&self) -> Option<String> {
        Some(self.database.user.clone())
    }

    pub fn password(&self) -> Option<String> {
        self.database.password.clone()
    }

    pub fn filters(&self) -> Option<Filters> {
        self.filters.clone()
    }

    pub fn topic(&self) -> String {
        if let Some(fluvio) = &self.fluvio {
            fluvio.topic.clone()
        } else {
            DEFAULT_TOPIC.to_owned()
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use std::path::PathBuf;

    const TEST_PATH: &str = "test_files";
    const PROFILE_FULL: &str = "consumer_profile_full.toml";
    const PROFILE_MIN: &str = "consumer_profile_min.toml";

    fn get_base_dir() -> PathBuf {
        let program_dir = std::env::current_dir().unwrap();
        program_dir.join(TEST_PATH)
    }

    #[test]
    fn test_full_profile() {
        let base_path = expand_tilde(&PathBuf::from("~/data")).unwrap();
        let last_offset_file = PathBuf::from("consumer.offset");
        let profile_path = get_base_dir().join(PROFILE_FULL);
        let profile_file = Config::load(&profile_path);

        if let Err(err) = &profile_file {
            println!("{:?}", err);
        };

        assert!(profile_file.is_ok());
        let expected = Profile {
            data: Data {
                base_path: base_path.clone(),
                last_offset_file: base_path.join(last_offset_file.clone()),
            },
            database: Database {
                ip_or_host: "localhost".to_owned(),
                port: Some(3306),
                user: "root".to_owned(),
                password: Some("root".to_owned()),
            },
            filters: Some(Filters::Exclude {
                exclude_dbs: vec!["mysql".to_owned(), "sys".to_owned()],
            }),
            fluvio: Some(Fluvio {
                topic: "rust-mysql-cdc".to_owned(),
            }),
        };

        let profile = profile_file.as_ref().unwrap().profile();
        assert_eq!(profile, &expected);
        assert_eq!(
            profile.last_offset_file(),
            &base_path.join(last_offset_file)
        );
        assert_eq!(profile.ip_or_host(), Some("localhost".to_owned()));
        assert_eq!(profile.port(), 3306);
        assert_eq!(profile.user(), Some("root".to_owned()));
        assert_eq!(profile.password(), Some("root".to_owned()));
        assert_eq!(profile.topic(), "rust-mysql-cdc".to_owned());
    }

    #[test]
    fn test_min_profile() {
        let base_path = PathBuf::from("/tmp/data");
        let last_offset_file = PathBuf::from("consumer2.offset");
        let profile_path = get_base_dir().join(PROFILE_MIN);
        let profile_file = Config::load(&profile_path);

        if let Err(err) = &profile_file {
            println!("{:?}", err);
        };

        assert!(profile_file.is_ok());
        let expected = Profile {
            data: Data {
                base_path: base_path.clone(),
                last_offset_file: base_path.join(last_offset_file.clone()),
            },
            database: Database {
                ip_or_host: "localhost".to_owned(),
                user: "root".to_owned(),
                port: None,
                password: None,
            },
            filters: None,
            fluvio: None,
        };

        let profile = profile_file.as_ref().unwrap().profile();
        assert_eq!(profile, &expected);
        assert_eq!(
            profile.last_offset_file(),
            &base_path.join(last_offset_file)
        );
        assert_eq!(profile.ip_or_host(), Some("localhost".to_owned()));
        assert_eq!(profile.port(), 3306);
        assert_eq!(profile.user(), Some("root".to_owned()));
        assert_eq!(profile.password(), None);
        assert_eq!(profile.topic(), DEFAULT_TOPIC.to_owned());
    }
}

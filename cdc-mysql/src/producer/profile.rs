//!
//! # Profile file
//!
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

use crate::util::expand_tilde;

const DEFAULT_TOPIC: &str = "rust-mysql-cdc";
const DEFAULT_REPLICAS: i16 = 1;
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
            profile.data.binlog_index_file =
                profile.data.base_path.join(profile.data.binlog_index_file);
            profile.data.resume_offset_file =
                profile.data.base_path.join(profile.data.resume_offset_file);
            profile.data.local_store_file =
                profile.data.base_path.join(profile.data.local_store_file);
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
    pub mysql_resource_name: String,
    pub data: Data,
    pub filters: Option<Filters>,
    pub fluvio: Option<Fluvio>,
}
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct Data {
    pub base_path: PathBuf,
    pub binlog_index_file: PathBuf,
    pub resume_offset_file: PathBuf,
    pub local_store_file: PathBuf,
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
            Filters::Include { include_dbs } => {
                for name in include_dbs {
                    name.make_ascii_lowercase();
                }
            }
            Filters::Exclude { exclude_dbs } => {
                for name in exclude_dbs {
                    name.make_ascii_lowercase();
                }
            }
        }
    }
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Fluvio {
    topic: String,
    replicas: Option<i16>,
}

impl Profile {
    #[allow(dead_code)] // used in unit
    pub fn set_binlog_index_file(&mut self, bn_file_idx: PathBuf) {
        self.data.binlog_index_file = bn_file_idx;
    }

    pub fn binlog_index_file(&self) -> &PathBuf {
        &self.data.binlog_index_file
    }

    pub fn resume_offset_file(&self) -> &Path {
        &self.data.resume_offset_file
    }

    pub fn local_store_file(&self) -> &Path {
        &self.data.local_store_file
    }

    pub fn mysql_resource_name(&self) -> &String {
        &self.mysql_resource_name
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

    pub fn replicas(&self) -> i16 {
        if let Some(fluvio) = &self.fluvio {
            if let Some(replicas) = fluvio.replicas {
                return replicas;
            }
        }
        DEFAULT_REPLICAS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PATH: &str = "test_files";
    const PROFILE_FULL: &str = "producer_profile_full.toml";
    const PROFILE_MIN: &str = "producer_profile_min.toml";

    fn get_base_dir() -> PathBuf {
        let program_dir = std::env::current_dir().unwrap();
        program_dir.join(TEST_PATH)
    }

    #[test]
    fn test_full_producer_profile() {
        let mysql_resource_name = ("mysql-docker-80").to_owned();
        let base_path = expand_tilde(&PathBuf::from("~/data")).unwrap();
        let binlog_index_file = PathBuf::from("binlog.index");
        let resume_offset_file = PathBuf::from("producer.offset");
        let local_store_file = PathBuf::from("producer.store");
        let profile_path = get_base_dir().join(PROFILE_FULL);
        let profile_file = Config::load(&profile_path);

        if let Err(err) = &profile_file {
            println!("{:?}", err);
        };

        assert!(profile_file.is_ok());
        let expected = Profile {
            mysql_resource_name: mysql_resource_name.clone(),
            data: Data {
                base_path: base_path.clone(),
                binlog_index_file: base_path.join(binlog_index_file.clone()),
                resume_offset_file: base_path.join(resume_offset_file.clone()),
                local_store_file: base_path.join(local_store_file.clone()),
            },
            filters: Some(Filters::Include {
                include_dbs: vec!["flvtest".to_owned()],
            }),
            fluvio: Some(Fluvio {
                topic: "rust-mysql-cdc".to_owned(),
                replicas: Some(2),
            }),
        };

        let profile = profile_file.as_ref().unwrap().profile();
        assert_eq!(profile, &expected);
        assert_eq!(profile.mysql_resource_name(), &mysql_resource_name);
        assert_eq!(
            profile.binlog_index_file(),
            &base_path.join(binlog_index_file)
        );
        assert_eq!(
            profile.resume_offset_file(),
            &base_path.join(resume_offset_file)
        );
        assert_eq!(
            profile.local_store_file(),
            &base_path.join(local_store_file)
        );
        assert_eq!(profile.topic(), "rust-mysql-cdc".to_owned());
        assert_eq!(profile.replicas(), 2);
    }

    #[test]
    fn test_min_producer_profile() {
        let mysql_resource_name = ("mysql-local").to_owned();
        let base_path = expand_tilde(&PathBuf::from("~/mysql-cdc/producer")).unwrap();
        let binlog_index_file = PathBuf::from("binlog.index");
        let resume_offset_file = PathBuf::from("producer.offset");
        let local_store_file = PathBuf::from("producer.store");
        let profile_path = get_base_dir().join(PROFILE_MIN);
        let profile_file = Config::load(&profile_path);

        if let Err(err) = &profile_file {
            println!("{:?}", err);
        };

        assert!(profile_file.is_ok());
        let expected = Profile {
            mysql_resource_name: mysql_resource_name.clone(),
            data: Data {
                base_path: base_path.clone(),
                binlog_index_file: base_path.join(binlog_index_file.clone()),
                resume_offset_file: base_path.join(resume_offset_file.clone()),
                local_store_file: base_path.join(local_store_file.clone()),
            },
            filters: None,
            fluvio: None,
        };

        let profile = profile_file.as_ref().unwrap().profile();
        assert_eq!(profile, &expected);
        assert_eq!(profile.mysql_resource_name(), &mysql_resource_name);
        assert_eq!(
            profile.binlog_index_file(),
            &base_path.join(binlog_index_file)
        );
        assert_eq!(
            profile.resume_offset_file(),
            &base_path.join(resume_offset_file)
        );
        assert_eq!(
            profile.local_store_file(),
            &base_path.join(local_store_file)
        );
        assert_eq!(profile.topic(), "rust-mysql-cdc".to_owned());
        assert_eq!(profile.replicas(), 1);
    }

    #[test]
    fn test_normalize_filter() {
        let mut filter = Filters::Include {
            include_dbs: vec![
                "flvDb".to_string(),
                "fluviodatabase".to_string(),
                "FLUVIO_DB".to_string(),
            ],
        };
        filter.normalize();
        match filter {
            Filters::Exclude { .. } => panic!("wrong variant"),
            Filters::Include { include_dbs } => {
                assert_eq!(&include_dbs[0], "flvdb");
                assert_eq!(&include_dbs[1], "fluviodatabase");
                assert_eq!(&include_dbs[2], "fluvio_db");
            }
        }
    }
}

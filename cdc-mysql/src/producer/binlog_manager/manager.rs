/// BinLogManager
///
/// File manager listens for changes in the bin-log directory and notifies receiver.
///
use crossbeam_channel::Sender;
use std::cmp;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tracing::{debug, error, instrument, trace};

use super::parse_records_from_file;
use super::IndexFile;
use super::LocalStore;
use super::Resume;
use super::{get_file_id, BinLogFile};
use crate::error::CdcError;
use crate::producer::{Filters, Profile};
use crate::util::expand_tilde;

const DELAY_MIN_MILIS: u64 = 500;

#[derive(Debug)]
pub struct BinLogManager {
    sender: Sender<String>,
    base_dir: PathBuf,
    filters: Option<Filters>,

    index_file: IndexFile,
    current_file: Option<BinLogFile>,

    local_store: LocalStore,
    urn: String,
}

impl BinLogManager {
    pub fn new(profile: &Profile, sender: Sender<String>) -> Result<Self, Error> {
        let (base_dir, bn_index_file) = get_base_path_and_file_tuple(profile.binlog_index_file());

        Ok(Self {
            sender,
            base_dir: base_dir.clone(),
            filters: profile.filters(),
            index_file: IndexFile::new(&base_dir, bn_index_file)?,
            current_file: None,
            local_store: LocalStore::new(profile.local_store_file())?,
            urn: profile.mysql_resource_name().clone(),
        })
    }

    #[instrument(skip(self, resume, frequency_mili))]
    pub fn run(mut self, resume: Resume, frequency_mili: Option<u64>) {
        let mut init = true;

        thread::spawn(move || loop {
            if let Err(err) = self.inner_run(&resume, init) {
                error!("Error: {}", err);
            }
            init = false;

            // sleep a bit
            let sleep = cmp::max(frequency_mili.unwrap_or(0), DELAY_MIN_MILIS);
            thread::sleep(Duration::from_millis(sleep));
        });
    }

    #[instrument(skip(self, resume, init))]
    fn inner_run(&mut self, resume: &Resume, init: bool) -> Result<(), CdcError> {
        if init {
            self.set_current_file(resume)?;
            self.send_current_file_records()?;

            self.send_all_files_records()?;
        } else {
            if self.has_current_file_changed() {
                self.send_current_file_records()?;
            }

            if self.has_index_file_changed() {
                self.send_all_files_records()?;
            }
        }

        Ok(())
    }

    fn set_current_file(&mut self, resume: &Resume) -> Result<(), Error> {
        let (file, offset) = match &resume.binfile {
            Some(binfile) => (binfile.file_name.clone(), binfile.offset),
            None => (self.get_first_index_file()?, None),
        };

        self.current_file = Some(BinLogFile::new(&self.base_dir, &file, offset)?);
        Ok(())
    }

    fn get_first_index_file(&self) -> Result<String, Error> {
        let all_files = self.index_file.get_bin_log_files()?;
        if let Some(first_file) = all_files.first() {
            Ok(first_file.to_owned())
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                "Internal Error: Index file is empty",
            ))
        }
    }

    fn has_current_file_changed(&mut self) -> bool {
        if let Some(current_file) = self.current_file.as_mut() {
            if let Ok(has_changed) = current_file.has_changed() {
                return has_changed;
            }
        }
        false
    }

    fn has_index_file_changed(&mut self) -> bool {
        if let Ok(has_changed) = self.index_file.has_changed() {
            return has_changed;
        }
        false
    }

    #[instrument(skip(self))]
    fn send_current_file_records(&mut self) -> Result<(), CdcError> {
        let current_file = self.current_file.as_ref().unwrap();
        trace!("Sending file: {:?}", current_file);

        let new_offset = parse_records_from_file(
            &self.sender,
            &current_file.path_to_string(),
            current_file.file_name(),
            current_file.offset(),
            self.filters.as_ref(),
            &mut self.local_store,
            &self.urn,
        )?;
        self.current_file.as_mut().unwrap().set_offset(new_offset);

        Ok(())
    }

    #[instrument(skip(self))]
    fn send_all_files_records(&mut self) -> Result<(), CdcError> {
        let files = self.get_files_from_bn_index()?;

        for file in files {
            // update current_file
            let next_bn_file = BinLogFile::new(&self.base_dir, &file, None)?;
            debug!("Next bin file: {:?}", &next_bn_file);
            self.current_file = Some(next_bn_file);

            // send_records
            let current_file = self.current_file.as_ref().unwrap();
            let new_offset = parse_records_from_file(
                &self.sender,
                &current_file.path_to_string(),
                current_file.file_name(),
                current_file.offset(),
                self.filters.as_ref(),
                &mut self.local_store,
                &self.urn,
            )?;

            self.current_file.as_mut().unwrap().set_offset(new_offset);
        }

        Ok(())
    }

    fn get_files_from_bn_index(&self) -> Result<Vec<String>, Error> {
        let mut files = vec![];
        let all_files = self.index_file.get_bin_log_files()?;

        if let Some(current_file) = self.current_file.as_ref() {
            for file in &all_files {
                let file_id = get_file_id(&self.base_dir.join(Path::new(file).to_path_buf()));
                if file_id > current_file.file_id() {
                    files.push(file.clone());
                }
            }
        }

        Ok(files)
    }
}

fn get_base_path_and_file_tuple(bn_file_path: &Path) -> (PathBuf, String) {
    let mut base_dir = bn_file_path.parent().unwrap().to_path_buf();

    // expand tilde if used
    if let Some(home_path) = expand_tilde(&base_dir) {
        base_dir = home_path;
    }

    let file = bn_file_path
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    (base_dir, file)
}

#[cfg(test)]
mod test {
    use crossbeam_channel::bounded;
    use std::fs;
    use std::path::PathBuf;

    use crate::messages::BnFile;
    use crate::producer::{Data, Profile};

    use super::BinLogFile;
    use super::BinLogManager;
    use super::Resume;

    const TEST_PATH: &str = "test_files";
    const BL_INDEX: &str = "binlog.index";
    const BL_FILE1: &str = "binlog.000001";
    const BL_FILE2: &str = "binlog.000002";
    const LOCAL_STORE: &str = "local.store";
    const RESUME_OFFSET: &str = "resume.offset";

    fn get_base_dir() -> PathBuf {
        let program_dir = std::env::current_dir().unwrap();
        program_dir.join(TEST_PATH)
    }

    fn build_profile() -> Profile {
        let base_path = get_base_dir();
        Profile {
            mysql_resource_name: "mysql_resource".to_owned(),
            data: Data {
                base_path: base_path.clone(),
                binlog_index_file: base_path.join(BL_INDEX),
                resume_offset_file: base_path.join(LOCAL_STORE),
                local_store_file: base_path.join(RESUME_OFFSET),
            },
            filters: None,
            fluvio: None,
        }
    }

    fn clean_up(profile: &Profile) {
        let _ = fs::remove_file(profile.resume_offset_file());
        let _ = fs::remove_file(profile.local_store_file());
    }

    #[test]
    fn test_set_current_file() {
        let base_dir = get_base_dir();
        let profile = build_profile();
        let (sender, _) = bounded::<String>(100);
        let fm_res = BinLogManager::new(&profile, sender);

        assert!(fm_res.is_ok());
        let mut fm = fm_res.unwrap();

        // test - resume: file1, offset: None,
        let bn_file = BnFile::new(BL_FILE1.to_owned(), None);
        let resume_path = get_base_dir().join("resume");
        let resume = Resume::new(&resume_path, bn_file).unwrap();
        let set_current_res = fm.set_current_file(&resume);
        assert!(set_current_res.is_ok());

        let exp_file = BL_FILE1.to_owned();
        let exp_offset = None;
        let bn_file_res = BinLogFile::new(&base_dir, &exp_file, exp_offset);
        assert!(bn_file_res.is_ok());
        assert_eq!(
            fm.current_file.as_ref().unwrap(),
            bn_file_res.as_ref().unwrap()
        );

        // test - resume: file2, offset: 2000,
        let bn_file = BnFile::new(BL_FILE2.to_owned(), Some(200));
        let resume = Resume::new(&resume_path, bn_file).unwrap();
        let set_current_res = fm.set_current_file(&resume);
        assert!(set_current_res.is_ok());

        let exp_file = BL_FILE2.to_owned();
        let exp_offset = Some(200);
        let bn_file_res = BinLogFile::new(&base_dir, &exp_file, exp_offset);
        assert!(bn_file_res.is_ok());
        assert_eq!(
            fm.current_file.as_ref().unwrap(),
            bn_file_res.as_ref().unwrap()
        );

        // test - resume: invalid, offset: Some(1000),
        let resume = Resume::empty(&resume_path).unwrap();
        let set_current_res = fm.set_current_file(&resume);
        assert!(set_current_res.is_ok());

        let exp_file = BL_FILE1.to_owned();
        let exp_offset = None;
        let bn_file_res = BinLogFile::new(&base_dir, &exp_file, exp_offset);
        assert!(bn_file_res.is_ok());
        assert_eq!(
            fm.current_file.as_ref().unwrap(),
            bn_file_res.as_ref().unwrap()
        );

        clean_up(&profile);
    }
}

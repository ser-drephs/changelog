#[allow(unused_imports)]
use crate::Repository;
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

const DIRECTORY: &str = ".changelog";
const CONFIG: &str = "changelog.config";

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    pub source: ChangelogSource,
    pub last_generation: SystemTime,
    pub date_format: String,
    pub diff_format: String,
    pub commit_detail_page_format: String,
}

#[derive(Serialize, Deserialize, Debug, Display)]
pub enum ChangelogSource {
    File,
    Tag,
}

impl Configuration {
    pub fn new() -> Self {
        let config_file = get_config_file();
        if !config_file.exists() {
            prepare_config(&config_file, false);
        }
        let mut config = read_config(&config_file);

        if let Err(_) = &config {
            warn!("no config file found");
            debug!("creating new one");
            prepare_config(&config_file, true);
            config = read_config(&config_file);
        }
        config.expect("config file cannot be opened")
    }

    pub fn save(&self) {
        let config_file = get_config_file();
        let settings_str = serde_json::to_string(&self).unwrap();
        info!("Settings: {}", settings_str);
        let mut file = OpenOptions::new()
            .write(true)
            .open(&config_file)
            .expect("could not open config file for writing");
        file.write_all(&settings_str.as_bytes())
            .expect("could not save settings");
        info!("configuration file saved");
    }
}

fn get_config_file() -> PathBuf {
    let repo = Repository::new();
    let config_file = Path::new(repo.get_location()).join(DIRECTORY).join(CONFIG);
    config_file
}

fn read_config(config_file: &Path) -> Result<Configuration, serde_json::Error> {
    let file_content_raw =
        String::from_utf8_lossy(&fs::read(&config_file).expect("could not read config file"))
            .parse::<String>()
            .unwrap();
    trace!("Raw Config: {}", &file_content_raw);
    let config = serde_json::from_str::<Configuration>(&file_content_raw);
    config
}

fn prepare_config(config_file: &Path, overwrite: bool) {
    let directory = config_file
        .parent()
        .expect("no parent found for config file");
    if !directory.exists() {
        fs::create_dir_all(&directory).expect("creating config directory");
        debug!("changelog config directory created");
    }
    if !config_file.exists() || overwrite {
        let settings = Configuration {
            source: ChangelogSource::File,
            last_generation: SystemTime::UNIX_EPOCH,
            date_format: "%Y-%m-%d".to_string(),
            diff_format:
                "{{repositoryUri}}/branchCompare?baseVersion=GC{{base}}&targetVersion=GC{{latest}}&_a=files"
                    .to_string(),
            commit_detail_page_format:"{{repositoryUri}}/commit/{{commit}}".to_string()
        };
        fs::File::create(&config_file).expect("could not create config file");
        settings.save();
        debug!("new config file created");
    }
}

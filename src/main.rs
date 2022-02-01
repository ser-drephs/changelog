#[macro_use]
extern crate enum_display_derive;
extern crate handlebars;
extern crate lazy_static;
extern crate regex;
extern crate serde;
extern crate serde_json;

use crate::configuration::Configuration;
use crate::repository::Repository;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use simple_logger::SimpleLogger;
use std::path::Path;

mod changelog;
mod configuration;
mod repository;

fn main() {
    SimpleLogger::new().init().unwrap();
    let configuration = Configuration::new();
    debug!("Source Configuration: {}", configuration.source);
    build_changelog();
}

fn build_changelog() {
    let repo = Repository::new();
    debug!("{} is a repository", repo.get_location().display());
    info!("building changelog...");
    let commits = repo.get_commits();
    changelog::generate_changelog(&repo, &commits)
}

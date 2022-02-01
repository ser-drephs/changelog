use crate::Path;
use git2::RepositoryOpenFlags;
use lazy_static::lazy_static;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use regex::Regex;
use std::fmt;
use std::fmt::Display;

const REPOSITORY: &str = "target/conventionalcommits.org";

#[derive(Display, Debug, PartialEq)]
pub enum ConventionalType {
    Feat,
    Fix,
    Breaking,
    Other,
}

pub struct Repository {
    repository: git2::Repository,
}

impl Repository {
    pub fn new() -> Self {
        let working_repository = Repository::get_repository_instance(REPOSITORY);
        if !&working_repository.is_ok() {
            error!("invalid path\nplease run changelog inside of a git repository");
        }
        Self {
            repository: working_repository.expect("not a repository"),
        }
    }
    pub fn get_location(&self) -> &Path {
        self.repository.path().parent().unwrap()
    }
    pub fn get_remote_uri(&self, remote: &str) -> String {
        let remotes = self.repository.find_remote(remote);
        if remotes.is_err() {
            warn!("remote {} does not exist", remote);
        }

        if remotes.is_ok() {
            remotes.unwrap().url().unwrap().to_string()
        } else {
            // in case there is no remote defined, at least any uri is returned
            "http://git.localhost".to_string()
        }
    }
    pub fn get_commits(&self) -> Vec<Commit> {
        let mut revwalk = self.repository.revwalk().expect("cannot walk the rev");
        revwalk.push_head().expect("head cannot be pushed");
        let mut commits = Vec::<Commit>::new();
        for rev in revwalk {
            let commit = self
                .repository
                .find_commit(rev.expect("no rev found"))
                .expect("commit not found");
            if !Commit::is_merge_commit(&commit) {
                let commit = Commit::new(
                    commit.id().to_string(),
                    String::from(commit.summary().expect("summary invalid")),
                    String::from(commit.message().expect("message invalid")),
                );
                trace!("Custom Commit Object: {}", commit.to_string());
                commits.push(commit);
            }
        }
        commits
    }
    fn get_repository_instance(location: &str) -> Result<git2::Repository, git2::Error> {
        git2::Repository::open_ext(location, RepositoryOpenFlags::empty(), Vec::<&Path>::new())
    }
}

#[derive(Debug)]
pub struct Commit {
    pub id: String,
    pub summary: String,
    pub conventional_type: ConventionalType,
}

impl Commit {
    pub fn new(id: String, summary: String, message: String) -> Self {
        let conventional_type = get_commit_type(&summary, &message);
        // remove conventional tag from summary text
        let summary_split = &summary.split(":").collect::<Vec<&str>>();
        let summary_text;
        if &summary_split.len() > &1_usize {
            summary_text = &summary_split[1]
        } else {
            summary_text = &summary_split[0]
        }
        Self {
            id: id,
            summary: summary_text.trim().to_string(),
            conventional_type: conventional_type,
        }
    }
    pub fn is_merge_commit(commit: &git2::Commit) -> bool {
        commit.parent_count() > 1
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "id={},summary={},type={}",
            self.id, self.summary, self.conventional_type
        )
    }
}

lazy_static! {
    static ref BREAKING_CHANGE_SUM_RE: Regex = Regex::new(r"^.+!:.*$").unwrap();
    static ref FEAT_RE: Regex = Regex::new(r"^feat.*:.*$").unwrap();
    static ref FIX_RE: Regex = Regex::new(r"^fix.*:.*$").unwrap();
    static ref BREAKING_CHANGE_MSG_RE: Regex = Regex::new(r"(?mi)^breaking change:.*").unwrap();
}

fn get_commit_type(summary: &str, message: &str) -> ConventionalType {
    let mut commit_type = ConventionalType::Other;

    if BREAKING_CHANGE_SUM_RE.is_match(&summary) {
        commit_type = ConventionalType::Breaking
    } else if FEAT_RE.is_match(&summary) {
        commit_type = ConventionalType::Feat
    } else if FIX_RE.is_match(&summary) {
        commit_type = ConventionalType::Fix
    }
    if BREAKING_CHANGE_MSG_RE.is_match(&message) {
        commit_type = ConventionalType::Breaking;
    }
    commit_type
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_map {
        ($name:ident, $($summary:expr, $message:expr, $expected:expr),+) => {
            #[test]
            fn $name() {
                let result = get_commit_type($($summary),+, $($message),+);
                assert_eq!(result, $($expected),+);
            }
        };
    }

    test_map!(
        test_map_commit_type_feat,
        "feat: normal",
        "",
        ConventionalType::Feat
    );
    test_map!(
        test_map_commit_type_feat_scope,
        "feat(scope): normal",
        "",
        ConventionalType::Feat
    );
    test_map!(
        test_map_commit_type_fix,
        "fix: normal",
        "",
        ConventionalType::Fix
    );
    test_map!(
        test_map_commit_type_fix_scope,
        "fix(scope): normal",
        "",
        ConventionalType::Fix
    );
    test_map!(
        test_map_commit_type_fix_bc,
        "fix!: normal",
        "",
        ConventionalType::Breaking
    );
    test_map!(
        test_map_commit_type_fix_bc_scope,
        "fix(scope)!: normal",
        "",
        ConventionalType::Breaking
    );
    test_map!(
        test_map_commit_type_feat_bc,
        "feat!: normal",
        "",
        ConventionalType::Breaking
    );
    test_map!(
        test_map_commit_type_feat_bc_scope,
        "feat(scope)!: normal",
        "",
        ConventionalType::Breaking
    );
    test_map!(
        test_map_commit_type_message_bc,
        "feat: normal",
        "Loren\nBREAKING CHANGE: bla\nsome other text",
        ConventionalType::Breaking
    );
    test_map!(
        test_map_commit_type_message_bc_as_end,
        "chore: normal",
        "Loren\nBREAKING CHANGE: bla",
        ConventionalType::Breaking
    );
}

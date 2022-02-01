use crate::repository::Commit;
use crate::repository::ConventionalType;
use crate::Configuration;
use crate::Repository;
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
#[allow(unused_imports)]
use log::{debug, info, trace, warn};
use markdown_gen::markdown::Heading;
use markdown_gen::markdown::Link;
use markdown_gen::markdown::List;
use markdown_gen::markdown::Markdown;
use markdown_gen::markdown::Paragraph;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

// macro_rules! create_list {
//     ($md:expr, $a:expr) => {
//         for commit in $a.into_iter() {
//             $md.write(List::new(false).title(map_commit_to_list(commit)))
//                 .unwrap();
//         }
//     };
// }

pub fn generate_changelog(repository: &Repository, commits: &Vec<Commit>) {
    // initialize objects
    let configuration = Configuration::new();
    let reg = Handlebars::new();
    // create draft file
    let changelog = get_draft_changelog_file(&repository);
    let draft_file = fs::File::create(&changelog).expect("could not create changelog");
    info!("creating new changelog file");
    // select types of commits
    let bc_commits: Vec<&Commit> = commits
        .iter()
        .filter(|voc| voc.conventional_type == ConventionalType::Breaking)
        .collect();
    let feat_commits: Vec<&Commit> = commits
        .iter()
        .filter(|voc| voc.conventional_type == ConventionalType::Feat)
        .collect();
    let fix_commits: Vec<&Commit> = commits
        .iter()
        .filter(|voc| voc.conventional_type == ConventionalType::Fix)
        .collect();
    // get todays date
    let now: DateTime<Utc> = Utc::now();
    let date = now.format(&configuration.date_format).to_string();
    // get remote repository link
    let repository_link = repository.get_remote_uri("origin");
    // get diff link format from configuration
    let first_commit = &commits[commits.len() - 1].id.to_string();
    let latest_commit = &commits[0].id.to_string();
    let diff_uri = reg.render_template(
        &configuration.diff_format,
        &json!({"repositoryUri": repository_link, "base": first_commit, "latest": latest_commit}),
    ).expect("configured diff format doesn't have the correct variables defined - for more information see readme");
    // create changelog draft markdown file
    let mut md = Markdown::new(draft_file);
    // write first heading
    md.write(
        Heading::new(2)
            .append(Link::new(&diff_uri).append("Draft Version"))
            .append::<&str>(
                &reg.render_template(" ({{date}})", &json!({ "date": date }))
                    .unwrap(),
            ),
    )
    .unwrap();
    if bc_commits.len() > 0 {
        md.write(Heading::new(3).append("BREAKING CHANGES"))
            .unwrap();
        // create_list!(md, bc_commits);
    }
    md.write(Heading::new(3).append("Features")).unwrap();
    // md.write(Paragraph::new()).unwrap();
    md.write(List::new(false).item("sample")).unwrap();
    for commit in feat_commits.into_iter() {
        // link to commit uri
        let commit_uri = &reg
            .render_template(
                &configuration.commit_detail_page_format,
                &json!({ "repositoryUri": repository_link, "commit": &commit.id }),
            )
            .unwrap();
        let commit_id_repr = &commit.id[..7];
        let commit_link = Link::new(commit_uri).append::<&str>(commit_id_repr);
        // text for item
        let commit_summary = &reg
            .render_template("{{subject}} (", &json!({ "subject": &commit.summary}))
            .unwrap();
        let entry = Paragraph::new()
            .append::<&str>(&commit_summary)
            .append(commit_link)
            .append(")");
        //write list item
        md.write(List::new(false).item(entry)).unwrap();
    }
    md.write(Paragraph::new()).unwrap();
    md.write(Heading::new(3).append("Bug Fixes")).unwrap();
    // create_list!(md, fix_commits);

    // md.write("Heading".heading(1)).unwrap();
    // md.write("Subheading".italic().heading(2)).unwrap();

    // md.write("bold".bold()).unwrap();

    // md.write("first paragraph").unwrap();
    // md.write(
    //     "Links: "
    //         .paragraph()
    //         .append("Rust".bold().link_to("https://rust-lang.org"))
    //         .append(", ")
    //         .append("Google".italic().link_to("https://google.com")),
    // )
    // .unwrap();
}

fn get_changelog_file(repository: &Repository) -> PathBuf {
    let changelog = Path::new(repository.get_location()).join("changelog.md");
    changelog
}

fn get_draft_changelog_file(repository: &Repository) -> PathBuf {
    let changelog = Path::new(repository.get_location()).join("changelog_DRAFT.md");
    debug!(
        "changelog will be created at {}",
        changelog.to_str().unwrap()
    );
    changelog
}

// todo changelog_DRAFT prepend to changelog

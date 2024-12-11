use git2::{Repository, Signature};
use reqwest::blocking::Client;
use serde_json::json;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{args:?}");

    if args.len() < 4 {
        return println!("Less arguments than expeted passed");
    }

    let path = &args[1];
    let commit_message = &args[2];
    let repo_name = &args[3];
    let my_github_name = String::from("devdannetworks");
    let my_github_email = String::from("officialdevduncan@gmail.com");
    let git_hub_token = env::var("GITHUB_TOKEN").expect("Failed to fetch git hub token");
    println!("{git_hub_token}");

    //Add all necessary files to repo
    let repo = Repository::init(path).expect("Failed to initialize a  repo!");
    fs::write(
        Path::new(path).join("Readme.md"),
        "#This is my initial readmemd to be updated",
    )
    .expect("Failed to add a readme.md file");
    let mut index = repo.index().expect("Failed to index the repo");
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .expect("Failed to add files to index");
    index.write().expect("Failed to save/write files to index");

    //Commit changes to repo
    let object_id = index.write_tree().expect("Failed to write object tree");
    let tree = repo
        .find_tree(object_id)
        .expect("Failed to find the object tree");
    let signature =
        Signature::now(&my_github_name, &my_github_email).expect("Failed to verify signature");
    let head = repo.head().ok().and_then(|h| h.target());
    let parent_commit = head.and_then(|oid| repo.find_commit(oid).ok());
    let parents = parent_commit.iter().collect::<Vec<_>>();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &commit_message,
        &tree,
        &parents,
    )
    .expect("Failed to commit repo");
    println!("Repo committed successfully");

    // Create a new repo on git-hub
    let client = Client::new();
    let url = format!("https://api.github.com/user/repos");
    let response = client
        .post(&url)
        .bearer_auth(git_hub_token)
        .json(&json!({"name": repo_name}))
        .header("User-Agent", "my_rust_script")
        .send()
        .expect("Failed to get a response creating a github repo");
    if response.status().is_success() {
        println!("Successfully created a github repo");
    } else if response.status() == 422 {
        println!("Skipping this step, repo already exists....");
    } else {
        eprintln!(
            "Failure creating a github repo: {}",
            response.text().unwrap()
        );
    }

    //Add git remote and add changes
    let remote_url = format!("https://@github.com/{}/{}.git", my_github_name, repo_name);
    println!("{remote_url}");
    if repo.find_remote("origin").is_err() {
        repo.remote("origin", &remote_url)
            .expect("Failed to add remote origin");
        println!("Remote origin added successfully!")
    } else {
        println!("Remote origin already exists, continuing execution");
    }

    // git remote set-url origin https://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@github.com/devdannetworks/api_call.git
    let git_auth_url =
        "https://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@github.com/devdannetworks/api_call.git";
    let set_url = Command::new("git")
        .arg("remote")
        .arg("set-url")
        .arg("origin")
        .arg(git_auth_url)
        .output()
        .expect("Failed to execute git set-url command");

    if !set_url.status.success() {
        eprintln!(
            "Error setting remote url to github: {}",
            String::from_utf8_lossy(&set_url.stderr)
        );
    } else {
        println!("Successfully set the url folder: {} repo to github", path);
    }

    //git push -u origin main
    let push_command = Command::new("git")
        .arg("push")
        .arg("-u")
        .arg("origin")
        .arg("main")
        .output()
        .expect("Failed to execute git push command");

    if !push_command.status.success() {
        eprintln!(
            "Error pushing to github, {}",
            String::from_utf8_lossy(&push_command.stderr)
        );
    } else {
        println!("Successfully pushed folder: {} repo to github", path);
    }
}

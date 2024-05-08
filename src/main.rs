use git::{FetchOptions, Oid};
use git2 as git;
use octocrab::Octocrab;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use store::Store;
use tokio::time::sleep;

use config::Config;

mod config;
mod store;

const COMMENT_FAIL: &str = r#"Hello.

TSC has reported issues with this PR.
Please verify on your end to confirm and fix any reported problems.

----

<sub>Beep, boop; I am just a little bot trying to be helpful.</sub>
"#;

fn repo_dir() -> PathBuf {
    let mut path = env::current_dir().expect("Unable to open current working directory.");
    path.push("repo");
    path
}

fn fetch_options(config: &Config) -> git::FetchOptions {
    let git_credentials_callback =
        |_user: &str, _user_from_url: Option<&str>, _cred: git::CredentialType| {
            git2::Cred::userpass_plaintext(&config.token, &config.token)
        };

    let mut callbacks = git::RemoteCallbacks::new();
    callbacks.credentials(git_credentials_callback);

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    fetch_options
}

fn clone_repo(config: &Config, repo_dir: &Path) -> git::Repository {
    println!("Cloning repo...");

    git::build::RepoBuilder::new()
        .fetch_options(fetch_options(config))
        .clone(&config.repo_url, repo_dir)
        .expect("Unable to clone repository")
}

fn ensure_repo(config: &Config) -> git::Repository {
    let repo_dir = repo_dir();

    if repo_dir.exists() {
        println!("Repo already exists, fetching...");

        let repo = git::Repository::open(repo_dir).expect("Unable to open repository");
        repo.find_remote("origin")
            .expect("Repository does not have a remote named 'origin'")
            .fetch(&[] as &[&str], Some(&mut fetch_options(config)), None)
            .expect("Unable to fetch remote");
        repo
    } else {
        clone_repo(config, &repo_dir)
    }
}

fn checkout(repo: &git::Repository, sha: &str) {
    let oid = Oid::from_str(sha).expect("Invalid commit sha");
    repo.set_head_detached(oid).expect("Unable to update HEAD");
    repo.checkout_head(None).expect("Unable to checkout");
}

fn test_commit(repo: &git::Repository, sha: &str) -> bool {
    checkout(repo, sha);

    let mut command = Command::new("yarn");
    let output = command.current_dir(repo.path()).output().unwrap();

    // TODO: Run a configurable script instead.
    let (err, out) = unsafe {
        (
            String::from_utf8_unchecked(output.stderr),
            String::from_utf8_unchecked(output.stdout),
        )
    };

    println!("=== STDERR ===\n{}", err);
    println!("=== STDOUT ===\n{}", out);

    let mut command = Command::new("tsc");
    let output = command.current_dir(repo.path()).output().unwrap();

    let (err, out) = unsafe {
        (
            String::from_utf8_unchecked(output.stderr),
            String::from_utf8_unchecked(output.stdout),
        )
    };

    println!("=== STDERR ===\n{}", err);
    println!("=== STDOUT ===\n{}", out);

    output.status.success()
}

async fn test_prs(
    github: &Octocrab,
    repo: &git::Repository,
    config: &Config,
    store: &mut Store,
) -> octocrab::Result<()> {
    let pulls = github
        .pulls(&config.username, &config.repo_name)
        .list()
        .per_page(50)
        .send()
        .await?;

    let mut hashes = Vec::<String>::new();

    for pull in pulls {
        let label = &pull.head.label.unwrap();
        println!("Testing PR: {}", label);

        if store.is_commit_checked(&pull.head.sha) {
            println!("Previously tested, skipping.");
        } else if test_commit(repo, &pull.head.sha) {
            println!("PASSED.");
        } else {
            println!("FAILED.");
            github
                .issues(&config.username, &config.repo_name)
                .create_comment(1, COMMENT_FAIL)
                .await
                .unwrap_or_else(|_| panic!("Unable comment on PR #{}.", pull.number));
        }

        hashes.push(pull.head.sha);
    }

    store.save(hashes);

    Ok(())
}

#[tokio::main]
async fn main() -> octocrab::Result<()> {
    let config = Config::load();

    let github = Octocrab::builder()
        .personal_token(config.token.clone())
        .build()?;

    let mut store = Store::load();

    loop {
        let repo = ensure_repo(&config);

        test_prs(&github, &repo, &config, &mut store).await?;
        sleep(Duration::from_secs(60)).await;
    }
}

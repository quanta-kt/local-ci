use std::env;

pub struct Config {
    pub token: String,
    pub repo_url: String,
    pub username: String,
    pub repo_name: String,
}

impl Config {
    pub fn load() -> Self {
        Self {
            token: env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN environment variable is not set"),
            repo_url: env::var("REPO_URL").expect("REPO_URL environment variable is not set"),
            username: env::var("GITHUB_USERNAME")
                .expect("GITHUB_USERNAME environment variable is not set"),
            repo_name: env::var("GITHUB_REPO_NAME")
                .expect("GITHUB_REPO_NAME environment variable is not set"),
        }
    }
}

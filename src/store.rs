use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::Path,
};

pub struct Store {
    checked_commits: Vec<String>,
}

impl Store {
    const FILENAME: &str = "checked_commits.txt";

    pub fn load() -> Self {
        let content = if Path::new(Self::FILENAME).exists() {
            let mut file = OpenOptions::new()
                .read(true)
                .open(Self::FILENAME)
                .unwrap_or_else(|_| {
                    panic!("Unable to open file '{}' for reading.", Self::FILENAME)
                });

            let mut content = String::new();

            file.read_to_string(&mut content)
                .expect("Unable to read checked_commits.txt");

            Some(content)
        } else {
            None
        };

        let checked_commits = content
            .map(|it| it.lines().map(|it| it.to_owned()).collect())
            .unwrap_or_default();

        Self { checked_commits }
    }

    pub fn is_commit_checked(&self, sha: &str) -> bool {
        self.checked_commits.iter().any(|it| it == sha)
    }

    pub fn save(&mut self, hashes: Vec<String>) {
        self.checked_commits.clear();
        self.checked_commits.extend(hashes);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(Self::FILENAME)
            .unwrap_or_else(|_| panic!("Unable to open file '{}' for writing.", Self::FILENAME));

        for sha in self.checked_commits.iter() {
            file.write_all(sha.as_bytes()).unwrap();
            file.write_all(&[b'\n']).unwrap();
        }

        file.flush().unwrap();
    }
}

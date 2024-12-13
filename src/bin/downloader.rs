use std::error::Error;
use std::fs;
use serde_derive::Deserialize;

const CRON_FILENAME: &str = "repositories.toml";

#[derive(Deserialize, Debug, Clone)]
pub struct RepoList {
    pub repo: Vec<String>
}

impl RepoList {
    pub fn read() -> Self {
        let contents = fs::read_to_string(CRON_FILENAME).expect("Could not read file");
        toml::from_str(&contents).expect("Unable to load data")
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let list = RepoList::read();
    println!("{:#?}", list);
    Ok(())
}
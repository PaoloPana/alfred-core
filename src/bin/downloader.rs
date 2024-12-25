use std::error::Error;
use std::{fs, io};
use std::fs::{create_dir, exists, remove_dir_all, File};
use std::io::Cursor;
use std::process::exit;
use log::{error, info};
use reqwest::Client;
use reqwest::redirect::Policy;
use serde_derive::Deserialize;

const REPO_FILENAME: &str = "repositories.toml";
const TMP_DIR: &str = "/tmp/alfred";

#[derive(Deserialize, Debug, Clone)]
pub struct RepoList {
    pub repo: Vec<String>
}

impl RepoList {
    pub fn read() -> Self {
        let contents = fs::read_to_string(REPO_FILENAME).expect("Could not read file");
        toml::from_str(&contents).expect("Unable to load data")
    }
}

fn get_asset_url(repo_url: &str, version: &str, filename: &str) -> String {
    format!("{repo_url}/releases/download/{version}/{filename}")
}

fn get_archive_url(repo_url: &str, version: &str, filename: &str, arch: &str) -> String {
    let archive_filename = format!("{filename}_{arch}.tar.gz");
    get_asset_url(repo_url, version, archive_filename.as_str())
}

async fn download_file(url: &str, out: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let mut output_file = File::create(out)?;
    let mut content =  Cursor::new(response.bytes().await?);
    io::copy(&mut content, &mut output_file)?;
    Ok(())
}

async fn download_repo(module_name: &str, repo: &str, version: Option<&str>) -> Result<String, Box<dyn Error>> {
    // TODO: stop if already executed
    // download from repo
    let latest_version = get_latest_version(repo).await?;
    let version = version.unwrap_or(latest_version.as_str());
    info!("Version; {version}");
    let current_arch = std::env::consts::ARCH.to_string();
    let archive_url = get_archive_url(repo, version, module_name, current_arch.as_str());
    let output_dir = TMP_DIR;
    if exists(output_dir)? {
        remove_dir_all(output_dir)?;
    }
    create_dir(output_dir)?;
    let output_archive = format!("{output_dir}/{module_name}.tar.gz");
    download_file(archive_url.as_str(), output_archive.as_str()).await?;
    let alfred_dir = std::env::current_dir().expect("Error trying to retrieve the tmp directory").display().to_string();
    decompress_archive(output_archive.as_str(), alfred_dir.as_str())?;
    Ok(version.to_string())
}

fn decompress_archive(archive_path: &str, output_path: &str) -> Result<(), Box<dyn Error>> {
    let tar_gz = File::open(archive_path)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);
    archive.unpack(output_path)?;
    Ok(())
}

async fn get_latest_version(repo: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("{repo}/releases/latest");
    let client = Client::builder()
        .redirect(Policy::none())
        .build()?;

    let response = client.get(url).send().await?;

    // Check if the response status is a redirect
    if response.status().is_redirection() {
        if let Some(location) = response.headers().get("Location") {
            let location_str = location.to_str().unwrap_or_default();
            return Ok(location_str.split("/").last()
                .unwrap_or("latest").to_string()
            );
        }
    }
    Ok("latest".to_string())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let repo_list = RepoList::read().repo;
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        error!("Not enough arguments. Structure: {} [module name]", args[0]);
        exit(1);
    }
    let module_name = &args[1];
    let version = args.get(2).map(String::as_str);
    let repo = repo_list.iter().find(|repo| repo.ends_with(module_name));
    match repo {
        None => error!("Repository {module_name} not found"),
        Some(repo) => {
            match download_repo(module_name.as_str(), repo.as_str(), version).await {
                Ok(version) => {
                    info!("Download succeeded from repo {repo} (version {version})")
                },
                Err(e) => {
                    error!("Error downloading repository {repo}: {e}");
                }
            }
        }
    };
}

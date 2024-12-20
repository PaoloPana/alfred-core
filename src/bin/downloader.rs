use std::error::Error;
use std::{fs, io};
use std::fs::{create_dir, File};
use std::process::exit;
use log::{debug, error};
use serde_derive::Deserialize;

const REPO_FILENAME: &str = "repositories.toml";
const MANIFEST_FILENAME: &str = "manifest.toml";
const ALFRED_DIR: &str = "/usr/share/alfred";

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

#[derive(Deserialize, Debug, Clone)]
pub struct Manifest {
    version: String,
    archs: Vec<String>,
    files: Vec<String>
}
impl Manifest {
    pub fn convert(manifest_content: String) -> Result<Self, Box<dyn Error>> {
        toml::from_str(&manifest_content)
            .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}

fn get_asset_url(repo_url: &str, version: &str, filename: &str) -> String {
    format!("{repo_url}/releases/{version}/{filename}")
}

async fn download_manifest(url: &str) -> Result<Manifest, Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    response.text().await.map(Manifest::convert)?
}

async fn download_file(url: &str, out: &str) -> Result<(), Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let text = response.text().await?;
    let mut body = text.as_bytes();
    let mut output_file = File::create(out)?;
    io::copy(&mut body, &mut output_file)?;
    Ok(())
}

async fn download_bin_files(base_url: &str, input_files: &Vec<String>, folder: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut response = Vec::new();
    for file in input_files {
        let out_file = format!("{folder}{file}");
        let file_url = format!("{base_url}{file}");
        download_file(file_url.as_str(), out_file.as_str()).await?;
        response.push(out_file);
    }
    Ok(response)
}

async fn download_repo(repo: &str, version: &str) -> Result<(), Box<dyn Error>> {
    // TODO: stop if already executed
    // download from repo
    //// download manifest: releases/latest/download/manifest.toml
    let manifest_url = get_asset_url(repo, version, MANIFEST_FILENAME);
    let manifest = download_manifest(manifest_url.as_str()).await?;
    //// analyse manifest and download necessary files (in tmp folder): releases/latest/download/{ARCH}/{FILES}
    let current_arch = std::env::consts::ARCH.to_string();
    if manifest.archs.iter().find(|arch| {
        arch.to_string() == current_arch
    }).is_none() {
        error!("Arch {current_arch} not supported");
        exit(1);
    }
    let version = manifest.version;
    let base_url = get_asset_url(repo, version.as_str(), format!("{current_arch}/").as_str());
    let output_dir = format!("/tmp/{repo}");
    create_dir(&output_dir)?;
    download_bin_files(base_url.as_str(), &manifest.files, output_dir.as_str()).await?;
    // for each downloaded files
    //// remove local if already exists
    //// move to main folder
    for file in manifest.files {
        let source_file = format!("/tmp/{repo}/{file}");
        let dest_file = format!("{ALFRED_DIR}/{file}");
        fs::copy(source_file, dest_file)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let repo_list = RepoList::read().repo;
    let args = std::env::args().collect::<Vec<String>>();
    debug!("{:?}", args);
    if args.len() < 2 {
        error!("Not enough arguments. Structure: {} [module name]", args[0]);
    }
    let module_name = &args[1];
    let version = "latest";
    let repo = repo_list.iter().find(|repo| repo.ends_with(module_name));
    match repo {
        None => error!("Repository {module_name} not found"),
        Some(repo) => download_repo(repo.as_str(), version).await?
    };
    Ok(())
}
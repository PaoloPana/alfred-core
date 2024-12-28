use std::collections::HashMap;
use serde_derive::Deserialize;
use std::fs;
use std::path::Path;
use toml;
use envconfig::Envconfig;
use toml::{Table, Value};

pub const CONFIG_FILENAME: &str = "config.toml";
const DEFAULT_TMP_DIR: &'static str = "/tmp";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub alfred: AlfredConfig,
    module: HashMap<String, String>
}

impl Config {
    pub fn read(module_name: Option<&str>) -> Self {
        let alfred = Self::read_alfred_config();
        let module = module_name.map_or_else(HashMap::new, Self::read_module_config);
        Self { alfred, module }
    }

    fn read_module_config(module_name: &str) -> HashMap<String, String> {
        let contents = fs::read_to_string(CONFIG_FILENAME).expect("Could not read file");
        let table: Table = contents.parse().expect("Could not parse toml file");
        table.get(&module_name.to_string())
            .and_then(Value::as_table)
            .map_or_else(HashMap::new, |module_config| module_config
                .iter()
                .map(|(k, v)| (k.to_string(), v.as_str().unwrap_or("").to_string()))
                .collect())
    }

    fn read_alfred_config() -> AlfredConfig {
        let from_env = EnvConfig::from_env();
        let from_file_config = FromFileConfig::read();
        let url = from_env.alfred.url.unwrap_or(from_file_config.alfred.url);
        let pub_port = from_env.alfred.pub_port.unwrap_or(from_file_config.alfred.pub_port);
        let sub_port = from_env.alfred.sub_port.unwrap_or(from_file_config.alfred.sub_port);
        let tmp_dir = from_env.alfred.tmp_dir
            .or(from_file_config.alfred.tmp_dir)
            .unwrap_or(DEFAULT_TMP_DIR.to_string());
        AlfredConfig { url, pub_port, sub_port, tmp_dir, modules: from_file_config.alfred.modules }
    }

    pub fn get_alfred_pub_url(&self) -> String {
        format!("{}:{}", self.alfred.url, self.alfred.pub_port)
    }
    pub fn get_alfred_sub_url(&self) -> String {
        format!("{}:{}", self.alfred.url, self.alfred.sub_port)
    }
    pub fn get_module_value(&self, key: &str) -> Option<String> {
        self.module.get(key).cloned()
    }
}

#[derive(Deserialize, Debug)]
pub struct AlfredConfig {
    pub url: String,
    pub pub_port: u32,
    pub sub_port: u32,
    pub tmp_dir: String,
    pub modules: Vec<String>
}

#[derive(Deserialize, Debug)]
struct FromFileConfig {
    alfred: FromFileAlfredConfig
}

impl FromFileConfig {

    fn get_config_filename() -> String {
        std::env::var("ALFRED_CONFIG")
            .ok()
            .and_then(|path| Path::new(&path.as_str()).exists().then_some(path))
            .unwrap_or_else(|| CONFIG_FILENAME.to_string())
    }

    fn read() -> Self {
        let contents = fs::read_to_string(Self::get_config_filename()).expect("Could not read file");
        toml::from_str(&contents).expect("Unable to load data")
    }
}

#[derive(Deserialize, Debug)]
struct FromFileAlfredConfig {
    url: String,
    pub_port: u32,
    sub_port: u32,
    tmp_dir: Option<String>,
    #[serde(default)]
    modules: Vec<String>
}

#[derive(Deserialize, Debug)]
struct EnvConfig {
    alfred: EnvAlfredConfig,
}
impl EnvConfig {

    fn from_env() -> Self {
        Self {
            alfred: EnvAlfredConfig::init_from_env().expect("Error during initialization using env variable"),
        }
    }
}

#[derive(Deserialize, Debug, Envconfig)]
struct EnvAlfredConfig {
    #[envconfig(from = "ALFRED_URL")]
    url: Option<String>,
    #[envconfig(from = "ALFRED_PUB_PORT")]
    pub_port: Option<u32>,
    #[envconfig(from = "ALFRED_SUB_PORT")]
    sub_port: Option<u32>,
    #[envconfig(from = "ALFRED_TMP_DIR")]
    tmp_dir: Option<String>
}

use std::collections::HashMap;
use serde_derive::Deserialize;
use std::fs;
use toml;
use envconfig::Envconfig;
use toml::Table;
use crate::error::Error;

pub const CONFIG_FILENAME: &str = "config.toml";

#[derive(Deserialize, Debug)]
pub struct Config {
    pub alfred: AlfredConfig,
    module: HashMap<String, String>
}

impl Config {
    pub fn read(module_name: Option<&str>) -> Result<Config, Error> {
        let alfred = Self::read_alfred_config()?;
        let module = Self::read_module_config(module_name)?;
        Ok(Config { alfred, module })
    }

    fn read_module_config(module_name: Option<&str>) -> Result<HashMap<String, String>, Error> {
        if module_name.is_none() {
            return Ok(HashMap::new());
        }
        let contents = fs::read_to_string(CONFIG_FILENAME).expect("Could not read file");
        let table: Table = contents.parse().unwrap();
        let module_config = table.get(&module_name.unwrap().to_string())
            .map(|val| val.as_table()).unwrap_or(None);
        if module_config.is_none() {
            return Ok(HashMap::new());
        }
        let module_config = module_config.unwrap();
        let mut res = HashMap::new();
        for (k, v) in module_config.into_iter() {
            res.insert(k.to_string(), v.as_str().unwrap().to_string());
        }
        Ok(res)
    }

    fn read_alfred_config() -> Result<AlfredConfig, Error> {
        let from_env = EnvConfig::from_env();
        let from_file_config = FromFileConfig::read();
        let url = from_env.alfred.url.unwrap_or(from_file_config.alfred.url);
        let pub_port = from_env.alfred.pub_port.unwrap_or(from_file_config.alfred.pub_port);
        let sub_port = from_env.alfred.sub_port.unwrap_or(from_file_config.alfred.sub_port);
        let tmp_dir = from_env.alfred.tmp_dir.unwrap_or(from_file_config.alfred.tmp_dir.unwrap_or(format!("{}", std::env::current_dir().unwrap().display())));
        Ok(AlfredConfig { url, pub_port, sub_port, tmp_dir, modules: from_file_config.alfred.modules })
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
    pub fn read() -> FromFileConfig {
        // TODO: remove expect
        let contents = fs::read_to_string(CONFIG_FILENAME).expect("Could not read file");
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

    pub fn from_env() -> EnvConfig {
        EnvConfig {
            alfred: EnvAlfredConfig::init_from_env().unwrap()
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

use std::collections::HashMap;
use serde_derive::Deserialize;
use std::fs;
use toml;
use envconfig::Envconfig;

#[derive(Deserialize, Debug)]
pub struct Config {
    alfred: AlfredConfig,
    module: HashMap<String, String>
}

impl Config {
    pub fn read() -> Config {
        let from_env = EnvConfig::from_env();
        let from_file = EnvConfig::from_file();
        Config {
            alfred: AlfredConfig {
                url: from_env.alfred.url.unwrap_or(from_file.alfred.url.expect("Missing ALFRED_URL env variable or alfred.url TOML property")),
                pub_port: from_env.alfred.pub_port.unwrap_or(from_file.alfred.pub_port.expect("Missing ALFRED_PUB_PORT env variable or alfred.pub_port TOML property")),
                sub_port: from_env.alfred.sub_port.unwrap_or(from_file.alfred.sub_port.expect("Missing ALFRED_SUB_PORT env variable or alfred.sub_port TOML property")),
            },
            module: from_file.module.unwrap_or(HashMap::new())
        }

    }
    pub fn get_alfred_url(&self) -> String { self.alfred.get_url() }
    pub fn get_alfred_pub_port(&self) -> u32 { self.alfred.get_pub_port() }
    pub fn get_alfred_sub_port(&self) -> u32 { self.alfred.get_sub_port() }
    fn get_alfred_url_by_port(&self, port: u32) -> String {
        format!("{}:{}", self.get_alfred_url(),  port)
    }
    pub fn get_alfred_pub_url(&self) -> String {
        self.get_alfred_url_by_port(self.get_alfred_pub_port())
    }
    pub fn get_alfred_sub_url(&self) -> String {
        format!("{}:{}", self.get_alfred_url(), self.get_alfred_sub_port())
    }
    pub fn get_module_value(&self, key: String) -> Option<String> {
        self.module.get(&key).cloned()
    }
}

#[derive(Deserialize, Debug, Envconfig)]
pub struct AlfredConfig {
    #[envconfig(from = "ALFRED_URL")]
    url: String,
    #[envconfig(from = "ALFRED_PUB_PORT")]
    pub_port: u32,
    #[envconfig(from = "ALFRED_SUB_PORT")]
    sub_port: u32
}

impl AlfredConfig {
    pub fn get_url(&self) -> String { self.url.clone() }
    pub fn get_pub_port(&self) -> u32 { self.pub_port }
    pub fn get_sub_port(&self) -> u32 { self.sub_port }
}

#[derive(Deserialize, Debug)]
pub struct EnvConfig {
    alfred: EnvAlfredConfig,
    module: Option<HashMap<String, String>>
}
impl EnvConfig {
    pub fn from_file() -> EnvConfig {
        let filename = "config.toml";
        let contents = fs::read_to_string(filename).expect("Could not read file");
        let config = toml::from_str(&contents).expect("Unable to load data");
        config
    }
    pub fn from_env() -> EnvConfig {
        EnvConfig {
            alfred: EnvAlfredConfig::init_from_env().unwrap(),
            module: Some(HashMap::new())
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
    sub_port: Option<u32>
}

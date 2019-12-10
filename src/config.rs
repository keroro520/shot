use ckb_logger::Config as LoggerConfig;
use clap::{App, Arg};
use serde_derive::{Deserialize, Serialize};
use std::clone::Clone;
use std::fs::{create_dir_all, read_to_string};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub basedir: PathBuf,
    pub alice: String,
    pub logger: LoggerConfig,
    pub chain: ChainConfig,
    pub constructor: ConstructorConfig,
    pub controller: ControllerConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChainConfig {
    pub rpc_urls: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConstructorConfig {
    pub fee_rate: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ControllerConfig {
    pub tps: f32,
}

pub fn setup() -> Result<Config, String> {
    let matches = App::new("shot")
        .version("0.1")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .default_value("config.toml")
                .help("Sets a custom config file"),
        )
        .get_matches();
    Config::load(matches.value_of("config").unwrap())
}

impl Config {
    pub fn load(path: &str) -> Result<Self, String> {
        let content = read_to_string(path).map_err(|err| format!("{}", err))?;
        let config: Self = toml::from_str(&content).map_err(|err| format!("{}", err))?;

        {
            let mut log_dir = config.basedir.clone();
            log_dir.push("logs");
            create_dir_all(log_dir).map_err(|err| format!("{}", err))?;
        }

        if config.chain.rpc_urls.is_empty() {
            return Err("ckb_nodes is empty".to_string());
        }
        Ok(config)
    }
}

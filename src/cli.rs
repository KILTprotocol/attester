use clap::Parser;
use std::path::PathBuf;

use crate::Configuration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE", env)]
    config: PathBuf,
}

impl Cli {
    pub fn get_config(&self) -> Configuration {
        let config_file = std::fs::File::open(&self.config).expect("Config File does not exists");
        let config: Configuration =
            serde_yaml::from_reader(config_file).expect("Error parsing the config file");
        config
    }
}

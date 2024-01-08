use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;

use crate::Configuration;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(env)]
    config: PathBuf,
}

impl Cli {
    pub fn get_config(&self) -> anyhow::Result<Configuration> {
        let config_file =
            std::fs::File::open(&self.config).context("Config File does not exists")?;
        let config: Configuration =
            serde_yaml::from_reader(config_file).context("Error parsing the config file")?;
        Ok(config)
    }
}

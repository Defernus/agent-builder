use envstruct::prelude::*;
use eyre::Context;

#[derive(EnvStruct)]
pub struct Config {
    pub ai_api_token: String,
    pub ai_api_url: String,
}

impl Config {
    pub fn init() -> eyre::Result<Self> {
        if dotenvy::dotenv().is_ok() {
            println!("Loaded .env file");
        }

        Self::with_prefix("AGENT").wrap_err("failed to load config")
    }
}

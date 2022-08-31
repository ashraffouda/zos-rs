use anyhow::Result;
use std::env;
use std::{fmt::Display, str::FromStr};

use super::kernel;
lazy_static! {
    static ref DEV_SUBSTRATE_URL: Vec<String> = vec![String::from("wss://tfchain.dev.grid.tf/")];
    static ref QA_SUBSTRATE_URL: Vec<String> = vec![String::from("wss://tfchain.qa.grid.tf/")];
    static ref PROD_SUBSTRATE_URL: Vec<String> = vec![
        String::from("wss://tfchain.grid.tf/"),
        String::from("wss://02.tfchain.grid.tf/"),
        String::from("wss://03.tfchain.grid.tf/"),
        String::from("wss://04.tfchain.grid.tf/"),
    ];
    static ref FLIST_URL: &'static str = "redis://hub.grid.tf:9900";
    static ref DEV_ACTIVATION_URL: &'static str =
        "https://activation.dev.grid.tf/activation/activate";
    static ref QA_ACTIVATION_URL: &'static str =
        "https://activation.qa.grid.tf/activation/activate";
    static ref PROD_ACTIVATION_URL: &'static str = "https://activation.grid.tf/activation/activate";
    static ref DEV_BIN_REPO: &'static str = "tf-zos-v3-bins.dev";
    static ref QA_BIN_REPO: &'static str = "tf-zos-v3-bins.qanet";
    static ref PROD_BIN_REPO: &'static str = "tf-zos-v3-bins";
}

// possible Running modes
#[derive(Debug, Clone)]
pub enum RunMode {
    Dev,
    Qa,
    Test,
    Main,
}

impl Display for RunMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunMode::Dev => write!(f, "development"),
            RunMode::Qa => write!(f, "qa"),
            RunMode::Test => write!(f, "testing"),
            RunMode::Main => write!(f, "production"),
        }
    }
}

impl FromStr for RunMode {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<RunMode, Self::Err> {
        match input {
            "dev" => Ok(RunMode::Dev),
            "development" => Ok(RunMode::Dev),
            "qa" => Ok(RunMode::Qa),
            "test" => Ok(RunMode::Test),
            "main" => Ok(RunMode::Main),
            "production" => Ok(RunMode::Main),
            _ => Err("Invalid Running mode"),
        }
    }
}
// Environment holds information about running environment of a node
// it defines the different constant based on the running mode (dev, test, prod)
#[derive(Debug, Clone)]
pub struct Environment {
    pub run_mode: RunMode,
    pub flist_url: String,
    pub bin_repo: String,
    pub farmer_id: i32,
    pub orphan: bool,
    pub farmer_secret: String,
    pub substrate_url: Vec<String>,
    pub activation_url: String,
    pub extended_config_url: String,
}
fn get_env(run_mode: RunMode) -> Environment {
    Environment {
        flist_url: (*FLIST_URL).to_string(),
        farmer_id: 0,
        orphan: true,
        extended_config_url: String::from("Unknown"),
        farmer_secret: String::from("Unknown"),
        run_mode: match run_mode {
            RunMode::Dev => RunMode::Dev,
            RunMode::Qa => RunMode::Qa,
            RunMode::Test => RunMode::Test,
            RunMode::Main => RunMode::Main,
        },
        bin_repo: match &run_mode {
            RunMode::Dev => (*DEV_BIN_REPO).to_string(),
            RunMode::Qa => (*QA_BIN_REPO).to_string(),
            RunMode::Test => (*QA_BIN_REPO).to_string(),
            RunMode::Main => (*PROD_BIN_REPO).to_string(),
        },
        substrate_url: match run_mode {
            RunMode::Dev => DEV_SUBSTRATE_URL.iter().map(|s| s.to_string()).collect(),
            RunMode::Qa => QA_SUBSTRATE_URL.iter().map(|s| s.to_string()).collect(),
            RunMode::Test => QA_SUBSTRATE_URL.iter().map(|s| s.to_string()).collect(),
            RunMode::Main => PROD_SUBSTRATE_URL.iter().map(|s| s.to_string()).collect(),
        },
        activation_url: match run_mode {
            RunMode::Dev => (*DEV_ACTIVATION_URL).to_string(),
            RunMode::Qa => (*QA_ACTIVATION_URL).to_string(),
            RunMode::Test => (*QA_ACTIVATION_URL).to_string(),
            RunMode::Main => (*PROD_ACTIVATION_URL).to_string(),
        },
    }
}

pub fn get() -> Result<Environment> {
    let params = kernel::get();
    get_env_from_params(params)
}
fn get_env_from_params(params: kernel::Params) -> Result<Environment> {
    let run_mode = match params.values("runmode") {
        Some(runmode) => {
            if runmode.len() > 0 {
                runmode[0].clone()
            } else {
                "main".to_string()
            }
        }
        None => match env::var("ZOS_RUNMODE") {
            Ok(run_mode) => run_mode,
            Err(_) => "Unknown".to_string(),
        },
    };

    let run_mode = RunMode::from_str(&run_mode).unwrap();
    let mut env = get_env(run_mode);
    if let Some(extended) = params.values("config_url") {
        if extended.len() > 0 {
            env.extended_config_url = extended[0].clone();
        }
    }

    if let Some(substrate) = params.values("substrate") {
        if substrate.len() > 0 {
            env.substrate_url = substrate.to_vec();
        }
    };
    match params.values("activation") {
        Some(activation) => {
            if activation.len() > 0 {
                env.activation_url = activation[activation.len() - 1].clone();
            }
        }
        None => {}
    };

    match params.values("secret") {
        Some(farm_secret) => {
            if farm_secret.len() > 0 {
                env.farmer_secret = farm_secret[farm_secret.len() - 1].clone();
            }
        }
        None => {}
    };

    match params.values("farmer_id") {
        Some(farmer_id) => {
            if farmer_id.len() < 1 {
                env.orphan = true;
                env.farmer_id = 0;
            } else {
                env.orphan = false;
                let id = farmer_id[0].parse::<i32>()?;
                env.farmer_id = id;
            }
        }
        None => {
            env.orphan = true;
            env.farmer_id = 0;
        }
    };

    // Checking if there environment variable
    // override default settings
    match env::var("ZOS_SUBSTRATE_URL") {
        // let urls: Vec<&str> =  substrate.iter().map(|s| s as &str).collect();
        Ok(substrate_url) => {
            env.substrate_url = vec![substrate_url];
        }
        Err(_) => {}
    };
    match env::var("ZOS_FLIST_URL") {
        Ok(flist_url) => env.flist_url = flist_url,
        Err(_) => {}
    };

    match env::var("ZOS_BIN_REPO") {
        Ok(bin_repo) => env.bin_repo = bin_repo,
        Err(_) => {}
    };

    Ok(env)
}

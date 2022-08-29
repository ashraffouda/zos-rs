use anyhow::Result;
use std::env;
use std::{collections::HashMap, fmt::Display, str::FromStr};

use super::kernel;

// possible Running modes
#[derive(Debug)]
pub enum RunningMode {
    Dev,
    Qa,
    Test,
    Main,
}
impl Display for RunningMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunningMode::Dev => write!(f, "development"),
            RunningMode::Qa => write!(f, "qa"),
            RunningMode::Test => write!(f, "testing"),
            RunningMode::Main => write!(f, "production"),
        }
    }
}
impl FromStr for RunningMode {
    type Err = ();

    fn from_str(input: &str) -> Result<RunningMode, Self::Err> {
        match input {
            "dev" => Ok(RunningMode::Dev),
            "qa" => Ok(RunningMode::Qa),
            "test" => Ok(RunningMode::Test),
            "main" => Ok(RunningMode::Main),
            _ => Err(()),
        }
    }
}
// Environment holds information about running environment of a node
// it defines the different constant based on the running mode (dev, test, prod)
#[derive(Debug)]
pub struct Environment {
    pub running_mode: RunningMode,
    pub flist_url: String,
    pub bin_repo: String,
    pub farmer_id: i32,
    pub orphan: bool,
    pub farmer_secret: String,
    pub substrate_url: Vec<String>,
    pub activation_url: String,
    pub extended_config_url: String,
}

fn envs(run_mode: RunningMode) -> Environment {
    let env_dev = Environment {
        running_mode: RunningMode::Dev,
        substrate_url: vec![String::from("wss://tfchain.dev.grid.tf/")],
        activation_url: String::from("https://activation.dev.grid.tf/activation/activate"),
        flist_url: String::from("redis://hub.grid.tf:9900"),
        bin_repo: String::from("tf-zos-v3-bins.dev"),
        orphan: true,
        farmer_secret: String::from("Unknown"),
        extended_config_url: String::from("Unknown"),
        farmer_id: 0,
    };
    let env_test = Environment {
        running_mode: RunningMode::Test,
        substrate_url: vec![String::from("wss://tfchain.test.grid.tf/")],
        activation_url: String::from("https://activation.test.grid.tf/activation/activate"),
        flist_url: String::from("redis://hub.grid.tf:9900"),
        bin_repo: String::from("tf-zos-v3-bins.test"),
        orphan: true,
        farmer_secret: String::from("Unknown"),
        extended_config_url: String::from("Unknown"),
        farmer_id: 0,
    };
    let env_qa = Environment {
        running_mode: RunningMode::Qa,
        substrate_url: vec![String::from("wss://tfchain.qa.grid.tf/")],
        activation_url: String::from("https://activation.qa.grid.tf/activation/activate"),
        flist_url: String::from("redis://hub.grid.tf:9900"),
        bin_repo: String::from("tf-zos-v3-bins.qanet"),
        orphan: true,
        farmer_secret: String::from("Unknown"),
        extended_config_url: String::from("Unknown"),
        farmer_id: 0,
    };
    // same as testnet for now. will be updated the day of the launch of production network
    let env_prod = Environment {
        running_mode: RunningMode::Main,
        substrate_url: vec![
            String::from("wss://tfchain.grid.tf/"),
            String::from("wss://02.tfchain.grid.tf/"),
            String::from("wss://03.tfchain.grid.tf/"),
            String::from("wss://04.tfchain.grid.tf/"),
        ],
        activation_url: String::from("https://activation.grid.tf/activation/activate"),
        flist_url: String::from("redis://hub.grid.tf:9900"),
        bin_repo: String::from("tf-zos-v3-bins"),
        orphan: true,
        farmer_secret: String::from("Unknown"),
        extended_config_url: String::from("Unknown"),
        farmer_id: 0,
    };
    match run_mode {
        RunningMode::Dev => env_dev,
        RunningMode::Qa => env_qa,
        RunningMode::Test => env_test,
        RunningMode::Main => env_prod,
    }
}
pub fn get() -> Result<Environment> {
    let params = kernel::get_params();
    get_env_from_params(params)
}
fn get_env_from_params(params: HashMap<String, Vec<String>>) -> Result<Environment> {
    let run_mode = match params.get("runmode") {
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

    let run_mode = RunningMode::from_str(&run_mode).unwrap();
    let mut env = envs(run_mode);

    match params.get("config_url") {
        Some(extended) => {
            if extended.len() > 0 {
                env.extended_config_url = extended[0].clone();
            }
        }
        None => {}
    };
    match params.get("substrate") {
        Some(substrate) => {
            if substrate.len() > 0 {
                env.substrate_url = substrate.to_vec();
            }
        }
        None => {}
    };
    match params.get("activation") {
        Some(activation) => {
            if activation.len() > 0 {
                env.activation_url = activation[activation.len() - 1].clone();
            }
        }
        None => {}
    };

    match params.get("secret") {
        Some(farm_secret) => {
            if farm_secret.len() > 0 {
                env.farmer_secret = farm_secret[farm_secret.len() - 1].clone();
            }
        }
        None => {}
    };

    match params.get("farmer_id") {
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
        Ok(substrate_url) => env.substrate_url = vec![substrate_url],
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

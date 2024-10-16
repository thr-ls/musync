use config::ConfigError;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{env, fs, io};

#[derive(Deserialize)]
pub struct Settings {
    pub local_path: String,
    pub remote_settings: RemoteSettings,
    pub api_settings: ApiSettings,
}

#[derive(Deserialize)]
pub struct RemoteSettings {
    pub remote_user: String,
    pub remote_host: String,
    pub remote_path: String,
    pub ssh_key_path: String,
}

#[derive(Deserialize)]
pub struct ApiSettings {
    pub api_base_url: String,
    pub api_username: String,
    pub api_password: String,
}

impl ApiSettings {
    pub fn new(url: &str, username: &str, password: &str) -> Self {
        Self {
            api_base_url: url.to_string(),
            api_username: username.to_string(),
            api_password: password.to_string(),
        }
    }
}

pub fn get_configuration(cfg_file: &str) -> Result<Settings, ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(cfg_file, config::FileFormat::Yaml))
        .build()?;

    settings.try_deserialize::<Settings>()
}

pub struct ConfigFolder {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub musync_db: PathBuf,
}

impl ConfigFolder {
    pub fn new() -> Self {
        let home_dir = env::var("HOME").expect("Failed to get HOME environment variable");

        Self {
            config_dir: get_config_dir_name(&home_dir),
            config_file: get_config_file_name(&home_dir),
            musync_db: get_musync_db_name(&home_dir),
        }
    }
}

fn get_config_dir_name(home_dir: &String) -> PathBuf {
    Path::new(&home_dir).join(".musync")
}

fn get_config_file_name(home_dir: &String) -> PathBuf {
    Path::new(&home_dir).join(".musync").join("config.yaml")
}

fn get_musync_db_name(home_dir: &String) -> PathBuf {
    Path::new(&home_dir).join(".musync").join("musync_db")
}

pub fn create_config(cfg_folder: ConfigFolder) -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[1m\x1b[32mCreating configuration...\x1b[0m");
    let config_dir = cfg_folder.config_dir;

    if config_dir.exists() && !confirm_overwrite()? {
        println!("\x1b[33mOperation cancelled.\x1b[0m");
        return Ok(());
    }

    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&cfg_folder.musync_db)?;

    let config_content = include_str!("config_template.yaml");
    fs::write(&cfg_folder.config_file, config_content)?;

    println!("\x1b[32mConfiguration folder created at:");
    println!("  -> ~/.musync");
    println!("Configuration file created at:");
    println!("  -> ~/.musync/config.yaml");
    println!("musync_db folder created at:");
    println!("  -> ~/.musync/musync_db");
    println!("\x1b[0mPlease edit the configuration file with your specific settings.");

    Ok(())
}

fn confirm_overwrite() -> Result<bool, io::Error> {
    println!("\x1b[31mThe configuration folder already exists.");
    println!("Do you want to overwrite it? Everything will be lost. (y/N)\x1b[0m");

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() == "y" {
        Ok(true)
    } else {
        Ok(false)
    }
}

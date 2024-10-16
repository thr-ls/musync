/// # The Main Entry Point of Synchronization
///
/// This function serves as the primary driver of our application. It orchestrates the entire process
/// from configuration loading to album synchronization with a remote API.
///
/// # Steps:
/// 1. Loads the configuration
/// 2. Opens the local database
/// 3. Processes the root directory
/// 4. Compares local data with the API
/// 5. Uploads any missing albums
///
use crate::{api_client, configuration, foundation::database, process};
use configuration::ConfigFolder;
use std::path::Path;

pub async fn run(cfg_folder: ConfigFolder) -> Result<(), Box<dyn std::error::Error>> {
    if !cfg_folder.config_dir.exists() || !cfg_folder.config_file.exists() {
        eprintln!(
            "\x1b[1m\x1b[31mConfiguration folder or config.yaml not found. Please run 'musync config' first.\x1b[0m"
        );
        return Ok(());
    }

    println!("\x1b[1m\x1b[34mStarting synchronization...\x1b[0m");
    start_sync(cfg_folder).await
}

async fn start_sync(config_folder: ConfigFolder) -> Result<(), Box<dyn std::error::Error>> {
    let config_file = config_folder.config_file.to_str().unwrap();
    let config = configuration::get_configuration(config_file)
        .map_err(|_| "Unable to parse configuration file")?;

    let db_path_as_str = config_folder
        .musync_db
        .to_str()
        .ok_or_else(|| "Failed to convert the database path to a string".to_string())?;

    let db = database::open_database(db_path_as_str)?;

    if let Err(e) = process::process_root(Path::new(&config.local_path), &db) {
        eprintln!(
            "\x1b[1m\x1b[31mFailed to process the root directory: {}\x1b[0m",
            e
        );
        return Ok(()); // Return Ok to prevent propagating the error further
    }

    let missing_albums = api_client::compare_with_api(&db, &config.api_settings)
        .await
        .unwrap_or_else(|e| {
            eprintln!("\x1b[31mError comparing with API: {}\x1b[0m", e);
            Vec::new()
        });

    if missing_albums.is_empty() {
        println!("\x1b[32mNo missing albums to upload. Everything is up-to-date!\x1b[0m");
    } else {
        println!("\x1b[1m\x1b[34mUploading missing albums to server...\x1b[0m");
        if let Err(e) = api_client::upload_missing_albums(&missing_albums, &config.remote_settings)
        {
            eprintln!("\x1b[31mFailed to upload albums: {}\x1b[0m", e);
        } else {
            println!("\x1b[32mSuccessfully uploaded missing albums.\x1b[0m");
        }
    }

    Ok(())
}

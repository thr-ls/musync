use clap::Command;
use musync::configuration::{create_config, ConfigFolder};
use musync::startup::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Command::new("musync")
        .about("🎵 Music synchronization tool utilizing the Subsonic API 🎵")
        .subcommand(
            Command::new("run")
                .about("🚀 Run the synchronization process to keep your music in sync"),
        )
        .subcommand(
            Command::new("config").about("🛠️ Create or update configuration file for musync"),
        )
        .get_matches();

    let cfg_folder = ConfigFolder::new();

    match args.subcommand() {
        Some(("run", _)) => {
            println!("\x1b[1m\x1b[34mStarting the synchronization process...\x1b[0m");
            run(cfg_folder).await
        }
        Some(("config", _)) => {
            println!("\x1b[1m\x1b[34mConfiguring musync...\x1b[0m");
            create_config(cfg_folder)
        }
        _ => {
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    println!("\x1b[1m\x1b[31mInvalid command!\x1b[0m\n");
    println!("📖 Available Commands:");
    println!("  \x1b[1m\x1b[32mmusync run\x1b[0m    - 🚀 Start synchronization");
    println!("  \x1b[1m\x1b[32mmusync config\x1b[0m - 🛠️  Create or update configuration file");
    println!("\x1b[33mUse these commands to manage your music library more effectively!\x1b[0m\n");
}

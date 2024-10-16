# MuSync

MuSync is a basic Rust-based CLI application designed to synchronize your local music library with a Subsonic API server (Navidrome in my case). It efficiently processes your music collection, compares it with a remote API, and uploads missing albums.

This project has a limited scope, tailored to my specific needs. I have a Navidrome server which is compatible with Subsonic API v1.16.1, and it reflects my local music library. While I've used more robust solutions in the past, they often required high maintenance or had issues that needed fixing. I simply wanted a tool where I could run a command to compare the albums on the server and upload any missing ones. Navidrome requires the files to be stored locally on the server, but I also want to keep the files on my laptop for local access.

## Features

- Scans and processes local music directories
- Stores artist and album information in a local database
- Compares local data with a remote API (Subsonic compatible)
- Identifies missing albums
- Uploads missing albums to a remote server via SCP
- Provides progress tracking for uploads
- One-way communication. (It does not handle deletion on the server)

## Installation

You can install MuSync using one of the following methods:

### Option 1: Install from release binary

1. Go to the [Releases](https://github.com/thr-ls/musync/releases) page of the MuSync repository.
2. Download the latest release binary for your operating system.
3. Extract the archive and move the `musync` binary to a directory in your system PATH (e.g., `/usr/local/bin` on Unix-like systems).

### Option 2: Build from source

1. Ensure you have Rust installed (rustc 1.81.0 or later).
2. Clone the repository:
   ```
   git clone https://github.com/yourusername/musync.git
   cd musync
   ```
3. Build the project:
   ```
   cargo build --release
   ```
4. The binary will be available at `target/release/musync`.

## Configuration

After installation, you need to create a configuration file:

1. Run the following command to create the initial configuration:
   ```
   musync config
   ```
2. This will create a configuration folder at `~/.musync` with a `config.yaml` file and a `musync_db` directory.
3. Edit the `~/.musync/config.yaml` file with your specific settings:

```yaml
local_path: "/path/to/your/music/library"
db_path: "/path/to/local/database"
remote_settings:
  remote_user: "remote_username"
  remote_host: "remote.host.com"
  remote_path: "/path/on/remote/server"
  ssh_key_path: "/path/to/your/ssh/key"
api_settings:
  api_base_url: "https://your-api-server.com"
  api_username: "your_username"
  api_password: "your_password"
```

Ensure you update the paths and credentials to match your setup.

## Usage

To run MuSync and start the synchronization process:

```
musync run
```

The application will process your local music library, compare it with the remote API, and upload any missing albums. It will provide progress information and status updates during the synchronization process.

## Project Structure

- `src/main.rs`: Entry point of the application
- `src/startup.rs`: Main application logic and orchestration
- `src/process/`: Handles local music library processing
- `src/foundation/`: Core functionality including database operations and utility functions
- `src/api_client/`: Manages communication with the remote API and file uploads

## Todo

- [ ] Create more tests

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

//! This module provides functionality for uploading missing albums to a remote location
//! with progress tracking. It includes functions for creating progress bars, extracting
//! album information from file paths, constructing remote paths, and performing the
//! actual upload using SCP.

use crate::configuration::RemoteSettings;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use regex::Regex;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

/// Uploads missing albums to a remote location with progress tracking.
///
/// This function takes a slice of album paths and remote settings, then uploads each album
/// to the specified remote location. It provides visual feedback using progress bars for
/// both overall progress and individual album uploads.
///
/// # Arguments
///
/// * `missing_albums` - A slice of strings representing paths to albums that need to be uploaded.
/// * `settings` - Remote settings containing information like host, user, and SSH key path.
///
/// # Example
///
/// ```
/// use musync::RemoteSettings;
/// use musync::upload_missing_albums;
///
/// let missing_albums = vec![
///     String::from("/path/to/Artist1/Album1"),
///     String::from("/path/to/Artist2/Album2"),
/// ];
///
/// let settings = RemoteSettings {
///     remote_user: String::from("user"),
///     remote_host: String::from("example.com"),
///     remote_path: String::from("/music"),
///     ssh_key_path: String::from("/path/to/ssh_key"),
/// };
///
/// upload_missing_albums(&missing_albums, &settings).expect("Failed to upload albums");
/// ```
///
pub fn upload_missing_albums(
    missing_albums: &[String],
    settings: &RemoteSettings,
) -> io::Result<()> {
    let multi_progress = MultiProgress::new();
    let overall_progress =
        create_progress_bar(&multi_progress, missing_albums.len() as u64, "albums");
    let re = Regex::new(r"(\d+)%").unwrap();

    for album_path in missing_albums {
        let (artist, album_name) = extract_artist_and_album(album_path)?;
        let remote_album_path = create_remote_path(settings, &artist, &album_name);

        overall_progress.set_message(format!("Uploading: {artist} - {album_name}"));

        let album_progress = create_progress_bar(&multi_progress, 100, "%");
        album_progress.set_message(format!("{artist} - {album_name}"));

        match upload_album(
            album_path,
            &remote_album_path,
            settings,
            &re,
            &album_progress,
        ) {
            Ok(()) => {
                album_progress.finish_with_message(format!("Uploaded: {artist} - {album_name}"));
                overall_progress.inc(1);
            }
            Err(e) => {
                album_progress.finish_with_message(format!("Failed: {artist} - {album_name}"));
                eprintln!("Failed to upload {artist} - {album_name}: {e}");
            }
        }
    }

    overall_progress.finish_with_message("All uploads completed");
    Ok(())
}

/// Creates a stylized progress bar for tracking upload progress.
///
/// This helper function sets up a progress bar with a custom style, making it easier
/// to visualize the upload process for both individual albums and overall progress.
///
/// # Arguments
///
/// * `multi_progress` - A reference to the MultiProgress instance for managing multiple progress bars.
/// * `total` - The total number of steps or items to track.
/// * `unit` - A string representing the unit of measurement (e.g., "albums" or "%").
fn create_progress_bar(multi_progress: &MultiProgress, total: u64, unit: &str) -> ProgressBar {
    let progress = multi_progress.add(ProgressBar::new(total));
    progress.set_style(
        ProgressStyle::default_bar()
            .template(&format!(
                "{{elapsed_precise}} [{{bar:40.cyan/blue}}] {{pos}}/{{len}} {unit} {{msg}}"
            ))
            .unwrap()
            .progress_chars("##-"),
    );
    progress
}

/// Extracts the artist and album name from a given album path.
///
/// This function parses the provided album path to extract the artist name (from the parent
/// directory) and the album name (from the directory name). It performs some basic validation
/// to ensure the path structure is as expected.
///
/// # Arguments
///
/// * `album_path` - A string slice representing the path to the album directory.
///
/// ```
///
fn extract_artist_and_album(album_path: &str) -> io::Result<(String, String)> {
    let path = Path::new(album_path);
    let artist = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid artist path"))?;

    let album_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid album path"))?;

    if album_name.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Empty album name",
        ));
    }

    Ok((artist.to_string(), album_name.to_string()))
}

/// Constructs the remote path for an album based on the provided settings and album information.
///
/// This function formats the remote path using the RemoteSettings and the extracted
/// artist and album names. The resulting path is used as the destination for the album upload.
///
/// # Arguments
///
/// * `settings` - A reference to the RemoteSettings containing remote user, host, and path information.
/// * `artist` - The name of the artist.
/// * `album_name` - The name of the album.
fn create_remote_path(settings: &RemoteSettings, artist: &str, album_name: &str) -> String {
    format!(
        "{}@{}:{}/{}/{}",
        settings.remote_user, settings.remote_host, settings.remote_path, artist, album_name
    )
}

/// Uploads a single album to the remote location using SCP.
///
/// This function spawns an SCP process to upload the album, capturing and parsing the
/// progress output to update the progress bar. It handles potential errors and ensures
/// the upload process completes successfully.
///
/// # Arguments
///
/// * `album_path` - The local path of the album to be uploaded.
/// * `remote_path` - The constructed remote path where the album will be uploaded.
/// * `settings` - A reference to the RemoteSettings containing the SSH key path.
/// * `re` - A reference to a Regex for parsing the SCP progress output.
/// * `progress` - A reference to the ProgressBar for updating upload progress.
///
fn upload_album(
    album_path: &str,
    remote_path: &str,
    settings: &RemoteSettings,
    re: &Regex,
    progress: &ProgressBar,
) -> io::Result<()> {
    let mut child = Command::new("scp")
        .args(&["-r", "-i", &settings.ssh_key_path, album_path, remote_path])
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stderr) = child.stderr.take() {
        for line in BufReader::new(stderr).lines().filter_map(Result::ok) {
            if let Some(cap) = re.captures(&line) {
                if let Some(percent) = cap.get(1).and_then(|m| m.as_str().parse::<u64>().ok()) {
                    progress.set_position(percent);
                }
            }
        }
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("SCP command failed with status: {}", status),
        ));
    }

    Ok(())
}

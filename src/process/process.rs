//! This module provides functionality for processing music directories and managing artist data.
//!
//! It includes functions for traversing directory structures, identifying audio files,
//! and updating artist information in a database.

use crate::foundation::database::{get_artist_data, store_artist_data};
use crate::foundation::utils::{clean_album_name, normalize_unicode};
use rayon::prelude::*;
use sled::Db;
use std::path::Path;
use std::time::UNIX_EPOCH;
use std::{fs, io};
use walkdir::WalkDir;

/// Supported audio file extensions.
const AUDIO_EXTENSIONS: [&str; 4] = ["mp3", "flac", "wav", "m4a"];

/// Process the root directory of the music collection.
///
/// This function walks through the immediate subdirectories of the root,
/// treating each as an artist folder, and processes them in parallel.
///
/// # Arguments
///
/// * `root` - The path to the root directory of the music collection.
/// * `db` - A reference to the database where artist information is stored.
///
pub fn process_root(root: &Path, db: &Db) -> io::Result<()> {
    WalkDir::new(root)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .par_bridge()
        .try_for_each(|entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_dir() && has_sub_folders(path)? {
                    let artist_name =
                        path.file_name().and_then(|n| n.to_str()).ok_or_else(|| {
                            io::Error::new(io::ErrorKind::InvalidData, "Invalid artist name")
                        })?;

                    process_artist_folder(path, artist_name, db)
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Failed to read directory entry. Details: {}", e),
            )),
        })
}

/// Process an individual artist folder.
///
/// This function checks if the artist's data needs updating, collects album information,
/// and stores the updated data in the database.
///
/// # Arguments
///
/// * `path` - The path to the artist's folder.
/// * `artist_name` - The name of the artist.
/// * `db` - A reference to the database.
///
fn process_artist_folder(path: &Path, artist_name: &str, db: &Db) -> io::Result<()> {
    let normalized_name = normalize_unicode(artist_name);
    let last_modified = get_last_modified_time(path)?;

    if let Some(stored_data) = get_artist_data(db, &normalized_name)? {
        if last_modified <= stored_data.last_modified {
            println!("Artist: {} (unchanged)", artist_name);
            return Ok(());
        }
    }

    let albums = collect_albums(path)?;
    let album_count = albums.len();

    store_artist_data(db, &normalized_name, album_count, last_modified, albums)?;
    println!("Artist: {}, Albums: {} (updated)", artist_name, album_count);
    Ok(())
}

/// Collect album information for an artist.
///
/// This function scans the artist's directory for subdirectories containing audio files,
/// which are considered albums.
///
fn collect_albums(artist_path: &Path) -> io::Result<Vec<(String, String)>> {
    WalkDir::new(artist_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let album_name = entry.file_name().to_str()?;
            if album_name != artist_path.file_name()?.to_str()? && has_audio_files(entry.path()) {
                let cleaned_name = clean_album_name(album_name);
                let full_path = entry.path().to_string_lossy().into_owned();
                Some(Ok((cleaned_name, full_path)))
            } else {
                None
            }
        })
        .collect()
}

/// Check if a directory contains any audio files.
///
/// # Arguments
///
/// * `path` - The path to check for audio files.
///
fn has_audio_files(path: &Path) -> bool {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .any(|e| is_audio_file(e.path()))
}

/// Check if a file is an audio file based on its extension.
///
fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Get the last modified time of a file or directory.
///
fn get_last_modified_time(path: &Path) -> io::Result<u64> {
    path.metadata()?
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
}

/// Check if a directory contains any sub-folders.
/// Ensure that only valid artist directories with sub-folders (potential albums) are processed,
/// and artists without any albums are skipped.
fn has_sub_folders(path: &Path) -> io::Result<bool> {
    Ok(fs::read_dir(path)?
        .filter_map(Result::ok)
        .any(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn create_test_directory(structure: &[(&str, &[&str])]) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        for (artist, albums) in structure {
            let artist_path = temp_dir.path().join(artist);
            fs::create_dir(&artist_path).unwrap();
            for album in *albums {
                let album_path = artist_path.join(album);
                fs::create_dir(&album_path).unwrap();
            }
        }
        temp_dir
    }

    #[test]
    fn test_process_root() {
        let temp_dir =
            create_test_directory(&[("Artist1", &["Album1", "Album2"]), ("Artist2", &["Album3"])]);

        // Add audio files to each album
        File::create(
            temp_dir
                .path()
                .join("Artist1")
                .join("Album1")
                .join("test1.mp3"),
        )
        .unwrap();
        File::create(
            temp_dir
                .path()
                .join("Artist1")
                .join("Album2")
                .join("test2.flac"),
        )
        .unwrap();
        File::create(
            temp_dir
                .path()
                .join("Artist2")
                .join("Album3")
                .join("test3.wav"),
        )
        .unwrap();

        let db = sled::Config::new().temporary(true).open().unwrap();

        process_root(temp_dir.path(), &db).unwrap();

        let artist1_data = get_artist_data(&db, "Artist1").unwrap().unwrap();
        assert_eq!(artist1_data.album_count, 2);

        let artist2_data = get_artist_data(&db, "Artist2").unwrap().unwrap();
        assert_eq!(artist2_data.album_count, 1);
    }

    #[test]
    fn test_collect_albums() {
        let temp_dir = create_test_directory(&[("Artist", &["Album1", "Album2", "NotAnAlbum"])]);
        let artist_path = temp_dir.path().join("Artist");

        // Add an audio file to Album1 and Album2, but not to NotAnAlbum
        File::create(artist_path.join("Album1").join("test.mp3")).unwrap();
        File::create(artist_path.join("Album2").join("test.flac")).unwrap();

        let albums = collect_albums(&artist_path).unwrap();

        assert_eq!(albums.len(), 2);
        assert!(albums.iter().any(|(name, _)| name == "Album1"));
        assert!(albums.iter().any(|(name, _)| name == "Album2"));
        assert!(!albums.iter().any(|(name, _)| name == "NotAnAlbum"));
    }

    #[test]
    fn test_has_audio_files() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test");
        fs::create_dir(&test_path).unwrap();

        assert!(!has_audio_files(&test_path));

        File::create(test_path.join("test.mp3")).unwrap();
        assert!(has_audio_files(&test_path));
    }

    #[test]
    fn test_is_audio_file() {
        assert!(is_audio_file(Path::new("test.mp3")));
        assert!(is_audio_file(Path::new("test.FLAC")));
        assert!(is_audio_file(Path::new("test.wav")));
        assert!(is_audio_file(Path::new("test.m4a")));
        assert!(!is_audio_file(Path::new("test.txt")));
    }

    #[test]
    fn test_get_last_modified_time() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        File::create(&test_file).unwrap();

        let last_modified = get_last_modified_time(&test_file).unwrap();
        assert!(last_modified > 0);
    }
}

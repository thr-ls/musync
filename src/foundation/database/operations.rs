use crate::foundation::database::ArtistData;
use crate::foundation::utils::normalize_unicode;
use sled::Db;
use std::io;

/// Opens a database at the specified path.
///
/// This function creates a new database or opens an existing one at the given path.
/// It's a friendly wrapper around `sled::open` that converts the error to a standard
/// IO error for easier error handling.
///
/// # Examples
///
/// ```
/// use musync::open_database;
/// let db = open_database("/path/to/my/database")?;
/// ```
pub fn open_database(path: &str) -> io::Result<Db> {
    sled::open(path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

/// Stores artist data in the database.
///
/// This function takes various pieces of information about an artist and stores
/// them in the database. It normalizes the artist name to ensure consistent storage
/// and retrieval, even with different Unicode representations.
///
/// # Arguments
///
/// * `db` - A reference to the opened database.
/// * `artist_name` - The name of the artist to store.
/// * `album_count` - The number of albums by the artist.
/// * `last_modified` - A timestamp indicating when the data was last modified.
/// * `albums` - A vector of tuples containing album names and years.
///
/// # Examples
///
/// ```
/// use musync::store_artist_data;
/// use musync::open_database;
///
/// let db = open_database("/path/to/my/database")?;
///
/// let albums = vec![("Album Name".to_string(), "2023".to_string())];
/// store_artist_data(&db, "Artist Name", 1, 1234567890, albums)?;
/// ```
pub fn store_artist_data(
    db: &Db,
    artist_name: &str,
    album_count: usize,
    last_modified: u64,
    albums: Vec<(String, String)>,
) -> io::Result<()> {
    let normalized_name = normalize_unicode(artist_name);

    let data = ArtistData {
        album_count,
        last_modified,
        albums,
    };

    let serialized = bincode::serialize(&data)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    db.insert(normalized_name.as_bytes(), serialized)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    Ok(())
}

/// Retrieves artist data from the database.
///
/// This function fetches the stored data for a given artist. It normalizes the
/// artist name to ensure consistent retrieval, regardless of Unicode representation.
///
/// # Arguments
///
/// * `db` - A reference to the opened database.
/// * `artist_name` - The name of the artist to retrieve data for.
///
/// # Returns
///
/// Returns a `Result` containing an `Option<ArtistData>`. The `Option` will be
/// `Some(ArtistData)` if the artist was found in the database, or `None` if not.
/// An `io::Error` is returned if the retrieval operation fails.
///
/// # Examples
///
/// ```
/// use musync::get_artist_data;
/// use musync::open_database;
///
/// let db = open_database("/path/to/my/database")?;
///
/// match get_artist_data(&db, "Artist Name")? {
///     Some(data) => println!("Found artist data: {:?}", data),
///     None => println!("Artist not found in database"),
/// }
/// ```
pub fn get_artist_data(db: &Db, artist_name: &str) -> io::Result<Option<ArtistData>> {
    let normalized_name = normalize_unicode(artist_name);

    db.get(normalized_name.as_bytes())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
        .map(|ivec| {
            bincode::deserialize(&ivec)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_open_database() {
        let temp_dir = tempdir().unwrap();
        let binding = temp_dir.path().join("test_db");
        let db_path = binding.to_str().unwrap();

        let result = open_database(db_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_store_and_get_artist_data() {
        let temp_dir = tempdir().unwrap();
        let binding = temp_dir.path().join("test_db");
        let db_path = binding.to_str().unwrap();

        let db = open_database(db_path).unwrap();

        let artist_name = "Test Artist";
        let album_count = 2;
        let last_modified = 1234567890;
        let albums = vec![
            ("Album 1".to_string(), "2020".to_string()),
            ("Album 2".to_string(), "2022".to_string()),
        ];

        // Store artist data
        let store_result =
            store_artist_data(&db, artist_name, album_count, last_modified, albums.clone());
        assert!(store_result.is_ok());

        // Retrieve artist data
        let get_result = get_artist_data(&db, artist_name);
        assert!(get_result.is_ok());

        let artist_data = get_result.unwrap().unwrap();
        assert_eq!(artist_data.album_count, album_count);
        assert_eq!(artist_data.last_modified, last_modified);
        assert_eq!(artist_data.albums, albums);
    }

    #[test]
    fn test_get_nonexistent_artist() {
        let temp_dir = tempdir().unwrap();
        let binding = temp_dir.path().join("test_db");
        let db_path = binding.to_str().unwrap();

        let db = open_database(db_path).unwrap();

        let result = get_artist_data(&db, "Nonexistent Artist");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_unicode_normalization() {
        let temp_dir = tempdir().unwrap();
        let binding = temp_dir.path().join("test_db");
        let db_path = binding.to_str().unwrap();

        let db = open_database(db_path).unwrap();

        let artist_name = "Björk";
        let normalized_name = normalize_unicode(artist_name);
        let album_count = 1;
        let last_modified = 1234567890;
        let albums = vec![("Album".to_string(), "2020".to_string())];

        println!("Original artist name: {}", artist_name);
        println!("Normalized artist name: {}", normalized_name);

        // Store using non-normalized name
        let store_result =
            store_artist_data(&db, artist_name, album_count, last_modified, albums.clone());
        assert!(store_result.is_ok(), "Failed to store artist data");

        // Print all keys in the database after storing
        println!("Keys in the database after storing:");
        for key in db.iter().keys() {
            println!("{:?}", String::from_utf8_lossy(&key.unwrap()));
        }

        // Retrieve using normalized name
        let get_result = get_artist_data(&db, &normalized_name);
        assert!(get_result.is_ok(), "Failed to get artist data");
        let artist_data = get_result.unwrap();
        println!("Retrieved data using normalized name: {:?}", artist_data);
        assert!(artist_data.is_some(), "No data found for normalized name");

        // Retrieve using original name
        let get_result_original = get_artist_data(&db, artist_name);
        assert!(
            get_result_original.is_ok(),
            "Failed to get artist data with original name"
        );
        let artist_data_original = get_result_original.unwrap();
        println!(
            "Retrieved data using original name: {:?}",
            artist_data_original
        );
        assert!(
            artist_data_original.is_some(),
            "No data found for original name"
        );
    }

    #[test]
    fn test_overwrite_artist_data() {
        let temp_dir = tempdir().unwrap();
        let binding = temp_dir.path().join("test_db");
        let db_path = binding.to_str().unwrap();

        let db = open_database(db_path).unwrap();

        let artist_name = "Test Artist";
        let album_count = 1;
        let last_modified = 1234567890;
        let albums = vec![("Album 1".to_string(), "2020".to_string())];

        // Store initial data
        let store_result =
            store_artist_data(&db, artist_name, album_count, last_modified, albums.clone());
        assert!(store_result.is_ok());

        // Overwrite with new data
        let new_album_count = 2;
        let new_last_modified = 1234567891;
        let new_albums = vec![
            ("Album 1".to_string(), "2020".to_string()),
            ("Album 2".to_string(), "2022".to_string()),
        ];

        let overwrite_result = store_artist_data(
            &db,
            artist_name,
            new_album_count,
            new_last_modified,
            new_albums.clone(),
        );
        assert!(overwrite_result.is_ok());

        // Retrieve and verify overwritten data
        let get_result = get_artist_data(&db, artist_name);
        assert!(get_result.is_ok());

        let artist_data = get_result.unwrap().unwrap();
        assert_eq!(artist_data.album_count, new_album_count);
        assert_eq!(artist_data.last_modified, new_last_modified);
        assert_eq!(artist_data.albums, new_albums);
    }
}

/// This module provides functionality to compare local music data with a remote API.
///
/// It includes structures and functions to fetch artist data, compare album lists,
/// and identify discrepancies between local and remote music libraries.
/// This module provides functionality to compare local music data with a remote API.
///
/// It includes structures and functions to fetch artist data, compare album lists,
/// and identify discrepancies between local and remote music libraries.
use crate::api_client::CompareError;
use crate::configuration::ApiSettings;
use crate::foundation::database::get_artist_data;
use crate::foundation::utils::{clean_album_name, normalize_unicode};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled::Db;
use std::collections::HashSet;

/// Represents a response from the Subsonic API.
#[derive(Debug, Serialize, Deserialize)]
struct SubsonicResponse {
    error: Option<ErrorDetails>,
    #[serde(rename = "openSubsonic")]
    open_subsonic: bool,
    #[serde(rename = "serverVersion")]
    server_version: String,
    status: String,
    #[serde(rename = "type")]
    response_type: String,
    version: String,
}

/// Contains details about an error returned by the Subsonic API.
#[derive(Debug, Serialize, Deserialize)]
struct ErrorDetails {
    code: i32,
    message: String,
}

/// Compares local music data with the remote API and returns a list of missing album paths.
///
/// # Arguments
///
/// * `db` - A reference to the local database.
/// * `settings` - API settings for authentication and connection.
///
/// # Example
///
/// ```
/// use musync::compare_with_api;
/// use sled::Db;
/// use musync::ApiSettings;
///
/// async fn example() {
///     let db = Db::open("path/to/db").unwrap();
///     let settings = ApiSettings::new("http://api.example.com", "username", "password");
///
///     match compare_with_api(&db, &settings).await {
///         Ok(missing_albums) => println!("Missing albums: {:?}", missing_albums),
///         Err(e) => eprintln!("Error: {:?}", e),
///     }
/// }
/// ```
pub async fn compare_with_api(
    db: &Db,
    settings: &ApiSettings,
) -> Result<Vec<String>, CompareError> {
    let client = Client::new();

    println!("\x1b[1m\x1b[34mFetching artist data from the remote API...\x1b[0m");
    let artists = fetch_artists(&client, settings).await?;

    let mut all_missing_album_paths = Vec::new();

    for artist in artists {
        let missing_albums = process_artist(db, &client, settings, artist).await?;
        all_missing_album_paths.extend(missing_albums);
    }

    Ok(all_missing_album_paths)
}

/// Fetches artist data from the remote API.
///
/// # Arguments
///
/// * `client` - An HTTP client for making requests.
/// * `settings` - API settings for authentication and connection.
///
async fn fetch_artists(
    client: &Client,
    settings: &ApiSettings,
) -> Result<Vec<Value>, CompareError> {
    let artists_url = format!(
        "{}/getArtists?u={}&p={}&v=1.16.1&c=navidrome&f=json",
        settings.api_base_url, settings.api_username, settings.api_password
    );

    let response: Value = client.get(&artists_url).send().await?.json().await?;

    if let Some(error) = response["subsonic-response"]["error"].as_object() {
        return Err(CompareError::ApiError {
            code: error["code"].as_i64().unwrap_or(0) as i32,
            message: error["message"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string(),
        });
    }

    let mut artists = Vec::new();

    if let Some(indexes) = response["subsonic-response"]["artists"]["index"].as_array() {
        for index in indexes {
            if let Some(index_artists) = index["artist"].as_array() {
                artists.extend(index_artists.iter().cloned());
            }
        }
    }

    Ok(artists)
}

/// Processes an individual artist, comparing local and remote data.
///
/// # Arguments
///
/// * `db` - A reference to the local database.
/// * `client` - An HTTP client for making requests.
/// * `settings` - API settings for authentication and connection.
/// * `artist` - Artist data from the API.
///
async fn process_artist(
    db: &Db,
    client: &Client,
    settings: &ApiSettings,
    artist: Value,
) -> Result<Vec<String>, CompareError> {
    let name = artist["name"].as_str().unwrap_or("");
    let api_album_count = artist["albumCount"].as_u64().unwrap_or(0) as usize;
    let id = artist["id"].as_str().unwrap_or("");

    let normalized_name = normalize_unicode(name);
    if let Some(local_data) = get_artist_data(db, &normalized_name)? {
        if local_data.album_count != api_album_count {
            println!(
                "\x1b[33mMismatch for artist '{}': Local count: {}, API count: {} - Artist id: {}\x1b[0m",
                normalized_name, local_data.album_count, api_album_count, id
            );
            let missing_albums =
                compare_album_lists(client, &settings.api_base_url, id, &local_data.albums).await?;
            Ok(missing_albums)
        } else {
            Ok(Vec::new())
        }
    } else {
        println!(
            "\x1b[31mNo local data found for artist '{}'\x1b[0m",
            normalized_name
        );
        Ok(Vec::new())
    }
}

async fn compare_album_lists(
    client: &Client,
    base_url: &str,
    artist_id: &str,
    local_albums: &[(String, String)],
) -> Result<Vec<String>, CompareError> {
    let artist_url = format!(
        "{}/getArtist?id={}&u=thiago&p=Lopp1010&v=1.16.1&c=navidrome&f=json",
        base_url, artist_id
    );

    let response: Value = client.get(&artist_url).send().await?.json().await?;

    let api_albums: HashSet<String> = response["subsonic-response"]["artist"]["album"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|album| album["name"].as_str().map(clean_album_name))
        .collect();

    let local_set: HashSet<String> = local_albums.iter().map(|(name, _)| name.clone()).collect();

    println!("\x1b[34mAPI albums: {:?}\x1b[0m", api_albums);
    println!("\x1b[34mLocal albums: {:?}\x1b[0m", local_set);

    let missing_locally: Vec<_> = api_albums.difference(&local_set).collect();
    let missing_in_api: Vec<_> = local_set.difference(&api_albums).collect();

    print_missing_albums(&missing_locally, &missing_in_api);

    Ok(missing_in_api
        .into_iter()
        .filter_map(|album_name| {
            local_albums
                .iter()
                .find(|(name, _)| name == album_name)
                .map(|(_, path)| path.clone())
        })
        .collect())
}

fn print_missing_albums(missing_locally: &[&String], missing_in_api: &[&String]) {
    if !missing_locally.is_empty() {
        println!(
            "\x1b[33mAlbums missing locally: {:?}\x1b[0m",
            missing_locally
        );
    }
    if !missing_in_api.is_empty() {
        println!("\x1b[33mAlbums missing in API: {:?}\x1b[0m", missing_in_api);
    }
}

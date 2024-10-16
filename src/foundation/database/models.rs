use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArtistData {
    pub album_count: usize,
    pub last_modified: u64,
    pub albums: Vec<(String, String)>, // (album name, full path)
}

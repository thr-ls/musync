use regex::Regex;
use unicode_normalization::UnicodeNormalization;

/// Cleans up an album name by removing text within square brackets.
///
/// This function takes a string slice representing an album name and removes
/// any content enclosed in square brackets (e.g., "[Deluxe Edition]", "[2024]"). It then
/// trims any leading or trailing whitespace.
///
/// # Arguments
///
/// * `name` - A string slice that holds the album name to be cleaned.
///
/// # Examples
///
/// ```
/// use musync::foundation::utils::clean_album_name;
///
/// let album = "Dark Side of the Moon [Remastered]";
/// let cleaned = clean_album_name(album);
/// assert_eq!(cleaned, "Dark Side of the Moon");
/// ```
pub fn clean_album_name(name: &str) -> String {
    let re = Regex::new(r"\[.*?\]").unwrap();
    let cleaned = re.replace_all(name, "");
    let result = cleaned.trim().to_string();
    result
}

/// Normalizes Unicode characters and converts text to lowercase.
///
/// This function takes a string slice, decomposes its Unicode characters
/// (NFD normalization), and then converts the result to lowercase. This is
/// useful for creating consistent, comparable versions of strings that may
/// contain diacritics or other Unicode variations.
///
/// # Arguments
///
/// * `input` - A string slice that holds the text to be normalized.
///
/// # Examples
///
/// ```
/// use musync::foundation::utils::normalize_unicode;
///
/// let text = "Café";
/// let normalized = normalize_unicode(text);
/// assert_eq!(normalized, "cafe");
/// ```
pub fn normalize_unicode(input: &str) -> String {
    input.nfd().collect::<String>().to_lowercase()
}

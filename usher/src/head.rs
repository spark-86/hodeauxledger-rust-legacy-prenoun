use rusqlite::Connection;

pub fn get_head(scope: &str, keyfile: &str, cache_path: &str) -> Result<String, anyhow::Error> {
    let cache = Connection::open(cache_path)?;

    Ok("".to_string())
}

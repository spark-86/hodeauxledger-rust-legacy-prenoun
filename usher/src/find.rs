use hl_core::Config;
use rusqlite::Connection;

use crate::argv::FindArgs;

pub async fn find_scope(
    find_args: &FindArgs,
    config_path: &str,
    scope: &str,
) -> Result<(), anyhow::Error> {
    let config = hl_services::config::load_config(config_path);
    let host = &find_args.host;
    let port = &find_args.port;
    let config = match config {
        Ok(config) => config,
        Err(_) => Config {
            host: host.clone(),
            port: port.parse()?,
            bin_dir: "./bin".to_string(),
            fs_dir: "./ledger/fs".to_string(),
            data_dir: "./ledger".to_string(),
            cache_db: "./ledger/cache.db".to_string(),
            hot_keys: vec![],
            verbose: false,
        },
    };
    let cache = Connection::open(&config.cache_db)?;
    let scope = hl_services::scope::get::get_scope(&cache, scope).await?;
    println!("{:#?}", scope);
    Ok(())
}

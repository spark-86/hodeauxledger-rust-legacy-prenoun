use anyhow::bail;
use hl_core::scope::scope::Scope;
use hl_io::db::scope::{retrieve_scope, retrieve_scope_full, store_scope};
use rusqlite::Connection;

use crate::request::request;

pub async fn fetch_from_network(
    host: &str,
    port: &str,
    scope: &str,
) -> Result<Scope, anyhow::Error> {
    let record_types = vec!["scope:*"];
    let keyfile = "./Verondu3.sk"; // TODO: Get from config
    let rhex_stream = request(host, port, scope, &record_types, keyfile).await?;
    for rhex in rhex_stream {
        let scope_obj: Scope = serde_json::from_value(rhex.intent.data)?;
        return Ok(scope_obj);
    }
    bail!("No scope object received from network")
}

pub async fn get_scope_full(
    cache: &Connection,
    scope: &str,
    record_types: &[&str],
    keyfile: &str,
) -> Result<Scope, anyhow::Error> {
    match retrieve_scope(cache, scope) {
        Ok(scope_data) => Ok(scope_data),
        Err(_) => {
            // Not cached — request from network
            let host = "127.0.0.1"; // TODO: Get from config
            let port = "1984"; // TODO: Get from config
            let rhex_stream = request(host, port, scope, record_types, keyfile).await?;
            for rhex in rhex_stream {
                let scope_obj: Scope = serde_json::from_value(rhex.intent.data)?;
                // Store the full scope object, including policy, authorities, and ushers
                hl_io::db::scope::store_scope_full(cache, &scope_obj)?;
                return Ok(scope_obj);
            }
            bail!("No scope object received from network")
        }
    }
}

/// Recursively walks up the scope hierarchy until it finds a parent.
/// Once found, caches the target_scope so subsequent lookups don't recurse again.
async fn get_parent(
    cache: &Connection,
    initial_scope: &str,
    target_scope: &str,
) -> Result<Scope, anyhow::Error> {
    // 👇 own the string so it lives across loop iterations
    let mut current_scope = initial_scope.to_string();

    loop {
        if current_scope.is_empty() {
            bail!("Cannot get parent of root scope");
        }

        let scope_parts: Vec<&str> = current_scope.split('.').collect();
        if scope_parts.len() <= 1 {
            bail!("No parent exists for this scope");
        }

        // 👇 This is a new String every loop, but that's fine because current_scope is owned
        let parent = scope_parts[..scope_parts.len() - 1].join(".");

        match retrieve_scope_full(cache, &parent) {
            Ok(parent_scope) => {
                let host = &parent_scope.ushers[0].host;
                let port = &parent_scope.ushers[0].port.to_string();
                let target_scope_obj = fetch_from_network(host, port, target_scope).await?;
                store_scope(cache, &target_scope_obj)?;
                return Ok(parent_scope);
            }
            Err(_) => {
                // 👇 Reassign ownership — no dangling reference
                current_scope = parent;
            }
        }
    }
}

/// Gets the scope, pulling and caching if missing.
pub async fn get_scope(cache: &Connection, scope: &str) -> Result<Scope, anyhow::Error> {
    match retrieve_scope_full(cache, scope) {
        Ok(scope_data) => Ok(scope_data),
        Err(_) => {
            // Not cached — climb up until we find a known parent, then cache this one.
            get_parent(cache, scope, scope).await?;
            retrieve_scope(cache, scope)
        }
    }
}

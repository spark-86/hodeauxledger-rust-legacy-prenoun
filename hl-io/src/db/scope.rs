use hl_core::scope::scope::Scope;
use rusqlite::{Connection, params};

use crate::db::{
    authority::get_authorities, connect_db, policy::retrieve_policy, usher::get_ushers,
};

pub fn store_scope(cache: &Connection, scope: &Scope) -> Result<(), anyhow::Error> {
    let status = cache.execute(
        "INSERT OR REPLACE INTO scopes (scope, role, last_synced) VALUES (?1, ?2, ?3)",
        params![scope.name, scope.role.to_string(), scope.last_synced],
    );
    match status {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(anyhow::anyhow!("Failed to store scope"));
        }
    }
    Ok(())
}

pub fn store_scope_full(cache: &Connection, scope: &Scope) -> Result<(), anyhow::Error> {
    let status = cache.execute(
        "INSERT OR REPLACE INTO scopes (scope, role, last_synced) VALUES (?1, ?2, ?3)",
        params![scope.name, scope.role.to_string(), scope.last_synced],
    );
    match status {
        Ok(_) => {
            for auth in scope.authorities.iter() {
                crate::db::authority::store_authority(&cache, &scope.name, &auth)?;
            }
            if scope.policy.is_some() {
                crate::db::policy::store_policy_full(
                    cache,
                    &scope.name,
                    &scope.policy.clone().unwrap(),
                )?;
            }
            for usher in scope.ushers.iter() {
                crate::db::usher::store_usher(cache, &scope.name, &usher)?;
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    Ok(())
}

pub fn scope_exists(cache: &Connection, scope_name: &str) -> Result<bool, anyhow::Error> {
    let mut stmt = cache.prepare("SELECT 1 FROM scopes WHERE scope = ?1 LIMIT 1")?;
    let mut rows = stmt.query(params![scope_name])?;
    Ok(rows.next()?.is_some())
}

pub fn retrieve_scope(cache: &Connection, scope_name: &str) -> Result<Scope, anyhow::Error> {
    let mut stmt = cache.prepare("SELECT scope, role, last_synced FROM scopes WHERE scope = ?1")?;
    let mut rows = stmt.query(params![scope_name])?;

    if let Some(row) = rows.next()? {
        let scope = Scope {
            name: row.get("scope")?,
            role: row.get::<_, String>("role")?.into(),
            last_synced: row.get("last_synced")?,
            policy: None,
            authorities: vec![],
            ushers: vec![],
        };
        Ok(scope)
    } else {
        Err(anyhow::anyhow!("Scope not found"))
    }
}

pub fn retrieve_scope_full(cache: &Connection, scope: &str) -> Result<Scope, anyhow::Error> {
    let mut scope_data = retrieve_scope(cache, scope)?;
    scope_data.policy = Some(retrieve_policy(cache, scope)?);
    scope_data.authorities = get_authorities(cache, scope)?;
    scope_data.ushers = get_ushers(cache, scope)?;
    Ok(scope_data)
}

pub fn flush_scope(scope_name: &str) -> Result<(), anyhow::Error> {
    let cache = connect_db("./ledger/cache/cache.db")?;
    cache.execute("DELETE FROM scopes WHERE scope = ?1", params![scope_name])?;
    Ok(())
}

pub fn flush_scope_full(cache: &Connection, scope_name: &str) -> Result<(), anyhow::Error> {
    cache.execute("DELETE FROM scopes WHERE scope = ?1", params![scope_name])?;
    cache.execute(
        "DELETE FROM authorities WHERE scope = ?1",
        params![scope_name],
    )?;
    cache.execute("DELETE FROM policies WHERE scope = ?1", params![scope_name])?;
    cache.execute("DELETE FROM ushers WHERE scope = ?1", params![scope_name])?;
    Ok(())
}

pub fn flush_scopes(cache: &Connection) -> Result<(), anyhow::Error> {
    cache.execute("DELETE FROM scopes", params![])?;
    Ok(())
}

pub fn build_table(cache: &Connection) -> Result<(), anyhow::Error> {
    cache.execute(
        "CREATE TABLE scopes (
                scope TEXT,
                role TEXT,
                last_synced INTEGER,
                PRIMARY KEY (scope)
            )
        ",
        params![],
    )?;
    Ok(())
}

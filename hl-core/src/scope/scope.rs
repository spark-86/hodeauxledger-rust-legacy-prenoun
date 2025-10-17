use std::fmt::{self, Display};

use crate::{Authority, Usher, policy::policy::Policy};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Scope {
    pub name: String,
    pub role: ScopeRoles,
    pub last_synced: u64,
    pub policy: Option<Policy>,
    pub authorities: Vec<Authority>,
    pub ushers: Vec<Usher>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum ScopeRoles {
    Authority = 0,
    Mirror = 1,
    Cache = 2,
    NoCache = 3,
}

impl Display for ScopeRoles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ScopeRoles::Authority => "Authority",
            ScopeRoles::Mirror => "Mirror",
            ScopeRoles::Cache => "Cache",
            ScopeRoles::NoCache => "NoCache",
        };
        write!(f, "{}", s)
    }
}
impl From<String> for ScopeRoles {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Authority" => ScopeRoles::Authority,
            "Mirror" => ScopeRoles::Mirror,
            "Cache" => ScopeRoles::Cache,
            "NoCache" => ScopeRoles::NoCache,
            _ => ScopeRoles::NoCache, // Default to NoCache if unknown
        }
    }
}
impl Scope {
    pub fn new(name: &str, role: ScopeRoles) -> Self {
        Self {
            name: name.to_string(),
            role,
            last_synced: 0,
            policy: None,
            authorities: vec![],
            ushers: vec![],
        }
    }
}

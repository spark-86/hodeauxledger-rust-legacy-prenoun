use crate::Key;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Authority {
    pub scope: String,
    pub key: Key,
    pub roles: Vec<String>,
    pub eff: Option<u64>,
    pub exp: Option<u64>,
    pub note: Option<String>,
}

impl Authority {
    pub fn new() -> Self {
        Self {
            scope: String::new(),
            key: Key::new(),
            roles: Vec::new(),
            eff: None,
            exp: None,
            note: None,
        }
    }

    pub fn is_valid(&self, current_time: u64) -> bool {
        if self.eff.is_some() && self.eff.unwrap() > current_time {
            return false;
        }
        if self.exp.is_some() && self.exp.unwrap() < current_time {
            return false;
        }
        true
    }
}

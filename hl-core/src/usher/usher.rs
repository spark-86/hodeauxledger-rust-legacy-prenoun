#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Usher {
    pub note: String,
    pub public_key: [u8; 32],
    pub host: String,
    pub port: u16,
    pub proto: String,
    pub priority: u8,
}

impl Usher {
    pub fn new() -> Self {
        Self {
            note: "".to_string(),
            host: "localhost".to_string(),
            public_key: [0u8; 32],
            port: 1984,
            proto: "rhex".to_string(),
            priority: 255,
        }
    }
}

# Intent data structure

```rust
pub struct Intent {                      // 🎯
    pub previous_hash: Option<[u8; 32]>, // ⬅️🧬
    pub scope: String,                   // 🌐
    pub nonce: String,                   // 🎲
    pub author_pk: [u8; 32],             // ✍️🔓
    pub usher_pk: [u8; 32],              // 📣🔓
    pub record_type: String,             // 📄
    pub data: serde_json::Value          // 📊
}
```

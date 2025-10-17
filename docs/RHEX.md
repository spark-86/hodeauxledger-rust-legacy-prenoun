# R⬢ (Rhex) Data structure

```rust
pub struct Rhex {
    pub magic: [u8; 6],                  // 🪄
    pub intent: Intent,                  // 🎯
    pub context: Context,                // 🖼️
    pub signatures: Vec<Signature>,      // 🖊️🖊️🖊️
    pub current_hash: Option<[u8; 32]>,  // ⬇️🧬
}
```

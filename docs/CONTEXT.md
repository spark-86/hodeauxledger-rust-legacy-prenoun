# Context data structure

This is supplied by the usher and verified via quorum.

`at` must be within policy tolerence for new appends. Spacial coordinates are all or nothing, meaning if one is set, they all must be set or get an invalid R⬢ response.

```rust
pub struct Context {
    pub at: u64,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub z: Option<f64>,
    pub refer: Option<String>
}
```

# Signature data structure

## SigType enum

```rust
pub enum SigType {
    Author,
    Usher,
    Quorum,
}
```

## Signature structure

```rust
pub struct Signature {
    sig_type: SigType,
    public_key: [u8; 32],
    signature: [u8; 64]
}
```

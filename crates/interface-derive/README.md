# `interface-derive`

Derive macro for [`interface`](https://docs.rs/interface) — typed, lossy-aware
interface translation between API versions.

> ⚠️ Work in progress. The macro is not yet implemented.

## Planned usage

```rust
use interface_derive::{Upgrade, Downgrade};

#[derive(Upgrade, Downgrade)]
pub struct User {
    pub name: String,
}
```

See [`interface`](https://docs.rs/interface) for the full model.

## License

MIT

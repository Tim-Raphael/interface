# `interface` *(WIP)*

Typed, lossy-aware interface translation between API versions.

## Overview

`interface` models data migration between versioned API schemas. Each
conversion is a [`Translation<Source, Target, Lossiness>`] — a deferred,
inspectable value that carries a [`Diff`] and encodes at the type level
whether the conversion is information-preserving ([`Lossless`]) or
destructive ([`Lossy`]).

## Usage

### Implement `Upgrade` / `Downgrade`

```rust
use interface::{Diff, Lossless, Lossy, Translation, Upgrade, Downgrade};

mod v1 {
    #[derive(Debug, PartialEq, Eq)]
    pub struct User { pub name: String }
}

mod v2 {
    #[derive(Debug, PartialEq, Eq)]
    pub struct User { pub name: String, pub email: String }
}

impl Upgrade<v2::User> for v1::User {
    type Lossiness = Lossless;

    fn upgrade(self) -> Translation<Self, v2::User, Lossless> {
        let diff = Diff::new().add("email", "default@example.com");
        Translation::new(self, Box::new(|s| v2::User {
            name: s.name,
            email: "default@example.com".into(),
        }), diff)
    }
}

impl Downgrade<v1::User> for v2::User {
    type Lossiness = Lossy;

    fn downgrade(self) -> Translation<Self, v1::User, Lossy> {
        let diff = Diff::new().sub("email", &self.email);
        Translation::new(self, Box::new(|s| v1::User { name: s.name }), diff)
    }
}
```

### Execute a translation

```rust
// Lossless: `.translate()` is only available on `Translation<_, _, Lossless>`
let t = v1::User { name: "Alice".into() }.upgrade();
assert!(!t.is_lossy());
let v2_user = t.translate();

// Lossy: must call `.translate_lossy()` — the distinct method name forces acknowledgement
let t = v2_user.downgrade();
assert!(t.is_lossy());
println!("{t}");             // prints the diff
let v1_user = t.translate_lossy();
```

## License

MIT

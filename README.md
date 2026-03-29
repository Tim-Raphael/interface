# interface *(WIP)*

When APIs evolve, structs change shape. Fields get added, renamed, or dropped.
Code that handles multiple versions of a schema — REST API consumers,
database migration layers, message bus subscribers — has to deal with this,
usually through ad-hoc conversion functions that silently discard data.

`interface` makes those conversions explicit and type-safe:

- **Lossy vs. lossless is a compile-time distinction.** Downgrading a `v2::User`
  to `v1::User` drops the `email` field. That destruction is encoded in the
  type as `Translation<_, _, Lossy>`, and the call site must use
  `.translate_lossy()` — the compiler won't let you silently throw data away.

- **Every translation carries a diff.** Before committing to a conversion you
  can inspect exactly which fields are added or removed.

- **Conversions are deferred values, not immediate calls.** `.upgrade()` and
  `.downgrade()` return a `Translation` you can log, forward, or discard
  before executing.

## Crates

| Crate | Description |
|-------|-------------|
| [`interface`](crates/interface) | Core traits and types *(WIP)* |
| [`interface-derive`](crates/interface-derive) | Derive macros *(WIP)* |

## License

MIT

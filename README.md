# bevy_simple_prefs

An unambitious Bevy plugin for persisting multiple Bevy `Resources` into a single preferences file, suitable for small projects.

- Automatically persists prefs when they are modified
- Persists to a single `ron` file
- Does IO in Bevy's async task pool
- WASM compatible

## Compatibility

| `bevy_simple_prefs` | `bevy` |
| :--                 | :--    |
| `0.1`               | `0.14` |

## Contributing

Please feel free to open a PR, but keep in mind this project's goals. This is meant to be a very lightweight crate. There should be zero additional dependencies on other Bevy ecosystem crates.

Please keep PRs small and scoped to a single feature or fix.

## Alternatives

If you need more features, check out [`bevy-persistent`](https://crates.io/crates/bevy-persistent) or [`bevy-settings`](https://crates.io/crates/bevy-settings).

# bevy_simple_prefs

[![crates.io](https://img.shields.io/crates/v/bevy_simple_prefs.svg)](https://crates.io/crates/bevy_simple_prefs)
[![docs](https://docs.rs/bevy_simple_prefs/badge.svg)](https://docs.rs/bevy_simple_prefs)
[![Following released Bevy versions](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://bevyengine.org/learn/book/plugin-development/#main-branch-tracking)

An unambitious Bevy plugin for persisting multiple Bevy `Resource`s into a single preferences file, suitable for small projects like jam games.

- Persists to a single `ron` file
- Does IO in Bevy's async task pool
- WASM compatible

## Usage

- Derive `Prefs` on a `struct` with members that are `Resource`s you want to be saved
- Simply modify your `Resource`s to initiate a save
- Write code that reacts to those `Resource`s changing, if you want

See [examples/prefs.rs](./bevy_simple_prefs/examples/prefs.rs)

## Compatibility

| `bevy_simple_prefs` | `bevy` |
| :--                 | :--    |
| `0.5`-`0.6`         | `0.16` |
| `0.4`               | `0.15` |
| `0.1`-`0.3`         | `0.14` |

## Contributing

Please feel free to open a PR, but keep in mind this project's goals. This is meant to be a very lightweight crate. There should be zero additional dependencies on other Bevy ecosystem crates.

Please keep PRs small and scoped to a single feature or fix.

## Alternatives

If you need more features, check out [`bevy-settings`](https://crates.io/crates/bevy-settings). There are also a few other options in the [persistence section](https://bevyengine.org/assets/#persistence) of Bevy Assets.
